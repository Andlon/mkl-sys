use std::env;
use std::path::PathBuf;
use bindgen::callbacks::{ParseCallbacks, IntKind};

#[cfg(feature="all")]
const WRAP_ALL: bool = true;

// TODO: There must be a better way than enumerating all features?
#[cfg(not(any(feature = "dss")))]
const WRAP_ALL: bool = true;

#[cfg(any(feature = "dss"))]
const WRAP_ALL: bool = false;

#[derive(Debug)]
pub struct Callbacks;

impl ParseCallbacks for Callbacks {
    fn int_macro(&self, name: &str, _value: i64) -> Option<IntKind> {
        // This forces all MKL constants to be signed. Otherwise `bindgen` might
        // give different types to different constants, which is inconvenient.
        // MKL expects these constants to be compatible with MKL_INT.
        if &name[..4] == "MKL_" {
            // TODO: This should be the same as MKL_INT, so need to take care to
            // reflect that.
            Some(IntKind::I32)
        } else {
            None
        }
    }
}

fn main() {
    pkg_config::probe_library("mkl-dynamic-lp64-seq").unwrap();

    #[allow(unused_mut)]
    let mut builder = bindgen::Builder::default()
        .parse_callbacks(Box::new(Callbacks));

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
            let dss_regex = "(.*dss.*)|(.*DSS.*)";
            builder = builder.whitelist_function(dss_regex)
                .whitelist_type(dss_regex)
                .whitelist_var(dss_regex)
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