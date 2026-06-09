//! 应用全局设置模块
//!
//! 负责持久化用户偏好（默认 vault 存储位置、自动锁、语言等）。
//! 设置文件存放于 `${app_local_data_dir}/settings.json`，跨平台一致。

use crate::error::TauriError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

/// 应用全局设置
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AppSettings {
    /// 用户自定义的默认 vault 存储目录（绝对路径）。
    /// `None` 表示使用软件内置默认值（`${app_local_data_dir}/Vaults`）。
    pub default_vault_dir: Option<String>,

    /// 自动锁定超时时间（秒），0 表示不自动锁定。
    pub auto_lock_timeout: Option<u64>,

    /// 界面语言（`zh` / `en`）。
    pub language: Option<String>,

    /// 是否启用生物识别解锁。
    pub biometrics_enabled: Option<bool>,
}

/// 设置状态（内存缓存 + 落盘）
pub struct AppSettingsState(pub Mutex<AppSettings>);

impl Default for AppSettingsState {
    fn default() -> Self {
        Self(Mutex::new(AppSettings::default()))
    }
}

/// 获取设置文件的完整路径
fn settings_file_path(app: &AppHandle) -> Result<PathBuf, TauriError> {
    let app_data = app
        .path()
        .app_local_data_dir()
        .map_err(|e| TauriError::FileOperationFailed(e.to_string()))?;
    Ok(app_data.join("settings.json"))
}

/// 从磁盘加载设置
pub fn load_settings_from_disk(app: &AppHandle) -> AppSettings {
    let path = match settings_file_path(app) {
        Ok(p) => p,
        Err(_) => return AppSettings::default(),
    };
    if !path.exists() {
        return AppSettings::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => AppSettings::default(),
    }
}

/// 将设置保存到磁盘
pub fn save_settings_to_disk(app: &AppHandle, settings: &AppSettings) -> Result<(), TauriError> {
    let path = settings_file_path(app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| TauriError::FileOperationFailed(e.to_string()))?;
    }
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| TauriError::Internal(e.to_string()))?;
    std::fs::write(&path, content)
        .map_err(|e| TauriError::FileOperationFailed(e.to_string()))?;
    Ok(())
}

/// 计算默认 vault 存储目录（软件内置）
fn compute_builtin_default_dir(app: &AppHandle) -> Result<PathBuf, TauriError> {
    let app_data = app
        .path()
        .app_local_data_dir()
        .map_err(|e| TauriError::FileOperationFailed(e.to_string()))?;
    Ok(app_data.join("Vaults"))
}

/// 解析当前生效的 vault 存储目录
pub fn resolve_default_vault_dir(app: &AppHandle) -> Result<PathBuf, TauriError> {
    let state = app.state::<AppSettingsState>();
    let settings = state
        .0
        .lock()
        .map_err(|e| TauriError::Internal(e.to_string()))?;

    if let Some(dir) = &settings.default_vault_dir {
        if !dir.is_empty() {
            return Ok(PathBuf::from(dir));
        }
    }

    compute_builtin_default_dir(app)
}

/// 获取当前默认 vault 存储目录
#[tauri::command]
pub fn get_default_vault_dir(app: AppHandle) -> Result<String, TauriError> {
    Ok(resolve_default_vault_dir(&app)?.to_string_lossy().to_string())
}

/// 获取软件内置的默认 vault 存储目录
#[tauri::command]
pub fn get_builtin_vault_dir(app: AppHandle) -> Result<String, TauriError> {
    Ok(compute_builtin_default_dir(&app)?.to_string_lossy().to_string())
}

