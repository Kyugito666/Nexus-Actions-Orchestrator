// cpp/crypto.h - Libsodium encryption interface

#ifndef NEXUS_CRYPTO_H
#define NEXUS_CRYPTO_H

#include <sodium.h>
#include <string>
#include <vector>
#include <cstdint>

extern "C" {
    // Initialize libsodium (call once at startup)
    int crypto_init();
    
    // Encrypt secret for GitHub Secrets API
    // Returns base64-encoded encrypted data
    // public_key: base64-encoded public key from GitHub
    // plaintext: secret value to encrypt
    // output: buffer to store encrypted data (must be pre-allocated)
    // output_len: pointer to size of output buffer
    int encrypt_secret(
        const char* public_key,
        const char* plaintext,
        char* output,
        size_t* output_len
    );
    
    // Free allocated memory
    void crypto_free(char* ptr);
}

#endif // NEXUS_CRYPTO_H
