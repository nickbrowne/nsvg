extern crate bindgen;
extern crate cc;

use std::env;
use std::path::PathBuf;

fn main() {
  cc::Build::new()
    .file("lib/nanosvg.c")
    .compile("nanosvg");

  let bindings = bindgen::Builder::default()
    .header("lib/nanosvg.h")
    .header("lib/nanosvgrast.h")
    .generate()
    .expect("Unable to generate bindings.");

  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

  bindings
    .write_to_file(out_path.join("bindings.rs"))
    .expect("Couldn't write bindings.");
}