/// 设置用户自定义默认 vault 存储目录
#[tauri::command]
pub fn set_default_vault_dir(
    path: String,
    app: AppHandle,
    state: State<'_, AppSettingsState>,
) -> Result<String, TauriError> {
    let trimmed = path.trim().to_string();
    if trimmed.is_empty() {
        return Err(TauriError::InvalidArgument(
            "Default vault directory cannot be empty".to_string(),
        ));
    }

    let dir = PathBuf::from(&trimmed);
    if !dir.is_absolute() {
        return Err(TauriError::InvalidPath(format!(
            "Default vault directory must be an absolute path: {}",
            trimmed
        )));
    }

    // 确保目录存在
    std::fs::create_dir_all(&dir)
        .map_err(|e| TauriError::FileOperationFailed(e.to_string()))?;

    {
        let mut settings = state
            .0
            .lock()
            .map_err(|e| TauriError::Internal(e.to_string()))?;
        settings.default_vault_dir = Some(trimmed.clone());
        save_settings_to_disk(&app, &settings)?;
    }

    Ok(trimmed)
}

/// 重置默认 vault 存储目录为软件内置默认值
#[tauri::command]
pub fn reset_default_vault_dir(
    app: AppHandle,
    state: State<'_, AppSettingsState>,
) -> Result<String, TauriError> {
    {
        let mut settings = state
            .0
            .lock()
            .map_err(|e| TauriError::Internal(e.to_string()))?;
        settings.default_vault_dir = None;
        save_settings_to_disk(&app, &settings)?;
    }

    let builtin = compute_builtin_default_dir(&app)?;
    std::fs::create_dir_all(&builtin)
        .map_err(|e| TauriError::FileOperationFailed(e.to_string()))?;
    Ok(builtin.to_string_lossy().to_string())
}

/// 获取完整应用设置
#[tauri::command]
pub fn get_app_settings(state: State<'_, AppSettingsState>) -> Result<AppSettings, TauriError> {
    let settings = state
        .0
        .lock()
        .map_err(|e| TauriError::Internal(e.to_string()))?;
    Ok(settings.clone())
}

/// 校验并规范化 vault 名称（去除不安全字符）
pub fn sanitize_vault_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// 解析 vault 完整路径：
/// 1. 显式传入 `path` 时直接使用
/// 2. 否则使用 `${default_dir}/<sanitized_name>.vault`
pub fn resolve_vault_path(
    app: &AppHandle,
    explicit_path: Option<&str>,
    vault_name: &str,
) -> Result<PathBuf, TauriError> {
    if let Some(p) = explicit_path {
        if !p.is_empty() {
            return Ok(PathBuf::from(p));
        }
    }

    let default_dir = resolve_default_vault_dir(app)?;
    std::fs::create_dir_all(&default_dir)
        .map_err(|e| TauriError::FileOperationFailed(e.to_string()))?;

    let sanitized = sanitize_vault_name(vault_name);
    let final_name = if sanitized.is_empty() {
        "vault".to_string()
    } else {
        sanitized
    };

    Ok(default_dir.join(format!("{}.vault", final_name)))
}

/// 用于扫描 vault 列表的目录：
/// 1. 当前生效的默认目录
/// 2. 软件内置默认目录（防止用户改路径后旧 vault 仍然可见）
/// 3. 当前工作目录（向后兼容）
pub fn resolve_vault_dir_for_listing(app: &AppHandle) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();

    if let Ok(d) = resolve_default_vault_dir(app) {
        dirs.push(d);
    }
    if let Ok(d) = compute_builtin_default_dir(app) {
        if !dirs.contains(&d) {
            dirs.push(d);
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        if !dirs.contains(&cwd) {
            dirs.push(cwd);
        }
    }

    dirs
}

/// 弹出系统文件夹选择器，让用户挑选 vault 存储目录
#[tauri::command]
pub async fn pick_vault_folder(app: AppHandle) -> Result<Option<String>, TauriError> {
    // 必须在独立线程中执行同步阻塞调用，避免阻塞 tokio runtime
    let result = tokio::task::spawn_blocking(move || {
        let folder = app
            .dialog()
            .file()
            .set_title("Select vault storage folder")
            .blocking_pick_folder();

        folder.and_then(|fp| fp.into_path().ok())
    })
    .await
    .map_err(|e| TauriError::Internal(e.to_string()))?;

    Ok(result.map(|p| p.to_string_lossy().to_string()))
}
