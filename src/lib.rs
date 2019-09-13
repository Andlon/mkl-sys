/*!

# mkl-sys

Auto-generated bindings to Intel MKL. Currently only tested on Linux, and not considered stable/ready for production use.

This crate relies on Intel MKL having been installed on the target system,
and that the environment is set up for use with MKL.
It uses `pkg-config` to determine library paths. The easiest way to make it work is to run the provided
`mklvars.sh` setup script that is bundled with MKL.

## Known issues
- `bindgen` does not seem to be able to properly handle many preprocessor macros, such as e.g. `dss_create`.
This appears to be related to [this issue](https://github.com/rust-lang/rust-bindgen/issues/753).
- Generating bindings for the entire MKL library takes a lot of time. This is a significant issue for debug
builds, as we currently have no way of forcing optimizations for bindgen when dependent projects are
built without optimizations. To circumvent this, you should use features to enable binding generation
only for the parts of the library that you will need. For example, the `dss` feature generates bindings for the
Direct Sparse Solver (DSS) interface.

The API exposed by this crate should be considered unstable until these issues have been resolved.

*/

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

/// The integer type used by the underlying MKL library.
///
/// TODO: Update this to account for use of i64 integers once we support this.
pub type MklInt = i32;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));