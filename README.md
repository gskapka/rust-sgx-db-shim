# :floppy_disk: Sealing SGX Data into an Untrusted Database in Rust Example

&nbsp;

A example of a database shim for sealing enclave data in an untrusted, external database. Written in Rust.

***

&nbsp;

### :wrench: Build It:

Prerequisites:

 - An SGX capable machine.
 - Intel SGX SDK 2.5 for Linux installed
 - Ubuntu 18.04
 - Docker.

Relies on: __`rust-sgx-sdk v1.0.9`__!

1) Clone the Rust SGX SDK:

__`❍ git clone https://github.com/apache/mesatee-sgx.git --branch v1.0.9`__

2) Clone this repo:

__`❍ git clone https://github.com/gskapka/rust-sgx-db-shim.git ./mesatee-sgx/samplecode/`__

3) Pull the correct SGX docker image:

__`❍ docker pull baiduxlab/sgx-rust-stable:1804-1.0.9`__

4) Start docker container pointing it to the SDK on your machine:

__`❍ docker run -v /your/path/to/rust-sgx:/root/sgx -ti --device /dev/isgx baiduxlab/sgx-rust`__

5) Start the AESM service inside docker:

__`❍ LD_LIBRARY_PATH=/opt/intel/libsgx-enclave-common/aesm /opt/intel/libsgx-enclave-common/aesm/aesm_service &`__

6) Enter EOS sample dir:

__`❍ cd sgx/samplecode/rust-sgx-db-shim`__

7) Build!

__`❍ make`__

&nbsp;

***

### :point_right: Run it:

7) After the above build, simply:

__`❍ cd bin && RUST_LOG=trace ./app`__

```

[2019-11-21T12:21:02Z INFO  sealed_db_enc] ✔ [Enc] Running example inside enclave!
[2019-11-21T12:21:02Z INFO  sealed_db_enc] ✔ [Enc] Creating data to save to db...
[2019-11-21T12:21:02Z INFO  sealed_db_enc] ✔ [Enc] Sealing data...
[2019-11-21T12:21:02Z INFO  sealed_db_enc] ✔ [Enc] Key to seal: [6, 6, 6]
[2019-11-21T12:21:02Z INFO  sealed_db_enc] ✔ [Enc] Value to seal: [1, 3, 3, 7]
[2019-11-21T12:21:02Z INFO  sealed_db_enc] ✔ [Enc] Sealed data written into app's scratch-pad!
[2019-11-21T12:21:02Z INFO  sealed_db_enc] ✔ [Enc] Sending db key & sealed data size via OCALL...
[2019-11-21T12:21:02Z INFO  sealed_db_app] ✔ [App] Saving sealed data into database...
[2019-11-21T12:21:02Z INFO  sealed_db_app] ✔ [App] Sealed data saved to database successfully!
[2019-11-21T12:21:02Z INFO  sealed_db_enc] ✔ [Enc] Getting item from external db...
[2019-11-21T12:21:02Z INFO  sealed_db_app] ✔ [App] Getting from database via OCALL...
[2019-11-21T12:21:02Z INFO  sealed_db_app] ✔ [App] Data retreived from database!
[2019-11-21T12:21:02Z INFO  sealed_db_app] ✔ [App] Copying data into enclave...
[2019-11-21T12:21:02Z INFO  sealed_db_enc] ✔ [Enc] External data written to enclave's scratch pad!
[2019-11-21T12:21:02Z INFO  sealed_db_enc] ✔ [Enc] Final unsealed key: [6, 6, 6]
[2019-11-21T12:21:02Z INFO  sealed_db_enc] ✔ [Enc] Final unsealed value: [1, 3, 3, 7]
[2019-11-21T12:21:02Z INFO  sealed_db_app] ✔ [App] Sample run successfully!

```

&nbsp;

***

&nbsp;

### :black_nib: Notes

- The sample has tracing to see what's going on. Run with __`RUST_LOG=trace`__ to see the traces.

 - The above build steps are monstrously fragile. Hopefully there's enough version-specific information above plus pinned dependencies inside the example to make it less so.

 - Note also that the method employed in this demonstration involves using a scratch-area of allocated memory that's larger than the size of data you expect to save/retrieve. This scratch space is needed both inside the enclave and externally in the app itself.

 - A second method could be employed to remove the above restriction, at the cost of each database call requiring two enclave boundary crossings rather than one in order to first pass in or out the size of the data in question.

&nbsp;

***

&nbsp;

### :guardsman: Tests

&nbsp;

There are no tests yet! :S

***

&nbsp;

### :black_nib: To Do:
