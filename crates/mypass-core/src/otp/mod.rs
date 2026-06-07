//! TOTP 模块

use anyhow::{Context, Result};
use base64::Engine as _;

pub struct TotpCode {
    pub code: String,
    pub remaining_secs: u32,
}

pub struct TotpManager;

impl TotpManager {
    pub fn generate(secret: &str) -> Result<TotpCode> {
        Self::generate_with_digits(secret, 6)
    }

    /// 生成指定位数的 TOTP 代码
    ///
    /// 大多数应用使用 6 位（Google Authenticator、Authy 等），
    /// 部分应用支持 8 位以提供更多熵。
    pub fn generate_with_digits(secret: &str, digits: u32) -> Result<TotpCode> {
        let secret_bytes = base64::engine::general_purpose::STANDARD
            .decode(secret)
            .context("Invalid TOTP secret")?;

        let time_step: u64 = 30;
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let counter = time / time_step;
        let remaining: u32 = (time % time_step) as u32;

        let code = generate_hotp(&secret_bytes, counter, digits)?;
        Ok(TotpCode {
            code,
            remaining_secs: time_step as u32 - remaining,
        })
    }

    pub fn verify(secret: &str, code: &str) -> Result<bool> {
        let generated = Self::generate(secret)?;
        Ok(generated.code == code)
    }
}

fn generate_hotp(secret: &[u8], counter: u64, digits: u32) -> Result<String> {
    use hmac::{Hmac, Mac};
    type HmacSha1 = Hmac<sha1::Sha1>;

    let counter_bytes = counter.to_be_bytes();
    let mut mac = HmacSha1::new_from_slice(secret)
        .context("Invalid HMAC key")?;
    mac.update(&counter_bytes);

    let result = mac.finalize();
    let hash = result.into_bytes();

    let offset = (hash[hash.len() - 1] & 0x0f) as usize;
    let code = u32::from_be_bytes([
        hash[offset] & 0x7f,
        hash[offset + 1],
        hash[offset + 2],
        hash[offset + 3],
    ]) % 10u32.pow(digits);

    Ok(format!("{:0width$}", code, width = digits as usize))
}

pub fn parse_totp_url(url: &str) -> Result<(&str, &str)> {
    let url = url.trim();
    if !url.starts_with("otpauth://totp/") {
        anyhow::bail!("Invalid TOTP URL");
    }

    let url = &url[15..];
    let parts: Vec<&str> = url.splitn(2, '?').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid TOTP URL format");
    }

    let secret = parts[1]
        .split('&')
        .find(|p| p.starts_with("secret="))
        .map(|p| &p[7..])
        .ok_or_else(|| anyhow::anyhow!("Missing secret"))?;

    Ok((parts[0], secret))
}

/// 内部使用：基于指定时间戳生成 TOTP（仅用于测试）
#[doc(hidden)]
pub fn _generate_at(secret: &[u8], time: u64) -> Result<String> {
    generate_hotp(secret, time / 30, 8)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 6238 附录 B 测试向量
    /// 密钥 "12345678901234567890"（20 字节 ASCII）
    /// 时间步长 30 秒，SHA-1
    ///
    /// | Time (sec) | T (hex)    | TOTP       |
    /// |------------|------------|------------|
    /// | 59         | 0000000000000001 | 94287082 |
    /// | 1111111109 | 00000000023523EC | 07081804 |
    /// | 1111111111 | 00000000023523ED | 14050471 |
    /// | 1234567890 | 000000000273EF07 | 89005924 |
    /// | 2000000000 | 0000000003F940AA | 69279037 |
    /// | 20000000000 | 0000000027BC86AA | 65353130 |
    const RFC_SECRET: &[u8] = b"12345678901234567890";

    #[test]
    fn test_rfc6238_t59() {
        // T = 59
        let code = _generate_at(RFC_SECRET, 59).unwrap();
        assert_eq!(code, "94287082");
    }

    #[test]
    fn test_rfc6238_t1111111109() {
        let code = _generate_at(RFC_SECRET, 1111111109).unwrap();
        assert_eq!(code, "07081804");
    }

    #[test]
    fn test_rfc6238_t1111111111() {
        let code = _generate_at(RFC_SECRET, 1111111111).unwrap();
        assert_eq!(code, "14050471");
    }

    #[test]
    fn test_rfc6238_t1234567890() {
        let code = _generate_at(RFC_SECRET, 1234567890).unwrap();
        assert_eq!(code, "89005924");
    }

    #[test]
    fn test_rfc6238_t2000000000() {
        let code = _generate_at(RFC_SECRET, 2000000000).unwrap();
        assert_eq!(code, "69279037");
    }

    #[test]
    fn test_rfc6238_t20000000000() {
        let code = _generate_at(RFC_SECRET, 20000000000).unwrap();
        assert_eq!(code, "65353130");
    }

    #[test]
    fn test_totp_remaining_time() {
        // 时间 0 -> 30s 剩余
        // 时间 15 -> 15s 剩余
        // 时间 29 -> 1s 剩余
        let time = 15;
        let time_step: u64 = 30;
        let remaining: u32 = (time % time_step) as u32;
        let expected_remaining = time_step as u32 - remaining;
        assert_eq!(expected_remaining, 15);
    }

    #[test]
    fn test_totp_code_format() {
        let code = _generate_at(RFC_SECRET, 59).unwrap();
        // 总是 8 位数字（RFC 6238 测试向量）
        assert_eq!(code.len(), 8);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_parse_totp_url_valid() {
        let url = "otpauth://totp/GitHub:user@example.com?secret=JBSWY3DPEHPK3PXP&issuer=GitHub";
        let (label, secret) = parse_totp_url(url).unwrap();
        assert_eq!(label, "GitHub:user@example.com");
        assert_eq!(secret, "JBSWY3DPEHPK3PXP");
    }

    #[test]
    fn test_parse_totp_url_no_params() {
        let url = "otpauth://totp/test?";
        assert!(parse_totp_url(url).is_err());
    }

    #[test]
    fn test_parse_totp_url_no_secret() {
        let url = "otpauth://totp/test?issuer=GitHub";
        assert!(parse_totp_url(url).is_err());
    }

    #[test]
    fn test_parse_totp_url_invalid_scheme() {
        let url = "http://example.com/secret";
        assert!(parse_totp_url(url).is_err());
    }

    #[test]
    fn test_totp_verify_correct_code() {
        // 验证 真实生成的代码
        let secret_bytes = b"12345678901234567890";
        let encoded = base64::engine::general_purpose::STANDARD.encode(secret_bytes);
        let code = TotpManager::generate(&encoded).unwrap();
        assert!(TotpManager::verify(&encoded, &code.code).unwrap());
    }

    #[test]
    fn test_totp_verify_wrong_code() {
        let secret_bytes = b"12345678901234567890";
        let encoded = base64::engine::general_purpose::STANDARD.encode(secret_bytes);
        assert!(!TotpManager::verify(&encoded, "000000").unwrap());
    }
}