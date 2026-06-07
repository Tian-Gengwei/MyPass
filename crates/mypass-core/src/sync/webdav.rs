//! WebDAV 同步协议（重写版）
//!
//! 使用纯 Rust TLS (rustls + aws-lc-rs)，无 ring/nasm 依赖。
//! 跨 Windows / Linux / macOS / Android 一致工作。
//!
//! ## 支持的 WebDAV 方法
//!
//! - `PROPFIND` - 测试连接 / 列出属性
//! - `GET` - 下载 manifest / 对象
//! - `PUT` - 上传 manifest / 对象
//! - `MKCOL` - 创建远程目录
//!
//! ## 不支持
//!
//! - `COPY` / `MOVE` - 当前不需要
//! - `DELETE` - 当前不需要（只支持完整同步）
//! - `LOCK` - 简单同步策略

use crate::error::TauriError;
use crate::sync::http::{self, Request};
use crate::sync::tls::{create_connector, TlsConnector};
use crate::vault::manifest::Manifest;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

const HTTP_TIMEOUT_SECS: u64 = 30;

/// WebDAV 客户端配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebDavConfig {
    /// 服务器 URL（如 https://nextcloud.example.com/remote.php/dav/files/user/）
    pub endpoint: String,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// Vault 目录名
    pub vault_name: String,
}

impl WebDavConfig {
    /// 构建 Vault 远程路径
    pub fn vault_path(&self) -> String {
        format!(
            "{}/{}.vault/",
            self.endpoint.trim_end_matches('/'),
            self.vault_name
        )
    }

    /// 构建 Manifest 路径
    pub fn manifest_path(&self) -> String {
        format!(
            "{}/{}.vault/manifest.enc",
            self.endpoint.trim_end_matches('/'),
            self.vault_name
        )
    }

    /// 构建对象路径
    pub fn object_path(&self, hash: &str) -> String {
        let safe_hash: String = hash
            .chars()
            .take(8)
            .map(|c| if c.is_ascii_hexdigit() { c } else { '0' })
            .collect();
        let bytes = safe_hash.as_bytes();
        let dir1 = std::str::from_utf8(&bytes[..2.min(bytes.len())]).unwrap_or("00");
        let dir2 = std::str::from_utf8(&bytes[2..4.min(bytes.len())]).unwrap_or("00");
        let filename = std::str::from_utf8(&bytes[4..8.min(bytes.len())]).unwrap_or("0000");
        format!(
            "{}/{}.vault/objects/{}/{}/{}.enc",
            self.endpoint.trim_end_matches('/'),
            self.vault_name,
            dir1,
            dir2,
            filename
        )
    }

    /// Basic Auth 头
    fn basic_auth(&self) -> String {
        let creds = format!("{}:{}", self.username, self.password);
        format!("Basic {}", BASE64.encode(creds.as_bytes()))
    }
}

/// WebDAV 同步结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebDavSyncResult {
    pub pulled_entries: usize,
    pub pushed_entries: usize,
    pub conflicts: usize,
    pub timestamp: i64,
}

/// WebDAV 同步器
pub struct WebDavSync {
    config: WebDavConfig,
    connector: Arc<dyn TlsConnector>,
}

impl WebDavSync {
    pub fn new(config: WebDavConfig) -> Self {
        Self {
            config,
            connector: create_connector(),
        }
    }

    /// 测试连接
    pub async fn test_connection(&self) -> Result<bool, TauriError> {
        let mut stream = self.connect().await?;
        let body = b"<?xml version=\"1.0\"?><d:propfind xmlns:d=\"DAV:\"><d:prop><d:resourcetype/></d:prop></d:propfind>".to_vec();
        let request = Request {
            method: "PROPFIND".to_string(),
            uri: self.config.vault_path(),
            headers: vec![
                ("Host".to_string(), self.config.endpoint.clone()),
                ("Authorization".to_string(), self.config.basic_auth()),
                ("Depth".to_string(), "0".to_string()),
                ("Content-Type".to_string(), "application/xml".to_string()),
            ],
            body,
        };
        let response = http::send_request(&mut *stream, &request).await?;
        // 207 Multi-Status, 404 Not Found (目录不存在但服务器可达), 或任何 2xx
        Ok(response.status == 207
            || response.status == 404
            || (200..300).contains(&response.status))
    }

    /// 拉取远程 Manifest
    pub async fn fetch_manifest(&self) -> Result<Option<Manifest>, TauriError> {
        let mut stream = self.connect().await?;
        let request = Request {
            method: "GET".to_string(),
            uri: self.config.manifest_path(),
            headers: vec![
                ("Host".to_string(), self.config.endpoint.clone()),
                ("Authorization".to_string(), self.config.basic_auth()),
            ],
            body: vec![],
        };
        let response = http::send_request(&mut *stream, &request).await?;

        if response.status == 404 {
            return Ok(None);
        }
        if !(200..300).contains(&response.status) {
            return Err(TauriError::SyncFailed(format!(
                "Fetch manifest failed: HTTP {}",
                response.status
            )));
        }

        let manifest: Manifest = serde_json::from_slice(&response.body)
            .map_err(|e| TauriError::SyncFailed(format!("Parse manifest: {}", e)))?;
        Ok(Some(manifest))
    }

