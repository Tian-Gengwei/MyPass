//! Bitwarden JSON 导入器
//!
//! 支持 Bitwarden 导出的 JSON 格式
//!
//! ## Bitwarden JSON 格式
//!
//! ```json
//! {
//!   "encrypted": false,
//!   "items": [
//!     {
//!       "id": "guid",
//!       "name": "entry name",
//!       "login": {
//!         "uris": [{ "uri": "https://example.com" }],
//!         "username": "user@example.com",
//!         "password": "secret",
//!         "totp": "otpauth://totp/..."
//!       },
//!       "notes": "notes",
//!       "folderId": "folder-guid"
//!     }
//!   ]
//! }
//! ```

use crate::import::ImportResult;
use crate::vault::entry::{Entry, Group};
use anyhow::Result;
use serde::Deserialize;

/// Bitwarden 导出格式
#[derive(Debug, Deserialize)]
pub struct BitwardenExport {
    /// 是否加密（加密格式暂不支持）
    pub encrypted: bool,
    /// 条目列表
    pub items: Vec<BitwardenItem>,
}

/// Bitwarden 条目
#[derive(Debug, Deserialize)]
pub struct BitwardenItem {
    /// ID
    pub id: Option<String>,
    /// 名称
    pub name: Option<String>,
    /// 登录信息
    pub login: Option<BitwardenLogin>,
    /// 备注
    pub notes: Option<String>,
    /// 分组 ID
    pub folder_id: Option<String>,
    /// 创建时间
    pub creation_date: Option<String>,
    /// 修改时间
    pub last_modified_date: Option<String>,
}

/// Bitwarden 登录信息
#[derive(Debug, Deserialize)]
pub struct BitwardenLogin {
    /// URI 列表
    pub uris: Option<Vec<BitwardenUri>>,
    /// 用户名
    pub username: Option<String>,
    /// 密码
    pub password: Option<String>,
    /// TOTP 密钥
    pub totp: Option<String>,
}

/// Bitwarden URI
#[derive(Debug, Deserialize)]
pub struct BitwardenUri {
    /// URI 字符串
    pub uri: Option<String>,
}

/// Bitwarden 分组
#[derive(Debug, Deserialize)]
pub struct BitwardenFolder {
    pub id: Option<String>,
    pub name: Option<String>,
}

/// Bitwarden JSON 导入器
pub struct BitwardenImporter {
    folders: Vec<BitwardenFolder>,
}

impl BitwardenImporter {
    pub fn new() -> Self {
        Self {
            folders: Vec::new(),
        }
    }

    /// 导入 Bitwarden JSON
    pub fn import(&self, data: &[u8]) -> Result<ImportResult> {
        // 解析 JSON
        let export: BitwardenExport = serde_json::from_slice(data)
            .map_err(|e| anyhow::anyhow!("Invalid Bitwarden JSON: {}", e))?;

        if export.encrypted {
            anyhow::bail!("Encrypted Bitwarden exports are not supported. Please use unencrypted export.");
        }

        // 转换为 Entry
        let entries = self.convert_items(export.items)?;

        // 分组（如果有）
        let groups = self.folders.iter().filter_map(|f| {
            f.name.as_ref().map(|name| {
                let group = Group::new(name.clone());
                group
            })
        }).collect();

        Ok(ImportResult { entries, groups, warnings: Vec::new() })
    }

    /// 转换条目
    fn convert_items(&self, items: Vec<BitwardenItem>) -> Result<Vec<Entry>> {
        let mut result = Vec::new();

        for item in items {
            let name = item.name.unwrap_or_else(|| "Untitled".to_string());
            let login = item.login.as_ref();

            let username = login
                .and_then(|l| l.username.clone())
                .unwrap_or_default();

            let password = login
                .and_then(|l| l.password.clone())
                .unwrap_or_default();

            let url = login
                .and_then(|l| l.uris.as_ref())
                .and_then(|uris| uris.first())
                .and_then(|u| u.uri.clone())
                .filter(|s| !s.is_empty());

            let totp = login
                .and_then(|l| l.totp.clone())
                .filter(|s| !s.is_empty());

            let notes = item.notes.filter(|s| !s.is_empty());

            let mut entry = Entry::new(name, username, password);

            if let Some(u) = url {
                entry = entry.with_url(u);
            }
            if let Some(n) = notes {
                entry = entry.with_notes(n);
            }
            if let Some(t) = totp {
                entry = entry.with_otp(t);
            }

            result.push(entry);
        }

        Ok(result)
    }
}

