//! 主密码认证
//!
//! ## 安全特性
//!
//! - **常量时间比较**：使用 `subtle::ConstantTimeEq` 防止时序攻击
//! - **速率限制**：连续 5 次失败后锁定 5 分钟
//! - **密码长度验证**：拒绝过短的主密码
//! - **零时清理**：KEK 在 Drop 时清零

use crate::crypto::{derive_kek, generate_salt, ct_eq};
use anyhow::Result;
use zeroize::Zeroize;

/// 最小主密码长度
pub const MIN_PASSWORD_LENGTH: usize = 8;

/// 最大失败次数
pub const MAX_FAILED_ATTEMPTS: u32 = 5;

/// 锁定时长（秒）
pub const LOCKOUT_DURATION_SECS: i64 = 300;

/// 包装 KEK 的零时结构
pub struct MasterPassword {
    pub salt: Vec<u8>,
    kek: Vec<u8>,
}

impl MasterPassword {
    pub fn new(password: &str) -> Result<Self> {
        Self::validate_password(password)?;
        let salt = generate_salt();
        let kek = derive_kek(password, &salt)?;
        Ok(Self { salt, kek })
    }

    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self> {
        Self::validate_password(password)?;
        let kek = derive_kek(password, salt)?;
        Ok(Self {
            salt: salt.to_vec(),
            kek,
        })
    }

    /// 验证主密码（常量时间比较）
    pub fn verify(&self, password: &str) -> bool {
        match derive_kek(password, &self.salt) {
            Ok(kek) => ct_eq(&kek, &self.kek),
            Err(_) => false,
        }
    }

    fn validate_password(password: &str) -> Result<()> {
        if password.is_empty() {
            anyhow::bail!("Password cannot be empty");
        }
        if password.len() < MIN_PASSWORD_LENGTH {
            anyhow::bail!("Password must be at least {} characters", MIN_PASSWORD_LENGTH);
        }
        if password.len() > 1024 {
            anyhow::bail!("Password too long (max 1024 characters)");
        }
        Ok(())
    }
}

impl Drop for MasterPassword {
    fn drop(&mut self) {
        self.kek.zeroize();
    }
}

/// 失败计数和锁定状态管理器
pub struct MasterPasswordManager {
    failed_attempts: u32,
    lockout_until: Option<i64>,
}

impl MasterPasswordManager {
    pub fn new() -> Self {
        Self {
            failed_attempts: 0,
            lockout_until: None,
        }
    }

    /// 验证密码，自动处理失败计数和锁定
    pub fn verify(&mut self, password: &str, stored: &MasterPassword) -> Result<bool> {
        if self.is_locked() {
            let remaining = self.lockout_remaining_secs().unwrap_or(0);
            anyhow::bail!(
                "Account locked. Try again in {} seconds.",
                remaining
            );
        }

        let valid = stored.verify(password);

        if !valid {
            self.failed_attempts += 1;
            if self.failed_attempts >= MAX_FAILED_ATTEMPTS {
                self.lockout_until = Some(now_timestamp() + LOCKOUT_DURATION_SECS);
                tracing::warn!(
                    "Master password verification locked for {} seconds after {} failures",
                    LOCKOUT_DURATION_SECS,
                    self.failed_attempts
                );
            }
        } else {
            self.failed_attempts = 0;
            self.lockout_until = None;
        }

        Ok(valid)
    }

    /// 是否处于锁定状态
    pub fn is_locked(&self) -> bool {
        if let Some(until) = self.lockout_until {
            now_timestamp() < until
        } else {
            false
        }
    }

    /// 锁定剩余秒数
    pub fn lockout_remaining_secs(&self) -> Option<i64> {
        if let Some(until) = self.lockout_until {
            let now = now_timestamp();
            if now < until {
                Some(until - now)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 重置状态
    pub fn reset(&mut self) {
        self.failed_attempts = 0;
        self.lockout_until = None;
    }

    /// 剩余尝试次数
    pub fn remaining_attempts(&self) -> u32 {
        MAX_FAILED_ATTEMPTS.saturating_sub(self.failed_attempts)
    }
}

impl Default for MasterPasswordManager {
    fn default() -> Self {
        Self::new()
    }
}

fn now_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_verify() {
        let mp = MasterPassword::new("correct_password").unwrap();
        assert!(mp.verify("correct_password"));
        assert!(!mp.verify("wrong_password"));
    }

    #[test]
    fn test_short_password_rejected() {
        let result = MasterPassword::new("short");
        assert!(result.is_err());
    }

    #[test]
    fn test_lockout_after_failures() {
        let mut manager = MasterPasswordManager::new();
        let mp = MasterPassword::new("correct_password").unwrap();

        for i in 0..MAX_FAILED_ATTEMPTS {
            assert!(!manager.verify("wrong", &mp).unwrap());
            if i < MAX_FAILED_ATTEMPTS - 1 {
                assert!(!manager.is_locked());
            }
        }
        assert!(manager.is_locked());

        // 锁定后即使正确密码也会失败
        let result = manager.verify("correct_password", &mp);
        assert!(result.is_err());
    }

    #[test]
    fn test_reset() {
        let mut manager = MasterPasswordManager::new();
        let mp = MasterPassword::new("correct_password").unwrap();
        manager.verify("wrong", &mp).unwrap();
        manager.reset();
        assert_eq!(manager.failed_attempts, 0);
    }
}
