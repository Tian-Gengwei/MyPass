//! 对象存储引擎
//!
//! 负责将 Entry/Group 序列化为 JSON -> 加密 -> 按 Hash 路径写入 objects/ 目录

use crate::crypto::{encrypt_aead, decrypt_aead};
use crate::error::TauriError;
use crate::vault::{Entry, Group, Manifest};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::path::PathBuf;
use zeroize::Zeroize;

/// 对象元数据（存储在 Manifest 中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMeta {
    pub id: String,
    pub obj_type: ObjectType,
    pub file_hash: String,
    pub version: u64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ObjectType {
    Entry,
    Group,
}

/// 对象存储引擎
///
/// ## 安全
///
/// - **MEK 零时清理**：`Drop` 时自动清零 MEK 内存
/// - **AEAD 加密**：所有对象使用 XChaCha20-Poly1305 加密
/// - **路径分片**：使用 Hash 前缀分散到子目录，避免单目录文件过多
pub struct ObjectStorage {
    vault_path: PathBuf,
    mek: Vec<u8>,
}

impl Drop for ObjectStorage {
    fn drop(&mut self) {
        self.mek.zeroize();
    }
}

impl ObjectStorage {
    pub fn new(vault_path: PathBuf, mek: Vec<u8>) -> Self {
        Self { vault_path, mek }
    }

    /// 构造对象路径（使用 `&Path` 而非 `String` 减少分配）
    fn object_path(&self, hash: &str) -> (PathBuf, String) {
        let clean = hash
            .as_bytes()
            .iter()
            .filter(|&&b| b != b':' && b != b'-')
            .take(8)
            .map(|&b| b as char)
            .collect::<String>();

        // 取前 4 个字符的字节（安全因为 hex 字符都是 ASCII）
        let bytes = clean.as_bytes();
        let (dir1, rest) = bytes.split_at_checked(2).unwrap_or((b"00", b"00"));
        let (dir2, filename) = rest.split_at_checked(2).unwrap_or((b"00", b"0000"));

        let dir1 = std::str::from_utf8(dir1).unwrap_or("00");
        let dir2 = std::str::from_utf8(dir2).unwrap_or("00");
        let filename = std::str::from_utf8(filename).unwrap_or("0000");

        let path = self.vault_path
            .join("objects")
            .join(dir1)
            .join(dir2)
            .join(format!("{}.enc", filename));

        (path, filename.to_string())
    }