    /// 推送本地 Manifest 到远程
    pub async fn push_manifest(&self, manifest: &Manifest, _mek: &[u8]) -> Result<(), TauriError> {
        let mut stream = self.connect().await?;
        let body = serde_json::to_vec(manifest)
            .map_err(|e| TauriError::SyncFailed(format!("Serialize: {}", e)))?;
        let request = Request {
            method: "PUT".to_string(),
            uri: self.config.manifest_path(),
            headers: vec![
                ("Host".to_string(), self.config.endpoint.clone()),
                ("Authorization".to_string(), self.config.basic_auth()),
                ("Content-Type".to_string(), "application/octet-stream".to_string()),
            ],
            body,
        };
        let response = http::send_request(&mut *stream, &request).await?;
        if !(200..300).contains(&response.status) {
            return Err(TauriError::SyncFailed(format!(
                "Push manifest failed: HTTP {}",
                response.status
            )));
        }
        Ok(())
    }

    /// 拉取单个对象
    pub async fn fetch_object(&self, hash: &str) -> Result<Vec<u8>, TauriError> {
        let mut stream = self.connect().await?;
        let request = Request {
            method: "GET".to_string(),
            uri: self.config.object_path(hash),
            headers: vec![
                ("Host".to_string(), self.config.endpoint.clone()),
                ("Authorization".to_string(), self.config.basic_auth()),
            ],
            body: vec![],
        };
        let response = http::send_request(&mut *stream, &request).await?;

        if response.status == 404 {
            return Err(TauriError::ObjectNotFound(hash.to_string()));
        }
        if !(200..300).contains(&response.status) {
            return Err(TauriError::SyncFailed(format!(
                "Fetch object failed: HTTP {}",
                response.status
            )));
        }
        Ok(response.body)
    }

    /// 推送单个对象到远程
    pub async fn push_object(&self, hash: &str, data: &[u8]) -> Result<(), TauriError> {
        let mut stream = self.connect().await?;
        let request = Request {
            method: "PUT".to_string(),
            uri: self.config.object_path(hash),
            headers: vec![
                ("Host".to_string(), self.config.endpoint.clone()),
                ("Authorization".to_string(), self.config.basic_auth()),
                ("Content-Type".to_string(), "application/octet-stream".to_string()),
            ],
            body: data.to_vec(),
        };
        let response = http::send_request(&mut *stream, &request).await?;
        if !(200..300).contains(&response.status) {
            return Err(TauriError::SyncFailed(format!(
                "Push object failed: HTTP {}",
                response.status
            )));
        }
        Ok(())
    }

    /// 确保远程目录存在
    pub async fn ensure_directories(&self) -> Result<(), TauriError> {
        self.mkcol(&self.config.vault_path()).await?;
        let objects_url = format!(
            "{}/{}.vault/objects/",
            self.config.endpoint.trim_end_matches('/'),
            self.config.vault_name
        );
        self.mkcol(&objects_url).await?;
        Ok(())
    }

    /// MKCOL 请求
    async fn mkcol(&self, url: &str) -> Result<(), TauriError> {
        let mut stream = self.connect().await?;
        let request = Request {
            method: "MKCOL".to_string(),
            uri: url.to_string(),
            headers: vec![
                ("Host".to_string(), self.config.endpoint.clone()),
                ("Authorization".to_string(), self.config.basic_auth()),
            ],
            body: vec![],
        };
        let response = http::send_request(&mut *stream, &request).await?;
        // 201 Created 或 405 Method Not Allowed (已存在) 都视为成功
        let status = response.status;
        if !(200..300).contains(&status) && status != 405 {
            return Err(TauriError::SyncFailed(format!(
                "MKCOL failed: HTTP {}",
                status
            )));
        }
        Ok(())
    }

