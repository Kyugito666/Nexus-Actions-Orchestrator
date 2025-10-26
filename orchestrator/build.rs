// build.rs - C++ compilation for libsodium crypto

use std::env;
use std::path::PathBuf;

fn main() {
    let mut build = cc::Build::new();
    
    build
        .cpp(true)
        .file("cpp/crypto.cpp")
        .flag("-std=c++17")
        .flag("-lsodium");
    
    // Platform-specific flags
    if cfg!(target_os = "windows") {
        build.flag("/EHsc");
    } else {
        build.flag("-fPIC");
    }
    
    build.compile("nexus_crypto");
    
    println!("cargo:rerun-if-changed=cpp/crypto.cpp");
    println!("cargo:rerun-if-changed=cpp/crypto.h");
    println!("cargo:rustc-link-lib=sodium");
}
