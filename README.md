# :floppy_disk: Sealing SGX Data into an Untrusted Database in Rust Example

&nbsp;

A example of a database shim for sealing enclave data in an untrusted, external database, written in Rust.

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

__`❍ cd bin && ./app`__

```
```

&nbsp;

***

&nbsp;

### :black_nib: Notes

The above build steps are monstrously fragile. Hopefully there's enough version-specific information above plus pinned dependencies inside the example to make it less so.

&nbsp;

***

&nbsp;

### :guardsman: Tests

&nbsp;

There are no tests yet! :S

***

&nbsp;

### :black_nib: To Do:
