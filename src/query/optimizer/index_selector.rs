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
    /// 不等于条件，无法使用索引
    NotEqual = 0,
    /// 范围条件
    Range = 1,
    /// 前缀匹配（等值）
    Prefix = 2,
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
}

impl IndexCandidate {
    /// 获取总评分（用于比较）
    pub fn total_score(&self) -> Vec<IndexScore> {
        self.scores.clone()
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
                }
                None => {
                    // 索引字段中断，后续字段无法使用索引前缀
                    break;
                }
            }
        }

        if hints.is_empty() {
            // 没有任何匹配的字段，返回全扫描候选
            return Some(IndexCandidate {
                index: index.clone(),
                scores: vec![IndexScore::NotEqual],
                column_hints: vec![],
            });
        }

        Some(IndexCandidate {
            index: index.clone(),
            scores,
            column_hints: hints,
        })
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
