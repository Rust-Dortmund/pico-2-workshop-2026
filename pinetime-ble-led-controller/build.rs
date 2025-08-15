//! This build script copies the `memory.x` file from the crate root into a directory where
//! the linker can always find it at build time.

use std::{env, fs, path::PathBuf};

fn main() {
    // Put memory layout in the output directory and ensure it's on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    fs::copy("memory.x", out.join("memory.x")).expect("memory.x should copy successfully");

    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=memory.x");
}
