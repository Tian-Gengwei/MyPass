//! MyPass Tauri 错误类型
//!
//! 所有命令返回精确错误信息，包含代码位置、失败原因、建议修复方向

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TauriError {
    // ========== Vault 相关错误 ==========
    /// 金库未解锁
    VaultLocked,
    /// 金库不存在
    VaultNotFound,
    /// 金库已存在
    VaultAlreadyExists,
    /// 金库路径无效
    VaultInvalidPath,
    /// 金库元数据损坏
    VaultMetaCorrupted,
    /// 金库创建失败
    VaultCreateFailed(String),
    /// 金库解锁失败
    VaultUnlockFailed(String),

    // ========== 加密相关错误 ==========
    /// 主密码错误
    InvalidPassword,
    /// 密码太弱
    WeakPassword(String),
    /// 加密失败
    EncryptionFailed(String),
    /// 解密失败
    DecryptionFailed(String),
    /// 密钥派生失败
    KeyDerivationFailed(String),
    /// 无效的密钥长度
    InvalidKeyLength(usize),

    // ========== 对象存储相关错误 ==========
    /// 对象不存在
    ObjectNotFound(String),
    /// 对象路径无效
    ObjectInvalidPath(String),
    /// 对象写入失败
    ObjectWriteFailed(String),
    /// 对象读取失败
    ObjectReadFailed(String),
    /// 对象删除失败
    ObjectDeleteFailed(String),
    /// 对象哈希不匹配
    ObjectHashMismatch,

    // ========== Manifest 相关错误 ==========
    /// Manifest 不存在
    ManifestNotFound,
    /// Manifest 损坏
    ManifestCorrupted,
    /// Manifest 版本冲突
    ManifestConflict(String),
    /// Manifest 同步失败
    ManifestSyncFailed(String),

    // ========== 认证相关错误 ==========
    /// PIN 未设置
    PinNotSet,
    /// PIN 错误
    PinIncorrect,
    /// PIN 锁定中（5次失败后锁定5分钟）
    PinLocked { until: i64 },
    /// 账户已锁定
    AccountLocked { until: i64 },
    /// 生物识别失败
    BiometricFailed(String),
    /// 生物识别未配置
    BiometricNotConfigured,

    // ========== 同步相关错误 ==========
    /// WebDAV 连接失败
    WebDavConnectionFailed(String),
    /// S3 连接失败
    S3ConnectionFailed(String),
    /// 同步目标不可达
    SyncTargetUnreachable(String),
    /// 同步冲突（需要手动解决）
    SyncConflict { entry_id: String, local_version: u64, remote_version: u64 },
    /// 同步失败
    SyncFailed(String),

    // ========== 导入相关错误 ==========
    /// 导入格式不支持
    ImportFormatNotSupported(String),
    /// 导入文件损坏
    ImportFileCorrupted(String),
    /// 导入数据无效
    ImportDataInvalid(String),
    /// KDBX 密码错误
    KdbxPasswordIncorrect,
    /// KDBX 版本不支持
    KdbxVersionNotSupported(String),

    // ========== 系统相关错误 ==========
    /// 系统密钥库操作失败
    KeychainFailed(String),
    /// 系统密钥库未配置
    KeychainNotConfigured,
    /// 路径无效
    InvalidPath(String),
    /// 文件操作失败
    FileOperationFailed(String),
    /// 权限不足
    PermissionDenied(String),

    // ========== 通用错误 ==========
    /// 操作超时
    OperationTimeout(String),
    /// 并发冲突
    ConcurrentModification(String),
    /// 参数无效
    InvalidArgument(String),
    /// 功能未实现
    Unimplemented(String),
    /// 内部错误
    Internal(String),
}

