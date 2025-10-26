// cpp/crypto.cpp - Libsodium encryption implementation

#include "crypto.h"
#include <sodium.h>
#include <cstring>
#include <memory>

extern "C" {

int crypto_init() {
    return sodium_init();
}

int encrypt_secret(
    const char* public_key_b64,
    const char* plaintext,
    char* output,
    size_t* output_len
) {
    if (!public_key_b64 || !plaintext || !output || !output_len) {
        return -1;
    }
    
    // Decode base64 public key
    size_t public_key_len = strlen(public_key_b64);
    std::vector<unsigned char> public_key(crypto_box_PUBLICKEYBYTES);
    
    size_t decoded_len;
    if (sodium_base642bin(
        public_key.data(),
        public_key.size(),
        public_key_b64,
        public_key_len,
        nullptr,
        &decoded_len,
        nullptr,
        sodium_base64_VARIANT_ORIGINAL
    ) != 0) {
        return -2; // Base64 decode failed
    }
    
    // Encrypt using sealed box
    size_t plaintext_len = strlen(plaintext);
    size_t ciphertext_len = crypto_box_SEALBYTES + plaintext_len;
    std::vector<unsigned char> ciphertext(ciphertext_len);
    
    if (crypto_box_seal(
        ciphertext.data(),
        reinterpret_cast<const unsigned char*>(plaintext),
        plaintext_len,
        public_key.data()
    ) != 0) {
        return -3; // Encryption failed
    }
    
    // Encode to base64
    size_t b64_len = sodium_base64_ENCODED_LEN(ciphertext_len, sodium_base64_VARIANT_ORIGINAL);
    
    if (b64_len > *output_len) {
        *output_len = b64_len;
        return -4; // Output buffer too small
    }
    
    char* b64_output = sodium_bin2base64(
        output,
        *output_len,
        ciphertext.data(),
        ciphertext_len,
        sodium_base64_VARIANT_ORIGINAL
    );
    
    if (!b64_output) {
        return -5; // Base64 encode failed
    }
    
    *output_len = strlen(output);
    return 0; // Success
}

void crypto_free(char* ptr) {
    if (ptr) {
        sodium_free(ptr);
    }
}

} // extern "C"
