use crate::types::Bytes;
use crate::constants::MEGA_BYTE;
use std::sync::Mutex;

lazy_static! {
    pub static ref SCRATCH_PAD: Mutex<Bytes> = {
        let scratch_pad_size = 1 * MEGA_BYTE;
        let mut scratch_pad: Vec<u8> = vec![0; scratch_pad_size];
        Mutex::new(scratch_pad)
    };
}

pub fn get_scratch_pad_pointer() -> *mut u8 { // TODO Wrap in Result!
    &mut SCRATCH_PAD
        .lock()
        .unwrap()[0]
}
