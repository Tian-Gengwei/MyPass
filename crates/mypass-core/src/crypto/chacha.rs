//! ChaCha20 流加密模块
//!
//! # 概述
//! 本模块提供 ChaCha20 流加密功能，用于加密操作日志等敏感数据。
//! 使用 XChaCha20-Poly1305 构造（与 xchacha20 模块相同的底层实现）。
//!
//! # 用途
//! - 日志流加密
//! - 操作历史记录保护

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use anyhow::Result;
use generic_array::GenericArray;

pub struct LogCipher {
    cipher: XChaCha20Poly1305,
    nonce: XNonce,
}

impl LogCipher {
    pub fn new(key: &[u8], nonce: &[u8]) -> Result<Self> {
        let key_array = GenericArray::from_slice(key);
        let nonce_array = GenericArray::from_slice(nonce);

        Ok(Self {
            cipher: XChaCha20Poly1305::new(&key_array),
            nonce: *nonce_array,
        })
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        self.cipher
            .encrypt(&self.nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("Log encryption failed: {}", e))
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        self.cipher
            .decrypt(&self.nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Log decryption failed: {}", e))
    }
}

pub fn derive_log_key(mek: &[u8], purpose: &[u8]) -> Vec<u8> {
    let info = [b"log-cipher-", purpose].concat();
    crate::crypto::hkdf_helper::derive_subkey(mek, &info, 32)
        .expect("HKDF derivation should not fail")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_cipher() {
        let key: [u8; 32] = crate::crypto::secure_random::generate_mek();
        let nonce: [u8; 24] = {
            let mut n = [0u8; 24];
            crate::crypto::secure_random::fill_random(&mut n);
            n
        };
        let log = LogCipher::new(&key, &nonce).unwrap();

        let plaintext = b"Test log entry";
        let encrypted = log.encrypt(plaintext).unwrap();
        let decrypted = log.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_log_cipher_authentication() {
        let key: [u8; 32] = crate::crypto::secure_random::generate_mek();
        let nonce: [u8; 24] = {
            let mut n = [0u8; 24];
            crate::crypto::secure_random::fill_random(&mut n);
            n
        };
        let log = LogCipher::new(&key, &nonce).unwrap();

        let encrypted = log.encrypt(b"secret").unwrap();
        let mut tampered = encrypted.clone();
        let len = tampered.len();
        tampered[len - 1] ^= 0x01;

        assert!(log.decrypt(&tampered).is_err());
    }
}