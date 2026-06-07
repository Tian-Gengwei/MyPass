//! Group 模型

use serde::{Deserialize, Serialize};
use crate::vault::entry::GroupId;

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
        let now = timestamp();
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
}

fn generate_group_id() -> String {
    crate::crypto::secure_random::generate_id()
}

fn timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}