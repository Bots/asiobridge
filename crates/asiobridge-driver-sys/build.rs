//! Build script for ASIO driver FFI bindings.

use std::env;
use std::path::PathBuf;

fn main() {
  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

  cc::Build::new()
    .cpp(true)
    .flag_if_supported("/std:c++17")
    .flag_if_supported("/W4")
    .file("src/asio_wrapper.cpp")
    .out_dir(&out_dir)
    .compile("asiobridge_asio_wrapper");

  let target = env::var("TARGET").unwrap_or_default();
  if target.contains("windows") {
    let bindings = bindgen::Builder::default()
      .header("src/asio_wrapper.h")
      .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
      .generate()
      .expect("Unable to generate bindings");

    bindings
      .write_to_file(out_dir.join("bindings.rs"))
      .expect("Unable to write bindings");
  }

  println!("cargo::rerun-if-changed=build.rs");
  println!("cargo::rerun-if-changed=src/asio_wrapper.cpp");
  println!("cargo::rerun-if-changed=src/asio_wrapper.h");
}
