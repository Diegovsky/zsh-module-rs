extern crate bindgen;

use std::env;
use std::path::PathBuf;

// If your build is failing, please, take a look at config.h and change its values accordingly to
// your machine.

fn main() {
    println!("cargo:rerun-if-changed=headers/wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("headers/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
