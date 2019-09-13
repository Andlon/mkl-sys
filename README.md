# mkl-sys

Auto-generated bindings to Intel MKL. Currently only tested on Linux, and not considered stable/ready for production use.

This crate relies on Intel MKL having been installed on the target system,
and that the environment is set up for use with MKL.
It uses `pkg-config` to determine library paths. The easiest way to make it work is to run the provided
`mklvars.sh` setup script that is bundled with MKL.

## Known issues
`bindgen` does not seem to be able to properly handle many preprocessor macros, such as e.g. `dss_create`.
This appears to be related to [this issue](https://github.com/rust-lang/rust-bindgen/issues/753).

The API exposed by this crate should be considered unstable until these issues have been resolved.

## License
Intel MKL is provided by Intel and licensed separately.

This crate is licensed under the MIT license. See `LICENSE` for details.

