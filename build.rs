#[cfg(feature = "bindgen")]
extern crate bindgen;

extern crate cc;

#[cfg(feature = "bindgen")]
fn generate_bindings() {
  use std::env;
  use std::path::PathBuf;

  let bindings = bindgen::Builder::default()
    .header("lib/nanosvg.h")
    .header("lib/nanosvgrast.h")
    .generate()
    .expect("Unable to generate bindings.");

  let out_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

  bindings
    .write_to_file(out_path.join("src").join("bindings.rs"))
    .expect("Couldn't write bindings.");
}

#[cfg(not(feature = "bindgen"))]
fn generate_bindings() {}

fn main() {
  cc::Build::new()
    .file("lib/nanosvg.c")
    .warnings(false)
    .compile("nanosvg");

  generate_bindings();
}
