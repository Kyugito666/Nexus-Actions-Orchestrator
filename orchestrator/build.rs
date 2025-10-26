// build.rs - Improved version
use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=cpp/crypto.cpp");
    println!("cargo:rerun-if-changed=cpp/crypto.h");
    
    // Try to link libsodium
    println!("cargo:rustc-link-lib=sodium");
    
    let mut build = cc::Build::new();
    
    build
        .cpp(true)
        .file("cpp/crypto.cpp")
        .flag_if_supported("-std=c++17")
        .warnings(false);
    
    // Platform-specific flags
    if cfg!(target_os = "windows") {
        build.flag_if_supported("/EHsc");
    } else {
        build.flag_if_supported("-fPIC");
    }
    
    // Try to find libsodium
    if let Ok(include_path) = env::var("SODIUM_INCLUDE_DIR") {
        build.include(include_path);
    }
    
    build.compile("nexus_crypto");
    
    // Print helpful message
    if cfg!(target_os = "linux") {
        println!("cargo:warning=Make sure libsodium-dev is installed: sudo apt-get install libsodium-dev");
    } else if cfg!(target_os = "macos") {
        println!("cargo:warning=Make sure libsodium is installed: brew install libsodium");
    } else if cfg!(target_os = "windows") {
        println!("cargo:warning=Make sure libsodium is installed. See: https://doc.libsodium.org/installation");
    }
}
