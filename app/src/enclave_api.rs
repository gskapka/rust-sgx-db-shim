use sgx_types::{
    sgx_status_t,
    sgx_enclave_id_t,
};

extern {
    pub fn run_sample(
        eid: sgx_enclave_id_t,
        retval: *mut sgx_status_t,
        scratch_pad_pointer: *mut u8,
        scratch_pad_size: *const u8,
    ) -> sgx_status_t;
}
