//! Vault 金库管理命令模块
//!
//! # 概述
//! 提供金库创建、解锁、锁定以及条目 CRUD 操作。
//!
//! # 命令列表
//! - `create_vault`: 创建新金库
//! - `unlock_vault`: 使用主密码解锁金库
//! - `lock_vault`: 锁定金库
//! - `get_entries`: 获取所有条目
//! - `create_entry`: 创建新条目
//! - `update_entry`: 更新条目
//! - `delete_entry`: 删除条目
//! - `get_groups`: 获取所有分组
//! - `create_group`: 创建新分组
//! - `search_entries`: 搜索条目
//! - `get_vault_info`: 获取金库信息
//! - `set_vault_path`: 设置金库路径（用于用户选择位置）

use crate::error::TauriError;
use mypass_core::vault::{Entry, Group, Vault};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::commands::vault::state::VaultState;

/// 金库状态全局单例
pub(crate) static VAULT_STATE: std::sync::OnceLock<Mutex<VaultState>> = std::sync::OnceLock::new();

/// 获取金库状态
pub(crate) fn get_state() -> &'static Mutex<VaultState> {
    VAULT_STATE.get_or_init(|| Mutex::new(VaultState::default()))
}

// ========== 请求/响应结构 ==========

#[derive(Serialize, Deserialize)]
pub struct VaultMetadata {
    pub is_unlocked: bool,
    pub entry_count: usize,
    pub group_count: usize,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateVaultRequest {
    pub password: String,
    pub name: String,
    pub path: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UnlockVaultRequest {
    pub password: String,
    pub path: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct CreateEntryRequest {
    pub name: String,
    pub username: String,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub group_id: Option<String>,
    pub otp_auth_url: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct UpdateEntryRequest {
    pub id: String,
    pub name: String,
    pub username: String,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub group_id: Option<String>,
    pub otp_auth_url: Option<String>,
}

// ========== Vault 状态管理 ==========

pub mod state {
    use mypass_core::vault::{Entry, Vault};
    use std::path::PathBuf;

    /// 金库内存状态
    pub(crate) struct VaultState {
        pub vault: Option<Vault>,
        pub vault_path: Option<PathBuf>,
    }

    impl Default for VaultState {
        fn default() -> Self {
            Self {
                vault: None,
                vault_path: None,
            }
        }
    }

    impl VaultState {
        /// 是否已解锁
        pub fn is_unlocked(&self) -> bool {
            self.vault.is_some()
        }

        /// 条目数量
        pub fn entries_len(&self) -> usize {
            self.vault.as_ref().map(|v| v.list_entries().len()).unwrap_or(0)
        }

        /// 获取脱敏条目（用于浏览器扩展）
        /// 返回的条目包含密码，前端需要决定是否展示
        pub fn safe_entries(&self) -> Vec<Entry> {
            self.vault.as_ref()
                .map(|v| v.list_entries())
                .unwrap_or_default()
        }

        /// 添加条目（用于浏览器扩展捕获保存的凭据）
        pub fn add_entry(
            &mut self,
            name: String,
            username: String,
            password: String,
            url: Option<String>,
        ) -> Result<Entry, mypass_core::error::TauriError> {
            let vault = self.vault.as_mut()
                .ok_or(mypass_core::error::TauriError::Internal("Vault not unlocked".into()))?;
            let mut entry = Entry::new(name, username, password);
            if let Some(u) = url {
                entry = entry.with_url(u);
            }
            vault.add_entry(entry.clone())?;
            Ok(entry)
        }
    }
}

// ========== 命令实现 ==========

/// 创建新金库
#[tauri::command]
pub fn create_vault(request: CreateVaultRequest) -> Result<VaultMetadata, TauriError> {
    tracing::info!("Creating vault: {}", request.name);

    let vault_path = if let Some(path) = request.path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()
            .map_err(|e| TauriError::InvalidPath(e.to_string()))?
    };

    let vault = Vault::create(vault_path.clone(), &request.password, &request.name)
        .map_err(|e| TauriError::VaultCreateFailed(e.to_string()))?;

    let info = vault.get_info();

    let mut state = get_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;
    state.vault = Some(vault);
    state.vault_path = Some(vault_path);

    Ok(VaultMetadata {
        is_unlocked: true,
        entry_count: info.entry_count,
        group_count: info.group_count,
        name: info.name,
    })
}

/// 使用主密码解锁金库
#[tauri::command]
pub fn unlock_vault(request: UnlockVaultRequest) -> Result<VaultMetadata, TauriError> {
    tracing::info!("Unlocking vault");

    let vault_path = if let Some(path) = request.path {
        PathBuf::from(path)
    } else {
        let state = get_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;
        state.vault_path.clone()
            .ok_or(TauriError::VaultNotFound)?
    };

    let vault = Vault::unlock(vault_path.clone(), &request.password)
        .map_err(|e| TauriError::VaultUnlockFailed(e.to_string()))?;

    let info = vault.get_info();

    let mut state = get_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;
    state.vault = Some(vault);
    state.vault_path = Some(vault_path);

    Ok(VaultMetadata {
        is_unlocked: true,
        entry_count: info.entry_count,
        group_count: info.group_count,
        name: info.name,
    })
}

/// 锁定金库
#[tauri::command]
pub fn lock_vault() -> Result<(), TauriError> {
    tracing::info!("Locking vault");

    let mut state = get_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;

    if let Some(mut vault) = state.vault.take() {
        vault.lock().map_err(|e| TauriError::Internal(e.to_string()))?;
    }

    Ok(())
}

/// 获取金库信息
#[tauri::command]
pub fn get_vault_info() -> Result<VaultMetadata, TauriError> {
    let state = get_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;

    let vault = state.vault.as_ref().ok_or(TauriError::VaultLocked)?;

    let info = vault.get_info();

    Ok(VaultMetadata {
        is_unlocked: true,
        entry_count: info.entry_count,
        group_count: info.group_count,
        name: info.name,
    })
}

/// 获取所有条目
#[tauri::command]
pub fn get_entries() -> Result<Vec<Entry>, TauriError> {
    let state = get_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;

    let vault = state.vault.as_ref().ok_or(TauriError::VaultLocked)?;

    Ok(vault.list_entries())
}

/// 创建新条目
#[tauri::command]
pub fn create_entry(request: CreateEntryRequest) -> Result<Entry, TauriError> {
    tracing::info!("Creating entry: {}", request.name);

    let mut state = get_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;
    let vault = state.vault.as_mut().ok_or(TauriError::VaultLocked)?;

    let mut entry = Entry::new(
        request.name,
        request.username,
        request.password,
    );

    if let Some(url) = request.url {
        entry = entry.with_url(url);
    }
    if let Some(notes) = request.notes {
        entry = entry.with_notes(notes);
    }
    if let Some(otp_url) = request.otp_auth_url {
        entry = entry.with_otp(otp_url);
    }
    if let Some(group_id) = request.group_id {
        entry = entry.with_group_id(group_id);
    }

    vault.add_entry(entry.clone())
        .map_err(|e| TauriError::ObjectWriteFailed(e.to_string()))?;

    Ok(entry)
}

/// 更新条目
#[tauri::command]
pub fn update_entry(request: UpdateEntryRequest) -> Result<Entry, TauriError> {
    tracing::info!("Updating entry: {}", request.id);

    let mut state = get_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;
    let vault = state.vault.as_mut().ok_or(TauriError::VaultLocked)?;

    let mut entry = Entry::new(
        request.name,
        request.username,
        request.password,
    );

    if let Some(url) = request.url {
        entry = entry.with_url(url);
    }
    if let Some(notes) = request.notes {
        entry = entry.with_notes(notes);
    }
    if let Some(otp_url) = request.otp_auth_url {
        entry = entry.with_otp(otp_url);
    }
    if let Some(group_id) = request.group_id {
        entry = entry.with_group_id(group_id);
    }

    vault.update_entry(&request.id, entry.clone())
        .map_err(|e| TauriError::ObjectWriteFailed(e.to_string()))?;

    Ok(entry)
}

/// 删除条目
#[tauri::command]
pub fn delete_entry(id: String) -> Result<(), TauriError> {
    tracing::info!("Deleting entry: {}", id);

    let mut state = get_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;
    let vault = state.vault.as_mut().ok_or(TauriError::VaultLocked)?;

    vault.delete_entry(&id)
        .map_err(|e| TauriError::ObjectDeleteFailed(e.to_string()))?;

    Ok(())
}

/// 获取所有分组
#[tauri::command]
pub fn get_groups() -> Result<Vec<Group>, TauriError> {
    let state = get_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;

    let vault = state.vault.as_ref().ok_or(TauriError::VaultLocked)?;

    Ok(vault.list_groups())
}

/// 创建新分组
#[tauri::command]
pub fn create_group(name: String) -> Result<Group, TauriError> {
    tracing::info!("Creating group: {}", name);

    let mut state = get_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;
    let vault = state.vault.as_mut().ok_or(TauriError::VaultLocked)?;

    let group = Group::new(name);
    vault.add_group(group.clone())
        .map_err(|e| TauriError::ObjectWriteFailed(e.to_string()))?;

    Ok(group)
}

/// 删除分组
#[tauri::command]
pub fn delete_group(id: String) -> Result<(), TauriError> {
    tracing::info!("Deleting group: {}", id);

    let mut state = get_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;
    let vault = state.vault.as_mut().ok_or(TauriError::VaultLocked)?;

    vault.delete_group(&id)
        .map_err(|e| TauriError::ObjectDeleteFailed(e.to_string()))?;

    Ok(())
}

/// 搜索条目
#[tauri::command]
pub fn search_entries(query: String) -> Result<Vec<Entry>, TauriError> {
    let state = get_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;

    let vault = state.vault.as_ref().ok_or(TauriError::VaultLocked)?;

    Ok(vault.search_entries(&query))
}

/// 获取单个条目详情
#[tauri::command]
pub fn get_entry(id: String) -> Result<Option<Entry>, TauriError> {
    let state = get_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;

    let vault = state.vault.as_ref().ok_or(TauriError::VaultLocked)?;

    Ok(vault.get_entry(&id).cloned())
}

/// Vault 列表项信息
#[derive(Serialize, Deserialize)]
pub struct VaultListItem {
    pub name: String,
    pub path: String,
    pub entry_count: usize,
    pub group_count: usize,
    pub last_modified: i64,
}

/// 获取所有 Vault 列表
#[tauri::command]
pub fn list_vaults() -> Result<Vec<VaultListItem>, TauriError> {
    tracing::info!("Listing vaults");

    let mut vaults = Vec::new();

    // 搜索当前目录下的 .vault 目录
    let current_dir = std::env::current_dir()
        .map_err(|e| TauriError::InvalidPath(e.to_string()))?;

    if let Ok(entries) = std::fs::read_dir(&current_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.extension().map(|e| e == "vault").unwrap_or(false) {
                if let Ok(meta) = std::fs::metadata(&path) {
                    let last_modified = meta.modified()
                        .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64)
                        .unwrap_or(0);

                    let meta_path = path.join("vault.meta.json");
                    let entry_count = path.join("objects").read_dir().map(|d| d.count()).unwrap_or(0);

                    vaults.push(VaultListItem {
                        name: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
                        path: path.to_string_lossy().to_string(),
                        entry_count,
                        group_count: 0,
                        last_modified,
                    });
                }
            }
        }
    }

    Ok(vaults)
}
