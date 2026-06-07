//!浏览器扩展通信模块
//!
//! 提供与浏览器扩展的长连接和消息处理

pub mod native;

pub use native::{ExtensionConnection, ExtensionMessage, ExtensionEntry};