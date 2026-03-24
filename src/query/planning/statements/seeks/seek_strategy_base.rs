//! 查找策略基础模块
//!
//! 定义查找策略的基础类型和选择器

use crate::core::types::expr::visitor_checkers::PropertyContainsChecker;
use crate::core::types::Expression;
use crate::core::Value;
use crate::storage::StorageClient;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekStrategyType {
    VertexSeek,
    IndexSeek,
    PropIndexSeek,
    VariablePropIndexSeek,
    EdgeSeek,
    ScanSeek,
}

#[derive(Debug)]
pub struct SeekStrategyContext {
    pub space_id: u64,
    pub node_pattern: NodePattern,
    pub predicates: Vec<Expression>,
    pub estimated_rows: usize,
    pub available_indexes: Vec<IndexInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodePattern {
    pub vid: Option<Value>,
    pub labels: Vec<String>,
    pub properties: Vec<(String, Value)>,
}

#[derive(Debug, Clone)]
pub struct IndexInfo {
    pub name: String,
    pub target_type: String,
    pub target_name: String,
    pub properties: Vec<String>,
    pub selectivity: f32,
    pub field_count: usize,
}

impl IndexInfo {
    pub fn new(
        name: String,
        target_type: String,
        target_name: String,
        properties: Vec<String>,
    ) -> Self {
        let field_count = properties.len();
        Self {
            name,
            target_type,
            target_name,
            properties,
            selectivity: 0.5,
            field_count,
        }
    }

    pub fn with_selectivity(mut self, selectivity: f32) -> Self {
        self.selectivity = selectivity;
        self
    }
}

#[derive(Debug)]
pub struct SeekResult {
    pub vertex_ids: Vec<Value>,
    pub strategy_used: SeekStrategyType,
    pub rows_scanned: usize,
}

impl SeekStrategyContext {
    pub fn new(space_id: u64, node_pattern: NodePattern, predicates: Vec<Expression>) -> Self {
        Self {
            space_id,
            node_pattern,
            predicates,
            estimated_rows: 0,
            available_indexes: Vec::new(),
        }
    }

    pub fn with_estimated_rows(mut self, rows: usize) -> Self {
        self.estimated_rows = rows;
        self
    }

    pub fn with_indexes(mut self, indexes: Vec<IndexInfo>) -> Self {
        self.available_indexes = indexes;
        self
    }

    pub fn has_explicit_vid(&self) -> bool {
        self.node_pattern.vid.is_some()
    }

    pub fn has_labels(&self) -> bool {
        !self.node_pattern.labels.is_empty()
    }

    pub fn has_predicates(&self) -> bool {
        !self.predicates.is_empty()
    }

    pub fn get_index_for_labels(&self, labels: &[String]) -> Option<&IndexInfo> {
        self.available_indexes
            .iter()
            .find(|idx| idx.target_type == "tag" && labels.contains(&idx.target_name))
    }

    /// 获取指定属性的索引
    pub fn get_index_for_property(&self, property: &str) -> Option<&IndexInfo> {
        self.available_indexes
            .iter()
            .find(|idx| idx.properties.contains(&property.to_string()))
    }

    /// 检查是否有属性谓词
    pub fn has_property_predicates(&self) -> bool {
        // 检查谓词中是否包含属性过滤条件
        self.predicates
            .iter()
            .any(|pred| matches!(pred, Expression::Binary { .. }))
    }

    /// 检查是否有属性索引
    pub fn has_index_for_properties(&self) -> bool {
        !self.available_indexes.is_empty() && !self.predicates.is_empty()
    }
}

#[derive(Debug)]
pub struct SeekStrategySelector {
    use_index_threshold: usize,
    scan_threshold: usize,
}

impl SeekStrategySelector {
    pub fn new() -> Self {
        Self {
            use_index_threshold: 1000,
            scan_threshold: 10000,
        }
    }

    pub fn with_thresholds(mut self, use_index: usize, scan: usize) -> Self {
        self.use_index_threshold = use_index;
        self.scan_threshold = scan;
        self
    }

    pub fn select_best_index<'a>(
        &self,
        indexes: &'a [IndexInfo],
        predicates: &[Expression],
    ) -> Option<&'a IndexInfo> {
        if indexes.is_empty() {
            return None;
        }

        let candidate_indexes: Vec<&IndexInfo> = indexes
            .iter()
            .filter(|idx| {
                idx.properties.iter().any(|prop| {
                    predicates
                        .iter()
                        .any(|pred| PropertyContainsChecker::check(pred, std::slice::from_ref(prop)))
                })
            })
            .collect();

        if candidate_indexes.is_empty() {
            return indexes.iter().min_by_key(|idx| idx.field_count);
        }

        candidate_indexes.into_iter().min_by(|a, b| {
            let field_cmp = a.field_count.cmp(&b.field_count);
            if field_cmp != std::cmp::Ordering::Equal {
                return field_cmp;
            }
            b.selectivity
                .partial_cmp(&a.selectivity)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn select_strategy<S: StorageClient + ?Sized>(
        &self,
        _storage: &S,
        context: &SeekStrategyContext,
    ) -> SeekStrategyType {
        if context.has_explicit_vid() {
            SeekStrategyType::VertexSeek
        } else if context.has_property_predicates() && context.has_index_for_properties() {
            SeekStrategyType::PropIndexSeek
        } else if context.node_pattern.labels.is_empty() {
            SeekStrategyType::ScanSeek
        } else if context.get_index_for_labels(&context.node_pattern.labels).is_some() {
            if context.estimated_rows < self.scan_threshold {
                SeekStrategyType::IndexSeek
            } else {
                SeekStrategyType::ScanSeek
            }
        } else if context.estimated_rows < self.use_index_threshold {
            SeekStrategyType::VertexSeek
        } else {
            SeekStrategyType::ScanSeek
        }
    }
}

impl Default for SeekStrategySelector {
    fn default() -> Self {
        Self::new()
    }
}
