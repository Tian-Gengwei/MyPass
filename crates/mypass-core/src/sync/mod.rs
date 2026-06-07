//! 同步模块
//!
//! 基于 Manifest 的增量同步，支持 WebDAV/S3

pub mod engine;
pub mod http;
pub mod s3;
pub mod tls;
pub mod tls_rustls;
pub mod webdav;

#[cfg(feature = "native-tls-compat")]
pub mod tls_native;

pub use engine::{SyncEngine, SyncDirection, ConflictResolution};
pub use http::{Request, Response, send_request, read_response, basic_auth_header};
pub use tls::{TlsConnector, TlsStream, create_connector};
pub use webdav::{WebDavSync, WebDavConfig, WebDavSyncResult};
pub use s3::{S3Sync, S3Config, S3SyncResult};
