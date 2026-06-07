//! Vault 模块
//!
//! 金库核心：条目存储、Manifest 索引、对象存储引擎

pub mod entry;
pub mod group;
pub mod manifest;
pub mod storage;
pub mod store;

pub use entry::{Entry, EntryId, GroupId};
pub use group::Group;
pub use manifest::Manifest;
pub use storage::{ObjectStorage, ObjectMeta, ObjectType};
pub use store::{Vault, VaultInfo, VaultMeta};