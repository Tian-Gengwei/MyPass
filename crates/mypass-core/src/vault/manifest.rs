//! Manifest 索引
//!
//! 记录所有条目的版本信息，用于增量同步

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::vault::entry::{EntryId, GroupId};
use crate::vault::storage::ObjectMeta;

/// Manifest 主索引
///
/// 记录所有对象的元数据，支持增量同步
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Manifest {
    /// 条目索引：EntryId -> ObjectMeta
    pub entries: HashMap<EntryId, ObjectMeta>,
    /// 分组索引：GroupId -> ObjectMeta
    pub groups: HashMap<GroupId, ObjectMeta>,
    /// Manifest 全局版本号
    pub version: u64,
    /// 最后更新时间戳
    pub updated_at: i64,
}

impl Manifest {
    pub fn new() -> Self {
        let now = timestamp();
        Self {
            entries: HashMap::new(),
            groups: HashMap::new(),
            version: 1,
            updated_at: now,
        }
    }

    /// 更新条目
    pub fn update_entry(&mut self, id: EntryId, meta: ObjectMeta) {
        let now = timestamp();
        self.entries.insert(id, meta);
        self.version += 1;
        self.updated_at = now;
    }

    /// 删除条目
    pub fn remove_entry(&mut self, id: &EntryId) {
        self.entries.remove(id);
        self.version += 1;
        self.updated_at = timestamp();
    }

    /// 更新分组
    pub fn update_group(&mut self, id: GroupId, meta: ObjectMeta) {
        let now = timestamp();
        self.groups.insert(id, meta);
        self.version += 1;
        self.updated_at = now;
    }

    /// 删除分组
    pub fn remove_group(&mut self, id: &GroupId) {
        self.groups.remove(id);
        self.version += 1;
        self.updated_at = timestamp();
    }

    /// 比较两个 Manifest，返回需要同步的条目
    ///
    /// 策略（C + D）：
    /// - 自动合并：version 不同的条目，取最新版本
    /// - 冲突标记：如果 version 相同但内容不同，标记为冲突
    ///
    /// # Returns
    /// * `SyncPlan` - 同步计划
    pub fn diff(&self, remote: &Manifest) -> SyncPlan {
        let mut plan = SyncPlan::default();

        // 遍历本地条目
        for (id, local_meta) in &self.entries {
            if let Some(remote_meta) = remote.entries.get(id) {
                // 条目存在于两端
                if remote_meta.version > local_meta.version {
                    // 远端更新，需要拉取
                    plan.pull.push(id.clone());
                } else if local_meta.version > remote_meta.version {
                    // 本地更新，需要推送
                    plan.push.push(id.clone());
                } else if local_meta.file_hash != remote_meta.file_hash {
                    // 版本相同但内容不同，冲突
                    plan.conflicts.push(ConflictEntry {
                        id: id.clone(),
                        local_hash: local_meta.file_hash.clone(),
                        remote_hash: remote_meta.file_hash.clone(),
                    });
                }
            } else {
                // 仅本地存在，需要推送
                plan.push.push(id.clone());
            }
        }

        // 远端有但本地没有的条目，需要拉取
        for (id, _remote_meta) in &remote.entries {
            if !self.entries.contains_key(id) {
                plan.pull.push(id.clone());
            }
        }

        // 分组同上
        for (id, local_meta) in &self.groups {
            if let Some(remote_meta) = remote.groups.get(id) {
                if remote_meta.version > local_meta.version {
                    plan.pull_groups.push(id.clone());
                } else if local_meta.version > remote_meta.version {
                    plan.push_groups.push(id.clone());
                } else if local_meta.file_hash != remote_meta.file_hash {
                    plan.conflicts.push(ConflictEntry {
                        id: id.clone(),
                        local_hash: local_meta.file_hash.clone(),
                        remote_hash: remote_meta.file_hash.clone(),
                    });
                }
            } else {
                plan.push_groups.push(id.clone());
            }
        }

        for (id, _remote_meta) in &remote.groups {
            if !self.groups.contains_key(id) {
                plan.pull_groups.push(id.clone());
            }
        }

        plan
    }

    /// 获取条目数量
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// 获取分组数量
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }
}

/// 同步计划
#[derive(Debug, Clone, Default)]
pub struct SyncPlan {
    /// 需要拉取的条目 ID
    pub pull: Vec<EntryId>,
    /// 需要推送的条目 ID
    pub push: Vec<EntryId>,
    /// 需要拉取的分组 ID
    pub pull_groups: Vec<GroupId>,
    /// 需要推送的分组 ID
    pub push_groups: Vec<GroupId>,
    /// 冲突条目（需要手动解决）
    pub conflicts: Vec<ConflictEntry>,
}

/// 冲突条目
#[derive(Debug, Clone)]
pub struct ConflictEntry {
    pub id: String,
    pub local_hash: String,
    pub remote_hash: String,
}

