//!附件处理模块
//!
//! 支持密码条目的文件附件（身份证照片、信用卡扫描件等）
//!
//! ## 设计
//!
//! - 附件独立存储，不混入 Vault JSON
//! - 每个附件有独立 ID 和元数据
//! - 支持压缩和加密存储
//! - 与 Vault 一起打包或独立同步

use crate::error::{Result, TauriError};
use anyhow::Result as AnyResult;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::fs;
use std::path::Path;

/// 附件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentMeta {
    ///附件 ID
    pub id: String,
    /// 所属条目 ID
    pub entry_id: String,
    /// 文件名
    pub name: String,
    /// MIME 类型
    pub mime_type: String,
    /// 文件大小（字节）
    pub size: u64,
    /// SHA-256 哈希
    pub hash: String,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 是否加密
    pub encrypted: bool,
}

/// 附件句柄
pub struct Attachment {
    /// 元数据
    meta: AttachmentMeta,
    /// 文件路径
    path: std::path::PathBuf,
}

/// 附件存储
pub struct AttachmentStore {
    /// 存储根目录
    root: std::path::PathBuf,
}

impl AttachmentStore {
    pub fn new(root: impl Into<std::path::PathBuf>) -> Self {
        Self {
            root: root.into(),
        }
    }

    /// 保存附件
    pub fn save(
        &self,
        entry_id: &str,
        name: &str,
        mime_type: &str,
        data: &[u8],
    ) -> AnyResult<AttachmentMeta> {
        let id = uuid::Uuid::new_v4().to_string();

        // 计算哈希
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hex::encode(hasher.finalize());

        // 创建目录
        let dir = self.root.join(entry_id);
        fs::create_dir_all(&dir)?;

        // 写入文件
        let ext = std::path::Path::new(name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin");
        let path = dir.join(format!("{}.{}", id, ext));
        fs::write(&path, data)?;

        let meta = AttachmentMeta {
            id,
            entry_id: entry_id.to_string(),
            name: name.to_string(),
            mime_type: mime_type.to_string(),
            size: data.len() as u64,
            hash,
            created_at: chrono::Utc::now(),
            encrypted: false,
        };

        Ok(meta)
    }

    /// 读取附件
    pub fn load(&self, entry_id: &str, attachment_id: &str) -> AnyResult<Vec<u8>> {
        // 查找文件
        let dir = self.root.join(entry_id);
        let entries = fs::read_dir(&dir)?;

        for entry in entries.flatten() {
            let filename = entry.file_name();
            let filename_str = filename.to_string_lossy();

            if filename_str.starts_with(attachment_id) {
                return Ok(fs::read(entry.path())?);
            }
        }

        anyhow::bail!("Attachment not found: {}/{}", entry_id, attachment_id)
    }

    /// 删除附件
    pub fn delete(&self, entry_id: &str, attachment_id: &str) -> AnyResult<()> {
        let dir = self.root.join(entry_id);
        let entries = fs::read_dir(&dir)?;

        for entry in entries.flatten() {
            let filename = entry.file_name();
            let filename_str = filename.to_string_lossy();

            if filename_str.starts_with(attachment_id) {
                fs::remove_file(entry.path())?;
                return Ok(());
            }
        }

        Ok(())
    }

    /// 删除条目的所有附件
    pub fn delete_all(&self, entry_id: &str) -> AnyResult<()> {
        let dir = self.root.join(entry_id);
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        Ok(())
    }

    /// 列出条目的附件
    pub fn list(&self, entry_id: &str) -> AnyResult<Vec<AttachmentMeta>> {
        let dir = self.root.join(entry_id);
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut metas = Vec::new();
        let entries = fs::read_dir(&dir)?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let metadata = fs::metadata(&path)?;
                let filename = path.file_name().unwrap().to_string_lossy();

                // 提取 ID（去掉扩展名）
                let id = filename.split('.').next().unwrap().to_string();

                metas.push(AttachmentMeta {
                    id,
                    entry_id: entry_id.to_string(),
                    name: filename.to_string(),
                    mime_type: "application/octet-stream".to_string(),
                    size: metadata.len(),
                    hash: String::new(), // 不计算哈希
                    created_at: chrono::Utc::now(),
                    encrypted: false,
                });
            }
        }

        Ok(metas)
    }
}

/// 从 Base64 导入附件
pub fn import_from_base64(
    store: &AttachmentStore,
    entry_id: &str,
    name: &str,
    mime_type: &str,
    base64_data: &str,
) -> AnyResult<AttachmentMeta> {
    use base64::Engine;
    let data = base64::engine::general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|e| anyhow::anyhow!("Invalid Base64: {}", e))?;

    store.save(entry_id, name, mime_type, &data)
}

/// 导出为 Base64
pub fn export_to_base64(store: &AttachmentStore, entry_id: &str, attachment_id: &str) -> AnyResult<String> {
    use base64::Engine;
    let data = store.load(entry_id, attachment_id)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_attachment_store() {
        let dir = tempdir().unwrap();
        let store = AttachmentStore::new(dir.path());

        let data = b"test file content";
        let meta = store.save("entry1", "test.txt", "text/plain", data).unwrap();

        assert_eq!(meta.name, "test.txt");
        assert_eq!(meta.size, 17);

        // 读取
        let loaded = store.load("entry1", &meta.id).unwrap();
        assert_eq!(loaded, data);

        // 列出
        let list = store.list("entry1").unwrap();
        assert_eq!(list.len(), 1);

        // 删除
        store.delete("entry1", &meta.id).unwrap();
        let list = store.list("entry1").unwrap();
        assert!(list.is_empty());
    }
}