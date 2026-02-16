//! 索引选择器
//! 根据查询条件选择最优索引
//! 参考 nebula-graph 的 OptimizerUtils::selectIndex 实现

use crate::core::{Expression, Value};
use crate::core::types::operators::BinaryOperator;
use crate::index::Index;
use crate::query::planner::plan::algorithms::{IndexLimit, ScanType};
use std::collections::HashMap;

/// 索引评分
/// 分数越高表示索引越适合该查询
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IndexScore {
    /// 无匹配，无法使用索引
    NoMatch = 0,
    /// 不等于条件，无法有效使用索引
    NotEqual = 1,
    /// 范围条件
    Range = 2,
    /// 前缀匹配（等值）
    Prefix = 3,
    /// 完全匹配（所有字段都有等值条件）
    FullMatch = 4,
}

/// 索引评分详情
#[derive(Debug, Clone)]
pub struct IndexScoreDetail {
    /// 索引ID
    pub index_id: i32,
    /// 索引名称
    pub index_name: String,
    /// 总评分
    pub total_score: i32,
    /// 匹配字段数
    pub matched_fields: usize,
    /// 总字段数
    pub total_fields: usize,
    /// 匹配率 (0.0 - 1.0)
    pub match_ratio: f64,
    /// 每个字段的评分
    pub field_scores: Vec<(String, IndexScore)>,
    /// 是否可以唯一确定记录
    pub is_unique_match: bool,
    /// 预估扫描行数
    pub estimated_rows: usize,
}

/// 列约束类型
#[derive(Debug, Clone)]
pub enum ColumnConstraint {
    /// 等值约束
    Equal(Value),
    /// 范围约束
    Range {
        start: Option<Value>,
        end: Option<Value>,
        include_start: bool,
        include_end: bool,
    },
}

/// 索引候选结果
#[derive(Debug, Clone)]
pub struct IndexCandidate {
    /// 索引定义
    pub index: Index,
    /// 每个字段的评分
    pub scores: Vec<IndexScore>,
    /// 列提示信息
    pub column_hints: Vec<IndexColumnHint>,
    /// 评分详情
    pub score_detail: Option<IndexScoreDetail>,
}

impl IndexCandidate {
    /// 获取总评分（用于比较）
    pub fn total_score(&self) -> Vec<IndexScore> {
        self.scores.clone()
    }

    /// 计算总评分值
    pub fn calculate_total_score(&self) -> i32 {
        self.scores.iter().map(|s| *s as i32).sum()
    }

    /// 获取评分详情
    pub fn get_score_detail(&self) -> Option<IndexScoreDetail> {
        self.score_detail.clone()
    }
}

impl PartialEq for IndexCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.index.id == other.index.id
    }
}

impl Eq for IndexCandidate {}

/// 索引列提示
#[derive(Debug, Clone)]
pub struct IndexColumnHint {
    /// 列名
    pub column_name: String,
    /// 扫描类型
    pub scan_type: ScanType,
    /// 起始值
    pub begin_value: Option<Value>,
    /// 结束值
    pub end_value: Option<Value>,
    /// 是否包含起始值
    pub include_begin: bool,
    /// 是否包含结束值
    pub include_end: bool,
}

/// 索引选择器
pub struct IndexSelector;

impl IndexSelector {
    /// 选择最优索引
    ///
    /// # 参数
    /// - `indexes`: 可用索引列表
    /// - `filter`: 过滤条件表达式
    ///
    /// # 返回
    /// 最优的索引候选，如果没有合适的索引则返回 None
    pub fn select_best_index(
        indexes: &[Index],
        filter: &Option<Expression>,
    ) -> Option<IndexCandidate> {
        if indexes.is_empty() {
            return None;
        }

        // 1. 从过滤条件中提取列约束
        let constraints = Self::extract_constraints(filter)?;

        // 2. 评估每个索引
        let mut candidates: Vec<IndexCandidate> = indexes
            .iter()
            .filter_map(|idx| Self::evaluate_index(idx, &constraints))
            .collect();

        if candidates.is_empty() {
            return None;
        }

        // 3. 选择最优索引
        // 先比较得分序列，再比较字段数量
        candidates.sort_by(|a, b| {
            for (s1, s2) in a.scores.iter().zip(b.scores.iter()) {
                match s2.cmp(s1) {
                    // 注意：这里是逆序，分数高的排在前面
                    std::cmp::Ordering::Equal => continue,
                    other => return other,
                }
            }
            // 字段多的排在前面
            b.scores.len().cmp(&a.scores.len())
        });

        candidates.into_iter().next()
    }

    /// 从表达式中提取列约束
    fn extract_constraints(filter: &Option<Expression>) -> Option<HashMap<String, ColumnConstraint>> {
        let filter = filter.as_ref()?;
        let mut constraints = HashMap::new();
        Self::extract_constraints_recursive(filter, &mut constraints);
        Some(constraints)
    }

