//! Vault 核心存储
//!
//! 管理金库的创建、解锁、条目操作
//!
//! ## 性能优化
//!
//! - **HashMap 索引**：O(1) 条目查找
//! - **预计算小写搜索索引**：避免每次 `to_lowercase()` 分配
//! - **批量迭代器**：使用 `iter().filter_map()` 减少中间分配

use crate::crypto::{derive_kek, encrypt_aead, decrypt_aead, generate_salt, zeroize};
use crate::error::TauriError;
use crate::vault::{Entry, Group, Manifest, ObjectStorage, ObjectType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;

/// Vault 元数据（存储在 vault.meta.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultMeta {
    pub version: u64,
    /// Argon2id 盐值
    pub salt: Vec<u8>,
    pub created_at: i64,
    pub name: String,
}

/// Vault 核心
///
/// 生命周期：
/// 1. `Vault::create()` - 创建新金库
/// 2. `Vault::unlock()` - 解锁已有金库
/// 3. `vault.lock()` - 锁定金库（保存状态）
///
/// ## 安全
///
/// - **MEK 零时清理**：`Drop` 时自动清零
/// - **预计算索引**：减少重复字符串处理
pub struct Vault {
    pub(crate) path: PathBuf,
    pub(crate) meta: VaultMeta,
    pub(crate) mek: Vec<u8>,
    pub(crate) manifest: Manifest,
    pub(crate) entries: HashMap<String, Entry>,
    pub(crate) groups: HashMap<String, Group>,
    pub(crate) storage: ObjectStorage,
    /// 预计算的搜索索引（小写化字段）
    search_index: HashMap<String, SearchEntry>,
}

/// 搜索索引条目（预计算的小写字段）
#[derive(Debug, Clone)]
struct SearchEntry {
    name_lower: String,
    username_lower: String,
    url_lower: String,
}

impl SearchEntry {
    fn from_entry(entry: &Entry) -> Self {
        Self {
            name_lower: entry.name.to_lowercase(),
            username_lower: entry.username.to_lowercase(),
            url_lower: entry.url.as_deref().unwrap_or("").to_lowercase(),
        }
    }

    fn matches(&self, query_lower: &str) -> bool {
        self.name_lower.contains(query_lower)
            || self.username_lower.contains(query_lower)
            || (!self.url_lower.is_empty() && self.url_lower.contains(query_lower))
    }
}

impl Drop for Vault {
    fn drop(&mut self) {
        // 关键安全：MEK 在 Vault 销毁时清零
        zeroize(&mut self.mek);
    }
}

impl Vault {
    /// 创建新金库
    pub fn create(path: PathBuf, master_password: &str, name: &str) -> Result<Vault, TauriError> {
        let salt = generate_salt();
        let kek = derive_kek(master_password, &salt)
            .map_err(|e| TauriError::KeyDerivationFailed(e.to_string()))?;
        let mek = generate_mek();

        let encrypted_mek = encrypt_aead(&mek, &kek)
            .map_err(|e| TauriError::EncryptionFailed(e.to_string()))?;

        let vault_path = path.join(format!("{}.vault", name));
        std::fs::create_dir_all(vault_path.join("objects"))
            .map_err(|e| TauriError::FileOperationFailed(e.to_string()))?;
        std::fs::create_dir_all(vault_path.join("logs"))
            .map_err(|e| TauriError::FileOperationFailed(e.to_string()))?;

        let meta = VaultMeta {
            version: 1,
            salt,
            created_at: timestamp(),
            name: name.to_string(),
        };

        std::fs::write(
            vault_path.join("vault.meta.json"),
            serde_json::to_string_pretty(&meta)
                .map_err(|e| TauriError::Internal(format!("Failed to serialize meta: {}", e)))?,
        )
        .map_err(|e| TauriError::FileOperationFailed(e.to_string()))?;

        std::fs::write(vault_path.join("master_key.enc"), encrypted_mek)
            .map_err(|e| TauriError::FileOperationFailed(e.to_string()))?;

        let storage = ObjectStorage::new(vault_path.clone(), mek.clone());
        let manifest = Manifest::new();

        Ok(Vault {
            path: vault_path,
            meta,
            mek,
            manifest,
            entries: HashMap::new(),
            groups: HashMap::new(),
            storage,
            search_index: HashMap::new(),
        })
    }

