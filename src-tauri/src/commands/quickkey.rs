//! QuickKey Tauri 命令

use crate::error::TauriError;
use mypass_core::auth::quickkey;
use serde::{Deserialize, Serialize};

static QUICKKEY_DATA: std::sync::OnceLock<std::sync::Mutex<Option<QuickKeyData>>> = std::sync::OnceLock::new();

struct QuickKeyData {
    quickkey: Vec<u8>,
    encrypted_mek: Vec<u8>,
    id: String,
}

fn get_quickkey_state() -> &'static std::sync::Mutex<Option<QuickKeyData>> {
    QUICKKEY_DATA.get_or_init(|| std::sync::Mutex::new(None))
}

#[derive(Serialize, Deserialize)]
pub struct EnableQuickKeyResponse {
    pub success: bool,
    pub quick_key_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct UnlockWithQuickKeyResponse {
    pub is_unlocked: bool,
    pub entry_count: usize,
    pub group_count: usize,
}

/// 启用 QuickKey（首次主密码解锁后调用）
#[tauri::command]
pub fn enable_quickkey(
    master_key: Vec<u8>,
    identifier: String,
) -> Result<EnableQuickKeyResponse, TauriError> {
    tracing::info!("Enabling QuickKey");

    // 1. 生成 QuickKey
    let qk = quickkey::quickkey_generate();

    // 2. 计算 QuickKey ID (SHA-256 前 16 字节)
    let quickkey_id = {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&qk);
        hex::encode(hasher.finalize())[..16].to_string()
    };

    // 3. 用 QuickKey 加密 MEK
    let encrypted_mek = quickkey::encrypt_mek_with_quickkey(&master_key, &qk)
        .map_err(|e| TauriError::EncryptionFailed(e.to_string()))?;

    // 4. 存储 QuickKey 到系统密钥库（文件回退）
    // 在 async 上下文之外运行 tokio runtime
    let qk_clone = qk.clone();
    let id_clone = identifier.clone();
    let _ = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        if let Ok(rt) = rt {
            let _ = rt.block_on(quickkey::quickkey_store(&qk_clone, &id_clone));
        }
    }).join();

    // 5. 存储到内存（用于当前会话）
    let mut state = get_quickkey_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;
    *state = Some(QuickKeyData {
        quickkey: qk,
        encrypted_mek,
        id: quickkey_id.clone(),
    });

    Ok(EnableQuickKeyResponse {
        success: true,
        quick_key_id: quickkey_id,
    })
}

/// 使用 QuickKey 解锁
#[tauri::command]
pub fn unlock_with_quickkey(quick_key_id: String) -> Result<UnlockWithQuickKeyResponse, TauriError> {
    tracing::info!("Unlocking with QuickKey: {}", quick_key_id);

    let state = get_quickkey_state().lock().map_err(|e| TauriError::Internal(e.to_string()))?;

    if let Some(data) = state.as_ref() {
        if data.id != quick_key_id {
            return Err(TauriError::KeychainNotConfigured);
        }

        // 用 QuickKey 解密获取 MEK
        let _mek = quickkey::decrypt_mek_with_quickkey(&data.encrypted_mek, &data.quickkey)
            .map_err(|e| TauriError::DecryptionFailed(e.to_string()))?;

        // 实际解锁 Vault 需要从当前 vault state 获取，这里返回 0 占位
        // 完整集成需要 uniffi 或共享内存
        Ok(UnlockWithQuickKeyResponse {
            is_unlocked: true,
            entry_count: 0,
            group_count: 0,
        })
    } else {
        Err(TauriError::KeychainNotConfigured)
    }
}

/// 检查 QuickKey 是否已启用
#[tauri::command]
pub fn is_quickkey_enabled() -> bool {
    get_quickkey_state()
        .lock()
        .map(|state| state.is_some())
        .unwrap_or(false)
}

/// 获取 QuickKey ID（如果有）
#[tauri::command]
pub fn get_quickkey_id() -> Option<String> {
    get_quickkey_state()
        .lock()
        .ok()
        .and_then(|state| state.as_ref().map(|data| data.id.clone()))
}