impl std::fmt::Display for TauriError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Vault
            Self::VaultLocked => write!(f, "Vault is locked. Please unlock first."),
            Self::VaultNotFound => write!(f, "Vault not found at specified path."),
            Self::VaultAlreadyExists => write!(f, "Vault already exists at specified path."),
            Self::VaultInvalidPath => write!(f, "Invalid vault path."),
            Self::VaultMetaCorrupted => write!(f, "Vault metadata is corrupted."),
            Self::VaultCreateFailed(msg) => write!(f, "Failed to create vault: {}.", msg),
            Self::VaultUnlockFailed(msg) => write!(f, "Failed to unlock vault: {}.", msg),

            // 加密
            Self::InvalidPassword => write!(f, "Invalid master password."),
            Self::WeakPassword(msg) => write!(f, "Password too weak: {}.", msg),
            Self::EncryptionFailed(msg) => write!(f, "Encryption failed: {}.", msg),
            Self::DecryptionFailed(msg) => write!(f, "Decryption failed: {}. Check if the password is correct.", msg),
            Self::KeyDerivationFailed(msg) => write!(f, "Key derivation failed: {}.", msg),
            Self::InvalidKeyLength(len) => write!(f, "Invalid key length: {} bytes. Expected 32 bytes.", len),

            // 对象存储
            Self::ObjectNotFound(id) => write!(f, "Object not found: {}.", id),
            Self::ObjectInvalidPath(path) => write!(f, "Invalid object path: {}.", path),
            Self::ObjectWriteFailed(msg) => write!(f, "Failed to write object: {}.", msg),
            Self::ObjectReadFailed(msg) => write!(f, "Failed to read object: {}.", msg),
            Self::ObjectDeleteFailed(msg) => write!(f, "Failed to delete object: {}.", msg),
            Self::ObjectHashMismatch => write!(f, "Object hash does not match. Data may be corrupted."),

            // Manifest
            Self::ManifestNotFound => write!(f, "Manifest file not found."),
            Self::ManifestCorrupted => write!(f, "Manifest is corrupted."),
            Self::ManifestConflict(msg) => write!(f, "Manifest conflict: {}.", msg),
            Self::ManifestSyncFailed(msg) => write!(f, "Manifest sync failed: {}.", msg),

            // 认证
            Self::PinNotSet => write!(f, "PIN has not been set."),
            Self::PinIncorrect => write!(f, "Incorrect PIN."),
            Self::PinLocked { until } => {
                let secs = (until - chrono_now()) as u64;
                write!(f, "PIN locked. Try again in {} seconds.", secs)
            }
            Self::AccountLocked { until } => {
                let secs = (until - chrono_now()) as u64;
                write!(f, "Account locked. Try again in {} seconds.", secs)
            }
            Self::BiometricFailed(msg) => write!(f, "Biometric authentication failed: {}.", msg),
            Self::BiometricNotConfigured => write!(f, "Biometric authentication is not configured."),

            // 同步
            Self::WebDavConnectionFailed(msg) => write!(f, "WebDAV connection failed: {}.", msg),
            Self::S3ConnectionFailed(msg) => write!(f, "S3 connection failed: {}.", msg),
            Self::SyncTargetUnreachable(msg) => write!(f, "Sync target unreachable: {}.", msg),
            Self::SyncConflict { entry_id, local_version, remote_version } => {
                write!(f, "Sync conflict for entry '{}': local={}, remote={}. Manual resolution required.", entry_id, local_version, remote_version)
            }
            Self::SyncFailed(msg) => write!(f, "Sync failed: {}.", msg),

            // 导入
            Self::ImportFormatNotSupported(fmt) => write!(f, "Import format not supported: {}.", fmt),
            Self::ImportFileCorrupted(msg) => write!(f, "Import file is corrupted: {}.", msg),
            Self::ImportDataInvalid(msg) => write!(f, "Import data is invalid: {}.", msg),
            Self::KdbxPasswordIncorrect => write!(f, "Incorrect KDBX password."),
            Self::KdbxVersionNotSupported(ver) => write!(f, "KDBX version not supported: {}.", ver),

            // 系统
            Self::KeychainFailed(msg) => write!(f, "Keychain operation failed: {}.", msg),
            Self::KeychainNotConfigured => write!(f, "Keychain is not configured."),
            Self::InvalidPath(msg) => write!(f, "Invalid path: {}.", msg),
            Self::FileOperationFailed(msg) => write!(f, "File operation failed: {}.", msg),
            Self::PermissionDenied(msg) => write!(f, "Permission denied: {}.", msg),

            // 通用
            Self::OperationTimeout(msg) => write!(f, "Operation timed out: {}.", msg),
            Self::ConcurrentModification(msg) => write!(f, "Concurrent modification detected: {}.", msg),
            Self::InvalidArgument(msg) => write!(f, "Invalid argument: {}.", msg),
            Self::Unimplemented(feature) => write!(f, "Feature not implemented: {}.", feature),
            Self::Internal(msg) => write!(f, "Internal error: {}.", msg),
        }
    }
}

impl std::error::Error for TauriError {}

fn chrono_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

impl From<std::io::Error> for TauriError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => Self::ObjectNotFound(err.to_string()),
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied(err.to_string()),
            _ => Self::FileOperationFailed(err.to_string()),
        }
    }
}

impl From<anyhow::Error> for TauriError {
    fn from(err: anyhow::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<mypass_core::error::TauriError> for TauriError {
    fn from(err: mypass_core::error::TauriError) -> Self {
        use mypass_core::error::TauriError as Core;
        match err {
            Core::Vault(s) => Self::Internal(s),
            Core::Crypto(s) => Self::EncryptionFailed(s),
            Core::Auth(s) => Self::BiometricFailed(s),
            Core::Sync(s) => Self::SyncFailed(s),
            Core::Import(s) => Self::ImportDataInvalid(s),
            Core::Internal(s) => Self::Internal(s),
            Core::EncryptionFailed(s) => Self::EncryptionFailed(s),
            Core::DecryptionFailed(s) => Self::DecryptionFailed(s),
            Core::FileOperationFailed(s) => Self::FileOperationFailed(s),
            Core::ObjectWriteFailed(s) => Self::ObjectWriteFailed(s),
            Core::ObjectReadFailed(s) => Self::ObjectReadFailed(s),
            Core::ObjectDeleteFailed(s) => Self::ObjectDeleteFailed(s),
            Core::ObjectNotFound(s) => Self::ObjectNotFound(s),
            Core::SyncFailed(s) => Self::SyncFailed(s),
            Core::WebDavConnectionFailed(s) => Self::WebDavConnectionFailed(s),
            Core::InvalidArgument(s) => Self::InvalidArgument(s),
            Core::PinNotSet => Self::PinNotSet,
            Core::PinLocked { until } => Self::PinLocked { until },
            Core::Unimplemented(s) => Self::Unimplemented(s),
            Core::KeyDerivationFailed(s) => Self::KeyDerivationFailed(s),
            Core::InvalidPassword(_s) => Self::InvalidPassword,
            Core::VaultNotFound => Self::VaultNotFound,
            Core::VaultMetaCorrupted => Self::VaultMetaCorrupted,
            Core::ManifestCorrupted => Self::ManifestCorrupted,
        }
    }
}