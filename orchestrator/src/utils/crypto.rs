// src/utils/crypto.rs - Rust wrapper for C++ crypto functions

use anyhow::{Result, Context};
use std::ffi::{CString, CStr};
use std::os::raw::c_char;

extern "C" {
    fn crypto_init() -> i32;
    fn encrypt_secret(
        public_key: *const c_char,
        plaintext: *const c_char,
        output: *mut c_char,
        output_len: *mut usize,
    ) -> i32;
    fn crypto_free(ptr: *mut c_char);
}

pub fn init_crypto() -> Result<()> {
    unsafe {
        let result = crypto_init();
        if result < 0 {
            anyhow::bail!("Failed to initialize libsodium: code {}", result);
        }
        Ok(())
    }
}

pub fn encrypt_for_github(public_key_b64: &str, secret_value: &str) -> Result<String> {
    let public_key_c = CString::new(public_key_b64)
        .context("Invalid public key string")?;
    let plaintext_c = CString::new(secret_value)
        .context("Invalid secret value string")?;
    
    const MAX_OUTPUT: usize = 8192; // 8KB should be enough
    let mut output: Vec<u8> = vec![0u8; MAX_OUTPUT];
    let mut output_len = MAX_OUTPUT;
    
    unsafe {
        let result = encrypt_secret(
            public_key_c.as_ptr(),
            plaintext_c.as_ptr(),
            output.as_mut_ptr() as *mut c_char,
            &mut output_len,
        );
        
        if result != 0 {
            anyhow::bail!("Encryption failed with code: {}", result);
        }
        
        // Convert output to Rust String
        let encrypted_str = CStr::from_bytes_until_nul(&output[..output_len])
            .context("Invalid C string from encryption")?
            .to_str()
            .context("Invalid UTF-8 in encrypted output")?
            .to_string();
        
        Ok(encrypted_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_crypto_init() {
        assert!(init_crypto().is_ok());
    }
    
    #[test]
    fn test_encrypt() {
        init_crypto().unwrap();
        
        // Example public key (base64 encoded, 32 bytes for Curve25519)
        let public_key = "hBSZF+rsRNIWNzMC2DUc6lE1R0CKT8pFqPxQH+2F6zk=";
        let secret = "test_secret_value";
        
        let result = encrypt_for_github(public_key, secret);
        assert!(result.is_ok());
        
        let encrypted = result.unwrap();
        assert!(!encrypted.is_empty());
        assert!(encrypted.len() > secret.len()); // Encrypted data is larger
    }
}
