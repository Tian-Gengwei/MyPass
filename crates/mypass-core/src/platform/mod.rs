//! 平台特定模块
//!
//! 提供跨平台系统集成接口

pub mod keychain;
pub mod biometric;

pub use keychain::{keychain_store, keychain_retrieve, keychain_delete, keychain_exists};
pub use biometric::{biometric_authenticate, biometric_is_available, biometric_get_type, BiometricType};