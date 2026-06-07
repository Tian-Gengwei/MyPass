//! 浏览器扩展通信协议
//!
//! ## 架构
//!
//! 浏览器扩展与 MyPass 桌面应用之间通过以下方式通信：
//!
//! 1. **WebSocket 长连接**（首选）
//!    - MyPass 启动时开启本地 WebSocket 服务（默认端口 9312）
//!    - 扩展连接后保持长连接
//!
//! 2. **Native Messaging**（回退）
//!    - 扩展通过 `chrome.runtime.sendNativeMessage` 调用 MyPass 客户端
//!    - 需要在系统中注册 native messaging host manifest
//!
//! ## 消息协议
//!
//! 所有消息为 JSON 格式，结构：
//! ```json
//! {
//!   "type": "request_type",
//!   "id": "unique_request_id",
//!   "payload": { ... }
//! }
//! ```

use serde::{Deserialize, Serialize};

/// 默认 WebSocket 端口
pub const DEFAULT_PORT: u16 = 9312;

/// 扩展条目（脱敏后的入口信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionEntry {
    /// 条目 ID
    pub id: String,
    /// 名称
    pub name: String,
    /// 用户名
    pub username: String,
    /// URL（用于匹配）
    pub url: Option<String>,
    /// 是否有密码（不返回密码明文）
    pub has_password: bool,
    /// 是否有 TOTP
    pub has_totp: bool,
}

/// 扩展消息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExtensionMessage {
    /// 心跳
    Ping,
    /// 心跳响应
    Pong,

    /// 获取条目列表
    GetEntries {
        id: String,
        url_filter: Option<String>,
    },
    /// 响应：条目列表
    EntriesResponse {
        id: String,
        entries: Vec<ExtensionEntry>,
    },

    /// 请求密码
    GetPassword {
        id: String,
        entry_id: String,
    },
    /// 响应：密码
    PasswordResponse {
        id: String,
        password: String,
    },

    /// 触发自动填充
    FillRequest {
        id: String,
        url: String,
        username: Option<String>,
    },
    /// 响应：填充结果
    FillResponse {
        id: String,
        success: bool,
    },

    /// 保存新凭据
    SaveCredential {
        id: String,
        url: String,
        username: String,
        password: String,
    },
    /// 响应：保存结果
    SaveResponse {
        id: String,
        entry_id: String,
    },

    /// 错误响应
    Error {
        id: Option<String>,
        message: String,
    },
}

/// 扩展连接（虚拟类型，表示与扩展的通信）
pub struct ExtensionConnection {
    port: u16,
}

impl ExtensionConnection {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn default() -> Self {
        Self::new(DEFAULT_PORT)
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Default for ExtensionConnection {
    fn default() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_port() {
        assert_eq!(DEFAULT_PORT, 9312);
    }

    #[test]
    fn test_message_serialize() {
        let msg = ExtensionMessage::GetEntries {
            id: "test-1".to_string(),
            url_filter: Some("github.com".to_string()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("get_entries"));
        assert!(json.contains("test-1"));
    }

    #[test]
    fn test_message_deserialize() {
        let json = r#"{"type":"ping"}"#;
        let msg: ExtensionMessage = serde_json::from_str(json).unwrap();
        matches!(msg, ExtensionMessage::Ping);
    }

    #[test]
    fn test_entry_serialize() {
        let entry = ExtensionEntry {
            id: "e1".to_string(),
            name: "GitHub".to_string(),
            username: "user@example.com".to_string(),
            url: Some("https://github.com".to_string()),
            has_password: true,
            has_totp: false,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("has_password"));
        assert!(json.contains("true"));
    }
}
