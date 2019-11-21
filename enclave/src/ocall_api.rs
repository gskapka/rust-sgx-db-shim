use sgx_types::sgx_status_t;

#[no_mangle]
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
