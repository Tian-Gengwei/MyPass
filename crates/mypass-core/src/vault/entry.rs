//! 条目模型

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

pub type EntryId = String;
pub type GroupId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: EntryId,
    pub name: String,
    pub username: String,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub otp_auth_url: Option<String>,
    pub group_id: Option<GroupId>,
    pub custom_fields: HashMap<String, String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub version: u64,
}

impl Entry {
    pub fn new(name: String, username: String, password: String) -> Self {
        let now = chrono_timestamp();
        Self {
            id: generate_id(),
            name,
            username,
            password,
            url: None,
            notes: None,
            otp_auth_url: None,
            group_id: None,
            custom_fields: HashMap::new(),
            created_at: now,
            updated_at: now,
            version: 1,
        }
    }

    pub fn with_url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    pub fn with_notes(mut self, notes: String) -> Self {
        self.notes = Some(notes);
        self
    }

    pub fn with_otp(mut self, otp_url: String) -> Self {
        self.otp_auth_url = Some(otp_url);
        self
    }

    pub fn with_group(mut self, group_id: GroupId) -> Self {
        self.group_id = Some(group_id);
        self
    }

    /// 设置 ID（用于从外部源导入时保留原 ID）
    pub fn with_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }

    /// 设置 group_id
    pub fn with_group_id(mut self, group_id: String) -> Self {
        self.group_id = Some(group_id);
        self
    }

    /// 设置时间戳
    pub fn with_timestamps(mut self, created_at: i64, updated_at: i64) -> Self {
        self.created_at = created_at;
        self.updated_at = updated_at;
        self
    }

    pub fn update(&mut self) {
        self.updated_at = chrono_timestamp();
        self.version += 1;
    }

    pub fn content_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.id.as_bytes());
        hasher.update(self.name.as_bytes());
        hasher.update(self.password.as_bytes());
        hasher.update(self.updated_at.to_le_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn storage_path(&self) -> (String, String, String) {
        let hash = self.content_hash();
        let hash = hash.replace(":", "").replace("-", "");
        let hash_str: String = hash.chars().take(8).collect();
        let part1 = hash_str[..2].to_string();
        let part2 = hash_str[2..4].to_string();
        let part3 = hash_str[4..8].to_string();
        (part1, part2, part3)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub parent_id: Option<GroupId>,
    pub created_at: i64,
    pub updated_at: i64,
    pub version: u64,
}

impl Group {
    pub fn new(name: String) -> Self {
        let now = chrono_timestamp();
        Self {
            id: generate_group_id(),
            name,
            parent_id: None,
            created_at: now,
            updated_at: now,
            version: 1,
        }
    }

    pub fn with_parent(mut self, parent_id: GroupId) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// 设置 ID（用于从外部源导入时保留原 ID）
    pub fn with_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }

    /// 设置时间戳
    pub fn with_timestamps(mut self, created_at: i64, updated_at: i64) -> Self {
        self.created_at = created_at;
        self.updated_at = updated_at;
        self
    }
}

fn generate_id() -> String {
    crate::crypto::secure_random::generate_id()
}

fn generate_group_id() -> String {
    crate::crypto::secure_random::generate_id()
}

fn chrono_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_entry() {
        let entry = Entry::new("GitHub".to_string(), "user@example.com".to_string(), "secret123".to_string());
        assert_eq!(entry.name, "GitHub");
        assert_eq!(entry.version, 1);
    }

    #[test]
    fn test_update_entry() {
        let mut entry = Entry::new("Test".to_string(), "user".to_string(), "pass".to_string());
        let old_updated = entry.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        entry.update();
        assert_eq!(entry.version, 2);
        assert!(entry.updated_at >= old_updated);
    }
}