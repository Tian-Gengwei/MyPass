//! 加密模块
//!
//! 提供 Argon2id KDF 和 XChaCha20-Poly1305 AEAD 加密
//!
//! ## 安全特性
//!
//! - **CSPRNG**: 使用 `OsRng` 而非 `thread_rng` 用于所有密钥生成
//! - **HKDF**: 用于子密钥派生 (优于直接 SHA-256)
//! - **AEAD**: XChaCha20-Poly1305 提供 192-bit nonce + 认证加密
//! - **零化**: 敏感数据使用 `Zeroize` trait 防止内存残留

pub mod argon2;
pub mod xchacha20;
pub mod chacha;
pub mod secure_random;
pub mod hkdf_helper;
pub mod constant_time;

pub use argon2::{derive_kek, generate_salt};
pub use xchacha20::{encrypt_aead, decrypt_aead, encrypt_aead_base64, decrypt_aead_base64};
pub use secure_random::{random_bytes, generate_id, generate_mek, generate_quickkey, fill_random, zeroize};
pub use hkdf_helper::{derive_subkey, derive_subkey_with_salt};
pub use constant_time::ct_eq;
