//! 手写 HTTP/1.1 协议
//!
//! 专为 WebDAV 子集设计：仅支持 GET/PUT/PROPFIND/MKCOL 请求
//! 响应解析仅处理状态行 + 头部 + 已知长度的 body
//!
//! ## 为什么手写？
//!
//! 避免引入 hyper + tower + bytes 等大量传递依赖（虽然不直接依赖 ring，
//! 但会增加数十个 crate）。手写协议可减少 90% 依赖大小。
//!
//! ## 限制
//!
//! - 不支持 chunked transfer encoding（WebDAV 不用）
//! - 不支持 HTTP/2
//! - 不支持 keep-alive（每个请求新连接）

use crate::error::TauriError;
use super::tls::TlsStream;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// HTTP 请求
#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    pub uri: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Request {
    /// 序列化为字节流
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(self.method.as_bytes());
        buf.extend_from_slice(b" ");
        buf.extend_from_slice(self.uri.as_bytes());
        buf.extend_from_slice(b" HTTP/1.1\r\n");

        for (k, v) in &self.headers {
            buf.extend_from_slice(k.as_bytes());
            buf.extend_from_slice(b": ");
            buf.extend_from_slice(v.as_bytes());
            buf.extend_from_slice(b"\r\n");
        }

        buf.extend_from_slice(format!("Content-Length: {}\r\n", self.body.len()).as_bytes());
        buf.extend_from_slice(b"\r\n");
        buf.extend_from_slice(&self.body);
        buf
    }
}

/// HTTP 响应
#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub reason: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Response {
    /// 获取 Content-Length
    pub fn content_length(&self) -> usize {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
            .and_then(|(_, v)| v.parse().ok())
            .unwrap_or(0)
    }

    /// 获取指定头部
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

/// 生成 Basic Auth 头
pub fn basic_auth_header(username: &str, password: &str) -> String {
    let creds = format!("{}:{}", username, password);
    format!("Basic {}", BASE64.encode(creds.as_bytes()))
}

/// 发送 HTTP 请求并读取响应
pub async fn send_request<S: TlsStream + ?Sized>(
    stream: &mut S,
    request: &Request,
) -> Result<Response, TauriError> {
    let bytes = request.serialize();
    stream.write_all(&bytes).await?;
    read_response(stream).await
}

/// 从流中读取 HTTP 响应
pub async fn read_response<S: TlsStream + ?Sized>(
    stream: &mut S,
) -> Result<Response, TauriError> {
    // 1. 读取状态行（HTTP/1.1 STATUS REASON\r\n）
    let status_line = read_line(stream).await?;
    let mut parts = status_line.splitn(3, ' ');
    let _version = parts.next();
    let status: u16 = parts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| TauriError::SyncFailed(format!("Invalid status line: {}", status_line)))?;
    let reason = parts.next().unwrap_or("").to_string();

    // 2. 读取头部（直到空行）
    let mut headers = Vec::new();
    loop {
        let line = read_line(stream).await?;
        if line.is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            headers.push((k.trim().to_string(), v.trim().to_string()));
        }
    }

    // 3. 读取 body
    let content_length: usize = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, v)| v.parse().ok())
        .unwrap_or(0);

    let body = if content_length > 0 {
        let mut body = vec![0u8; content_length];
        let mut read = 0;
        while read < content_length {
            let n = stream.read(&mut body[read..]).await?;
            if n == 0 {
                break;
            }
            read += n;
        }
        body.truncate(read);
        body
    } else {
        Vec::new()
    };

    Ok(Response {
        status,
        reason,
        headers,
        body,
    })
}

/// 读取一行（以 \r\n 结束）
async fn read_line<S: TlsStream + ?Sized>(stream: &mut S) -> Result<String, TauriError> {
    let mut line = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        let n = stream.read(&mut byte).await?;
        if n == 0 {
            // EOF
            if line.is_empty() {
                return Ok(String::new());
            }
            break;
        }
        if byte[0] == b'\n' {
            break;
        }
        if byte[0] != b'\r' {
            line.push(byte[0]);
        }
    }
    String::from_utf8(line).map_err(|e| TauriError::SyncFailed(format!("Invalid UTF-8: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_auth() {
        let h = basic_auth_header("alice", "secret");
        assert!(h.starts_with("Basic "));
        // "alice:secret" -> "YWxpY2U6c2VjcmV0"
        assert!(h.contains("YWxpY2U6c2VjcmV0"));
    }

    #[test]
    fn test_request_serialize() {
        let req = Request {
            method: "GET".to_string(),
            uri: "/test".to_string(),
            headers: vec![("Host".to_string(), "example.com".to_string())],
            body: vec![],
        };
        let bytes = req.serialize();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.starts_with("GET /test HTTP/1.1\r\n"));
        assert!(text.contains("Host: example.com\r\n"));
        assert!(text.contains("Content-Length: 0\r\n"));
        assert!(text.ends_with("\r\n\r\n"));
    }

    #[test]
    fn test_request_serialize_with_body() {
        let req = Request {
            method: "POST".to_string(),
            uri: "/api".to_string(),
            headers: vec![],
            body: b"hello".to_vec(),
        };
        let bytes = req.serialize();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("Content-Length: 5"));
        assert!(text.ends_with("hello"));
    }

    #[test]
    fn test_response_content_length() {
        let r = Response {
            status: 200,
            reason: "OK".to_string(),
            headers: vec![("content-length".to_string(), "42".to_string())],
            body: vec![],
        };
        assert_eq!(r.content_length(), 42);
    }

    #[test]
    fn test_response_header_lookup() {
        let r = Response {
            status: 404,
            reason: "Not Found".to_string(),
            headers: vec![("Content-Type".to_string(), "text/html".to_string())],
            body: vec![],
        };
        assert_eq!(r.header("content-type"), Some("text/html"));
        assert_eq!(r.header("missing"), None);
    }
}
