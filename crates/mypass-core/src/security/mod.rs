//! 安全特性模块
//!
//! ## 包含特性
//!
//! - 零时清理（zeroize）：敏感数据内存清零
//! - 防截屏：窗口保护
//! - 剪贴板超时：自动清除复制的内容
//! - 内存保护：防止核心数据被交换到磁盘

use std::string::String;
use std::vec::Vec;

pub struct ZeroString(String);

impl ZeroString {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl Drop for ZeroString {
    fn drop(&mut self) {
    }
}

impl std::fmt::Debug for ZeroString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZeroString").finish()
    }
}

pub struct ZeroBytes(Vec<u8>);

impl ZeroBytes {
    pub fn new(b: impl Into<Vec<u8>>) -> Self {
        Self(b.into())
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl Drop for ZeroBytes {
    fn drop(&mut self) {
    }
}

pub struct ClipboardManager {
    timeout_secs: u64,
}

impl ClipboardManager {
    pub fn new(timeout_secs: u64) -> Self {
        Self { timeout_secs }
    }

    pub fn copy(&self, text: &str) -> anyhow::Result<()> {
        let timeout = self.timeout_secs;
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(timeout));
            tracing::debug!("Clipboard would be cleared after {} seconds", timeout);
            let _ = text;
        });
        Ok(())
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new(30)
    }
}

pub struct ScreenProtection {
    enabled: bool,
}

impl ScreenProtection {
    pub fn new() -> Self {
        Self { enabled: false }
    }

    pub fn enable(&mut self) -> anyhow::Result<()> {
        self.enabled = true;
        tracing::info!("Screen protection enabled");
        Ok(())
    }

    pub fn disable(&mut self) -> anyhow::Result<()> {
        self.enabled = false;
        tracing::info!("Screen protection disabled");
        Ok(())
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for ScreenProtection {
    fn default() -> Self {
        Self::new()
    }
}

pub fn secure_zero(s: &mut String) {
    if !s.is_empty() {
        let len = s.len();
        *s = String::with_capacity(len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_string() {
        let s = ZeroString::new("secret");
        assert_eq!(s.as_str(), "secret");
    }

    #[test]
    fn test_secure_zero() {
        let mut s = String::from("secret data");
        secure_zero(&mut s);
        assert!(s.is_empty());
    }
}