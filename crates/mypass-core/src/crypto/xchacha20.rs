//! XChaCha20-Poly1305 AEAD 加密模块
//!
//! # 概述
//! 本模块提供基于 XChaCha20-Poly1305 的Authenticated Encryption with Associated Data (AEAD) 加密功能。
//! 每个条目使用独立的随机 Nonce 进行加密，确保相同明文产生不同的密文。
//!
//! # 安全性
//! - 使用 256-bit 密钥
//! - Nonce 长度 24 字节 (192-bit)
//! - 基于 RFC 8439 标准

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use generic_array::GenericArray;
use rand::rngs::OsRng;
use rand::RngCore;

const NONCE_LEN: usize = 24;

pub fn encrypt_aead(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let key_array = GenericArray::from_slice(key);
    let cipher = XChaCha20Poly1305::new(key_array);

    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    let mut result = nonce_bytes.to_vec();
    result.extend(ciphertext);
    Ok(result)
}

pub fn decrypt_aead(ciphertext_with_nonce: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let key_array = GenericArray::from_slice(key);
    let cipher = XChaCha20Poly1305::new(key_array);

    if ciphertext_with_nonce.len() < NONCE_LEN {
        anyhow::bail!("Ciphertext too short");
    }

    let nonce = XNonce::from_slice(&ciphertext_with_nonce[..NONCE_LEN]);
    let ciphertext = &ciphertext_with_nonce[NONCE_LEN..];

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    Ok(plaintext)
}

pub fn encrypt_aead_base64(plaintext: &[u8], key: &[u8]) -> Result<String> {
    let encrypted = encrypt_aead(plaintext, key)?;
    Ok(BASE64.encode(&encrypted))
}

pub fn decrypt_aead_base64(ciphertext_b64: &str, key: &[u8]) -> Result<Vec<u8>> {
    let ciphertext = BASE64.decode(ciphertext_b64).context("Invalid base64")?;
    decrypt_aead(&ciphertext, key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key: [u8; 32] = crate::crypto::secure_random::generate_mek();
        let plaintext = b"Hello, World!";

        let encrypted = encrypt_aead(plaintext, &key).unwrap();
        let decrypted = decrypt_aead(&encrypted, &key).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_base64_encode() {
        let key: [u8; 32] = crate::crypto::secure_random::generate_mek();
        let plaintext = b"Test data";

        let encrypted_b64 = encrypt_aead_base64(plaintext, &key).unwrap();
        let decrypted = decrypt_aead_base64(&encrypted_b64, &key).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_nonce_uniqueness() {
        let key: [u8; 32] = crate::crypto::secure_random::generate_mek();
        let plaintext = b"test";

        let mut nonces = std::collections::HashSet::new();
        for _ in 0..100 {
            let encrypted = encrypt_aead(plaintext, &key).unwrap();
            let nonce: [u8; 24] = encrypted[..24].try_into().unwrap();
            assert!(nonces.insert(nonce), "Nonce collision!");
        }
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1: [u8; 32] = crate::crypto::secure_random::generate_mek();
        let key2: [u8; 32] = crate::crypto::secure_random::generate_mek();
        let plaintext = b"secret";

        let encrypted = encrypt_aead(plaintext, &key1).unwrap();
        let result = decrypt_aead(&encrypted, &key2);
        assert!(result.is_err());
    }

    #[test]
    fn test_truncated_ciphertext_fails() {
        let key: [u8; 32] = crate::crypto::secure_random::generate_mek();
        let plaintext = b"hello";
        let encrypted = encrypt_aead(plaintext, &key).unwrap();
        let truncated = &encrypted[..encrypted.len() - 5];
        assert!(decrypt_aead(truncated, &key).is_err());
    }
}