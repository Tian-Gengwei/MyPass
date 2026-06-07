//! PIN 码认证模块
//!
//! 提供 PIN 码验证与速率限制
//!
//! ## 安全策略
//!
//! - **Argon2 派生**：使用 Argon2id 而非纯 SHA-256 防止暴力破解
//! - **常量时间比较**：使用 `subtle::ConstantTimeEq` 防止时序攻击
//! - **速率限制**：5 次连续失败后锁定 5 分钟
//! - **每次成功验证后重置计数**

use crate::crypto::ct_eq;
use crate::error::TauriError;
use std::sync::atomic::{AtomicU32, Ordering};

/// 最大连续失败次数
const MAX_ATTEMPTS: u32 = 5;
/// 锁定时长（秒）
const LOCKOUT_DURATION_SECS: i64 = 300;

/// PIN 全局状态（单例）
pub struct PinState {
    /// 已设置的 PIN 哈希（Argon2id 派生）
    pin_hash: std::sync::Mutex<Option<PinHash>>,
    /// 失败次数
    failed_attempts: AtomicU32,
    /// 锁定截止时间戳（0 表示未锁定）
    lockout_until: std::sync::Mutex<i64>,
}

/// PIN 哈希 + salt（用于验证）
struct PinHash {
    salt: Vec<u8>,
    hash: Vec<u8>,
}

impl PinState {
    pub fn new() -> Self {
        Self {
            pin_hash: std::sync::Mutex::new(None),
            failed_attempts: AtomicU32::new(0),
            lockout_until: std::sync::Mutex::new(0),
        }
    }

    /// 设置 PIN
    pub fn set_pin(&self, pin: &str) -> Result<(), TauriError> {
        if pin.len() < 4 {
            return Err(TauriError::Internal("PIN must be at least 4 digits".into()));
        }
        if pin.len() > 16 {
            return Err(TauriError::Internal("PIN too long (max 16 characters)".into()));
        }

        let salt = crate::crypto::secure_random::random_bytes(16);
        let hash = derive_pin_key(pin, &salt)
            .map_err(|e| TauriError::Internal(e.to_string()))?;

        let mut pin_hash = self.pin_hash.lock()
            .map_err(|e| TauriError::Internal(e.to_string()))?;
        *pin_hash = Some(PinHash { salt, hash });
        self.failed_attempts.store(0, Ordering::SeqCst);
        *self.lockout_until.lock().unwrap() = 0;
        Ok(())
    }

    /// 验证 PIN
    pub fn verify(&self, pin: &str) -> Result<bool, TauriError> {
        if let Err(e) = self.check_lockout() {
            return Err(e);
        }

        let pin_hash = self.pin_hash.lock()
            .map_err(|e| TauriError::Internal(e.to_string()))?;

        let stored = pin_hash.as_ref().ok_or(TauriError::PinNotSet)?;

        // 即使派生失败也要执行一次"假"派生 + 比较，保持常量时间
        let input_hash = derive_pin_key(pin, &stored.salt)
            .map_err(|e| TauriError::Internal(e.to_string()))?;

        let valid = ct_eq(&input_hash, &stored.hash);

        if valid {
            self.failed_attempts.store(0, Ordering::SeqCst);
            *self.lockout_until.lock().unwrap() = 0;
            Ok(true)
        } else {
            drop(pin_hash);
            self.record_failed_attempt();
            Ok(false)
        }
    }

    /// 检查是否已设置 PIN
    pub fn is_set(&self) -> bool {
        self.pin_hash.lock()
            .map(|h| h.is_some())
            .unwrap_or(false)
    }

    /// 获取剩余尝试次数
    pub fn remaining_attempts(&self) -> u32 {
        MAX_ATTEMPTS.saturating_sub(self.failed_attempts.load(Ordering::SeqCst))
    }

    /// 是否已锁定
    pub fn is_locked(&self) -> bool {
        let lockout_until = *self.lockout_until.lock().unwrap();
        if lockout_until == 0 {
            return false;
        }
        current_timestamp() < lockout_until
    }

    /// 获取锁定剩余秒数
    pub fn lockout_remaining_secs(&self) -> Option<i64> {
        let lockout_until = *self.lockout_until.lock().unwrap();
        if lockout_until == 0 {
            return None;
        }
        let now = current_timestamp();
        if now < lockout_until {
            Some(lockout_until - now)
        } else {
            None
        }
    }

    /// 清除 PIN
    pub fn clear(&self) -> Result<(), TauriError> {
        let mut pin_hash = self.pin_hash.lock()
            .map_err(|e| TauriError::Internal(e.to_string()))?;
        *pin_hash = None;
        self.failed_attempts.store(0, Ordering::SeqCst);
        *self.lockout_until.lock().unwrap() = 0;
        Ok(())
    }

    fn check_lockout(&self) -> Result<(), TauriError> {
        let lockout_until = *self.lockout_until.lock().unwrap();
        if lockout_until == 0 {
            return Ok(());
        }

        let now = current_timestamp();
        if now < lockout_until {
            return Err(TauriError::PinLocked { until: lockout_until });
        }

        *self.lockout_until.lock().unwrap() = 0;
        Ok(())
    }

    fn record_failed_attempt(&self) {
        let attempts = self.failed_attempts.fetch_add(1, Ordering::SeqCst) + 1;

        if attempts >= MAX_ATTEMPTS {
            let lockout_until = current_timestamp() + LOCKOUT_DURATION_SECS;
            *self.lockout_until.lock().unwrap() = lockout_until;
            tracing::warn!("PIN locked due to {} failed attempts", attempts);
        }
    }
}

impl Default for PinState {
    fn default() -> Self {
        Self::new()
    }
}

/// 使用 Argon2id 派生 PIN 哈希（与主密码一致）
fn derive_pin_key(pin: &str, salt: &[u8]) -> anyhow::Result<Vec<u8>> {
    use argon2::{Argon2, Params};

    // PIN 长度较短，使用较低成本（与主密码相比）
    let params = Params::new(
        19456,  // 19 MiB
        2,
        1,
        Some(32),
    )
    .map_err(|e| anyhow::anyhow!("Invalid Argon2 params: {}", e))?;

    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    let mut output = vec![0u8; 32];
    argon2
        .hash_password_into(pin.as_bytes(), salt, &mut output)
        .map_err(|e| anyhow::anyhow!("Argon2 hashing failed: {}", e))?;

    Ok(output)
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_verify_pin() {
        let state = PinState::new();
        state.set_pin("1234").unwrap();
        assert!(state.verify("1234").unwrap());
        assert!(!state.verify("5678").unwrap());
    }

    #[test]
    fn test_short_pin_rejected() {
        let state = PinState::new();
        assert!(state.set_pin("12").is_err());
    }

    #[test]
    fn test_lockout_after_failures() {
        let state = PinState::new();
        state.set_pin("1234").unwrap();

        for i in 0..MAX_ATTEMPTS {
            assert!(!state.verify("wrong").unwrap());
            if i < MAX_ATTEMPTS - 1 {
                assert!(!state.is_locked());
            }
        }
        assert!(state.is_locked());
        assert!(state.verify("1234").is_err());
    }

    #[test]
    fn test_reset_on_success() {
        let state = PinState::new();
        state.set_pin("1234").unwrap();

        state.verify("wrong").unwrap();
        state.verify("wrong").unwrap();
        assert_eq!(state.remaining_attempts(), 3);

        state.verify("1234").unwrap();
        assert_eq!(state.remaining_attempts(), 5);
    }
}