impl Default for BitwardenImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::import::Importer for BitwardenImporter {
    fn can_import(&self, data: &[u8]) -> bool {
        is_bitwarden_json(data)
    }

    fn import(&self, data: &[u8]) -> anyhow::Result<ImportResult> {
        self.import(data)
    }
}

// ============================================================================
// Bitwarden CSV 导入器
// ============================================================================

/// Bitwarden CSV 导入器
///
/// 支持 Bitwarden 导出的 CSV 格式
///
/// ## Bitwarden CSV 格式
/// ```
/// folder,name,group,login_uri,login_username,login_password,login_totp,notes,fields,reprompt,filename
/// ```

/// Bitwarden CSV 导入器
pub struct BitwardenCsvImporter;

impl BitwardenCsvImporter {
    pub fn new() -> Self {
        Self
    }

    /// 解析 CSV 行（处理引号和转义）
    fn parse_csv_line(&self, line: &str) -> Vec<String> {
        let mut fields = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;

        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            let ch = chars[i];

            if ch == '"' {
                if in_quotes {
                    // 检查是否是转义引号 ""
                    if i + 1 < len && chars[i + 1] == '"' {
                        current.push('"');
                        i += 2;
                        continue;
                    }
                    in_quotes = false;
                } else {
                    in_quotes = true;
                }
                i += 1;
                continue;
            }

            if ch == ',' && !in_quotes {
                fields.push(current.trim().to_string());
                current.clear();
                i += 1;
                continue;
            }

            current.push(ch);
            i += 1;
        }

        fields.push(current.trim().to_string());
        fields
    }

    /// 查找列索引（不区分大小写）
    fn find_column_index(&self, headers: &[String], names: &[&str]) -> Option<usize> {
        for (i, header) in headers.iter().enumerate() {
            let lower = header.to_lowercase();
            for name in names {
                if lower.contains(*name) {
                    return Some(i);
                }
            }
        }
        None
    }

    /// 导入 Bitwarden CSV
    pub fn import(&self, data: &[u8]) -> anyhow::Result<ImportResult> {
        let text = String::from_utf8_lossy(data);
        let mut lines = text.lines();
        let mut result = ImportResult::default();

        // 解析标题行
        let header = match lines.next() {
            Some(h) => self.parse_csv_line(h),
            None => anyhow::bail!("Empty CSV file"),
        };

        // 查找各列索引
        let _folder_idx = self.find_column_index(&header, &["folder"]);
        let name_idx = self.find_column_index(&header, &["name", "title"]);
        let login_uri_idx = self.find_column_index(&header, &["login_uri", "uri", "url"]);
        let login_username_idx = self.find_column_index(&header, &["login_username", "username", "user"]);
        let login_password_idx = self.find_column_index(&header, &["login_password", "password", "pass"]);
        let login_totp_idx = self.find_column_index(&header, &["login_totp", "totp"]);
        let notes_idx = self.find_column_index(&header, &["notes", "note", "memo"]);

        // 处理数据行
        for line in lines {
            if line.trim().is_empty() {
                continue;
            }

            let fields = self.parse_csv_line(line);

            let name = name_idx
                .and_then(|i| fields.get(i).cloned())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "Unknown".to_string());

            let username = login_username_idx
                .and_then(|i| fields.get(i).cloned())
                .unwrap_or_default();

            let password = login_password_idx
                .and_then(|i| fields.get(i).cloned())
                .unwrap_or_default();

            let url = login_uri_idx
                .and_then(|i| fields.get(i).cloned())
                .filter(|s| !s.is_empty());

            let totp = login_totp_idx
                .and_then(|i| fields.get(i).cloned())
                .filter(|s| !s.is_empty());

            let notes = notes_idx
                .and_then(|i| fields.get(i).cloned())
                .filter(|s| !s.is_empty());

            // 如果没有密码且没有用户名，可能是无效行
            if password.is_empty() && username.is_empty() {
                result.warnings.push(format!("Skipped entry without credentials: {}", name));
                continue;
            }

            let mut entry = Entry::new(name.clone(), username, password);

            if let Some(u) = url {
                entry = entry.with_url(u);
            }
            if let Some(n) = notes {
                entry = entry.with_notes(n);
            }
            if let Some(t) = totp {
                entry = entry.with_otp(t);
            }

            result.entries.push(entry);
        }

        Ok(result)
    }
}

