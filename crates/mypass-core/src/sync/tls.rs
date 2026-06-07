//! TLS 抽象层
//!
//! 提供跨平台 TLS 抽象，让 WebDAV 客户端不直接依赖具体实现。
//!
//! ## 后端选择
//!
//! - `rustls`（默认）：使用 rustls + aws-lc-rs，**纯 Rust**实现
//! - `native-tls-compat`：使用系统 TLS（schannel/Secure Transport/Conscrypt）

use crate::error::TauriError;
use std::pin::Pin;
use std::sync::Arc;

/// TLS 连接器（手动 async 实现以支持 dyn）
pub trait TlsConnector: Send + Sync {
    /// 连接到远程主机
    fn connect<'a>(
        &'a self,
        host: &'a str,
        port: u16,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Box<dyn TlsStream>, TauriError>> + Send + 'a>>;

    /// 获取连接器名称
    fn name(&self) -> &'static str;
}

/// TLS 流（使用 Pin<Box<dyn Future>> 模式）
pub trait TlsStream: Send + Sync {
    /// 写入所有数据
    fn write_all<'a>(
        &'a mut self,
        data: &'a [u8],
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), TauriError>> + Send + 'a>>;

    /// 读取数据到缓冲区
    fn read<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> Pin<Box<dyn std::future::Future<Output = Result<usize, TauriError>> + Send + 'a>>;

    /// 关闭 TLS 连接
    fn shutdown<'a>(
        &'a mut self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), TauriError>> + Send + 'a>> {
        Box::pin(async { Ok(()) })
    }
}

/// 工厂函数：创建默认 TLS 连接器
pub fn create_connector() -> Arc<dyn TlsConnector> {
    #[cfg(not(feature = "native-tls-compat"))]
    {
        Arc::new(crate::sync::tls_rustls::RustlsConnector::new_skip_verify())
    }
    #[cfg(feature = "native-tls-compat")]
    {
        Arc::new(crate::sync::tls_native::NativeTlsConnector::new())
    }
}
