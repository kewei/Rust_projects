extern crate bindgen;

use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let path = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("path: {:?}", Path::new(&path));

    // println!("cargo:rustc-link-search=/usr/lib/x86_64-linux-gnu");
    println!("cargo:rustc-link-lib=usb-1.0");

    println!("cargo:rerun-if-changed=wrapper.h");
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let output_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(output_path.join("bindings.rs"))
        .expect("Could not write bindings");
}