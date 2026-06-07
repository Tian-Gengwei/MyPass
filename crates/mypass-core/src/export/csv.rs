//! CSV 导出器模块

use super::Exporter;
use crate::vault::entry::Entry;

/// CSV 导出器
pub struct CsvExporter;

impl CsvExporter {
    pub fn new() -> Self {
        Self
    }
    
    /// 转义 CSV 字段
    fn escape_field(&self, field: &str) -> String {
        if field.contains(',') || field.contains('"') || field.contains('\n') {
            format!("\"{}\"", field.replace('"', "\"\""))
        } else {
            field.to_string()
        }
    }
}

impl Exporter for CsvExporter {
    fn export(&self, entries: &[Entry]) -> Vec<u8> {
        let mut csv = String::new();
        
        // CSV 标题行
        csv.push_str("name,username,password,url,notes,totp,group,id,created_at,updated_at\n");
        
        for entry in entries {
            let name = self.escape_field(&entry.name);
            let username = self.escape_field(&entry.username);
            let password = self.escape_field(&entry.password);
            let url = self.escape_field(entry.url.as_deref().unwrap_or(""));
            let notes = self.escape_field(entry.notes.as_deref().unwrap_or(""));
            let totp = self.escape_field(entry.otp_auth_url.as_deref().unwrap_or(""));
            let group = self.escape_field(entry.group_id.as_deref().unwrap_or(""));
            let id = self.escape_field(&entry.id);
            let created = entry.created_at.to_string();
            let updated = entry.updated_at.to_string();
            
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{}\n",
                name, username, password, url, notes, totp, group, id, created, updated
            ));
        }
        
        csv.into_bytes()
    }
    
    fn file_extension(&self) -> &str {
        "csv"
    }
    
    fn format_name(&self) -> &str {
        "CSV"
    }
}