    /// 内部：建立 TLS 连接（带超时）
    async fn connect(
        &self,
    ) -> Result<Box<dyn crate::sync::tls::TlsStream>, TauriError> {
        let url = url::Url::parse(&self.config.endpoint)
            .map_err(|e| TauriError::InvalidArgument(format!("Invalid URL: {}", e)))?;
        let host = url
            .host_str()
            .ok_or_else(|| TauriError::InvalidArgument("No host in URL".into()))?
            .to_string();
        let port = url.port().unwrap_or(match url.scheme() {
            "https" => 443,
            "http" => 80,
            _ => {
                return Err(TauriError::InvalidArgument(
                    "Unsupported scheme (only http/https)".into(),
                ))
            }
        });

        // http:// 不走 TLS
        if url.scheme() == "http" {
            return Err(TauriError::InvalidArgument(
                "Plain HTTP not supported, use HTTPS".into(),
            ));
        }

        timeout(
            Duration::from_secs(HTTP_TIMEOUT_SECS),
            self.connector.connect(&host, port),
        )
        .await
        .map_err(|_| TauriError::SyncFailed("Connection timeout".into()))?
    }

    /// 执行完整同步
    pub async fn sync(
        &self,
        local_manifest: &Manifest,
        mek: &[u8],
        object_loader: impl Fn(&str) -> Result<Vec<u8>, TauriError>,
    ) -> Result<WebDavSyncResult, TauriError> {
        // 确保目录存在
        self.ensure_directories().await?;

        // 1. 拉取远程 Manifest
        let remote_manifest = match self.fetch_manifest().await? {
            Some(m) => m,
            None => {
                tracing::info!("No remote manifest, performing initial sync");
                return self.initial_sync(local_manifest, mek, object_loader).await;
            }
        };

        // 2. 生成 SyncPlan
        let plan = local_manifest.diff(&remote_manifest);

        let mut result = WebDavSyncResult {
            pulled_entries: 0,
            pushed_entries: 0,
            conflicts: plan.conflicts.len(),
            timestamp: current_timestamp(),
        };

        // 3. 拉取远端更新
        for entry_id in &plan.pull {
            if let Some(meta) = remote_manifest.entries.get(entry_id) {
                if self.fetch_object(&meta.file_hash).await.is_ok() {
                    result.pulled_entries += 1;
                }
            }
        }

        // 4. 推送本地更新
        for entry_id in &plan.push {
            if let Some(meta) = local_manifest.entries.get(entry_id) {
                if let Ok(data) = object_loader(&meta.file_hash) {
                    if self.push_object(&meta.file_hash, &data).await.is_ok() {
                        result.pushed_entries += 1;
                    }
                }
            }
        }

        // 5. 推送更新后的 Manifest
        self.push_manifest(local_manifest, mek).await?;

        Ok(result)
    }

    /// 首次同步（远程为空）
    async fn initial_sync(
        &self,
        manifest: &Manifest,
        mek: &[u8],
        object_loader: impl Fn(&str) -> Result<Vec<u8>, TauriError>,
    ) -> Result<WebDavSyncResult, TauriError> {
        let mut count = 0;
        for meta in manifest.entries.values() {
            if let Ok(data) = object_loader(&meta.file_hash) {
                if self.push_object(&meta.file_hash, &data).await.is_ok() {
                    count += 1;
                }
            }
        }
        self.push_manifest(manifest, mek).await?;
        Ok(WebDavSyncResult {
            pulled_entries: 0,
            pushed_entries: count,
            conflicts: 0,
            timestamp: current_timestamp(),
        })
    }
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_path() {
        let config = WebDavConfig {
            endpoint: "https://example.com/dav/".to_string(),
            username: "u".to_string(),
            password: "p".to_string(),
            vault_name: "myvault".to_string(),
        };
        assert_eq!(config.vault_path(), "https://example.com/dav/myvault.vault/");
    }

    #[test]
    fn test_manifest_path() {
        let config = WebDavConfig {
            endpoint: "https://example.com/dav/".to_string(),
            username: "u".to_string(),
            password: "p".to_string(),
            vault_name: "vault".to_string(),
        };
        assert_eq!(
            config.manifest_path(),
            "https://example.com/dav/vault.vault/manifest.enc"
        );
    }

    #[test]
    fn test_object_path() {
        let config = WebDavConfig {
            endpoint: "https://example.com/dav".to_string(),
            username: "u".to_string(),
            password: "p".to_string(),
            vault_name: "vault".to_string(),
        };
        let path = config.object_path("a1b2c3d4e5f6");
        assert!(path.contains("a1/b2/c3d4.enc"));
    }

    #[test]
    fn test_object_path_with_non_hex() {
        let config = WebDavConfig {
            endpoint: "https://example.com/dav".to_string(),
            username: "u".to_string(),
            password: "p".to_string(),
            vault_name: "vault".to_string(),
        };
        // ':' and '-' are non-hex, replaced with '0'
        // 'g', 'h', 'i' are non-hex (a-f only), replaced with '0'
        // "abc:def-ghi" -> 'a','b','c','0','d','e','f','0','0','0','0'
        // take(8) = "abc0def0" -> dir1="ab", dir2="c0", filename="def0"
        let path = config.object_path("abc:def-ghi");
        assert!(path.contains("ab/c0/def0.enc"), "path was: {}", path);
    }
}