fn timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::storage::ObjectType;

    fn make_meta(id: &str, version: u64, hash: &str) -> ObjectMeta {
        ObjectMeta {
            id: id.to_string(),
            obj_type: ObjectType::Entry,
            file_hash: hash.to_string(),
            version,
            updated_at: 0,
        }
    }

    #[test]
    fn test_manifest_new() {
        let m = Manifest::new();
        assert!(m.entries.is_empty());
        assert!(m.groups.is_empty());
        assert_eq!(m.version, 1);
    }

    #[test]
    fn test_update_entry_increments_version() {
        let mut m = Manifest::new();
        let v0 = m.version;
        m.update_entry("e1".to_string(), make_meta("e1", 1, "hash1"));
        assert!(m.version > v0);
        assert_eq!(m.entries.len(), 1);
    }

    #[test]
    fn test_remove_entry_decreases_count() {
        let mut m = Manifest::new();
        m.update_entry("e1".to_string(), make_meta("e1", 1, "hash1"));
        assert_eq!(m.entries.len(), 1);
        m.remove_entry(&"e1".to_string());
        assert_eq!(m.entries.len(), 0);
    }

    #[test]
    fn test_diff_empty_both_sides() {
        let local = Manifest::new();
        let remote = Manifest::new();
        let plan = local.diff(&remote);
        assert!(plan.pull.is_empty());
        assert!(plan.push.is_empty());
        assert!(plan.conflicts.is_empty());
    }

    #[test]
    fn test_diff_local_only_should_push() {
        let mut local = Manifest::new();
        local.update_entry("e1".to_string(), make_meta("e1", 1, "hash1"));

        let remote = Manifest::new();
        let plan = local.diff(&remote);
        assert_eq!(plan.push.len(), 1);
        assert!(plan.pull.is_empty());
    }

    #[test]
    fn test_diff_remote_only_should_pull() {
        let local = Manifest::new();

        let mut remote = Manifest::new();
        remote.update_entry("e1".to_string(), make_meta("e1", 1, "hash1"));

        let plan = local.diff(&remote);
        assert_eq!(plan.pull.len(), 1);
        assert!(plan.push.is_empty());
    }

    #[test]
    fn test_diff_remote_newer_should_pull() {
        let mut local = Manifest::new();
        local.update_entry("e1".to_string(), make_meta("e1", 1, "hash1"));

        let mut remote = Manifest::new();
        remote.update_entry("e1".to_string(), make_meta("e1", 2, "hash2"));

        let plan = local.diff(&remote);
        assert_eq!(plan.pull.len(), 1);
        assert_eq!(plan.pull[0], "e1");
    }

    #[test]
    fn test_diff_local_newer_should_push() {
        let mut local = Manifest::new();
        local.update_entry("e1".to_string(), make_meta("e1", 3, "hash3"));

        let mut remote = Manifest::new();
        remote.update_entry("e1".to_string(), make_meta("e1", 1, "hash1"));

        let plan = local.diff(&remote);
        assert_eq!(plan.push.len(), 1);
        assert_eq!(plan.push[0], "e1");
    }

    #[test]
    fn test_diff_same_version_same_hash_no_action() {
        let mut local = Manifest::new();
        local.update_entry("e1".to_string(), make_meta("e1", 5, "samehash"));

        let mut remote = Manifest::new();
        remote.update_entry("e1".to_string(), make_meta("e1", 5, "samehash"));

        let plan = local.diff(&remote);
        assert!(plan.pull.is_empty());
        assert!(plan.push.is_empty());
        assert!(plan.conflicts.is_empty());
    }

    #[test]
    fn test_diff_same_version_different_hash_creates_conflict() {
        let mut local = Manifest::new();
        local.update_entry("e1".to_string(), make_meta("e1", 5, "hash_a"));

        let mut remote = Manifest::new();
        remote.update_entry("e1".to_string(), make_meta("e1", 5, "hash_b"));

        let plan = local.diff(&remote);
        assert_eq!(plan.conflicts.len(), 1);
        assert_eq!(plan.conflicts[0].id, "e1");
        assert_eq!(plan.conflicts[0].local_hash, "hash_a");
        assert_eq!(plan.conflicts[0].remote_hash, "hash_b");
    }

    #[test]
    fn test_diff_groups_handled_separately() {
        let mut local = Manifest::new();
        local.update_group("g1".to_string(), make_meta("g1", 1, "ghash1"));

        let mut remote = Manifest::new();
        remote.update_group("g1".to_string(), make_meta("g1", 2, "ghash2"));

        let plan = local.diff(&remote);
        assert_eq!(plan.pull_groups.len(), 1);
        assert!(plan.pull.is_empty(), "条目和分组应分别处理");
    }

    #[test]
    fn test_diff_complex_scenario() {
        let mut local = Manifest::new();
        let mut remote = Manifest::new();

        // 本地独有 -> push
        local.update_entry("e_local".to_string(), make_meta("e_local", 1, "h1"));

        // 远端独有 -> pull
        remote.update_entry("e_remote".to_string(), make_meta("e_remote", 1, "h2"));

        // 远端更新 -> pull
        local.update_entry("e_pull".to_string(), make_meta("e_pull", 1, "h3"));
        remote.update_entry("e_pull".to_string(), make_meta("e_pull", 2, "h3b"));

        // 本地更新 -> push
        local.update_entry("e_push".to_string(), make_meta("e_push", 3, "h4"));
        remote.update_entry("e_push".to_string(), make_meta("e_push", 1, "h4"));

        let plan = local.diff(&remote);
        assert_eq!(plan.push.len(), 2); // e_local + e_push
        assert_eq!(plan.pull.len(), 2); // e_remote + e_pull
        assert!(plan.conflicts.is_empty());
    }

    #[test]
    fn test_manifest_serialize_deserialize() {
        let mut m = Manifest::new();
        m.update_entry("e1".to_string(), make_meta("e1", 1, "hash1"));
        m.update_group("g1".to_string(), make_meta("g1", 1, "ghash1"));

        let json = serde_json::to_string(&m).unwrap();
        let restored: Manifest = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.entries.len(), 1);
        assert_eq!(restored.groups.len(), 1);
        assert_eq!(restored.entries.get("e1").unwrap().file_hash, "hash1");
    }
}