//! 安全命令模块
//!
//! 提供锁定、超时等安全相关命令

use crate::error::TauriError;
use std::sync::atomic::{AtomicU64, Ordering};

/// 锁定状态
pub struct LockState {
    /// 自动锁定超时（秒），0 表示不自动锁定
    pub auto_lock_timeout: AtomicU64,
    /// 最后活动时间戳
    pub last_activity: AtomicU64,
}

impl LockState {
    pub fn new() -> Self {
        Self {
            auto_lock_timeout: AtomicU64::new(300), // 默认 5 分钟
            last_activity: AtomicU64::new(current_timestamp() as u64),
        }
    }

    /// 更新活动时间
    pub fn touch(&self) {
        self.last_activity.store(current_timestamp() as u64, Ordering::SeqCst);
    }

    /// 检查是否应该自动锁定
    #[allow(dead_code)]
    pub fn should_auto_lock(&self) -> bool {
        let timeout = self.auto_lock_timeout.load(Ordering::SeqCst);
        if timeout == 0 {
            return false;
        }
        let last = self.last_activity.load(Ordering::SeqCst) as i64;
        let now = current_timestamp();
        now - last > timeout as i64
    }

    pub fn set_auto_lock_timeout(&self, seconds: u64) {
        self.auto_lock_timeout.store(seconds, Ordering::SeqCst);
    }

    pub fn get_auto_lock_timeout(&self) -> u64 {
        self.auto_lock_timeout.load(Ordering::SeqCst)
    }
}

impl Default for LockState {
    fn default() -> Self {
        Self::new()
    }
}

static LOCK_STATE: std::sync::OnceLock<LockState> = std::sync::OnceLock::new();

fn lock_state() -> &'static LockState {
    LOCK_STATE.get_or_init(LockState::new)
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// 设置自动锁定超时时间
#[tauri::command]
pub fn set_auto_lock_timeout(seconds: u64) -> Result<(), TauriError> {
    tracing::info!("Setting auto-lock timeout to {} seconds", seconds);
    lock_state().set_auto_lock_timeout(seconds);
    Ok(())
}

/// 获取自动锁定超时时间
#[tauri::command]
pub fn get_auto_lock_timeout() -> Result<u64, TauriError> {
    Ok(lock_state().get_auto_lock_timeout())
}

/// 保持会话活跃
#[tauri::command]
pub fn keep_alive() -> Result<(), TauriError> {
    lock_state().touch();
    Ok(())
}

/// 获取会话状态
#[tauri::command]
pub fn get_session_status() -> Result<SessionStatus, TauriError> {
    Ok(SessionStatus {
        is_unlocked: true,
        auto_lock_timeout: lock_state().get_auto_lock_timeout(),
        last_activity: lock_state().last_activity.load(Ordering::SeqCst) as i64,
    })
}

#[derive(serde::Serialize)]
pub struct SessionStatus {
    pub is_unlocked: bool,
    pub auto_lock_timeout: u64,
    pub last_activity: i64,
}
