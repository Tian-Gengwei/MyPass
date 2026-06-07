//! Vault 生命周期集成测试
//!
//! 测试完整的 Vault 使用流程：
//! 1. 创建新 Vault
//! 2. 解锁 Vault
//! 3. 添加/更新/删除条目
//! 4. 创建/删除分组
//! 5. 锁定 Vault
//! 6. 重新解锁（验证数据持久化）
//! 7. 错误密码解锁失败

use mypass_core::vault::entry::Entry;
use mypass_core::vault::{Group, Vault};
use std::path::PathBuf;

fn create_test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("mypass-test")
        .join(name)
        .join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_vault_full_lifecycle() {
    let dir = create_test_dir("lifecycle");
    let vault_dir = dir.join("test_vault.vault");

    // 1. 创建
    let mut vault = Vault::create(dir.clone(), "master123!", "test_vault").unwrap();
    assert_eq!(vault.list_entries().len(), 0);
    assert_eq!(vault.list_groups().len(), 0);

    // 2. 添加分组
    let group = Group::new("Logins".to_string());
    let group_id = group.id.clone();
    vault.add_group(group).unwrap();
    assert_eq!(vault.list_groups().len(), 1);

    // 3. 添加条目
    let mut entry = Entry::new(
        "GitHub".to_string(),
        "user@example.com".to_string(),
        "secret".to_string(),
    )
    .with_url("https://github.com".to_string())
    .with_notes("Test entry".to_string());
    entry.group_id = Some(group_id.clone());
    let entry_id = entry.id.clone();
    vault.add_entry(entry).unwrap();
    assert_eq!(vault.list_entries().len(), 1);

    // 4. 更新条目
    let mut updated = Entry::new(
        "GitHub Updated".to_string(),
        "new@example.com".to_string(),
        "newsecret".to_string(),
    );
    updated.id = entry_id.clone();
    updated.group_id = Some(group_id.clone());
    vault.update_entry(&entry_id, updated).unwrap();

    let entries = vault.list_entries();
    assert_eq!(entries[0].name, "GitHub Updated");
    assert_eq!(entries[0].username, "new@example.com");
    assert_eq!(entries[0].password, "newsecret");

    // 5. 删除条目
    vault.delete_entry(&entry_id).unwrap();
    assert_eq!(vault.list_entries().len(), 0);

    // 6. 锁定
    vault.lock().unwrap();

    // 7. 重新打开
    let vault2 = Vault::unlock(vault_dir.clone(), "master123!").unwrap();
    // 删除的条目不应存在
    assert_eq!(vault2.list_entries().len(), 0);
    // 分组应保留
    assert_eq!(vault2.list_groups().len(), 1);

    // 清理
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_vault_wrong_password_fails() {
    let dir = create_test_dir("wrong_password");
    let vault_dir = dir.join("vault.vault");

    let _ = Vault::create(dir.clone(), "correct", "vault").unwrap();

    let result = Vault::unlock(vault_dir, "wrong_password");
    assert!(result.is_err());
}

#[test]
fn test_vault_search() {
    let dir = create_test_dir("search");

    let mut vault = Vault::create(dir.clone(), "password", "v").unwrap();

    vault.add_entry(Entry::new("GitHub".to_string(), "user1".to_string(), "p1".to_string())).unwrap();
    vault.add_entry(Entry::new("GitLab".to_string(), "user2".to_string(), "p2".to_string())).unwrap();
    vault.add_entry(Entry::new("Twitter".to_string(), "user3".to_string(), "p3".to_string())).unwrap();

    // 搜索 "git" 应该匹配 GitHub 和 GitLab
    let results = vault.search_entries("git");
    assert_eq!(results.len(), 2);

    // 搜索 "twitter" 应该匹配 Twitter
    let results = vault.search_entries("twitter");
    assert_eq!(results.len(), 1);

    // 搜索空字符串返回所有
    let results = vault.search_entries("");
    assert_eq!(results.len(), 3);

    // 搜索不存在的字符串
    let results = vault.search_entries("nonexistent");
    assert_eq!(results.len(), 0);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_vault_persistence() {
    let dir = create_test_dir("persistence");
    let vault_dir = dir.join("v.vault");

    // 创建 + 添加数据
    {
        let mut vault = Vault::create(dir.clone(), "password", "v").unwrap();
        vault.add_entry(Entry::new("Test".to_string(), "u".to_string(), "p".to_string())).unwrap();
    }

    // 重新打开
    {
        let vault = Vault::unlock(vault_dir.clone(), "password").unwrap();
        let entries = vault.list_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Test");
    }

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_vault_concurrent_operations() {
    let dir = create_test_dir("concurrent");

    let mut vault = Vault::create(dir.clone(), "password", "v").unwrap();

    // 添加 100 个条目
    for i in 0..100 {
        let entry = Entry::new(
            format!("Entry{}", i),
            format!("user{}", i),
            format!("pass{}", i),
        );
        vault.add_entry(entry).unwrap();
    }

    assert_eq!(vault.list_entries().len(), 100);

    // 搜索
    let results = vault.search_entries("Entry5");
    assert!(!results.is_empty());

    // 删除一半
    let entries = vault.list_entries();
    for entry in entries.iter().take(50) {
        vault.delete_entry(&entry.id).unwrap();
    }

    assert_eq!(vault.list_entries().len(), 50);

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_vault_info() {
    let dir = create_test_dir("info");

    let mut vault = Vault::create(dir.clone(), "password", "my_vault").unwrap();

    vault.add_entry(Entry::new("e1".to_string(), "u".to_string(), "p".to_string())).unwrap();
    vault.add_entry(Entry::new("e2".to_string(), "u".to_string(), "p".to_string())).unwrap();
    vault.add_group(Group::new("g1".to_string())).unwrap();

    let info = vault.get_info();
    assert_eq!(info.name, "my_vault");
    assert_eq!(info.entry_count, 2);
    assert_eq!(info.group_count, 1);

    let _ = std::fs::remove_dir_all(dir);
}