    /// 解锁金库
    pub fn unlock(path: PathBuf, master_password: &str) -> Result<Vault, TauriError> {
        let meta = Self::load_meta(&path)?;

        // 派生 KEK
        let kek = derive_kek(master_password, &meta.salt)
            .map_err(|e| TauriError::KeyDerivationFailed(e.to_string()))?;

        // 读取加密的 MEK
        let encrypted_mek = std::fs::read(path.join("master_key.enc"))
            .map_err(|e| TauriError::FileOperationFailed(e.to_string()))?;

        // 解密 MEK
        let mek = decrypt_aead(&encrypted_mek, &kek)
            .map_err(|e| TauriError::InvalidPassword(e.to_string()))?;

        // 加载 Manifest
        let manifest = Self::load_manifest(&path, &mek)?;

        let storage = ObjectStorage::new(path.clone(), mek.clone());

        // 加载所有对象
        let entries_list = storage.load_all_entries(&manifest)
            .map_err(|e| TauriError::ObjectReadFailed(e.to_string()))?;
        let groups_list = storage.load_all_groups(&manifest)
            .map_err(|e| TauriError::ObjectReadFailed(e.to_string()))?;

        // 构建 HashMap 和搜索索引
        let mut entries: HashMap<String, Entry> = HashMap::with_capacity(entries_list.len());
        let mut search_index: HashMap<String, SearchEntry> = HashMap::with_capacity(entries_list.len());
        for entry in entries_list {
            search_index.insert(entry.id.clone(), SearchEntry::from_entry(&entry));
            entries.insert(entry.id.clone(), entry);
        }

        let groups: HashMap<String, Group> = groups_list
            .into_iter()
            .map(|g| (g.id.clone(), g))
            .collect();

        Ok(Vault {
            path,
            meta,
            mek,
            manifest,
            entries,
            groups,
            storage,
            search_index,
        })
    }

    /// 锁定金库
    pub fn lock(&mut self) -> Result<(), TauriError> {
        self.save_manifest()?;
        // 锁定后清空内存中的数据
        self.entries.clear();
        self.groups.clear();
        self.search_index.clear();
        zeroize(&mut self.mek);
        Ok(())
    }

    /// 添加条目
    pub fn add_entry(&mut self, entry: Entry) -> Result<(), TauriError> {
        let meta = self.storage.save_object(&entry, &entry.id, ObjectType::Entry)
            .map_err(|e| TauriError::ObjectWriteFailed(e.to_string()))?;
        self.manifest.update_entry(entry.id.clone(), meta);
        self.search_index.insert(entry.id.clone(), SearchEntry::from_entry(&entry));
        self.entries.insert(entry.id.clone(), entry);
        self.save_manifest()?;
        Ok(())
    }

    /// 更新条目
    pub fn update_entry(&mut self, id: &str, entry: Entry) -> Result<(), TauriError> {
        let old_hash = self.manifest.entries.get(id)
            .map(|m| m.file_hash.clone())
            .unwrap_or_default();

        let meta = self.storage.update_object(&entry, id, ObjectType::Entry, &old_hash)
            .map_err(|e| TauriError::ObjectWriteFailed(e.to_string()))?;
        self.manifest.update_entry(id.to_string(), meta);
        self.search_index.insert(id.to_string(), SearchEntry::from_entry(&entry));
        self.entries.insert(id.to_string(), entry);
        self.save_manifest()?;
        Ok(())
    }

    /// 删除条目
    pub fn delete_entry(&mut self, id: &str) -> Result<(), TauriError> {
        if let Some(meta) = self.manifest.entries.get(id).cloned() {
            self.storage.delete_object_by_hash(&meta.file_hash)
                .map_err(|e| TauriError::ObjectDeleteFailed(e.to_string()))?;
            self.manifest.remove_entry(&id.to_string());
            self.entries.remove(id);
            self.search_index.remove(id);
            self.save_manifest()?;
        }
        Ok(())
    }

    /// 获取条目
    pub fn get_entry(&self, id: &str) -> Option<&Entry> {
        self.entries.get(id)
    }

    /// 列出所有条目
    pub fn list_entries(&self) -> Vec<Entry> {
        self.entries.values().cloned().collect()
    }

    /// 添加分组
    pub fn add_group(&mut self, group: Group) -> Result<(), TauriError> {
        let meta = self.storage.save_object(&group, &group.id, ObjectType::Group)
            .map_err(|e| TauriError::ObjectWriteFailed(e.to_string()))?;
        self.manifest.update_group(group.id.clone(), meta);
        self.groups.insert(group.id.clone(), group);
        self.save_manifest()?;
        Ok(())
    }

