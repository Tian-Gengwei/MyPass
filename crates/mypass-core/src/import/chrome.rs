//! Chrome CSV 导入器
//!
//! 支持 Chrome/Firefox 导出的密码 CSV 格式
//!
//! ## CSV 格式
//!
//! ```text
//! name,url,username,password
//! "GitHub","https://github.com","user@example.com","secret123"
//! ```

use crate::import::ImportResult;
use crate::vault::entry::Entry;
use anyhow::Result;
use std::io::Cursor;

/// Chrome/Firefox CSV 导入器
pub struct ChromeCsvImporter;

impl ChromeCsvImporter {
    pub fn new() -> Self {
        Self
    }

    /// 导入 CSV
    pub fn import(&self, data: &[u8]) -> Result<ImportResult> {
        let cursor = Cursor::new(data);
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(cursor);

        let mut entries = Vec::new();

        for result in reader.records() {
            let record = result.map_err(|e| anyhow::anyhow!("CSV read error: {}", e))?;

            // 解析字段
            let name = self.get_field(&record, 0).unwrap_or_else(|| "Untitled".to_string());
            let url = self.get_field(&record, 1);
            let username = self.get_field(&record, 2).unwrap_or_default();
            let password = self.get_field(&record, 3).unwrap_or_default();

            // 跳过空记录
            if name == "name" || (name.is_empty() && username.is_empty() && password.is_empty()) {
                continue;
            }

            let mut entry = Entry::new(
                name,
                username,
                password,
            );

            if let Some(u) = url {
                if !u.is_empty() {
                    entry = entry.with_url(u);
                }
            }

            entries.push(entry);
        }

        Ok(ImportResult {
            entries,
            groups: Vec::new(),
            warnings: Vec::new(),
        })
    }

    /// 获取字段（处理引号）
    fn get_field(&self, record: &csv::StringRecord, index: usize) -> Option<String> {
        record.get(index).map(|s| {
            let s = s.trim();
            // 去除首尾引号
            if (s.starts_with('"') && s.ends_with('"'))
                || (s.starts_with('\'') && s.ends_with('\''))
            {
                s[1..s.len()-1].to_string()
            } else {
                s.to_string()
            }
        }).filter(|s| !s.is_empty())
    }

    /// 从字符串导入
    pub fn import_str(&self, data: &str) -> Result<ImportResult> {
        self.import(data.as_bytes())
    }
}

impl Default for ChromeCsvImporter {
    fn default() -> Self {
        Self::new()
    }
}

/// 检查是否是 Chrome CSV
pub fn is_chrome_csv(data: &[u8]) -> bool {
    // 检查是否有 CSV header: name,url,username,password
    if let Ok(first_line) = std::str::from_utf8(data) {
        let first_line = first_line.trim().to_lowercase();
        first_line.contains("name")
            && first_line.contains("url")
            && first_line.contains("username")
            && first_line.contains("password")
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_chrome_csv() {
        let valid = b"name,url,username,password\nGitHub,https://github.com,user,pass";
        assert!(is_chrome_csv(valid));

        let invalid = b"not,a,csv,file";
        assert!(!is_chrome_csv(invalid));
    }

    #[test]
    fn test_import_csv() {
        let csv = r#"name,url,username,password
"GitHub","https://github.com","user@example.com","secret123"
"Twitter","https://twitter.com","@user","password456""#;

        let importer = ChromeCsvImporter::new();
        let result = importer.import_str(csv).unwrap();

        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.entries[0].name, "GitHub");
        assert_eq!(result.entries[0].url.as_deref(), Some("https://github.com"));
    }
}

impl crate::import::Importer for ChromeCsvImporter {
    fn can_import(&self, data: &[u8]) -> bool {
        is_chrome_csv(data)
    }

    fn import(&self, data: &[u8]) -> anyhow::Result<ImportResult> {
        self.import(data)
    }
}