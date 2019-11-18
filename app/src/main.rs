#[macro_use]
extern crate lazy_static;
extern crate sgx_types;
extern crate sgx_urts;
extern crate dirs;
use sgx_types::*;
use sgx_urts::SgxEnclave;

use std::{
    fs,
    path,
    slice,
    ptr::copy_nonoverlapping,
    io::{
        Read,
        Write
    },
};

static ENCLAVE_FILE: &'static str = "enclave.signed.so";
static ENCLAVE_TOKEN: &'static str = "enclave.token";

lazy_static! { // NOTE Gives us enc. as global but now can't destroy it!
    pub static ref ENCLAVE: SgxEnclave = {
        let mut launch_token: sgx_launch_token_t = [0; 1024];
        let mut launch_token_updated: i32 = 0;
        let mut home_dir = path::PathBuf::new();
        let use_token = match dirs::home_dir() {
            Some(path) => {
                println!("[+] Home dir is {}", path.display());
                home_dir = path;
                true
            },
            None => {
                println!("[-] Cannot get home dir");
                false
            }
        };
        let token_file: path::PathBuf = home_dir.join(ENCLAVE_TOKEN);;
        if use_token == true {
            match fs::File::open(&token_file) {
                Err(_) => {
                    println!(
                        "[-] Open token file {} error! Will create one.",
                        token_file.as_path().to_str().unwrap()
                        );
                },
                Ok(mut f) => {
                    println!("[+] Open token file success! ");
                    match f.read(&mut launch_token) {
                        Ok(1024) => {
                            println!("[+] Token file valid!");
                        },
                        _ => println!("[+] Token file invalid, will create new token file"),
                    }
                }
            }
        }
        let debug = 1;
        let mut misc_attr = sgx_misc_attribute_t {secs_attr: sgx_attributes_t { flags:0, xfrm:0}, misc_select:0};
        let enclave = match SgxEnclave::create(
            ENCLAVE_FILE,
            debug,
            &mut launch_token,
            &mut launch_token_updated,
            &mut misc_attr
        ) {
            Ok(enc) => enc,
            Err(e) => panic!("[-] Failed to create enclave: {}", e),
        };
        if use_token == true && launch_token_updated != 0 {
            match fs::File::create(&token_file) {
                Ok(mut f) => {
                    match f.write_all(&launch_token) {
                        Ok(()) => println!("[+] Saved updated launch token!"),
                        Err(_) => println!("[-] Failed to save updated launch token!"),
                    }
                },
                Err(_) => {
                    println!("[-] Failed to save updated enclave token, but doesn't matter");
                },
            }
        }
        enclave
    };
}

extern {
    pub fn run(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
    ) -> sgx_status_t;
}

#[no_mangle]
pub extern "C"
fn get_from_db(
    key_pointer: *mut u8,
    key_size: u32,
    value_pointer: *mut u8,
    value_size: u32,
) -> sgx_status_t {
    println!("✔ Doing thing via OCALL!");
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

fn main() {
    let result = unsafe {
        run(
            ENCLAVE.geteid(),
            &mut sgx_status_t::SGX_SUCCESS,
        )
    };
    match result {
        sgx_status_t::SGX_SUCCESS => println!("✔ `Run` fxn success!!"),
        _ => {
            println!("✘ ECALL Failed: {}", result);
            return;
        }
    };
    //ENCLAVE.destroy();
}
