//! HKDF 子密钥派生
//!
//! 使用 HKDF-SHA256 从主密钥派生子密钥
//!
//! # 为什么用 HKDF 而不是直接 SHA-256?
//!
//! - HKDF 是 RFC 5869 标准的密钥派生函数
//! - 提取+扩展两步设计，避免长度扩展攻击
//! - 接受信息（info）参数允许为不同用途派生独立子密钥
//! - 比直接哈希更安全，更标准化

use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::Zeroize;
use anyhow::Result;
use subtle::ConstantTimeEq;

/// 派生子密钥（不带 salt）
///
/// # Arguments
/// * `master_key` - 主密钥（输入密钥材料）
/// * `info` - 上下文信息（如 "vault-key" 或 "log-cipher"）
/// * `len` - 输出字节数
pub fn derive_subkey(master_key: &[u8], info: &[u8], len: usize) -> Result<Vec<u8>> {
    let hk = Hkdf::<Sha256>::new(None, master_key);
    let mut okm = vec![0u8; len];
    hk.expand(info, &mut okm)
        .map_err(|e| anyhow::anyhow!("HKDF expand failed: {}", e))?;
    Ok(okm)
}

/// 派生子密钥（带 salt）
///
/// salt 提供额外的随机性/上下文分离
pub fn derive_subkey_with_salt(master_key: &[u8], salt: &[u8], info: &[u8], len: usize) -> Result<Vec<u8>> {
    let hk = Hkdf::<Sha256>::new(Some(salt), master_key);
    let mut okm = vec![0u8; len];
    hk.expand(info, &mut okm)
        .map_err(|e| anyhow::anyhow!("HKDF expand failed: {}", e))?;
    Ok(okm)
}

/// 安全比较两个字节切片（常量时间）
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.ct_eq(b).into()
}

/// 带自动零化的子密钥派生
///
/// 子密钥在 drop 时自动清零
pub fn derive_subkey_zeroizing(master_key: &[u8], info: &[u8], len: usize) -> Result<ZeroizingKey> {
    let key = derive_subkey(master_key, info, len)?;
    Ok(ZeroizingKey(key))
}

/// 自动零化的密钥包装
pub struct ZeroizingKey(pub Vec<u8>);

impl Drop for ZeroizingKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl std::ops::Deref for ZeroizingKey {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hkdf_deterministic() {
        let master = b"test-master-key";
        let info = b"test-info";
        let k1 = derive_subkey(master, info, 32).unwrap();
        let k2 = derive_subkey(master, info, 32).unwrap();
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_hkdf_different_info() {
        let master = b"test-master-key";
        let k1 = derive_subkey(master, b"purpose-a", 32).unwrap();
        let k2 = derive_subkey(master, b"purpose-b", 32).unwrap();
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_hkdf_different_keys() {
        let k1 = derive_subkey(b"master-1", b"info", 32).unwrap();
        let k2 = derive_subkey(b"master-2", b"info", 32).unwrap();
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"hello", b"helloo"));
    }
}