    /// 保存对象
    pub fn save_object<T: Serialize>(&self, obj: &T, id: &str, obj_type: ObjectType) -> Result<ObjectMeta, TauriError> {
        // 1. 序列化为 JSON
        let json = serde_json::to_vec(obj)
            .map_err(|e| TauriError::Internal(format!("Failed to serialize: {}", e)))?;

        // 2. 使用 MEK 加密
        let encrypted = encrypt_aead(&json, &self.mek)
            .map_err(|e| TauriError::EncryptionFailed(e.to_string()))?;

        // 3. 计算内容 Hash（用于路径）
        let hash = compute_hash(&json);

        // 4. 构造路径
        let (file_path, _) = self.object_path(&hash);

        // 5. 确保目录存在
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| TauriError::FileOperationFailed(e.to_string()))?;
        }

        // 6. 写入加密文件
        std::fs::write(&file_path, &encrypted)
            .map_err(|e| TauriError::ObjectWriteFailed(e.to_string()))?;

        Ok(ObjectMeta {
            id: id.to_string(),
            obj_type,
            file_hash: hash,  // 存储完整 hash 用于后续加载
            version: 1,
            updated_at: timestamp(),
        })
    }

    /// 更新已有对象
    pub fn update_object<T: Serialize>(&self, obj: &T, id: &str, obj_type: ObjectType, old_hash: &str) -> Result<ObjectMeta, TauriError> {
        if !old_hash.is_empty() {
            let _ = self.delete_object_by_hash(old_hash);
        }

        let mut meta = self.save_object(obj, id, obj_type)?;
        meta.version += 1;
        Ok(meta)
    }

    /// 加载对象
    pub fn load_object<T: for<'de> Deserialize<'de>>(&self, hash: &str, _obj_type: ObjectType) -> Result<T, TauriError> {
        let (file_path, _) = self.object_path(hash);

        let encrypted = std::fs::read(&file_path)
            .map_err(|e| TauriError::ObjectReadFailed(format!("{}: {}", file_path.display(), e)))?;

        let json = decrypt_aead(&encrypted, &self.mek)
            .map_err(|e| TauriError::DecryptionFailed(e.to_string()))?;

        let obj = serde_json::from_slice(&json)
            .map_err(|e| TauriError::Internal(format!("Failed to deserialize: {}", e)))?;

        Ok(obj)
    }

    /// 通过 hash 删除对象
    pub fn delete_object_by_hash(&self, hash: &str) -> Result<(), TauriError> {
        let (file_path, _) = self.object_path(hash);
        std::fs::remove_file(&file_path)
            .map_err(|e| TauriError::ObjectDeleteFailed(format!("{}: {}", file_path.display(), e)))?;
        Ok(())
    }

    /// 删除对象（通过 Manifest 中的信息）
    pub fn delete_object(&self, hash: &str) -> Result<(), TauriError> {
        self.delete_object_by_hash(hash)
    }

    /// 批量保存条目
    pub fn save_entries(&self, entries: &[Entry]) -> Result<Vec<ObjectMeta>, TauriError> {
        entries
            .iter()
            .map(|entry| self.save_object(entry, &entry.id, ObjectType::Entry))
            .collect()
    }

    /// 批量保存分组
    pub fn save_groups(&self, groups: &[Group]) -> Result<Vec<ObjectMeta>, TauriError> {
        groups
            .iter()
            .map(|group| self.save_object(group, &group.id, ObjectType::Group))
            .collect()
    }

    /// 批量加载所有 Entry
    pub fn load_all_entries(&self, manifest: &Manifest) -> Result<Vec<Entry>, TauriError> {
        manifest.entries
            .iter()
            .filter(|(_, meta)| meta.obj_type == ObjectType::Entry)
            .filter_map(|(id, meta)| {
                match self.load_object::<Entry>(&meta.file_hash, ObjectType::Entry) {
                    Ok(entry) => Some(Ok(entry)),
                    Err(e) => {
                        tracing::warn!("Failed to load entry {}: {}", id, e);
                        None
                    }
                }
            })
            .collect()
    }

    /// 批量加载所有 Group
    pub fn load_all_groups(&self, manifest: &Manifest) -> Result<Vec<Group>, TauriError> {
        manifest.groups
            .iter()
            .filter(|(_, meta)| meta.obj_type == ObjectType::Group)
            .filter_map(|(id, meta)| {
                match self.load_object::<Group>(&meta.file_hash, ObjectType::Group) {
                    Ok(group) => Some(Ok(group)),
                    Err(e) => {
                        tracing::warn!("Failed to load group {}: {}", id, e);
                        None
                    }
                }
            })
            .collect()
    }
}

/// 计算内容的 SHA256 哈希
fn compute_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

fn timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash() {
        let content = b"test entry";
        let hash = compute_hash(content);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_object_path_no_clean_chars() {
        let storage = ObjectStorage::new(PathBuf::from("/tmp/vault"), vec![0u8; 32]);
        let (path, filename) = storage.object_path("a1b2c3d4e5f6g7h8");
        assert!(path.ends_with("a1/b2/c3d4.enc"));
        assert_eq!(filename, "c3d4");
    }

    #[test]
    fn test_object_path_strips_dashes() {
        let storage = ObjectStorage::new(PathBuf::from("/tmp/vault"), vec![0u8; 32]);
        let (path, _) = storage.object_path("a1:b2-c3d4e5f6g7h8");
        assert!(path.ends_with("a1/b2/c3d4.enc"));
    }
}