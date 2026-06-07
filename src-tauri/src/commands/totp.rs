//! TOTP 命令模块
//!
//! 提供 TOTP 生成与验证

use crate::error::TauriError;
use mypass_core::otp::{TotpManager, parse_totp_url};

/// 生成 TOTP 验证码
///
/// # Arguments
/// * `secret` - TOTP 密钥（Base64 编码）
///
/// # Returns
/// * 包含6 位验证码和剩余秒数的对象
#[tauri::command]
pub fn generate_totp(secret: String) -> Result<TotpCode, TauriError> {
    let code = TotpManager::generate(&secret)
        .map_err(|e| TauriError::Internal(e.to_string()))?;

    Ok(TotpCode {
        code: code.code,
        remaining_secs: code.remaining_secs,
    })
}

/// 验证 TOTP 验证码
///
/// # Arguments
/// * `secret` - TOTP 密钥
/// * `code` - 用户输入的 6 位验证码
///
/// # Returns
/// * 验证是否成功
#[tauri::command]
pub fn verify_totp(secret: String, code: String) -> Result<bool, TauriError> {
    let valid = TotpManager::verify(&secret, &code)
        .map_err(|e| TauriError::Internal(e.to_string()))?;

    Ok(valid)
}

/// 从 otpauth:// URL 解析 TOTP 密钥
///
/// # Arguments
/// * `url` - otpauth:// URL
///
/// # Returns
/// * (账户名, 密钥) 元组
#[tauri::command]
pub fn parse_totp_url_command(url: String) -> Result<(String, String), TauriError> {
    let (account, secret) = parse_totp_url(&url)
        .map_err(|e| TauriError::Internal(e.to_string()))?;

    Ok((account.to_string(), secret.to_string()))
}

#[derive(serde::Serialize)]
pub struct TotpCode {
    pub code: String,
    pub remaining_secs: u32,
}