impl Default for BitwardenCsvImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::import::Importer for BitwardenCsvImporter {
    fn can_import(&self, data: &[u8]) -> bool {
        is_bitwarden_csv(data)
    }

    fn import(&self, data: &[u8]) -> anyhow::Result<ImportResult> {
        self.import(data)
    }
}

/// 检查是否是 Bitwarden CSV
pub fn is_bitwarden_csv(data: &[u8]) -> bool {
    let text = String::from_utf8_lossy(data);
    let lower = text.to_lowercase();

    // 检查是否包含 Bitwarden CSV 特征列
    lower.contains("folder") &&
    lower.contains("name") &&
    (lower.contains("login_uri") || lower.contains("login_username") || lower.contains("login_password"))
}

/// 检查是否是 Bitwarden JSON
pub fn is_bitwarden_json(data: &[u8]) -> bool {
    serde_json::from_slice::<BitwardenExport>(data).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_bitwarden_csv() {
        let valid_csv = "folder,name,login_uri,login_username,login_password\n";
        assert!(is_bitwarden_csv(valid_csv.as_bytes()));

        let invalid_csv = "name,password,url\n";
        assert!(!is_bitwarden_csv(invalid_csv.as_bytes()));
    }

    #[test]
    fn test_parse_bitwarden_csv() {
        let csv = r#"folder,name,login_uri,login_username,login_password,login_totp,notes
My Folder,GitHub,https://github.com,user@example.com,secret123,otpauth://totp/GitHub:user@example.com?secret=JBSWY3DPEHPK3PXP,My GitHub account"#;

        let importer = BitwardenCsvImporter::new();
        let result = importer.import(csv.as_bytes()).unwrap();

        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].name, "GitHub");
        assert_eq!(result.entries[0].username, "user@example.com");
        assert_eq!(result.entries[0].password, "secret123");
        assert_eq!(result.entries[0].url.as_deref(), Some("https://github.com"));
        assert_eq!(result.entries[0].otp_auth_url.as_deref().unwrap().starts_with("otpauth://totp/"), true);
        assert_eq!(result.entries[0].notes.as_deref(), Some("My GitHub account"));
    }

    #[test]
    fn test_is_bitwarden_json() {
        let valid_json = br#"{"encrypted":false,"items":[]}"#;
        assert!(is_bitwarden_json(valid_json));

        let invalid_json = b"not json at all";
        assert!(!is_bitwarden_json(invalid_json));
    }

    #[test]
    fn test_parse_bitwarden() {
        let json = r#"{
            "encrypted": false,
            "items": [{
                "id": "123",
                "name": "GitHub",
                "login": {
                    "uris": [{"uri": "https://github.com"}],
                    "username": "user@example.com",
                    "password": "secret123",
                    "totp": "otpauth://totp/GitHub/user@example.com?secret=JBSWY3DPEHPK3PXP"
                },
                "notes": "My GitHub account"
            }]
        }"#;

        let importer = BitwardenImporter::new();
        let result = importer.import(json.as_bytes()).unwrap();

        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].name, "GitHub");
        assert_eq!(result.entries[0].username, "user@example.com");
        assert_eq!(result.entries[0].password, "secret123");
    }
}