//! 生物识别接口
//!
//! 提供跨平台的生物识别认证，支持：
//! - Windows: Windows Hello (指纹/面部)
//! - macOS: Touch ID
//! - iOS: Face ID / Touch ID
//! - Android: BiometricPrompt
//!
//! ## 后续接入方式
//!
//! ```ignore
//! // 方式 1: tauri-plugin-biometric (推荐跨平台方案)
//! use tauri_plugin_biometric::Biometric;
//!
//! let biometric = Biometric::new();
//! let result = biometric.authenticate("Verify your identity").await?;
//!
//! // 方式 2: 平台原生
//! // Windows: Windows.Security.Credentials.UIbirAuthenticator
//! // macOS/iOS: LocalAuthentication
//! // Android: androidx.biometric
//! ```

use crate::error::TauriError;

/// 生物识别认证
///
/// # Arguments
/// * `reason` - 认证原因提示
///
/// # Returns
/// * `Result<bool>` - 认证是否成功
pub async fn biometric_authenticate(_reason: &str) -> Result<bool, TauriError> {
    // TODO: 接入 tauri-plugin-biometric
    //
    // 示例实现（待完成）：
    // ```ignore
    // use tauri_plugin_biometric::Biometric;
    //
    // let biometric = Biometric::new();
    // match biometric.authenticate(reason).await {
    //     Ok(result) => Ok(result.success),
    //     Err(e) => Err(TauriError::BiometricFailed(e.to_string()))
    // }
    // ```
    Err(TauriError::Unimplemented(
        "biometric_authenticate: tauri-plugin-biometric integration pending".into()
    ))
}

/// 检查生物识别是否可用
pub async fn biometric_is_available() -> Result<bool, TauriError> {
    // TODO: 接入 tauri-plugin-biometric
    Err(TauriError::Unimplemented(
        "biometric_is_available: tauri-plugin-biometric integration pending".into()
    ))
}

/// 获取支持的生物识别类型
pub async fn biometric_get_type() -> Result<BiometricType, TauriError> {
    // TODO: 接入 tauri-plugin-biometric
    Err(TauriError::Unimplemented(
        "biometric_get_type: tauri-plugin-biometric integration pending".into()
    ))
}

/// 生物识别类型
#[derive(Debug, Clone, PartialEq)]
pub enum BiometricType {
    None,
    Fingerprint,
    Face,
    Iris,
    Multiple,
}