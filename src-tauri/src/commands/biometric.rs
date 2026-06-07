//! 生物识别 Tauri 命令
//!
//! 使用 tauri-plugin-biometric 实现指纹/面部识别解锁
//!
//! 当前实现：返回模拟值，等待接入实际插件

use crate::error::TauriError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BiometricStatus {
    pub available: bool,
    pub biometry_type: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct AuthenticateResponse {
    pub success: bool,
    pub error: Option<String>,
}

/// 检查生物识别是否可用
#[tauri::command]
pub async fn check_biometric_available() -> Result<BiometricStatus, TauriError> {
    // TODO: 集成 tauri-plugin-biometric
    // 临时返回基于 OS 的默认值
    let (available, biometry_type) = platform_default_biometry();
    Ok(BiometricStatus {
        available,
        biometry_type,
    })
}

/// 触发生物识别认证
#[tauri::command]
pub async fn authenticate_biometric(reason: String) -> Result<AuthenticateResponse, TauriError> {
    tracing::info!("Biometric authentication requested: {}", reason);

    // TODO: 集成实际生物识别 API
    // 当前仅记录日志，需要用户在 UI 中确认
    Ok(AuthenticateResponse {
        success: true,
        error: None,
    })
}

/// 获取支持的生物识别类型
#[tauri::command]
pub fn get_biometry_type() -> Option<String> {
    platform_default_biometry().1
}

fn platform_default_biometry() -> (bool, Option<String>) {
    #[cfg(target_os = "windows")]
    { (true, Some("windows_hello".to_string())) }
    #[cfg(target_os = "macos")]
    { (true, Some("touchid".to_string())) }
    #[cfg(target_os = "ios")]
    { (true, Some("faceid".to_string())) }
    #[cfg(target_os = "android")]
    { (true, Some("fingerprint".to_string())) }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "ios", target_os = "android")))]
    { (false, None) }
}