    /// 递归提取约束
    fn extract_constraints_recursive(expr: &Expression, constraints: &mut HashMap<String, ColumnConstraint>) {
        match expr {
            Expression::Binary { left, op, right } => {
                if let (Some(col_name), Some(value)) = (Self::extract_column_name(left), Self::extract_value(right)) {
                    match op {
                        BinaryOperator::Equal => {
                            constraints.insert(col_name, ColumnConstraint::Equal(value));
                        }
                        BinaryOperator::GreaterThan => {
                            constraints.insert(col_name, ColumnConstraint::Range {
                                start: Some(value),
                                end: None,
                                include_start: false,
                                include_end: false,
                            });
                        }
                        BinaryOperator::GreaterThanOrEqual => {
                            constraints.insert(col_name, ColumnConstraint::Range {
                                start: Some(value),
                                end: None,
                                include_start: true,
                                include_end: false,
                            });
                        }
                        BinaryOperator::LessThan => {
                            constraints.insert(col_name, ColumnConstraint::Range {
                                start: None,
                                end: Some(value),
                                include_start: false,
                                include_end: false,
                            });
                        }
                        BinaryOperator::LessThanOrEqual => {
                            constraints.insert(col_name, ColumnConstraint::Range {
                                start: None,
                                end: Some(value),
                                include_start: false,
                                include_end: true,
                            });
                        }
                        _ => {}
                    }
                }
            }
            Expression::Function { name, args } if name.eq_ignore_ascii_case("and") => {
                // 合并 AND 条件的约束
                for operand in args {
                    Self::extract_constraints_recursive(operand, constraints);
                }
            }
            _ => {}
        }
    }

    /// 提取列名
    fn extract_column_name(expr: &Expression) -> Option<String> {
        match expr {
            Expression::Property { property, .. } => Some(property.clone()),
            Expression::Variable(name) => Some(name.clone()),
            Expression::TagProperty { property, .. } => Some(property.clone()),
            Expression::EdgeProperty { property, .. } => Some(property.clone()),
            _ => None,
        }
    }

    /// 提取常量值
    fn extract_value(expr: &Expression) -> Option<Value> {
        match expr {
            Expression::Literal(value) => Some(value.clone()),
            _ => None,
        }
    }

    /// 评估索引匹配度
    fn evaluate_index(index: &Index, constraints: &HashMap<String, ColumnConstraint>) -> Option<IndexCandidate> {
        if index.fields.is_empty() {
            return None;
        }

        let mut hints = Vec::new();
        let mut scores = Vec::new();
        let mut field_scores = Vec::new();
        let mut matched_fields = 0;
        let total_fields = index.fields.len();
        let mut is_unique_match = index.is_unique;

        // 按索引字段顺序匹配约束
        for field in &index.fields {
            match constraints.get(&field.name) {
                Some(ColumnConstraint::Equal(value)) => {
                    hints.push(IndexColumnHint {
                        column_name: field.name.clone(),
                        scan_type: ScanType::Unique,
                        begin_value: Some(value.clone()),
                        end_value: Some(value.clone()),
                        include_begin: true,
                        include_end: true,
                    });
                    scores.push(IndexScore::Prefix);
                    field_scores.push((field.name.clone(), IndexScore::Prefix));
                    matched_fields += 1;
                }
                Some(ColumnConstraint::Range { start, end, include_start, include_end }) => {
                    hints.push(IndexColumnHint {
                        column_name: field.name.clone(),
                        scan_type: ScanType::Range,
                        begin_value: start.clone(),
                        end_value: end.clone(),
                        include_begin: *include_start,
                        include_end: *include_end,
                    });
                    scores.push(IndexScore::Range);
                    field_scores.push((field.name.clone(), IndexScore::Range));
                    matched_fields += 1;
                    // 范围条件不能唯一确定记录
                    is_unique_match = false;
                }
                None => {
                    // 索引字段中断，后续字段无法使用索引前缀
                    field_scores.push((field.name.clone(), IndexScore::NoMatch));
                    is_unique_match = false;
                    break;
                }
            }
        }

        // 如果所有字段都匹配且都是等值条件，则为完全匹配
        if matched_fields == total_fields && scores.iter().all(|s| matches!(s, IndexScore::Prefix)) {
            scores = vec![IndexScore::FullMatch; scores.len()];
            for (_, score) in field_scores.iter_mut() {
                *score = IndexScore::FullMatch;
            }
        }

        // 计算匹配率
        let match_ratio = if total_fields > 0 {
            matched_fields as f64 / total_fields as f64
        } else {
            0.0
        };

        // 计算预估扫描行数
        let estimated_rows = Self::estimate_rows(&hints, index.is_unique);

        // 创建评分详情
        let score_detail = IndexScoreDetail {
            index_id: index.id,
            index_name: index.name.clone(),
            total_score: scores.iter().map(|s| *s as i32).sum(),
            matched_fields,
            total_fields,
            match_ratio,
            field_scores,
            is_unique_match,
            estimated_rows,
        };

        if hints.is_empty() {
            // 没有任何匹配的字段，返回全扫描候选
            return Some(IndexCandidate {
                index: index.clone(),
                scores: vec![IndexScore::NotEqual],
                column_hints: vec![],
                score_detail: Some(score_detail),
            });
        }

        Some(IndexCandidate {
            index: index.clone(),
            scores,
            column_hints: hints,
            score_detail: Some(score_detail),
        })
    }

