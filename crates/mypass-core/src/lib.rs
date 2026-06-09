//! MyPass Core Library
//!
//! 本地优先、跨平台、端到端加密的密码管理器核心

#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::nursery)]

pub mod crypto;
pub mod vault;
pub mod sync;
pub mod auth;
pub mod otp;
pub mod import;
pub mod export;
pub mod platform;
pub mod security;
pub mod performance;
pub mod error;

pub use error::TauriError;
pub use error::Result;
