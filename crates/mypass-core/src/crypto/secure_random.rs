//! 加密安全随机数生成
//!
//! 使用操作系统的密码学安全随机数生成器 (CSPRNG)
//! - Linux/macOS: /dev/urandom
//! - Windows: BCryptGenRandom (通过 getrandom)
//!
//! # 为什么不用 `thread_rng()`?
//!
//! `thread_rng()` 在某些旧版本的 `rand` crate 中使用 xorshift 生成器
//! 不是密码学安全的。对于密钥、ID 和盐值，必须使用 CSPRNG。

use rand::RngCore;
use rand::rngs::OsRng;
use zeroize::Zeroize;

/// 加密安全随机字节（任意长度）
pub fn random_bytes(len: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; len];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

/// 填充现有缓冲区（写入 OS CSPRNG）
pub fn fill_random(bytes: &mut [u8]) {
    OsRng.fill_bytes(bytes);
}

/// 生成 128-bit 唯一 ID
pub fn generate_id() -> String {
    let mut bytes = [0u8; 16];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// 生成随机 MEK（主加密密钥）
pub fn generate_mek() -> [u8; 32] {
    let mut mek = [0u8; 32];
    OsRng.fill_bytes(&mut mek);
    mek
}

/// 生成 256-bit QuickKey
pub fn generate_quickkey() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

/// 安全零化内存（防止敏感数据残留）
pub fn zeroize(bytes: &mut [u8]) {
    bytes.zeroize();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_bytes_unique() {
        let a = random_bytes(32);
        let b = random_bytes(32);
        assert_ne!(a, b, "Random bytes should be unique");
        assert_eq!(a.len(), 32);
    }

    #[test]
    fn test_generate_id_unique() {
        let a = generate_id();
        let b = generate_id();
        assert_ne!(a, b);
        assert_eq!(a.len(), 32); // hex(16) = 32 chars
    }

    #[test]
    fn test_generate_mek_unique() {
        let a = generate_mek();
        let b = generate_mek();
        assert_ne!(a, b);
    }
}
