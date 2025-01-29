use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=include/kyber/api.h");
    println!("cargo:rerun-if-changed=include/kyber/params.h");
    println!("cargo:rerun-if-changed=include/sphincs/api.h");
    println!("cargo:rerun-if-changed=include/sphincs/params.h");

    // Link against native libraries
    println!("cargo:rustc-link-search=native=libs");
    println!("cargo:rustc-link-lib=dylib=pqcrystals_kyber768_ref");
    println!("cargo:rustc-link-lib=dylib=sphincsshake128f");

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-Iinclude")
        .clang_arg("-DPARAMS=sphincs-sha-256-128f")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
