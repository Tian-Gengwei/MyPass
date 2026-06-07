//! Argon2id 密钥派生
//!
//! 从主密码派生 KEK (密钥加密密钥)

use argon2::{
    Argon2, Params,
};
use anyhow::Result;

const SALT_LEN: usize = 16;
const OUTPUT_LEN: usize = 32;

pub fn derive_kek(master_password: &str, salt: &[u8]) -> Result<Vec<u8>> {
    let params = Params::new(
        65536,
        3,
        4,
        Some(OUTPUT_LEN),
    )
    .map_err(|e| anyhow::anyhow!("Invalid Argon2 params: {}", e))?;

    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    let mut output = vec![0u8; OUTPUT_LEN];
    argon2
        .hash_password_into(master_password.as_bytes(), salt, &mut output)
        .map_err(|e| anyhow::anyhow!("Argon2 hashing failed: {}", e))?;

    Ok(output)
}

pub fn generate_salt() -> Vec<u8> {
    crate::crypto::secure_random::random_bytes(SALT_LEN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_kek() {
        let password = "test_password";
        let salt = generate_salt();
        let kek = derive_kek(password, &salt).unwrap();
        assert_eq!(kek.len(), 32);
    }

    #[test]
    fn test_different_salts() {
        let password = "test_password";
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        let kek1 = derive_kek(password, &salt1).unwrap();
        let kek2 = derive_kek(password, &salt2).unwrap();
        assert_ne!(kek1, kek2);
    }
}