use std::{
    fs,
    path,
    io::{
        Read,
        Write
    },
};
use sgx_types::*;
use sgx_urts::SgxEnclave;

static ENCLAVE_TOKEN: &'static str = "enclave.token";
static ENCLAVE_FILE: &'static str = "enclave.signed.so";

lazy_static! { // NOTE Gives us enc. as global but now can't destroy it!
    pub static ref ENCLAVE: SgxEnclave = {
        let mut launch_token: sgx_launch_token_t = [0; 1024];
        let mut launch_token_updated: i32 = 0;
        let mut home_dir = path::PathBuf::new();
        let use_token = match dirs::home_dir() {
            Some(path) => {
                println!("✔ [App] Home dir is {}", path.display());
                home_dir = path;
                true
            },
            None => {
                println!("✘ [App] Cannot get home dir");
                false
            }
        };
        let token_file: path::PathBuf = home_dir.join(ENCLAVE_TOKEN);;
        if use_token == true {
            match fs::File::open(&token_file) {
                Err(_) => {
                    println!(
                        "✘ [App] Open token file {} error! Will create one.",
                        token_file.as_path().to_str().unwrap()
                        );
                },
                Ok(mut f) => {
                    println!("✔ Open token file success! ");
                    match f.read(&mut launch_token) {
                        Ok(1024) => {
                            println!("✔ [App] Token file valid!");
                        },
                        _ => println!("✔[App] Token file invalid, will create new token file"),
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
