//! 认证模块
//!
//! 提供主密码、PIN、QuickKey 等认证机制

pub mod master_password;
pub mod pin;
pub mod quickkey;

pub use master_password::{MasterPassword, MasterPasswordManager};
pub use pin::PinState;