#[macro_use]
extern crate lazy_static;
extern crate sgx_types;
extern crate sgx_urts;
extern crate dirs;

pub mod db;
pub mod types;
pub mod constants;
pub mod enclave_api;
pub mod scratch_pad;
pub mod init_enclave;

use sgx_types::*;
use db::DATABASE;
use init_enclave::ENCLAVE;
use enclave_api::run_sample;
use scratch_pad::SCRATCH_PAD;
use scratch_pad::get_scratch_pad_pointer;
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
    value_size: u32,
) -> sgx_status_t {
    println!("✔ Getting from database via OCALL!");
    let db_key = unsafe {
        slice::from_raw_parts(key_pointer, key_size as usize)
    };
    println!("✔ Outside enclave db_key: {:?}", db_key);
    let mut data: Vec<u8> = vec![6,6,6];
    let data_length = data.len() as u32;
    let mut data_length_bytes: Vec<u8> = data_length
        .to_le_bytes()
        .to_vec();
    println!("✔ Value pointer:  {:?}", value_pointer);
    println!("✔ Data length:  {:?}", data_length);
    println!("✔ Data length bytes:  {:?}", data_length);
    data_length_bytes.append(&mut data);
    unsafe {
        copy_nonoverlapping(
            &data_length_bytes[0],
            value_pointer,
            data_length_bytes.len()
        )
    }
    sgx_status_t::SGX_SUCCESS
}


#[no_mangle]
pub extern "C"
fn seal_into_db(
    key_pointer: *mut u8,
    key_size: u32,
    value_pointer: *mut u8,
    value_size: u32,
) -> sgx_status_t {
    println!("✔ Sealing into DB via OCALL!");
    /*
    let db_key = unsafe {
        slice::from_raw_parts(key_pointer, key_size as usize)
    };
    println!("✔ Outside enclave db_key: {:?}", db_key);
    let mut data: Vec<u8> = vec![6,6,6];
    let data_length = data.len() as u32;
    let mut data_length_bytes: Vec<u8> = data_length
        .to_le_bytes()
        .to_vec();
    println!("✔ Value pointer:  {:?}", value_pointer);
    println!("✔ Data length:  {:?}", data_length);
    println!("✔ Data length bytes:  {:?}", data_length);
    data_length_bytes.append(&mut data);
    unsafe {
        copy_nonoverlapping(
            &data_length_bytes[0],
            value_pointer,
            data_length_bytes.len()
        )
    }
    */
    sgx_status_t::SGX_SUCCESS
}

fn main() {
    println!("Scratch pad before: {:?}", &SCRATCH_PAD.lock().unwrap()[..50]);
    let result = unsafe {
        run_sample(
            ENCLAVE.geteid(),
            &mut sgx_status_t::SGX_SUCCESS,
            get_scratch_pad_pointer(),
            SCRATCH_PAD_SIZE as *const u8,
        )
    };
    match result {
        sgx_status_t::SGX_SUCCESS => {
            println!("✔ Sample run successfully!");
            println!("Scratch pad after: {:?}", &SCRATCH_PAD.lock().unwrap()[..50]);
        }
        _ => {
            println!("✘ ECALL Failed: {}", result);
            return;
        }
    };
    //ENCLAVE.destroy(); // FIXME: CAN'T USE THIS :S
}
