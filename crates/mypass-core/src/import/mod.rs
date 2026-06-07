//! 导入模块
//!
//! # 概述
//! 提供从各种密码管理器格式导入数据的功能。
//!
//! # 支持格式
//! - KeePass KDBX (需要密码)
//! - Bitwarden JSON/CSV
//! - Microsoft Edge CSV
//! - 通用 CSV

pub mod keepass;
pub mod bitwarden;
pub mod edge;
pub mod chrome;

pub use keepass::KeepassImporter;
pub use bitwarden::BitwardenImporter;
pub use edge::EdgeImporter;
pub use chrome::ChromeCsvImporter;

use crate::vault::entry::{Entry, Group};

/// 导入器 trait
pub trait Importer {
    /// 检查是否能导入此数据
    fn can_import(&self, data: &[u8]) -> bool;
    
    /// 执行导入
    fn import(&self, data: &[u8]) -> anyhow::Result<ImportResult>;
}

/// 导入结果
#[derive(Debug, Default)]
pub struct ImportResult {
    /// 导入的条目
    pub entries: Vec<Entry>,
    /// 导入的分组
    pub groups: Vec<Group>,
    /// 警告信息
    pub warnings: Vec<String>,
}

/// 导入格式类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImportFormat {
    /// KeePass KDBX
    Keepass,
    /// Bitwarden JSON/CSV
    Bitwarden,
    /// Microsoft Edge
    Edge,
    /// 通用 CSV
    Csv,
    /// 未知格式
    Unknown,
}

/// 通用导入服务
pub struct ImportService {
    importers: Vec<Box<dyn Importer + Send + Sync>>,
}

impl ImportService {
    /// 创建新的导入服务
    pub fn new() -> Self {
        Self {
            importers: Vec::new(),
        }
    }
    
    /// 添加导入器
    pub fn add_importer<I: Importer + Send + Sync + 'static>(&mut self, importer: I) {
        self.importers.push(Box::new(importer));
    }
    
    /// 自动检测格式
    pub fn detect_format(&self, data: &[u8]) -> ImportFormat {
        // 检查 KeePass
        if data.len() > 4 && &data[0..4] == b"KDBX" {
            return ImportFormat::Keepass;
        }
        
        // 检查 Bitwarden JSON
        if serde_json::from_slice::<BitwardenExportFormat>(data).is_ok() {
            return ImportFormat::Bitwarden;
        }
        
        // 检查 Edge CSV
        let text = String::from_utf8_lossy(data);
        if text.contains("\"URL\"") && text.contains("\"User Name\"") {
            return ImportFormat::Edge;
        }
        
        // 检查 Bitwarden CSV
        if text.contains("folder,name,group,username") {
            return ImportFormat::Bitwarden;
        }
        
        // 检查通用 CSV (至少包含 name,password 列)
        let lower = text.to_lowercase();
        if lower.contains("name") && (lower.contains("password") || lower.contains("pass")) {
            return ImportFormat::Csv;
        }
        
        ImportFormat::Unknown
    }
    
    /// 导入数据（自动检测格式）
    pub fn import_auto(&self, data: &[u8]) -> anyhow::Result<ImportResult> {
        for importer in &self.importers {
            if importer.can_import(data) {
                return importer.import(data);
            }
        }
        anyhow::bail!("No suitable importer found for this format")
    }
    
    /// 导入指定格式
    pub fn import_with_format(&self, data: &[u8], format: ImportFormat, password: Option<&str>) -> anyhow::Result<ImportResult> {
        let importer: Box<dyn Importer> = match format {
            ImportFormat::Keepass => {
                if let Some(pwd) = password {
                    Box::new(KeepassImporter::with_password(pwd.to_string()))
                } else {
                    Box::new(KeepassImporter::new())
                }
            }
            ImportFormat::Bitwarden => Box::new(BitwardenImporter::new()),
            ImportFormat::Edge => Box::new(EdgeImporter::new()),
            ImportFormat::Csv => Box::new(CsvImporter::new()),
            ImportFormat::Unknown => anyhow::bail!("Unknown format"),
        };
        
        importer.import(data)
    }
}

impl Default for ImportService {
    fn default() -> Self {
        let mut service = Self::new();
        service.add_importer(KeepassImporter::new());
        service.add_importer(BitwardenImporter::new());
        service.add_importer(EdgeImporter::new());
        service.add_importer(CsvImporter::new());
        service
    }
}

/// Bitwarden 导出格式检测
#[derive(serde::Deserialize)]
struct BitwardenExportFormat {
    // 用于格式检测，不需要使用这些字段
}

/// 通用 CSV 导入器
pub struct CsvImporter;

impl CsvImporter {
    pub fn new() -> Self {
        Self
    }
    
    fn parse_line(&self, line: &str) -> Vec<String> {
        let mut fields = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        
        for ch in line.chars() {
            match ch {
                '"' => in_quotes = !in_quotes,
                ',' if !in_quotes => {
                    fields.push(current.trim().to_string());
                    current.clear();
                }
                _ => current.push(ch),
            }
        }
        fields.push(current.trim().to_string());
        fields
    }
}

impl Importer for CsvImporter {
    fn can_import(&self, data: &[u8]) -> bool {
        // 检查是否有标题行包含 name/password 列
        let text = String::from_utf8_lossy(data);
        let lower = text.to_lowercase();
        (lower.contains("name") || lower.contains("title")) && 
           (lower.contains("password") || lower.contains("pass"))
    }
    
    fn import(&self, data: &[u8]) -> anyhow::Result<ImportResult> {
        let text = String::from_utf8_lossy(data);
        let mut result = ImportResult::default();
        let mut lines = text.lines();
        
        // 解析标题行
        let header = match lines.next() {
            Some(h) => self.parse_line(h),
            None => anyhow::bail!("Empty CSV file"),
        };
        
        // 查找列索引
        let name_idx = header.iter().position(|h| {
            let lower = h.to_lowercase();
            lower.contains("name") || lower.contains("title") || lower.contains("login")
        });
        let user_idx = header.iter().position(|h| {
            let lower = h.to_lowercase();
            lower.contains("user") || lower.contains("email")
        });
        let pass_idx = header.iter().position(|h| {
            let lower = h.to_lowercase();
            lower.contains("password") || lower.contains("pass")
        });
        let url_idx = header.iter().position(|h| {
            let lower = h.to_lowercase();
            lower.contains("url") || lower.contains("website") || lower.contains("site")
        });
        let note_idx = header.iter().position(|h| {
            let lower = h.to_lowercase();
            lower.contains("note") || lower.contains("memo") || lower.contains("comment")
        });
        
        // 处理数据行
        for line in lines {
            if line.trim().is_empty() {
                continue;
            }
            
            let fields = self.parse_line(line);
            
            let name = name_idx.and_then(|i| fields.get(i).cloned()).unwrap_or_else(|| "Unknown".to_string());
            let username = user_idx.and_then(|i| fields.get(i).cloned()).unwrap_or_default();
            let password = pass_idx.and_then(|i| fields.get(i).cloned()).unwrap_or_default();
            let url = url_idx.and_then(|i| fields.get(i).cloned()).filter(|s| !s.is_empty());
            let notes = note_idx.and_then(|i| fields.get(i).cloned()).filter(|s| !s.is_empty());
            
            let mut entry = Entry::new(name, username, password);
            
            if let Some(u) = url {
                entry = entry.with_url(u);
            }
            if let Some(n) = notes {
                entry = entry.with_notes(n);
            }
            
            result.entries.push(entry);
        }
        
        Ok(result)
    }
}
