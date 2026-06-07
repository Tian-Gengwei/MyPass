//! Rustls + aws-lc-rs 实现
//!
//! 纯 Rust TLS 实现，无 ring/nasm 依赖。
//! 跨 Windows / Linux / macOS / Android 一致工作。

use super::{TlsConnector as ConnectorTrait, TlsStream as StreamTrait};
use crate::error::TauriError;
use std::pin::Pin;
use std::sync::Arc;
use std::future::Future;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::{client::TlsStream, TlsConnector as TokioRustlsConnector};

/// Rustls 连接器（默认后端）
pub struct RustlsConnector {
    connector: TokioRustlsConnector,
}

impl RustlsConnector {
    /// 创建跳过证书验证的客户端（开发模式）
    pub fn new_skip_verify() -> Self {
        let config = rustls::ClientConfig::builder()
            .with_root_certificates(rustls::RootCertStore::empty())
            .with_no_client_auth();

        Self {
            connector: Arc::new(config).into(),
        }
    }
}

impl ConnectorTrait for RustlsConnector {
    fn connect<'a>(
        &'a self,
        host: &'a str,
        port: u16,
    ) -> Pin<Box<dyn Future<Output = Result<Box<dyn StreamTrait>, TauriError>> + Send + 'a>> {
        Box::pin(async move {
            // 1. TCP 连接
            let tcp = TcpStream::connect((host, port))
                .await
                .map_err(|e| TauriError::WebDavConnectionFailed(format!("TCP: {}", e)))?;

            // 2. 解析 server name
            let server_name = rustls::pki_types::ServerName::try_from(host.to_string())
                .map_err(|e| TauriError::WebDavConnectionFailed(format!("DNS: {}", e)))?;

            // 3. TLS 握手
            let tls_stream = self.connector
                .connect(server_name, tcp)
                .await
                .map_err(|e| TauriError::WebDavConnectionFailed(format!("TLS: {}", e)))?;

            Ok(Box::new(RustlsStream { inner: tls_stream }) as Box<dyn StreamTrait>)
        })
    }

    fn name(&self) -> &'static str {
        "rustls+aws-lc-rs"
    }
}

pub struct RustlsStream {
    inner: TlsStream<TcpStream>,
}

impl StreamTrait for RustlsStream {
    fn write_all<'a>(
        &'a mut self,
        data: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), TauriError>> + Send + 'a>> {
        Box::pin(async move {
            self.inner
                .write_all(data)
                .await
                .map_err(|e| TauriError::SyncFailed(format!("TLS write: {}", e)))
        })
    }

    fn read<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> Pin<Box<dyn Future<Output = Result<usize, TauriError>> + Send + 'a>> {
        Box::pin(async move {
            self.inner
                .read(buf)
                .await
                .map_err(|e| TauriError::SyncFailed(format!("TLS read: {}", e)))
        })
    }

    fn shutdown<'a>(
        &'a mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), TauriError>> + Send + 'a>> {
        Box::pin(async move {
            let _ = self.inner.shutdown().await;
            Ok(())
        })
    }
}
