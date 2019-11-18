use std::sync::Mutex;
use crate::types::Bytes;
use crate::constants::{
    MEGA_BYTE,
    SCRATCH_PAD_SIZE,
};

lazy_static! {
    pub static ref SCRATCH_PAD: Mutex<Bytes> = {
        let mut scratch_pad: Vec<u8> = vec![0; SCRATCH_PAD_SIZE];
        Mutex::new(scratch_pad)
    };
}

pub fn get_scratch_pad_pointer() -> *mut u8 { // TODO Wrap in Result!
    &mut SCRATCH_PAD
        .lock()
        .unwrap()[0]
}
