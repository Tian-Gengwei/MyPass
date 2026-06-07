//! 浏览器扩展通信命令模块
//!
//! # 概述
//! 提供与浏览器扩展的通信接口，支持自动填充和凭证保存。

use crate::commands::vault::get_state;
use crate::error::TauriError;

/// 检查金库状态
#[derive(serde::Serialize)]
pub struct VaultStatusResponse {
    pub is_unlocked: bool,
    pub entry_count: usize,
}

#[tauri::command]
pub fn vault_status() -> Result<VaultStatusResponse, TauriError> {
    let vault_state = get_state()
        .lock()
        .map_err(|e| TauriError::Internal(e.to_string()))?;

    Ok(VaultStatusResponse {
        is_unlocked: vault_state.is_unlocked(),
        entry_count: vault_state.entries_len(),
    })
}

/// 获取扩展可见的条目（脱敏版，不包含明文密码）
#[derive(serde::Serialize)]
pub struct SafeEntry {
    pub id: String,
    pub name: String,
    pub username: String,
    pub url: Option<String>,
}

#[tauri::command]
pub fn get_extension_entries() -> Result<Vec<SafeEntry>, TauriError> {
    let vault_state = get_state()
        .lock()
        .map_err(|e| TauriError::Internal(e.to_string()))?;

    if !vault_state.is_unlocked() {
        return Err(TauriError::VaultLocked);
    }

    let entries: Vec<SafeEntry> = vault_state
        .safe_entries()
        .into_iter()
        .map(|e| SafeEntry {
            id: e.id,
            name: e.name,
            username: e.username,
            url: e.url,
        })
        .collect();

    Ok(entries)
}

/// 保存凭证（从扩展调用）
#[tauri::command]
pub fn extension_save_credential(
    name: String,
    username: String,
    password: String,
    url: Option<String>,
) -> Result<crate::commands::vault::CreateEntryRequest, TauriError> {
    let mut vault_state = get_state()
        .lock()
        .map_err(|e| TauriError::Internal(e.to_string()))?;

    if !vault_state.is_unlocked() {
        return Err(TauriError::VaultLocked);
    }

    let entry = vault_state
        .add_entry(name.clone(), username, password, url)
        .map_err(|e| TauriError::ObjectWriteFailed(e.to_string()))?;

    // 返回创建结果，让扩展可以确认成功
    Ok(crate::commands::vault::CreateEntryRequest {
        name,
        username: entry.username,
        password: entry.password,
        url: entry.url,
        notes: entry.notes,
        group_id: entry.group_id,
        otp_auth_url: entry.otp_auth_url,
    })
}
