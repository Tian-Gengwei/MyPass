//! 导出模块
//!
//! # 概述
//! 提供将金库数据导出为标准格式的功能。
//!
//! # 支持格式
//! - JSON (通用)
//! - CSV (通用)
//! - Bitwarden JSON
//! - 1Password CSV (待实现)

mod json;
mod csv;

pub use json::JsonExporter;
pub use csv::CsvExporter;

use crate::vault::entry::Entry;

/// 导出器 trait
pub trait Exporter {
    /// 导出为字节数据
    fn export(&self, entries: &[Entry]) -> Vec<u8>;
    
    /// 获取导出格式的文件扩展名
    fn file_extension(&self) -> &str;
    
    /// 获取导出格式的名称
    fn format_name(&self) -> &str;
}

/// 导出格式枚举
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    /// JSON 格式
    Json,
    /// CSV 格式
    Csv,
    /// Bitwarden JSON 格式
    Bitwarden,
}

impl ExportFormat {
    /// 获取对应的导出器
    pub fn exporter(&self) -> Box<dyn Exporter> {
        match self {
            ExportFormat::Json => Box::new(JsonExporter::new()),
            ExportFormat::Csv => Box::new(CsvExporter::new()),
            ExportFormat::Bitwarden => Box::new(json::BitwardenJsonExporter::new()),
        }
    }
    
    /// 从文件扩展名获取格式
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "json" => Some(ExportFormat::Json),
            "csv" => Some(ExportFormat::Csv),
            _ => None,
        }
    }
}