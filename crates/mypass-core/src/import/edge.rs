//! Microsoft Edge 密码导入器模块
//!
//! # 概述
//! 支持从 Microsoft Edge 浏览器导出的密码数据。
//!
//! # 支持格式
//! - Edge CSV 导出格式
//! - Edge JSON 导出格式（如果支持）
//!
//! # 导入内容
//! - 登录条目（用户名、密码、URL）
//! - 备注

use super::{Importer, ImportResult};
use crate::vault::entry::Entry;
use serde::Deserialize;

/// Edge 浏览器导入器
pub struct EdgeImporter;

impl EdgeImporter {
    /// 创建新的 Edge 导入器
    pub fn new() -> Self {
        Self
    }
}

/// Edge JSON 导出结构
#[derive(Deserialize)]
struct EdgeJsonExport {
    /// 登录数据数组
    #[serde(rename = "logins")]
    logins: Option<Vec<EdgeJsonLogin>>,
}

/// Edge JSON 登录条目
#[derive(Deserialize)]
struct EdgeJsonLogin {
    /// URL
    #[serde(rename = "hostname")]
    hostname: Option<String>,
    /// 用户名
    #[serde(rename = "username")]
    username: Option<String>,
    /// 密码
    #[serde(rename = "password")]
    password: Option<String>,
    /// 备注
    #[serde(rename = "notes")]
    notes: Option<String>,
}

impl Importer for EdgeImporter {
    fn can_import(&self, data: &[u8]) -> bool {
        // 检查是否是 Edge CSV 格式（标题包含 URL, User Name, Password）
        let text = String::from_utf8_lossy(data);
        if text.contains("URL") && text.contains("User Name") && text.contains("Password") {
            return true;
        }
        // 检查是否是 Edge JSON 格式
        if serde_json::from_slice::<EdgeJsonExport>(data).is_ok() {
            return true;
        }
        false
    }

    fn import(&self, data: &[u8]) -> anyhow::Result<ImportResult> {
        // 尝试 CSV 格式
        let text = String::from_utf8_lossy(data);
        if text.contains("URL") && text.contains("User Name") {
            return import_csv(&text);
        }
        
        // 尝试 JSON 格式
        if let Ok(export) = serde_json::from_slice::<EdgeJsonExport>(data) {
            return import_json(&export);
        }
        
        anyhow::bail!("Unable to parse Edge format")
    }
}

/// 从 CSV 导入
fn import_csv(csv_data: &str) -> anyhow::Result<ImportResult> {
    let mut result = ImportResult::default();
    let mut lines = csv_data.lines();
    
    // 跳过标题行
    if let Some(header) = lines.next() {
        if !header.contains("URL") {
            anyhow::bail!("Invalid Edge CSV format");
        }
    }

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }

        let fields = parse_edge_csv_line(line);
        if fields.len() < 3 {
            continue;
        }

        let url = fields.get(0).cloned().unwrap_or_default();
        let username = fields.get(1).cloned().unwrap_or_default();
        let password = fields.get(2).cloned().unwrap_or_default();
        let note = fields.get(3).cloned().filter(|s| !s.is_empty());

        // 跳过空 URL
        if url.is_empty() {
            continue;
        }

        let name = extract_name_from_url(&url);
        let mut entry = Entry::new(name, username, password);
        entry = entry.with_url(url);

        if let Some(n) = note {
            entry = entry.with_notes(n);
        }

        result.entries.push(entry);
    }

    Ok(result)
}

/// 从 JSON 导入
fn import_json(export: &EdgeJsonExport) -> anyhow::Result<ImportResult> {
    let mut result = ImportResult::default();

    if let Some(logins) = &export.logins {
        for login in logins {
            let url = login.hostname.clone().unwrap_or_default();
            let username = login.username.clone().unwrap_or_default();
            let password = login.password.clone().unwrap_or_default();
            let notes = login.notes.clone().filter(|s| !s.is_empty());

            // 跳过空 URL
            if url.is_empty() {
                continue;
            }

            let name = extract_name_from_url(&url);
            let mut entry = Entry::new(name, username, password);
            entry = entry.with_url(url);

            if let Some(n) = notes {
                entry = entry.with_notes(n);
            }

            result.entries.push(entry);
        }
    }

    Ok(result)
}

/// 从 URL 提取域名作为名称
fn extract_name_from_url(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        parsed.host_str().unwrap_or(url).to_string()
    } else {
        // 尝试提取域名部分
        url.split('/')
            .nth(2) // 跳过协议和空部分
            .unwrap_or(url)
            .split(':')
            .next()
            .unwrap_or(url)
            .to_string()
    }
}

/// 解析 Edge CSV 行（处理引号和转义）
fn parse_edge_csv_line(line: &str) -> Vec<String> {
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
                // 检查是否是转义引号
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