    /// 删除分组
    pub fn delete_group(&mut self, id: &str) -> Result<(), TauriError> {
        if let Some(meta) = self.manifest.groups.get(id).cloned() {
            self.storage.delete_object_by_hash(&meta.file_hash)
                .map_err(|e| TauriError::ObjectDeleteFailed(e.to_string()))?;
            self.manifest.remove_group(&id.to_string());
            self.groups.remove(id);
            self.save_manifest()?;
        }
        Ok(())
    }

    /// 列出所有分组
    pub fn list_groups(&self) -> Vec<Group> {
        self.groups.values().cloned().collect()
    }

    /// 搜索条目（使用预计算索引，O(n) 但常数小）
    pub fn search_entries(&self, query: &str) -> Vec<Entry> {
        if query.is_empty() {
            return self.list_entries();
        }
        let query_lower = query.to_lowercase();
        self.entries
            .iter()
            .filter_map(|(id, entry)| {
                self.search_index
                    .get(id)
                    .filter(|idx| idx.matches(&query_lower))
                    .map(|_| entry.clone())
            })
            .collect()
    }

    /// 获取金库信息
    pub fn get_info(&self) -> VaultInfo {
        VaultInfo {
            name: self.meta.name.clone(),
            entry_count: self.entries.len(),
            group_count: self.groups.len(),
            created_at: self.meta.created_at,
        }
    }

    /// 获取 Manifest
    pub fn get_manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// 获取 MEK 引用
    pub fn get_mek(&self) -> &[u8] {
        &self.mek
    }

    // ========== 私有方法 ==========

    #[allow(dead_code)]
    fn save_meta(&self) -> Result<(), TauriError> {
        let meta_path = self.path.join("vault.meta.json");
        let meta_json = serde_json::to_string_pretty(&self.meta)
            .map_err(|e| TauriError::Internal(format!("Failed to serialize meta: {}", e)))?;
        std::fs::write(meta_path, meta_json)
            .map_err(|e| TauriError::FileOperationFailed(e.to_string()))?;
        Ok(())
    }

    fn save_manifest(&self) -> Result<(), TauriError> {
        let manifest_path = self.path.join("manifest.enc");
        let manifest_json = serde_json::to_vec(&self.manifest)
            .map_err(|e| TauriError::Internal(format!("Failed to serialize manifest: {}", e)))?;
        let encrypted = encrypt_aead(&manifest_json, &self.mek)
            .map_err(|e| TauriError::EncryptionFailed(e.to_string()))?;
        std::fs::write(manifest_path, encrypted)
            .map_err(|e| TauriError::FileOperationFailed(e.to_string()))?;
        Ok(())
    }

    fn load_meta(path: &PathBuf) -> Result<VaultMeta, TauriError> {
        let meta_path = if path.join("vault.meta.json").exists() {
            path.join("vault.meta.json")
        } else {
            let entries = std::fs::read_dir(path)
                .map_err(|e| TauriError::ObjectReadFailed(e.to_string()))?;
            for entry in entries.flatten() {
                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if name_str.ends_with(".vault") {
                        let vault_path = entry.path();
                        if vault_path.join("vault.meta.json").exists() {
                            return Self::load_meta(&vault_path);
                        }
                    }
                }
            }
            return Err(TauriError::VaultNotFound);
        };

        let meta_json = std::fs::read_to_string(&meta_path)
            .map_err(|e| TauriError::ObjectReadFailed(format!("vault.meta.json: {}", e)))?;
        let meta: VaultMeta = serde_json::from_str(&meta_json)
            .map_err(|_| TauriError::VaultMetaCorrupted)?;
        Ok(meta)
    }

    fn load_manifest(path: &PathBuf, mek: &[u8]) -> Result<Manifest, TauriError> {
        let manifest_path = path.join("manifest.enc");
        let encrypted = std::fs::read(&manifest_path)
            .map_err(|e| TauriError::ObjectReadFailed(format!("manifest.enc: {}", e)))?;
        let manifest_json = decrypt_aead(&encrypted, mek)
            .map_err(|e| TauriError::DecryptionFailed(e.to_string()))?;
        let manifest: Manifest = serde_json::from_slice(&manifest_json)
            .map_err(|_| TauriError::ManifestCorrupted)?;
        Ok(manifest)
    }
}

/// 金库信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct VaultInfo {
    pub name: String,
    pub entry_count: usize,
    pub group_count: usize,
    pub created_at: i64,
}

fn generate_mek() -> Vec<u8> {
    crate::crypto::secure_random::generate_mek().to_vec()
}

fn timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
