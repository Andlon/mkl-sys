use std::env;
use std::path::PathBuf;
use bindgen::callbacks::{ParseCallbacks, IntKind};

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
    if cfg!(not(any(feature = "all", feature = "dss", feature = "sparse-matrix-checker"))) {
        panic!(
"No MKL modules selected.
To use this library, please select the features corresponding \
to MKL modules that you would like to use, or enable the `all` feature if you would \
like to generate symbols for all modules.");
    }

    let name = build_config_name();
    let library = pkg_config::probe_library(&name).unwrap();

    // Use information obtained from pkg-config to setup args for clang used by bindgen.
    // Otherwise we don't get e.g. the correct MKL preprocessor definitions).
    let clang_args = {
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
    };

    #[allow(unused_mut)]
    let mut builder = bindgen::Builder::default()
        .parse_callbacks(Box::new(Callbacks))
        .clang_args(clang_args);

    if cfg!(feature = "all") {
        builder = builder.header("wrapper_all.h");
    }

    // If only part of MKL is needed, we use features to construct whitelists of
    // the needed functionality. These can be overridden with the "all" feature, which
    // avoids whitelisting and instead encompasses everything.
    #[cfg(not(feature="all"))]
    {
        #[cfg(feature="dss")]
        {
            let dss_regex = "(.*dss.*)|(.*DSS.*)";
            builder = builder.whitelist_function(dss_regex)
                .whitelist_type(dss_regex)
                .whitelist_var(dss_regex)
                .header("wrapper_dss.h");
        }

        #[cfg(feature="sparse-matrix-checker")]
        {
            builder = builder.whitelist_function("sparse_matrix_checker*")
                .whitelist_function("sparse_matrix_checker_init*")
                .header("wrapper_sparse_matrix_checker.h")
        }
    }

    let bindings = builder
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}