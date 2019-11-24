extern crate dirs;
extern crate sgx_urts;
extern crate sgx_types;
extern crate env_logger;
#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;

pub mod db;
pub mod types;
pub mod constants;
pub mod ecall_api;
pub mod init_enclave;

use sgx_types::*;
use db::DATABASE;
use init_enclave::ENCLAVE;
use ecall_api::run_sample;
use constants::SCRATCH_PAD_SIZE;
use std::{
    slice,
    ptr::copy_nonoverlapping,
};

#[no_mangle]
pub extern "C"
fn get_from_db(
    key_pointer: *mut u8,
    key_size: u32,
    value_pointer: *mut u8,
    _value_size: u32, // NOTE: Used only in EDL!
) -> sgx_status_t {
    info!("✔ [App] Getting from database via OCALL...");
    let db_key = unsafe {
        slice::from_raw_parts(key_pointer, key_size as usize)
    };
    trace!("✔ [App] Database key to query: {:?}", db_key);
    let mut data = DATABASE
        .lock()
        .unwrap()
        [db_key]
        .clone();
    info!("✔ [App] Data retreived from database!");
    let data_length = data.len() as u32;
    let mut final_bytes_to_copy: Vec<u8> = data_length
        .to_le_bytes()
        .to_vec();
    info!("✔ [App] Copying data into enclave...");
    final_bytes_to_copy.append(&mut data);
    unsafe {
        copy_nonoverlapping(
            &final_bytes_to_copy[0] as *const u8,
            value_pointer,
            final_bytes_to_copy.len()
        )
    }
    sgx_status_t::SGX_SUCCESS
}


#[no_mangle]
pub extern "C"
fn save_to_db(
    key_pointer: *mut u8,
    key_size: u32,
    sealed_log_size: u32,
    scratch_pad_pointer: *mut u8,
) -> sgx_status_t {
    let data_from_scratch_pad = unsafe {
        slice::from_raw_parts(scratch_pad_pointer, sealed_log_size as usize)
    };
    info!("✔ [App] Saving sealed data into database...");
    let db_key = unsafe {
        slice::from_raw_parts(key_pointer, key_size as usize)
    };
    trace!("✔ [App] Database key: {:?}", db_key);
    trace!("✔ [App] Sealed log size: {:?}", sealed_log_size);
    DATABASE
        .lock()
        .unwrap()
        .insert(
            db_key.to_vec(),
            data_from_scratch_pad.to_vec(),
        );
    info!("✔ [App] Sealed data saved to database successfully!");
    sgx_status_t::SGX_SUCCESS
}

fn main() {
    env_logger::init();
    let mut scratch_pad: Vec<u8> = vec![0; SCRATCH_PAD_SIZE];
    let scratch_pad_pointer: *mut u8 = &mut scratch_pad[0];
    let result = unsafe {
        run_sample(
            ENCLAVE.geteid(),
            &mut sgx_status_t::SGX_SUCCESS,
            scratch_pad_pointer,
            scratch_pad.len() as *const u8,
        )
    };
    match result {
        sgx_status_t::SGX_SUCCESS => {
            info!("✔ [App] Sample run successfully!");
        }
        _ => {
            info!("✘ [App] ECALL Failed: {}", result);
            return;
        }
    };
    //ENCLAVE.destroy(); // FIXME: CAN'T USE THIS :S
}
