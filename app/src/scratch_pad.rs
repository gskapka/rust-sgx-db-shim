use crate::types::Bytes;
use crate::constants::{
    MEGA_BYTE,
    SCRATCH_PAD_SIZE,
};
use std::sync::{
    Arc,
    Mutex,
};
use std::cell::{ RefCell };

lazy_static! {
    //pub static ref SCRATCH_PAD: Mutex<RefCell<Bytes>> = {
    pub static ref SCRATCH_PAD: Bytes = {
        let mut scratch_pad: Vec<u8> = vec![0; SCRATCH_PAD_SIZE];
        //Arc::new(Mutex::new(scratch_pad))
        //Rc::new(
        //Mutex::new(RefCell::new(scratch_pad))
        //Rc::new(RefCell::new(scratch_pad))
        scratch_pad
    };
}

pub fn get_scratch_pad_pointer() -> *mut u8 { // TODO Have as lazy static too?
    let mut sc: Vec<u8> = vec![0; SCRATCH_PAD_SIZE];
    let pointer = &mut sc[0] as * mut u8;
    pointer
    /*
    let pointer = unsafe {
        //let pointer =
            &mut SCRATCH_PAD[0] as * mut u8
    };
    */
}
