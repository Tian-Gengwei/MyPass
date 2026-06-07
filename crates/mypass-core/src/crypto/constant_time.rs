//! 常量时间比较
//!
//! 防止时序攻击。在密码哈希、令牌验证等场景中，
//! 普通 `==` 比较会因为不同字节数的不同处理时间泄露信息。

use subtle::ConstantTimeEq;

/// 常量时间字节比较
///
/// # 示例
///
/// ```
/// use mypass_core::crypto::ct_eq;
///
/// assert!(ct_eq(b"secret", b"secret"));
/// assert!(!ct_eq(b"secret", b"Secret"));
/// ```
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    a.ct_eq(b).into()
}

/// 常量时间字符串比较（仅 ASCII 字符）
pub fn ct_eq_str(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.as_bytes().ct_eq(b.as_bytes()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal() {
        assert!(ct_eq(b"hello", b"hello"));
        assert!(ct_eq(b"", b""));
    }

    #[test]
    fn test_not_equal() {
        assert!(!ct_eq(b"hello", b"world"));
        assert!(!ct_eq(b"hello", b"hell"));
        assert!(!ct_eq(b"hello", b"helloo"));
    }

    #[test]
    fn test_str() {
        assert!(ct_eq_str("secret", "secret"));
        assert!(!ct_eq_str("secret", "Secret"));
    }
}
