//! 导出命令模块
//!
//! 提供 JSON / CSV / Bitwarden JSON 导出功能

use crate::commands::vault::get_state;
use crate::error::TauriError;
use mypass_core::export::{CsvExporter, Exporter, JsonExporter};

/// 导出为 CSV 格式
#[tauri::command]
pub async fn export_csv(file_path: String) -> Result<(), TauriError> {
    tracing::info!("Exporting to CSV: {}", file_path);

    let state = get_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;
    let vault = state.vault.as_ref().ok_or(TauriError::VaultLocked)?;
    
    let entries = vault.list_entries();
    let exporter = CsvExporter::new();
    let data = exporter.export(&entries);
    
    std::fs::write(&file_path, data)
        .map_err(|e| TauriError::ObjectWriteFailed(format!("{}: {}", file_path, e)))?;
    
    tracing::info!("Exported {} entries to CSV", entries.len());
    Ok(())
}

/// 导出为 JSON 格式
#[tauri::command]
pub async fn export_json(file_path: String) -> Result<(), TauriError> {
    tracing::info!("Exporting to JSON: {}", file_path);

    let state = get_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;
    let vault = state.vault.as_ref().ok_or(TauriError::VaultLocked)?;
    
    let entries = vault.list_entries();
    let exporter = JsonExporter::new();
    let data = exporter.export(&entries);
    
    std::fs::write(&file_path, data)
        .map_err(|e| TauriError::ObjectWriteFailed(format!("{}: {}", file_path, e)))?;
    
    tracing::info!("Exported {} entries to JSON", entries.len());
    Ok(())
}

/// 获取支持的导出格式
#[tauri::command]
pub fn get_supported_export_formats() -> Vec<String> {
    vec![
        "CSV".to_string(),
        "JSON".to_string(),
        "Bitwarden JSON".to_string(),
    ]
}
