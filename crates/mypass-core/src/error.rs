//! 错误类型定义
//!
//! ## 设计
//!
//! 所有错误都携带人类可读的消息，集成到 Tauri IPC 时转为 String

use thiserror::Error;

pub type Result<T> = anyhow::Result<T>;

#[derive(Error, Debug)]
pub enum TauriError {
    #[error("Vault error: {0}")]
    Vault(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Auth error: {0}")]
    Auth(String),

    #[error("Sync error: {0}")]
    Sync(String),

    #[error("Import error: {0}")]
    Import(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("File operation failed: {0}")]
    FileOperationFailed(String),

    #[error("Object not found: {0}")]
    ObjectNotFound(String),

    #[error("Object write failed: {0}")]
    ObjectWriteFailed(String),

    #[error("Object read failed: {0}")]
    ObjectReadFailed(String),

    #[error("Object delete failed: {0}")]
    ObjectDeleteFailed(String),

    #[error("Sync failed: {0}")]
    SyncFailed(String),

    #[error("WebDAV connection failed: {0}")]
    WebDavConnectionFailed(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("PIN not set")]
    PinNotSet,

    #[error("PIN locked until {until}")]
    PinLocked { until: i64 },

    #[error("Unimplemented: {0}")]
    Unimplemented(String),

    // ===== Vault 特定错误 =====

    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    #[error("Invalid password")]
    InvalidPassword(String),

    #[error("Vault not found")]
    VaultNotFound,

    #[error("Vault metadata corrupted")]
    VaultMetaCorrupted,

    #[error("Manifest corrupted")]
    ManifestCorrupted,
}

impl TauriError {
    /// 转为 Tauri IPC 可序列化的字符串
    pub fn to_user_message(&self) -> String {
        self.to_string()
    }
}