    /// 预估扫描行数
    fn estimate_rows(hints: &[IndexColumnHint], is_unique: bool) -> usize {
        if hints.is_empty() {
            return 10000; // 默认全表扫描预估
        }

        if is_unique && hints.iter().all(|h| h.scan_type == ScanType::Unique) {
            return 1; // 唯一索引等值查询
        }

        // 根据扫描类型估算
        let mut rows = 1000usize;
        for hint in hints {
            match hint.scan_type {
                ScanType::Unique => rows = rows.saturating_div(10),
                ScanType::Range => rows = rows.saturating_div(5),
                ScanType::Prefix => rows = rows.saturating_div(8), // 前缀匹配介于等值和范围之间
                ScanType::Full => rows = rows.saturating_mul(10),
            }
        }

        rows.max(1)
    }

    /// 评估所有索引并返回评分列表
    pub fn evaluate_all_indexes(
        indexes: &[Index],
        filter: &Option<Expression>,
    ) -> Vec<IndexScoreDetail> {
        let constraints = match Self::extract_constraints(filter) {
            Some(c) => c,
            None => return vec![],
        };

        indexes
            .iter()
            .filter_map(|idx| Self::evaluate_index(idx, &constraints))
            .filter_map(|candidate| candidate.score_detail)
            .collect()
    }

    /// 选择最优索引并返回评分详情
    pub fn select_best_index_with_detail(
        indexes: &[Index],
        filter: &Option<Expression>,
    ) -> Option<(IndexCandidate, IndexScoreDetail)> {
        let candidate = Self::select_best_index(indexes, filter)?;
        let detail = candidate.score_detail.clone()?;
        Some((candidate, detail))
    }

    /// 将列提示转换为 IndexLimit
    pub fn hints_to_limits(hints: &[IndexColumnHint]) -> Vec<IndexLimit> {
        hints
            .iter()
            .map(|hint| IndexLimit {
                column: hint.column_name.clone(),
                begin_value: hint.begin_value.as_ref().map(|v| format!("{:?}", v)),
                end_value: hint.end_value.as_ref().map(|v| format!("{:?}", v)),
                include_begin: hint.include_begin,
                include_end: hint.include_end,
                scan_type: hint.scan_type,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::operators::BinaryOperator;
    use crate::index::{IndexField, IndexStatus, IndexType};

    fn create_test_index(id: i32, fields: Vec<&str>) -> Index {
        Index {
            id,
            name: format!("idx_{}", id),
            space_id: 1,
            schema_name: "test".to_string(),
            fields: fields
                .into_iter()
                .map(|name| IndexField::new(
                    name.to_string(),
                    crate::core::Value::String("".to_string()),
                    true,
                ))
                .collect(),
            properties: vec![],
            index_type: IndexType::TagIndex,
            status: IndexStatus::Active,
            is_unique: false,
            comment: None,
        }
    }

    #[test]
    fn test_select_best_index_with_equal() {
        let indexes = vec![
            create_test_index(1, vec!["name", "age"]),
            create_test_index(2, vec!["age"]),
        ];

        // name = 'test'
        let filter = Some(Expression::Binary {
            left: Box::new(Expression::Variable("name".to_string())),
            op: BinaryOperator::Equal,
            right: Box::new(Expression::Literal(Value::String("test".to_string()))),
        });

        let result = IndexSelector::select_best_index(&indexes, &filter);
        assert!(result.is_some());
        let candidate = result.unwrap();
        assert_eq!(candidate.index.id, 1); // 选择 name 开头的索引
        assert_eq!(candidate.scores, vec![IndexScore::Prefix]);
    }

    #[test]
    fn test_select_best_index_with_range() {
        let indexes = vec![
            create_test_index(1, vec!["name", "age"]),
            create_test_index(2, vec!["age"]),
        ];

        // age > 18
        let filter = Some(Expression::Binary {
            left: Box::new(Expression::Variable("age".to_string())),
            op: BinaryOperator::GreaterThan,
            right: Box::new(Expression::Literal(Value::Int(18))),
        });

        let result = IndexSelector::select_best_index(&indexes, &filter);
        assert!(result.is_some());
        let candidate = result.unwrap();
        assert_eq!(candidate.index.id, 2); // 选择 age 索引
        assert_eq!(candidate.scores, vec![IndexScore::Range]);
    }

    #[test]
    fn test_no_matching_index() {
        let indexes = vec![create_test_index(1, vec!["name"])];

        // age > 18 (没有匹配的索引)
        let filter = Some(Expression::Binary {
            left: Box::new(Expression::Variable("age".to_string())),
            op: BinaryOperator::GreaterThan,
            right: Box::new(Expression::Literal(Value::Int(18))),
        });

        let result = IndexSelector::select_best_index(&indexes, &filter);
        // 返回全扫描候选
        assert!(result.is_some());
        let candidate = result.unwrap();
        assert_eq!(candidate.scores, vec![IndexScore::NotEqual]);
    }
}
