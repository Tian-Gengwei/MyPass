//! native-tls 实现（feature-gated）
//!
//! - Windows: schannel (SChannel)
//! - macOS: Secure Transport
//! - Android: OpenSSL + Conscrypt
//! - Linux: OpenSSL

#![cfg(feature = "native-tls-compat")]

use super::{TlsConnector, TlsStream};
use crate::error::TauriError;
use std::future::Future;
use std::pin::Pin;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_native_tls::{TlsConnector as TokioNativeTlsConnector, TlsStream as TokioNativeTlsStream};

pub struct NativeTlsConnector {
    inner: TokioNativeTlsConnector,
}

impl NativeTlsConnector {
    pub fn new() -> Self {
        let mut builder = native_tls::TlsConnector::builder();
        builder.danger_accept_invalid_certs(true);
        let inner = builder.build().expect("Failed to build native-tls connector");
        Self {
            inner: TokioNativeTlsConnector::from(inner),
        }
    }
}

impl Default for NativeTlsConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl TlsConnector for NativeTlsConnector {
    fn connect<'a>(
        &'a self,
        host: &'a str,
        port: u16,
    ) -> Pin<Box<dyn Future<Output = Result<Box<dyn TlsStream>, TauriError>> + Send + 'a>> {
        Box::pin(async move {
            let tcp = TcpStream::connect((host, port))
                .await
                .map_err(|e| TauriError::WebDavConnectionFailed(format!("TCP: {}", e)))?;

            let tls = self.inner
                .connect(host, tcp)
                .await
                .map_err(|e| TauriError::WebDavConnectionFailed(format!("TLS: {}", e)))?;

            Ok(Box::new(NativeTlsStream { inner: tls }) as Box<dyn TlsStream>)
        })
    }

    fn name(&self) -> &'static str {
        "native-tls"
    }
}

pub struct NativeTlsStream {
    inner: TokioNativeTlsStream<TcpStream>,
}

impl TlsStream for NativeTlsStream {
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
