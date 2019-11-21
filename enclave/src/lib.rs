#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
#[cfg(not(target_env = "sgx"))]
#[macro_use] extern crate sgx_tstd as std;

extern crate sgx_types;
extern crate sgx_tseal;
extern crate serde_cbor;
extern crate env_logger;

pub mod ocall_api;

use sgx_types::*;
use sgx_tseal::SgxSealedData;
use sgx_types::marker::ContiguousMemory;
use ocall_api::{
    save_to_db,
    get_from_db,
};
use std::{
    u32,
    vec::Vec,
    mem::size_of,
    string::{
        String,
        ToString,
    },
};

pub type Bytes = Vec<u8>;
pub const U32_NUM_BYTES: usize = 4;
pub const MEGA_BYTE: usize = 1_000_000;
pub const SCRATCH_PAD_SIZE: usize = 1 * MEGA_BYTE;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
struct DatabaseKeyAndValue {
    key: Bytes,
    value: Bytes,
}

impl DatabaseKeyAndValue {
    pub fn new(key: Bytes, value: Bytes) -> Self {
        DatabaseKeyAndValue { key, value }
    }
}


fn to_sealed_log_for_slice<T: Copy + ContiguousMemory>(
    sealed_data: &SgxSealedData<[T]>,
    sealed_log: * mut u8,
    sealed_log_size: u32
) -> Option<* mut sgx_sealed_data_t> {
    unsafe {
        sealed_data
            .to_raw_sealed_data_t(
                sealed_log as * mut sgx_sealed_data_t,
                sealed_log_size
            )
    }
}

fn from_sealed_log_for_slice<'a, T: Copy + ContiguousMemory>(
    sealed_log: * mut u8,
    sealed_log_size: u32
) -> Option<SgxSealedData<'a, [T]>> {
    unsafe {
        SgxSealedData::<[T]>::from_raw_sealed_data_t(
            sealed_log as * mut sgx_sealed_data_t,
            sealed_log_size
        )
    }
}

fn get_item_from_db(
    mut key: Bytes,
    enclave_scratch_pad: &mut Bytes,
) -> Result<sgx_status_t, String> {
    info!("✔ [Enclave] Getting item from external db...");
    let key_pointer: *mut u8 = &mut key[0];
    let enclave_scratch_pad_pointer: *mut u8 = &mut enclave_scratch_pad[0];
    unsafe {
        get_from_db(
            &mut sgx_status_t::SGX_SUCCESS,
            key_pointer,
            key.len() as *const u32,
            enclave_scratch_pad_pointer,
            SCRATCH_PAD_SIZE as *const u32,
        )
    };
    let mut length_of_data_arr = [0u8; U32_NUM_BYTES];
    let bytes = &enclave_scratch_pad[..length_of_data_arr.len()];
    length_of_data_arr.copy_from_slice(bytes);
    let length_of_data = u32::from_le_bytes(length_of_data_arr) as usize;
    trace!("✔ [Enclave] Length of data received: {:?}", length_of_data);
    let final_data = enclave_scratch_pad[U32_NUM_BYTES..U32_NUM_BYTES + length_of_data].to_vec();
    trace!("✔ [Enclave] Final retrieved data length: {:?}", final_data.len());
    let mut copied_vector = Vec::new();
    for i in 0..final_data.len() {
        copied_vector.push(final_data[i]);
    }
    let copied_pointer: *mut u8 = &mut copied_vector[0];
    let x = from_sealed_log_for_slice::<u8>(
        copied_pointer,
        final_data.len() as u32
    );
    let y = match x {
        Some(data) => data,
        None => return Err(
            sgx_status_t::SGX_ERROR_INVALID_PARAMETER.to_string()
        )
    };
    trace!("✔ [Enclave] Additional text: {:?}",y.get_additional_txt());
    trace!("✔ [Enclave] Encrypted text: {:?}", y.get_encrypt_txt());
    trace!("✔ [Enclave] Payload: {:?}", y.get_payload_size());
    let unsealed_data = match y.unseal_data() {
        Ok(data) => data,
        Err(e) => return Err(e.to_string())
    };
    let something = unsealed_data.get_decrypt_txt();
    let data: DatabaseKeyAndValue = serde_cbor::from_slice(something).unwrap();
    info!("✔ [Enclave] Final unsealed data: {:?}", data);
    Ok(sgx_status_t::SGX_SUCCESS)
}

fn seal_item_into_db(
    mut key: Bytes,
    value: Bytes,
    scratch_pad_pointer: *mut u8,
) -> Result<sgx_status_t, String> {
    info!("✔ [Enclave] Sealing data...");
    let data = DatabaseKeyAndValue::new(key.clone(), value);
    trace!("✔ [Enclave] Data to seal: {:?}", data);
    let encoded_data = serde_cbor::to_vec(&data).unwrap();
    let encoded_slice = encoded_data.as_slice();
    let extra_data: [u8; 0] = [0u8; 0]; // TODO Abstract this away!
    let sealing_result = SgxSealedData::<[u8]>::seal_data(
        &extra_data,
        encoded_slice,
    );
    let sealed_data = match sealing_result {
        Ok(x) => x,
        Err(sgx_error) => return Err(sgx_error.to_string())
    };
    trace!("✔ [Enclave] Sealed-data additional data: {:?}", sealed_data.get_additional_txt());
    trace!("✔ [Enclave] Sealed-data encrypted txt: {:?}", sealed_data.get_encrypt_txt());
    trace!("✔ [Enclave] Sealed-data payload size: {:?}", sealed_data.get_payload_size());
    trace!("✔ [Enclave] Raw sealed data size: {:?}", SgxSealedData::<u8>::calc_raw_sealed_data_size(
        sealed_data.get_add_mac_txt_len(),
        sealed_data.get_encrypt_txt_len(),
    ));
    trace!("✔ [Enclave] Data sealed successfully!");
    let sealed_log_size = size_of::<sgx_sealed_data_t>() + encoded_slice.len();
    trace!("✔ [Enclave] Sealed log size: {}", sealed_log_size);
    let option = to_sealed_log_for_slice(
        &sealed_data,
        scratch_pad_pointer,
        sealed_log_size as u32,
    );
    if option.is_none() {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER.to_string())
    }
    info!("✔ [Enclave] Sealed data written into app's scratch-pad!");
    info!("✔ [Enclave] Sending db key & sealed data size via OCALL...");
    let key_pointer: *mut u8 = &mut key[0];
    unsafe {
        save_to_db(
            &mut sgx_status_t::SGX_SUCCESS,
            key_pointer,
            key.len() as *const u32,
            sealed_log_size as *const u32,
            scratch_pad_pointer,
        )
    };
    Ok(sgx_status_t::SGX_SUCCESS)
}

#[no_mangle]
pub extern "C" fn run_sample(
    app_scratch_pad_pointer: *mut u8,
    _app_scratch_pad_size: u32,
) -> sgx_status_t {
    env_logger::init();
    info!(
        "✔ [Enclave] Running example inside enclave...{}",
        "✔ [Enclave] Creating data..."
    );
    let mut enclave_scratch_pad: Vec<u8> = vec![0; SCRATCH_PAD_SIZE];
    let key: Bytes = vec![6, 6, 6];
    let value: Bytes = vec![1, 3, 3, 7];
    seal_item_into_db(key.clone(), value, app_scratch_pad_pointer)
        .and_then(|_| get_item_from_db(key, &mut enclave_scratch_pad))
        .unwrap()
}
