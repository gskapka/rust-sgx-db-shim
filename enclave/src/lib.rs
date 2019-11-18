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
};

pub type Bytes = Vec<u8>;

// Ocall API
extern "C" {
    pub fn seal_into_db(
        ret_val: *mut sgx_status_t,
        key_pointer: *mut u8,
        key_size: *const u8,
        value_pointer: *mut u8,
        value_size: *const u8,
    ) -> sgx_status_t;

    pub  fn get_from_db(
        ret_val: *mut sgx_status_t,
        key_pointer: *mut u8,
        key_size: *const u32,
        value_pointer: *mut u8,
        value_size: *const u32,
    ) -> sgx_status_t;
}

#[no_mangle]
pub extern "C" fn run_sample(
    scratch_pad_pointer: *mut u8,
    scratch_pad_size: *const u8,
) -> sgx_status_t {
    /*
     * So I wanna create some data
     * seal it outside
     * save it in hash map.
     *
     * Then run a function that queries that hashmap for that data.
     *
     */
    println!("✔ Running example inside enclave...");
    println!("✔ Creating data...");
    let key: Bytes = vec![1, 3, 3, 7];
    let value: Bytes = vec![1, 2, 3, 4, 5, 6];
    println!("✔ Sealing data...");
    let sealing_result = SgxSealedData::<[u8]>::seal_data(&key, &value[..]);
    let sealed_data = match sealing_result {
        Ok(x) => x,
        Err(sgx_error) => return sgx_error
    };
    /*
    let option = to_sealed_log(
        &sealed_data,
        sealed_log_pointer, // IE Where to write this too? // NEED A SCRATCH PAD ON THE OUTSIDE!
        sealed_log_size
    );
    if option.is_none() {
        return sgx_status_t::SGX_ERROR_INVALID_PARAMETER;
    }
    */
    sgx_status_t::SGX_SUCCESS
    /*
    let mut key: [u8; 3] = [107, 101, 121]; // NOTE: b"key";
    pub const MEGA_BYTE: usize = 1_000_000;
    pub const U32_BYTES: usize = 4;
    let value_size = 1 * MEGA_BYTE;
    let key_size = 3;
    let key_pointer: *mut u8 = &mut key[0];
    let mut value: Vec<u8> = vec![0; value_size];
    let value_pointer: *mut u8 = &mut value[0];
    println!("✔ Value before: {:?}", &value[..20]);
    let res = unsafe {
        get_from_db(
            &mut sgx_status_t::SGX_SUCCESS,
            key_pointer,
            key_size as *const u32,
            value_pointer,
            value_size as *const u32,
        )
    };
    println!("✔ Res after: {:?}", res);
    println!("✔ Value after: {:?}", &value[..20]);
    let mut length_of_data_arr = [0u8; U32_BYTES];
    let bytes = &value[..length_of_data_arr.len()];
    length_of_data_arr.copy_from_slice(bytes);
    let length_of_data = u32::from_le_bytes(length_of_data_arr) as usize;
    println!("✔ Length of data as u32: {:?}", length_of_data);
    let final_data = &value[U32_BYTES..U32_BYTES + length_of_data];
    println!("✔ Final data: {:?}", final_data);
    sgx_status_t::SGX_SUCCESS
    */
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
