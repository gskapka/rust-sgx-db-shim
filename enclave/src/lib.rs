#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_types;
extern crate sgx_tseal;

use sgx_types::*;
use sgx_tseal::SgxSealedData;
use sgx_types::marker::ContiguousMemory;
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

// Ocall API
extern "C" {
    pub fn save_to_db(
        ret_val: *mut sgx_status_t,
        key_pointer: *mut u8,
        key_size: *const u32,
        sealed_log_size: *const u32,
        scratch_pad_pointer: *mut u8,
    ) -> sgx_status_t;

    pub  fn get_from_db(
        ret_val: *mut sgx_status_t,
        key_pointer: *mut u8,
        key_size: *const u32,
        value_pointer: *mut u8,
        value_size: *const u32,
    ) -> sgx_status_t;
}

fn to_sealed_log<T: Copy + ContiguousMemory>(
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

fn get_item_from_db(mut key: Bytes) -> Result<sgx_status_t, String> {
    println!("✔ [Enclave] Getting item from external db...");
    pub const MEGA_BYTE: usize = 1_000_000;
    pub const U32_BYTES: usize = 4;
    let scratch_pad_size = 1 * MEGA_BYTE; // Create scratch-pad at `run_sample`!
    let key_pointer: *mut u8 = &mut key[0];
    let mut scratch_pad: Vec<u8> = vec![0; scratch_pad_size];
    let scratch_pad_pointer: *mut u8 = &mut scratch_pad[0];
    let ocall_result = unsafe {
        get_from_db(
            &mut sgx_status_t::SGX_SUCCESS,
            key_pointer,
            key.len() as *const u32,
            scratch_pad_pointer,
            scratch_pad_size as *const u32,
        )
    };
    let mut length_of_data_arr = [0u8; U32_BYTES];
    let bytes = &scratch_pad[..length_of_data_arr.len()];
    length_of_data_arr.copy_from_slice(bytes);
    let length_of_data = u32::from_le_bytes(length_of_data_arr) as usize;
    println!("✔ [Enclave] Length of data received: {:?}", length_of_data);
    let final_data = &scratch_pad[U32_BYTES..U32_BYTES + length_of_data];
    println!("✔ [Enclave] Final retreived data: {:?}", final_data);
    Ok(sgx_status_t::SGX_SUCCESS)
}

fn seal_item_into_db(
    mut key: Bytes,
    value: Bytes,
    scratch_pad_pointer: *mut u8,
    scratch_pad_size: u32,
) -> Result<sgx_status_t, String> {
    println!("✔ [Enclave] Sealing data...");
    let extra_data: [u8; 0] = [0u8; 0]; // TODO Abstract this away!
    let sealing_result = SgxSealedData::<[u8]>::seal_data(
        &extra_data,
        &value[..]
    );
    let sealed_data = match sealing_result {
        Ok(x) => x,
        Err(sgx_error) => return Err(sgx_error.to_string())
    };
    println!("✔ [Enclave] Data sealed!");
    let sealed_log_size = size_of::<sgx_sealed_data_t>() + value.len();
    println!("✔ [Enclave] Sealed log size: {}", sealed_log_size);
    let option = to_sealed_log(
        &sealed_data,
        scratch_pad_pointer,
        scratch_pad_size as u32,
    );
    if option.is_none() {
        return Err(sgx_status_t::SGX_ERROR_INVALID_PARAMETER.to_string())
    }
    println!("✔ [Enclave] Sealed data written into app's scratch-pad!");
    println!("✔ [Enclave] Sending db key & sealed data size via OCALL");
    let key_pointer: *mut u8 = &mut key[0];
    let ocall_result = unsafe {
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
    scratch_pad_pointer: *mut u8, // TODO: Rename to `app_scratch_pad_pointer`
    scratch_pad_size: u32,
) -> sgx_status_t {
    // TODO Use Result returning fxns and match against a pipeline in here!
    // Make an enclave scratch pad and save that pointer to state!
    println!(
        "✔ [Enclave] Running example inside enclave...{}",
        "✔ [Enclave] Creating data..."
    );
    let key: Bytes = vec![1, 3, 3, 7];
    let value: Bytes = vec![1, 2, 3, 4, 5, 6];
    seal_item_into_db(key.clone(), value, scratch_pad_pointer, scratch_pad_size)
        .and_then(|_| get_item_from_db(key))
        .unwrap()
}
