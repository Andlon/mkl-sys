use bindgen::callbacks::{IntKind, ParseCallbacks};
use bindgen::EnumVariation;
use std::env;
use std::path::PathBuf;

/// Build the name used for pkg-config library resolution, e.g. "mkl-dynamic-lp64-seq".
fn build_config_name() -> String {
    let parallelism = if cfg!(feature = "openmp") {
        "iomp"
    } else {
        "seq"
    };

    let integer_config = if cfg!(feature = "ilp64") {
        "ilp64"
    } else {
        "lp64"
    };

    format!("mkl-dynamic-{}-{}", integer_config, parallelism)
}

fn lib_dirs_windows(mklroot: &str) -> Vec<String> {
    let exec_prefix = PathBuf::from(mklroot);
    let libdir: PathBuf = [exec_prefix.clone(), PathBuf::from(r"lib/intel64_win")]
        .iter()
        .collect();
    let omplibdir: PathBuf = [
        exec_prefix.clone(),
        PathBuf::from(r"../compiler/lib/intel64_win"),
    ]
    .iter()
    .collect();

    let mut lib_dirs = vec![libdir];

    if cfg!(feature = "openmp") {
        lib_dirs.push(omplibdir);
    }

    lib_dirs
        .into_iter()
        .map(|p| p.to_str().unwrap().into())
        .collect()
}

fn libs_windows() -> Vec<String> {
    let libs_base = vec!["mkl_core_dll.lib"];
    let libs_seq = vec!["mkl_sequential_dll.lib"];
    let libs_omp = vec!["mkl_intel_thread_dll.lib", "libiomp5md.lib"];
    let libs_lp64 = vec!["mkl_intel_lp64_dll.lib"];
    let libs_ilp64 = vec!["mkl_intel_ilp64_dll.lib"];

    let mut libs = libs_base;

    if cfg!(feature = "openmp") {
        libs.extend(libs_omp);
    } else {
        libs.extend(libs_seq);
    };

    if cfg!(feature = "ilp64") {
        libs.extend(libs_ilp64)
    } else {
        libs.extend(libs_lp64)
    };

    libs.into_iter().map(|s| s.into()).collect()
}

fn cflags_windows(mklroot: &str) -> Vec<String> {
    let mut cflags = Vec::new();

    let prefix = PathBuf::from(mklroot);
    let includedir: PathBuf = [prefix.clone(), PathBuf::from(r"include")]
        .iter()
        .collect();

    if cfg!(feature = "ilp64") {
        cflags.push("-DMKL_ILP64".into());
    }

    cflags.push("--include-directory".into());
    cflags.push(format!("{}", includedir.to_str().unwrap()));
    cflags
}

#[derive(Debug)]
pub struct Callbacks;

impl ParseCallbacks for Callbacks {
    fn int_macro(&self, name: &str, _value: i64) -> Option<IntKind> {
        // This forces all MKL constants to be signed. Otherwise `bindgen` might
        // give different types to different constants, which is inconvenient.
        // MKL expects these constants to be compatible with MKL_INT.
        if &name[..4] == "MKL_" {
            // Important: this should be the same as MKL_INT
            if cfg!(feature = "ilp64") {
                Some(IntKind::I64)
            } else {
                Some(IntKind::I32)
            }
        } else {
            None
        }
    }
}

fn main() {
    if cfg!(not(any(
        feature = "all",
        feature = "dss",
        feature = "sparse-matrix-checker",
        feature = "extended-eigensolver",
        feature = "inspector-executor"
    ))) {
        panic!(
            "No MKL modules selected.
To use this library, please select the features corresponding \
to MKL modules that you would like to use, or enable the `all` feature if you would \
like to generate symbols for all modules."
        );
    }

    // Use information obtained from pkg-config to setup args for clang used by bindgen.
    // Otherwise we don't get e.g. the correct MKL preprocessor definitions).
    let clang_args = {
        let name = build_config_name();

        match pkg_config::probe_library(&name) {
            Ok(library) => {
                let mut args = Vec::new();
                for (key, val) in library.defines {
                    if let Some(value) = val {
                        args.push(format!("-D{}={}", key, value));
                    } else {
                        args.push(format!("-D{}", key));
                    }
                }
                for path in library.include_paths {
                    args.push(format!("-I{}", path.display()));
                }
                args
            }
            Err(_) => {
                let mklroot = match env::var("MKLROOT") {
                    Ok(mklroot) => mklroot,
                    Err(_) => panic!("Environment variable 'MKLROOT' does not exist.")
                };

                for lib_dir in lib_dirs_windows(&mklroot) {
                    println!("cargo:rustc-link-search=native=\"{}\"", lib_dir);
                }

                for lib in libs_windows() {
                    println!("cargo:rustc-link-lib={}", lib);
                }

                let args = cflags_windows(&mklroot);
                args
            }
        }
    };

    #[allow(unused_mut)]
    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(Callbacks))
        .default_enum_style(EnumVariation::ModuleConsts)
        .clang_args(clang_args);

    // If only part of MKL is needed, we use features to construct whitelists of
    // the needed functionality. These can be overridden with the "all" feature, which
    // avoids whitelisting and instead encompasses everything.
    #[cfg(not(feature = "all"))]
    {
        #[cfg(feature = "dss")]
        {
            let dss_regex = "(dss_.*)|(DSS_.*)|(MKL_DSS.*)";
            builder = builder
                .whitelist_function(dss_regex)
                .whitelist_type(dss_regex)
                .whitelist_var(dss_regex);
        }

        #[cfg(feature = "sparse-matrix-checker")]
        {
            builder = builder
                .whitelist_function("sparse_matrix_checker*")
                .whitelist_function("sparse_matrix_checker_init*");
        }

        #[cfg(feature = "extended-eigensolver")]
        {
            builder = builder
                .whitelist_function(".*feast.*")
                .whitelist_function("mkl_sparse_ee_init")
                .whitelist_function("mkl_sparse_._svd")
                .whitelist_function("mkl_sparse_._ev")
                .whitelist_function("mkl_sparse_._gv");
        }

        #[cfg(feature = "inspector-executor")]
        {
            builder = builder.whitelist_function("mkl_sparse_.*");
        }
    }

    let bindings = builder.generate().expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
