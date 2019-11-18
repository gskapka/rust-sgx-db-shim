#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

use sgx_types::*;
use std::{
    u32,
    vec::Vec,
};
// NOTE: RMing this causes duplicate lang item errs!
// Because otherwise vec declared below imports std!

// Ocall API
extern "C" {
    pub  fn get_from_db(
        ret_val: *mut sgx_status_t,
        key_pointer: *mut u8,
        key_size: *const u32,
        value_pointer: *mut u8,
        value_size: *const u32,
    ) -> sgx_status_t;
}

#[no_mangle]
pub extern "C" fn run() -> sgx_status_t {
    println!("✔ Running inside enclave...");
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
}
