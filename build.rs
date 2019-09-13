use std::env;
use std::path::PathBuf;

#[cfg(feature="all")]
const WRAP_ALL: bool = true;

// TODO: There must be a better way than enumerating all features?
#[cfg(not(any(feature = "dss")))]
const WRAP_ALL: bool = true;

#[cfg(any(feature = "dss"))]
const WRAP_ALL: bool = false;

fn main() {
    pkg_config::probe_library("mkl-dynamic-lp64-seq").unwrap();

    #[allow(unused_mut)]
    let mut builder = bindgen::Builder::default();

    if WRAP_ALL {
        builder = builder.header("wrapper_all.h");
    }

    // If only part of MKL is needed, we use features to construct whitelists of
    // the needed functionality. These can be overridden with the "all" feature, which
    // avoids whitelisting and instead encompasses everything.
    #[cfg(not(feature="all"))]
    {
        #[cfg(feature="dss")]
        {
            builder = builder.whitelist_function("dss_.*")
                .whitelist_function("DSS_.*")
                .header("wrapper_dss.h");
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