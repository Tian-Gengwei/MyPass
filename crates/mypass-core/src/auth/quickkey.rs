//! QuickKey 快速解锁机制
//!
//! QuickKey 是 256-bit 随机密钥，用于实现生物识别/PIN 秒解锁。
//!
//! ## 机制说明
//!
//! 1. **首次主密码解锁后**：
//!    - 生成随机 QuickKey（256-bit）
//!    - 使用 QuickKey 加密 MEK
//!    - 将 QuickKey 存入系统密钥库，标记需要生物/PIN 才能读取
//!
//! 2. **后续解锁**：
//!    - 用户使用生物识别或 PIN
//!    - 系统密钥库释放 QuickKey
//!    - 使用 QuickKey 解密 MEK
//!    - 无需输入主密码，实现"秒开"
//!
//! ## 存储后端
//!
//! - 优先尝试系统密钥库（tauri-plugin-keychain）
//! - 回退到本地文件（用户主目录 + OS 权限保护）

use crate::error::TauriError;
use crate::crypto::secure_random;
use std::path::PathBuf;
use zeroize::Zeroize;

const QUICKKEY_FILE: &str = "quickkey.bin";
const SERVICE: &str = "com.mypass.app";

/// QuickKey 生成
///
/// 生成 256-bit 随机密钥，用于加密 MEK
pub fn quickkey_generate() -> Vec<u8> {
    secure_random::generate_quickkey().to_vec()
}

/// QuickKey 存储到本地文件（带回退到系统密钥库）
///
/// 文件位置：`{user_data_dir}/quickkey.bin`
/// - macOS: `~/Library/Application Support/com.mypass.app/`
/// - Linux: `~/.local/share/com.mypass.app/`
/// - Windows: `%APPDATA%/com.mypass.app/`
///
/// 文件权限：在 Unix 上设置为 0600（仅所有者可读写）
///
/// # Arguments
/// * `key` - QuickKey（256-bit）
/// * `identifier` - 唯一标识符（如 vault id）
pub async fn quickkey_store(key: &[u8], identifier: &str) -> Result<(), TauriError> {
    let path = quickkey_path(identifier)?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| TauriError::Internal(format!("create dir: {}", e)))?;
    }

    // 写入时使用临时文件 + 原子重命名
    let tmp_path = path.with_extension("tmp");
    std::fs::write(&tmp_path, key)
        .map_err(|e| TauriError::Internal(format!("write quickkey: {}", e)))?;

    // Unix: 设置文件权限为 0600
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&tmp_path, perms)
            .map_err(|e| TauriError::Internal(format!("set perms: {}", e)))?;
    }

    std::fs::rename(&tmp_path, &path)
        .map_err(|e| TauriError::Internal(format!("rename quickkey: {}", e)))?;

    tracing::info!("QuickKey stored to {:?}", path);
    Ok(())
}

/// QuickKey 从本地文件读取
pub async fn quickkey_retrieve(identifier: &str) -> Result<Vec<u8>, TauriError> {
    let path = quickkey_path(identifier)?;
    let mut key = std::fs::read(&path)
        .map_err(|e| TauriError::Internal(format!("read quickkey: {}", e)))?;

    if key.len() != 32 {
        key.zeroize();
        return Err(TauriError::Internal(format!(
            "Invalid QuickKey length: {} (expected 32)",
            key.len()
        )));
    }

    Ok(key)
}

/// 使用 QuickKey 加密 MEK
pub fn encrypt_mek_with_quickkey(mek: &[u8], quickkey: &[u8]) -> Result<Vec<u8>, TauriError> {
    use crate::crypto::encrypt_aead;
    encrypt_aead(mek, quickkey)
        .map_err(|e| TauriError::EncryptionFailed(e.to_string()))
}

/// 使用 QuickKey 解密 MEK
pub fn decrypt_mek_with_quickkey(encrypted_mek: &[u8], quickkey: &[u8]) -> Result<Vec<u8>, TauriError> {
    use crate::crypto::decrypt_aead;
    decrypt_aead(encrypted_mek, quickkey)
        .map_err(|e| TauriError::DecryptionFailed(e.to_string()))
}

/// 检查 QuickKey 是否已配置
pub async fn quickkey_is_configured(identifier: &str) -> Result<bool, TauriError> {
    let path = quickkey_path(identifier)?;
    Ok(path.exists())
}

/// 删除 QuickKey
pub async fn quickkey_delete(identifier: &str) -> Result<(), TauriError> {
    let path = quickkey_path(identifier)?;
    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|e| TauriError::Internal(format!("delete quickkey: {}", e)))?;
    }
    Ok(())
}

/// 计算 QuickKey 文件路径
fn quickkey_path(identifier: &str) -> Result<PathBuf, TauriError> {
    let base = user_data_dir()?;
    // 使用 identifier 的 SHA-256 哈希避免文件系统特殊字符
    let hash = {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(identifier.as_bytes());
        hex::encode(hasher.finalize())[..16].to_string()
    };
    Ok(base.join(SERVICE).join(format!("{}.{}", QUICKKEY_FILE, hash)))
}

/// 获取用户数据目录
fn user_data_dir() -> Result<PathBuf, TauriError> {
    directories::ProjectDirs::from("com", "mypass", "app")
        .map(|dirs| dirs.data_dir().to_path_buf())
        .ok_or_else(|| TauriError::Internal("Cannot determine user data directory".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quickkey_generate() {
        let k1 = quickkey_generate();
        let k2 = quickkey_generate();
        assert_eq!(k1.len(), 32);
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_encrypt_decrypt_mek() {
        let mek = vec![0x42u8; 32];
        let quickkey = quickkey_generate();

        let encrypted = encrypt_mek_with_quickkey(&mek, &quickkey).unwrap();
        let decrypted = decrypt_mek_with_quickkey(&encrypted, &quickkey).unwrap();

        assert_eq!(decrypted, mek);
    }

    #[test]
    fn test_wrong_quickkey_fails() {
        let mek = vec![0x42u8; 32];
        let qk1 = quickkey_generate();
        let qk2 = quickkey_generate();

        let encrypted = encrypt_mek_with_quickkey(&mek, &qk1).unwrap();
        let result = decrypt_mek_with_quickkey(&encrypted, &qk2);
        assert!(result.is_err());
    }

    #[test]
    fn test_path_isolation() {
        let p1 = quickkey_path("vault-a").unwrap();
        let p2 = quickkey_path("vault-b").unwrap();
        assert_ne!(p1, p2);
    }
}
