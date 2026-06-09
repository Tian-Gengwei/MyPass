//! 导入命令模块
//!
//! 提供 KeePass KDBX / Bitwarden JSON / Chrome CSV 导入功能

use crate::commands::vault::get_state;
use crate::error::TauriError;
use mypass_core::import::{BitwardenImporter, BitwardenCsvImporter, ChromeCsvImporter, Importer as _, KeepassImporter};

/// 导入 KeePass KDBX 文件
#[tauri::command]
pub async fn import_keepass(file_path: String) -> Result<usize, TauriError> {
    tracing::info!("Importing KeePass file: {}", file_path);

    let importer = KeepassImporter::new();
    let data = std::fs::read(&file_path)
        .map_err(|e| TauriError::ObjectReadFailed(format!("{}: {}", file_path, e)))?;

    if !importer.can_import(&data) {
        return Err(TauriError::ImportFormatNotSupported("KeePass KDBX".into()));
    }

    let result = importer.import(&data)
        .map_err(|e| TauriError::ImportDataInvalid(e.to_string()))?;

    // 自动把导入的条目加入当前 vault
    let mut state = get_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;
    let vault = state.vault.as_mut().ok_or(TauriError::VaultLocked)?;
    let count = result.entries.len();
    for entry in result.entries {
        if let Err(e) = vault.add_entry(entry) {
            tracing::warn!("Failed to add imported entry: {}", e);
        }
    }

    tracing::info!("Imported {} entries", count);
    Ok(count)
}

/// 导入 Bitwarden JSON 文件
#[tauri::command]
pub async fn import_bitwarden(file_path: String) -> Result<usize, TauriError> {
    tracing::info!("Importing Bitwarden file: {}", file_path);

    let importer = BitwardenImporter::new();
    let data = std::fs::read(&file_path)
        .map_err(|e| TauriError::ObjectReadFailed(format!("{}: {}", file_path, e)))?;

    if !importer.can_import(&data) {
        return Err(TauriError::ImportFormatNotSupported("Bitwarden JSON".into()));
    }

    let result = importer.import(&data)
        .map_err(|e| TauriError::ImportDataInvalid(e.to_string()))?;

    let mut state = get_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;
    let vault = state.vault.as_mut().ok_or(TauriError::VaultLocked)?;
    let count = result.entries.len();
    for entry in result.entries {
        if let Err(e) = vault.add_entry(entry) {
            tracing::warn!("Failed to add imported entry: {}", e);
        }
    }

    tracing::info!("Imported {} entries", count);
    Ok(count)
}

/// 导入 Bitwarden CSV 文件
#[tauri::command]
pub async fn import_bitwarden_csv(file_path: String) -> Result<usize, TauriError> {
    tracing::info!("Importing Bitwarden CSV file: {}", file_path);

    let importer = BitwardenCsvImporter::new();
    let data = std::fs::read(&file_path)
        .map_err(|e| TauriError::ObjectReadFailed(format!("{}: {}", file_path, e)))?;

    if !importer.can_import(&data) {
        return Err(TauriError::ImportFormatNotSupported("Bitwarden CSV".into()));
    }

    let result = importer.import(&data)
        .map_err(|e| TauriError::ImportDataInvalid(e.to_string()))?;

    let mut state = get_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;
    let vault = state.vault.as_mut().ok_or(TauriError::VaultLocked)?;
    let count = result.entries.len();
    for entry in result.entries {
        if let Err(e) = vault.add_entry(entry) {
            tracing::warn!("Failed to add imported entry: {}", e);
        }
    }

    tracing::info!("Imported {} entries from Bitwarden CSV", count);
    Ok(count)
}

/// 导入 Chrome CSV 文件
#[tauri::command]
pub async fn import_chrome_csv(file_path: String) -> Result<usize, TauriError> {
    tracing::info!("Importing Chrome CSV file: {}", file_path);

    let importer = ChromeCsvImporter::new();
    let data = std::fs::read(&file_path)
        .map_err(|e| TauriError::ObjectReadFailed(format!("{}: {}", file_path, e)))?;

    if !importer.can_import(&data) {
        return Err(TauriError::ImportFormatNotSupported("Chrome CSV".into()));
    }

    let result = importer.import(&data)
        .map_err(|e| TauriError::ImportDataInvalid(e.to_string()))?;

    let mut state = get_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;
    let vault = state.vault.as_mut().ok_or(TauriError::VaultLocked)?;
    let count = result.entries.len();
    for entry in result.entries {
        if let Err(e) = vault.add_entry(entry) {
            tracing::warn!("Failed to add imported entry: {}", e);
        }
    }

    tracing::info!("Imported {} entries from Chrome CSV", count);
    Ok(count)
}

/// 获取支持的导入格式
#[tauri::command]
pub fn get_supported_import_formats() -> Vec<String> {
    vec![
        "KeePass KDBX".to_string(),
        "Bitwarden JSON".to_string(),
        "Bitwarden CSV".to_string(),
        "Chrome CSV".to_string(),
    ]
}
