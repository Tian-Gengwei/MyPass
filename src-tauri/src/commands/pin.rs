//! PIN 码 Tauri 命令

use crate::error::TauriError;
use mypass_core::auth::pin::PinState as PinManager;
use std::sync::Mutex;
use tauri::State;

/// PIN 管理器状态
pub struct PinManagerState(pub Mutex<PinManager>);

impl Default for PinManagerState {
    fn default() -> Self {
        Self(Mutex::new(PinManager::new()))
    }
}

#[derive(serde::Serialize)]
pub struct SetPinResponse {
    pub success: bool,
    pub message: String,
}

#[derive(serde::Serialize)]
pub struct VerifyPinResponse {
    pub valid: bool,
    pub locked: bool,
    pub remaining_attempts: Option<u32>,
    pub lockout_remaining_secs: Option<i64>,
}

/// 设置或更改 PIN 码
#[tauri::command]
pub fn set_pin(pin: String, state: State<PinManagerState>) -> Result<SetPinResponse, TauriError> {
    let manager = state.0.lock().map_err(|e| TauriError::Internal(e.to_string()))?;
    manager.set_pin(&pin)
        .map_err(|e| TauriError::Internal(e.to_string()))?;

    Ok(SetPinResponse {
        success: true,
        message: "PIN set successfully".to_string(),
    })
}

/// 验证 PIN 码
#[tauri::command]
pub fn verify_pin(pin: String, state: State<PinManagerState>) -> Result<VerifyPinResponse, TauriError> {
    let manager = state.0.lock().map_err(|e| TauriError::Internal(e.to_string()))?;

    if manager.is_locked() {
        return Ok(VerifyPinResponse {
            valid: false,
            locked: true,
            remaining_attempts: Some(0),
            lockout_remaining_secs: manager.lockout_remaining_secs(),
        });
    }

    let valid = manager.verify(&pin)
        .map_err(|e| TauriError::Internal(e.to_string()))?;

    Ok(VerifyPinResponse {
        valid,
        locked: false,
        remaining_attempts: Some(manager.remaining_attempts()),
        lockout_remaining_secs: None,
    })
}

/// 检查是否已设置 PIN
#[tauri::command]
pub fn is_pin_set(state: State<PinManagerState>) -> Result<bool, TauriError> {
    let manager = state.0.lock().map_err(|e| TauriError::Internal(e.to_string()))?;
    Ok(manager.is_set())
}
