//! JSON 导出器模块

use super::Exporter;
use crate::vault::entry::Entry;
use serde::Serialize;

/// MyPass 内部 JSON 格式
#[derive(Serialize)]
struct MyPassExport {
    /// 导出版本
    version: String,
    /// 导出时间戳
    exported_at: i64,
    /// 条目数量
    entry_count: usize,
    /// 条目列表
    entries: Vec<ExportEntry>,
}

/// 导出的条目结构
#[derive(Serialize)]
struct ExportEntry {
    /// 条目 ID
    id: String,
    /// 条目名称
    name: String,
    /// 用户名
    username: String,
    /// 密码（明文）
    password: String,
    /// URL
    url: Option<String>,
    /// 备注
    notes: Option<String>,
    /// TOTP URL
    totp: Option<String>,
    /// 分组
    group: Option<String>,
    /// 自定义字段
    custom_fields: std::collections::HashMap<String, String>,
    /// 创建时间
    created_at: i64,
    /// 更新时间
    updated_at: i64,
}

/// MyPass JSON 导出器
pub struct JsonExporter;

impl JsonExporter {
    pub fn new() -> Self {
        Self
    }
}

impl Exporter for JsonExporter {
    fn export(&self, entries: &[Entry]) -> Vec<u8> {
        let export = MyPassExport {
            version: "1.0".to_string(),
            exported_at: chrono_timestamp(),
            entry_count: entries.len(),
            entries: entries.iter().map(|e| convert_entry(e)).collect(),
        };
        
        serde_json::to_string_pretty(&export).unwrap().into_bytes()
    }
    
    fn file_extension(&self) -> &str {
        "json"
    }
    
    fn format_name(&self) -> &str {
        "MyPass JSON"
    }
}

/// Bitwarden JSON 导出器
pub struct BitwardenJsonExporter;

impl BitwardenJsonExporter {
    pub fn new() -> Self {
        Self
    }
}

impl Exporter for BitwardenJsonExporter {
    fn export(&self, entries: &[Entry]) -> Vec<u8> {
        use serde_json::json;
        
        let items: Vec<_> = entries.iter().map(|e| {
            json!({
                "name": e.name,
                "login": {
                    "username": e.username,
                    "password": e.password,
                    "uri": e.url,
                    "totp": e.otp_auth_url,
                },
                "notes": e.notes,
                "folderId": e.group_id,
                "type": 1,
                "favorite": false,
            })
        }).collect();
        
        let export = json!({
            "encrypted": false,
            "items": items,
        });
        
        serde_json::to_string_pretty(&export).unwrap().into_bytes()
    }
    
    fn file_extension(&self) -> &str {
        "json"
    }
    
    fn format_name(&self) -> &str {
        "Bitwarden JSON"
    }
}

/// 转换条目
fn convert_entry(entry: &Entry) -> ExportEntry {
    ExportEntry {
        id: entry.id.clone(),
        name: entry.name.clone(),
        username: entry.username.clone(),
        password: entry.password.clone(),
        url: entry.url.clone(),
        notes: entry.notes.clone(),
        totp: entry.otp_auth_url.clone(),
        group: entry.group_id.clone(),
        custom_fields: entry.custom_fields.clone(),
        created_at: entry.created_at,
        updated_at: entry.updated_at,
    }
}

/// 获取当前时间戳
fn chrono_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}