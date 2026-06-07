//! 同步命令模块
//!
//! 提供 WebDAV 同步功能（S3 接口已预留）

use crate::error::TauriError;
use mypass_core::sync::webdav::{WebDavConfig, WebDavSync};
use mypass_core::vault::manifest::Manifest;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SyncStatus {
    pub last_sync: Option<i64>,
    pub pending_changes: usize,
    pub is_syncing: bool,
    pub conflicts: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SyncConfig {
    pub sync_type: SyncType,
    pub endpoint: String,
    pub credentials: Option<SyncCredentials>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum SyncType {
    WebDav,
    S3,
}

#[derive(Serialize, Deserialize)]
pub struct SyncCredentials {
    pub username: Option<String>,
    pub password: Option<String>,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
}

#[derive(Default)]
struct SyncState {
    last_sync: Option<i64>,
    config: Option<SyncConfig>,
}

static SYNC_STATE: std::sync::OnceLock<std::sync::Mutex<SyncState>> = std::sync::OnceLock::new();

fn sync_state() -> &'static std::sync::Mutex<SyncState> {
    SYNC_STATE.get_or_init(|| std::sync::Mutex::new(SyncState::default()))
}

/// 触发同步
#[tauri::command]
pub async fn sync_vault(
    config: SyncConfig,
    local_manifest: Manifest,
) -> Result<SyncStatus, TauriError> {
    tracing::info!("Syncing vault");

    let result = match config.sync_type {
        SyncType::WebDav => {
            let creds = config.credentials.as_ref()
                .ok_or(TauriError::InvalidArgument("Missing credentials".into()))?;
            let username = creds.username.clone().unwrap_or_default();
            let password = creds.password.clone().unwrap_or_default();

            let webdav_config = WebDavConfig {
                endpoint: config.endpoint.clone(),
                username,
                password,
                vault_name: "default".to_string(),
            };
            let sync = WebDavSync::new(webdav_config);
            let mek = b""; // TODO: 实际从 vault state 获取 MEK
            sync.sync(&local_manifest, mek, |_hash| Ok(Vec::new())).await?
        }
        SyncType::S3 => {
            return Err(TauriError::Unimplemented("S3 sync is planned for future".into()));
        }
    };

    let mut state = sync_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;
    state.last_sync = Some(result.timestamp);

    Ok(SyncStatus {
        last_sync: Some(result.timestamp),
        pending_changes: 0,
        is_syncing: false,
        conflicts: Vec::new(),
    })
}

/// 获取同步状态
#[tauri::command]
pub async fn get_sync_status() -> Result<SyncStatus, TauriError> {
    let state = sync_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;

    Ok(SyncStatus {
        last_sync: state.last_sync,
        pending_changes: 0,
        is_syncing: false,
        conflicts: Vec::new(),
    })
}

/// 配置同步
#[tauri::command]
pub async fn configure_sync(config: SyncConfig) -> Result<(), TauriError> {
    let mut state = sync_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;
    state.config = Some(config);
    Ok(())
}

/// 测试同步连接
#[tauri::command]
pub async fn test_sync_connection(config: SyncConfig) -> Result<bool, TauriError> {
    match config.sync_type {
        SyncType::WebDav => {
            let creds = config.credentials.as_ref()
                .ok_or(TauriError::InvalidArgument("Missing credentials".into()))?;
            let username = creds.username.clone().unwrap_or_default();
            let password = creds.password.clone().unwrap_or_default();

            let webdav_config = WebDavConfig {
                endpoint: config.endpoint,
                username,
                password,
                vault_name: "default".to_string(),
            };
            let sync = WebDavSync::new(webdav_config);
            Ok(sync.test_connection().await?)
        }
        SyncType::S3 => Err(TauriError::Unimplemented("S3 sync is planned for future".into())),
    }
}
