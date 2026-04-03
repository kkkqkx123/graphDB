use crate::encoder::Encoder;
use crate::error::Result;
use crate::DocId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightOptions {
    pub template: String,
    pub boundary: Option<HighlightBoundaryOptions>,
    pub clip: Option<bool>,
    pub merge: Option<bool>,
    pub ellipsis: Option<HighlightEllipsisOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightBoundaryOptions {
    pub before: Option<i32>,
    pub after: Option<i32>,
    pub total: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightEllipsisOptions {
    pub template: String,
    pub pattern: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HighlightConfig {
    pub template: String,
    pub markup_open: String,
    pub markup_close: String,
    pub boundary: Option<HighlightBoundaryOptions>,
    pub clip: bool,
    pub merge: Option<String>,
    pub ellipsis: String,
    pub ellipsis_markup_length: usize,
}

impl HighlightConfig {
    pub fn from_options(options: &HighlightOptions) -> Result<Self> {
        let template = options.template.clone();

        let markup_open_pos = template.find("$1").ok_or_else(|| {
            crate::error::InversearchError::Encoder(crate::error::EncoderError::Encoding(
                "Invalid highlight template. The replacement pattern \"$1\" was not found"
                    .to_string(),
            ))
        })?;

        let markup_open = template[..markup_open_pos].to_string();
        let markup_close = template[markup_open_pos + 2..].to_string();

        let clip = options.clip.unwrap_or(true);
        let merge = if clip && !markup_open.is_empty() && !markup_close.is_empty() {
            Some(format!("{} {}", markup_close, markup_open))
        } else {
            None
        };

        let (ellipsis, ellipsis_markup_length) = if let Some(ellipsis_opts) = &options.ellipsis {
            let ellipsis_template = ellipsis_opts.template.clone();
            let ellipsis_markup_length = ellipsis_template.len() - 2;
            let ellipsis_pattern = ellipsis_opts.pattern.as_deref().unwrap_or("...");
            let ellipsis = ellipsis_template.replace("$1", ellipsis_pattern);
            (ellipsis, ellipsis_markup_length)
        } else {
            ("...".to_string(), 0)
        };

        Ok(HighlightConfig {
            template,
            markup_open,
            markup_close,
            boundary: options.boundary.clone(),
            clip,
            merge,
            ellipsis,
            ellipsis_markup_length,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SearchDocument {
    pub id: u64,
    pub doc: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct EnrichedSearchResult {
    pub id: u64,
    pub doc: Option<serde_json::Value>,
    pub highlight: Option<String>,
}

pub type EnrichedSearchResults = Vec<EnrichedSearchResult>;

#[derive(Debug, Clone)]
pub struct FieldSearchResult {
    pub field: String,
    pub result: EnrichedSearchResults,
}

pub type FieldSearchResults = Vec<FieldSearchResult>;

#[derive(Debug, Clone, Default)]
pub struct EncoderCache {
    cache: HashMap<String, Vec<String>>,
}

impl EncoderCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_encode(&mut self, query: &str, encoder: &Encoder) -> Result<Vec<String>> {
        // Use a simple string representation of encoder config as key
        let key = "encoder_key".to_string(); // Simplified for now

        if let Some(cached) = self.cache.get(&key) {
            return Ok(cached.clone());
        }

        let encoded = encoder.encode(query)?;
        self.cache.insert(key, encoded.clone());
        Ok(encoded)
    }
}

// ============================================================
// 新增：结构化高亮结果类型（方案 A - 并行架构）
// ============================================================

/// 单个匹配项的结构化信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightMatch {
    /// 匹配的原始文本
    pub text: String,
    /// 匹配的开始位置（字符级别）
    pub start_pos: usize,
    /// 匹配的结束位置
    pub end_pos: usize,
    /// 匹配的查询词
    pub matched_query: String,
    /// 匹配得分（可选，用于多匹配排序）
    pub score: Option<f64>,
}

/// 单个字段的高亮结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldHighlight {
    /// 字段名
    pub field: String,
    /// 所有匹配项
    pub matches: Vec<HighlightMatch>,
    /// 高亮后的完整文本（可选，方便前端直接使用）
    pub highlighted_text: Option<String>,
    /// 匹配的查询词列表
    pub matched_queries: Vec<String>,
}

/// 单个文档的高亮结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentHighlight {
    /// 文档ID
    pub id: DocId,
    /// 各字段的高亮结果
    pub fields: Vec<FieldHighlight>,
    /// 总匹配数
    pub total_matches: usize,
}

/// 搜索结果（不含高亮的基础结果）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: DocId,
    pub score: Option<f64>,
    pub doc: Option<serde_json::Value>,
}

/// 完整的搜索结果（含高亮）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultWithHighlight {
    pub results: Vec<SearchResult>,
    pub highlights: Vec<DocumentHighlight>,
    pub total: usize,
    pub query: String,
}
