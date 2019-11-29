# mkl-sys

Auto-generated bindings to Intel MKL. Currently only tested on Linux, and not considered stable/ready for production use.

This crate relies on Intel MKL having been installed on the target system,
and that the environment is set up for use with MKL.
It uses `pkg-config` to determine library paths. The easiest way to make it work is to run the provided
`mklvars.sh` setup script that is bundled with MKL.

## Windows support

To run `bindgen` a Clang installation is required. According to the `bindgen` [documentation](https://rust-lang.github.io/rust-bindgen/requirements.html#clang) version 3.9 should suffice. A recent pre-built version of Clang can be downloaded on the [LLVM release page](https://releases.llvm.org/download.html). To build `mkl-sys`, the following environment variables have to be present:
1. Clang requires the MSVC standard library on Windows. Therefore, the build process should be started from a
Visual Studio or Build Tools command prompt. The command prompt can be started from a start menu shortcut created by
Visual Studio or by running a `vcvars` script (e.g. `C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Auxiliary\Build\vcvars64.bat`) in an open command prompt. An IDE such as Clion with a configured MSVC toolchain should also work.
2. The environment variable `MKLROOT` has to be set properly to the path containing the `bin`, `lib`, `include`, etc. folders
of MKL (e.g. `C:\Program Files (x86)\IntelSWTools\compilers_and_libraries_2019.5.281\windows\mkl`). This can also be done by
running the `mklvars.bat` script in the `bin` folder of MKL.
3. The environment variable `LIBCLANG_PATH` used by `bindgen` has to be set to point to the `bin` folder of the Clang installation.
Note, that during runtime the corresponding DLLs (e.g. located in `C:\Program Files (x86)\IntelSWTools\compilers_and_libraries_2019.5.281\windows\redist\intel64_win\mkl`) have to be in `PATH`.

## Known issues
- `bindgen` does not seem to be able to properly handle many preprocessor macros, such as e.g. `dss_create`.
This appears to be related to [this issue](https://github.com/rust-lang/rust-bindgen/issues/753).
- Generating bindings for the entire MKL library takes a lot of time. This is a significant issue for debug
builds, as we currently have no way of forcing optimizations for `bindgen` when dependent projects are
built without optimizations. To circumvent this, you should use features to enable binding generation
only for the parts of the library that you will need. For example, the `dss` feature generates bindings for the
Direct Sparse Solver (DSS) interface.

The API exposed by this crate should be considered unstable until these issues have been resolved.

## License
Intel MKL is provided by Intel and licensed separately.

This crate is licensed under the MIT license. See `LICENSE` for details.

