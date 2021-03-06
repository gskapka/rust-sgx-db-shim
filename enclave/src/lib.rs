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

fn get_length_of_data_in_scratch_pad(scratch_pad: &Bytes) -> usize {
    let mut length_of_data_arr = [0u8; U32_NUM_BYTES];
    let bytes = &scratch_pad[..U32_NUM_BYTES];
    length_of_data_arr.copy_from_slice(bytes);
    u32::from_le_bytes(length_of_data_arr) as usize
}

fn get_data_from_scratch_pad(scratch_pad: &Bytes) -> Bytes {
    let length_of_data = get_length_of_data_in_scratch_pad(scratch_pad);
    scratch_pad[U32_NUM_BYTES..U32_NUM_BYTES + length_of_data].to_vec()
}

fn get_item_from_db(
    mut key: Bytes,
    scratch_pad: &mut Bytes,
) -> Result<sgx_status_t, String> {
    info!("✔ [Enc] Getting item from external db...");
    let key_pointer: *mut u8 = &mut key[0];
    let enclave_scratch_pad_pointer: *mut u8 = &mut scratch_pad[0];
    unsafe {
        get_from_db(
            &mut sgx_status_t::SGX_SUCCESS,
            key_pointer,
            key.len() as *const u32,
            enclave_scratch_pad_pointer,
            SCRATCH_PAD_SIZE as *const u32,
        )
    };
    let mut data = get_data_from_scratch_pad(&scratch_pad);
    info!("✔ [Enc] External data written to enclave's scratch pad!");
    trace!("✔ [Enc] Retreived data length: {:?}", data.len());
    let data_pointer: *mut u8 = &mut data[0];
    let maybe_sealed_data = from_sealed_log_for_slice::<u8>(
        data_pointer,
        data.len() as u32
    );
    let sealed_data = match maybe_sealed_data {
        Some(sealed_data) => sealed_data,
        None => return Err(
            sgx_status_t::SGX_ERROR_INVALID_PARAMETER.to_string()
        )
    };
    trace!(
        "✔ [Enc] Payload: {:?}",
        sealed_data.get_payload_size()
    );
    trace!(
        "✔ [Enc] Encrypted text: {:?}",
        sealed_data.get_encrypt_txt()
    );
    trace!(
        "✔ [Enc] Additional text: {:?}",
        sealed_data.get_additional_txt()
    );
    let unsealed_data = match sealed_data.unseal_data() {
        Ok(unsealed_data) => unsealed_data,
        Err(e) => return Err(e.to_string())
    };
    let cbor_encoded_slice = unsealed_data.get_decrypt_txt();
    let final_data: DatabaseKeyAndValue = serde_cbor::from_slice(
        cbor_encoded_slice
    ).unwrap();
    //info!("✔ [Enc] Final unsealed data: {:?}", final_data);
    info!("✔ [Enc] Final unsealed key: {:?}", final_data.key);
    info!("✔ [Enc] Final unsealed value: {:?}", final_data.value);
    Ok(sgx_status_t::SGX_SUCCESS)
}

fn seal_item_into_db(
    mut key: Bytes,
    value: Bytes,
    scratch_pad_pointer: *mut u8,
) -> Result<sgx_status_t, String> {
    info!("✔ [Enc] Sealing data...");
    let data = DatabaseKeyAndValue::new(key.clone(), value);
    info!("✔ [Enc] Key to seal: {:?}", data.key);
    info!("✔ [Enc] Value to seal: {:?}", data.value);
    let encoded_data = serde_cbor::to_vec(&data).unwrap();
    let encoded_slice = encoded_data.as_slice();
    let extra_data: [u8; 0] = [0u8; 0]; // TODO Abstract this away!
    let sealing_result = SgxSealedData::<[u8]>::seal_data(
        &extra_data,
        encoded_slice,
    );
    let sealed_data = match sealing_result {
        Ok(sealed_data) => sealed_data,
        Err(sgx_error) => return Err(sgx_error.to_string())
    };
    trace!(
        "✔ [Enc] Sealed-data additional data: {:?}",
        sealed_data.get_additional_txt()
    );
    trace!(
        "✔ [Enc] Sealed-data encrypted txt: {:?}",
        sealed_data.get_encrypt_txt()
    );
    trace!(
        "✔ [Enc] Sealed-data payload size: {:?}",
        sealed_data.get_payload_size()
    );
    trace!("✔ [Enc] Raw sealed data size: {:?}",
        SgxSealedData::<u8>::calc_raw_sealed_data_size(
            sealed_data.get_add_mac_txt_len(),
            sealed_data.get_encrypt_txt_len(),
        )
    );
    trace!("✔ [Enc] Data sealed successfully!");
    let sealed_log_size = size_of::<sgx_sealed_data_t>() + encoded_slice.len();
    trace!("✔ [Enc] Sealed log size: {}", sealed_log_size);
    let option = to_sealed_log_for_slice(
        &sealed_data,
        scratch_pad_pointer,
        sealed_log_size as u32,
    );
    if option.is_none() {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER.to_string())
    }
    info!("✔ [Enc] Sealed data written into app's scratch-pad!");
    info!("✔ [Enc] Sending db key & sealed data size via OCALL...");
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
    info!("✔ [Enc] Running example inside enclave!");
    info!("✔ [Enc] Creating data to save to db...");
    let mut enclave_scratch_pad: Vec<u8> = vec![0; SCRATCH_PAD_SIZE];
    let key: Bytes = vec![6, 6, 6];
    let value: Bytes = vec![1, 3, 3, 7];
    seal_item_into_db(key.clone(), value, app_scratch_pad_pointer)
        .and_then(|_| get_item_from_db(key, &mut enclave_scratch_pad))
        .unwrap()
}
