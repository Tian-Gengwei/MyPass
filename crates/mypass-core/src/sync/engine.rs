//! 同步引擎
//!
//! 基于 Manifest 的增量同步，支持 WebDAV/S3

use crate::vault::manifest::{Manifest, SyncPlan};

pub enum SyncDirection {
    Push,
    Pull,
    BiDirectional,
}

/// 同步引擎
///
/// 负责 Manifest 的比对与同步计划生成
#[allow(dead_code)]
pub struct SyncEngine {
    #[allow(dead_code)]
    direction: SyncDirection,
}

impl SyncEngine {
    pub fn new(direction: SyncDirection) -> Self {
        Self { direction }
    }

    /// 合并两个 Manifest，生成同步计划
    ///
    /// 策略（C + D）：
    /// - 按 version 决定谁更新
    /// - 如果 version 相同但内容不同，标记为冲突
    pub fn merge_manifests(&self, local: &Manifest, remote: &Manifest) -> SyncPlan {
        local.diff(remote)
    }

    /// 解析冲突
    ///
    /// 返回值表示选择保留哪个版本
    pub fn resolve_conflict(
        &self,
        local: &Manifest,
        remote: &Manifest,
        conflict_id: &str,
    ) -> ConflictResolution {
        let local_meta = local.entries.get(conflict_id);
        let remote_meta = remote.entries.get(conflict_id);

        match (local_meta, remote_meta) {
            (Some(l), Some(r)) => {
                if l.version == r.version && l.file_hash != r.file_hash {
                    // 版本相同但内容不同，无法自动解决
                    ConflictResolution::Manual
                } else if l.updated_at > r.updated_at {
                    ConflictResolution::KeepLocal
                } else {
                    ConflictResolution::KeepRemote
                }
            }
            _ => ConflictResolution::KeepLocal,
        }
    }
}

/// 冲突解决策略
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictResolution {
    /// 保留本地版本
    KeepLocal,
    /// 保留远端版本
    KeepRemote,
    /// 两者都保留（产生两个条目）
    KeepBoth,
    /// 需要手动解决
    Manual,
}