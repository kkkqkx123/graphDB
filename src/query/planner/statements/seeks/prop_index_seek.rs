//! 属性索引查找策略
//!
//! 基于属性条件的索引查找，支持等值、范围、前缀等查询
//!
//! 适用场景:
//! - MATCH (v:Person) WHERE v.age > 18
//! - MATCH (v:Person) WHERE v.name = "Alice"
//! - MATCH (v:Person) WHERE v.name STARTS WITH "A"

use super::seek_strategy::SeekStrategy;
use super::seek_strategy_base::{IndexInfo, SeekResult, SeekStrategyContext, SeekStrategyType};
use crate::core::{StorageError, Value};
use crate::storage::StorageClient;

/// 属性过滤条件
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyPredicate {
    pub property: String,
    pub op: PredicateOp,
    pub value: Value,
}

/// 谓词操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredicateOp {
    Eq,      // =
    Ne,      // !=
    Lt,      // <
    Le,      // <=
    Gt,      // >
    Ge,      // >=
    In,      // IN
    StartsWith, // STARTS WITH
}

impl PredicateOp {
    /// 从字符串解析操作符
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "=" | "==" => Some(PredicateOp::Eq),
            "!=" | "<>" => Some(PredicateOp::Ne),
            "<" => Some(PredicateOp::Lt),
            "<=" => Some(PredicateOp::Le),
            ">" => Some(PredicateOp::Gt),
            ">=" => Some(PredicateOp::Ge),
            "IN" | "in" => Some(PredicateOp::In),
            "STARTS WITH" | "starts with" => Some(PredicateOp::StartsWith),
            _ => None,
        }
    }

    /// 检查是否为范围操作
    pub fn is_range(&self) -> bool {
        matches!(self, PredicateOp::Lt | PredicateOp::Le | PredicateOp::Gt | PredicateOp::Ge)
    }

    /// 检查是否为等值操作
    pub fn is_equality(&self) -> bool {
        matches!(self, PredicateOp::Eq | PredicateOp::In)
    }
}

/// 属性索引查找策略
#[derive(Debug, Clone)]
pub struct PropIndexSeek {
    predicates: Vec<PropertyPredicate>,
}

impl PropIndexSeek {
    pub fn new(predicates: Vec<PropertyPredicate>) -> Self {
        Self { predicates }
    }

    /// 从表达式列表提取属性谓词
    pub fn extract_predicates(expressions: &[crate::core::Expression]) -> Vec<PropertyPredicate> {
        let mut predicates = Vec::new();
        
        for expr in expressions {
            if let Some(pred) = Self::extract_predicate(expr) {
                predicates.push(pred);
            }
        }
        
        predicates
    }

    /// 从单个表达式提取属性谓词
    fn extract_predicate(expr: &crate::core::Expression) -> Option<PropertyPredicate> {
        use crate::core::types::operators::BinaryOperator;
        
        match expr {
            crate::core::Expression::Binary { op, left, right } => {
                let op_str = match op {
                    BinaryOperator::Equal => "=",
                    BinaryOperator::NotEqual => "!=",
                    BinaryOperator::LessThan => "<",
                    BinaryOperator::LessThanOrEqual => "<=",
                    BinaryOperator::GreaterThan => ">",
                    BinaryOperator::GreaterThanOrEqual => ">=",
                    _ => return None,
                };
                
                // 尝试提取属性名和值
                if let (Some(prop), Some(val)) = (Self::extract_property(left), Self::extract_value(right)) {
                    if let Some(pred_op) = PredicateOp::from_str(op_str) {
                        return Some(PropertyPredicate {
                            property: prop,
                            op: pred_op,
                            value: val,
                        });
                    }
                }
                
                // 交换左右尝试
                if let (Some(prop), Some(val)) = (Self::extract_property(right), Self::extract_value(left)) {
                    let swapped_op = match op_str {
                        "<" => PredicateOp::Gt,
                        "<=" => PredicateOp::Ge,
                        ">" => PredicateOp::Lt,
                        ">=" => PredicateOp::Le,
                        _ => PredicateOp::from_str(op_str)?,
                    };
                    return Some(PropertyPredicate {
                        property: prop,
                        op: swapped_op,
                        value: val,
                    });
                }
                
                None
            }
            _ => None,
        }
    }

    /// 从表达式提取属性名
    fn extract_property(expr: &crate::core::Expression) -> Option<String> {
        match expr {
            crate::core::Expression::Property { object, property } => {
                // 检查是否为节点属性访问，如 v.name
                if matches!(object.as_ref(), crate::core::Expression::Variable(_)) {
                    Some(property.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// 从表达式提取值
    fn extract_value(expr: &crate::core::Expression) -> Option<Value> {
        match expr {
            crate::core::Expression::Literal(val) => Some(val.clone()),
            _ => None,
        }
    }

    /// 查找适合属性谓词的索引
    fn find_best_index<'a>(&'a self, context: &'a SeekStrategyContext) -> Option<(&'a IndexInfo, &'a PropertyPredicate)> {
        for pred in &self.predicates {
            if let Some(index) = context.get_index_for_property(&pred.property) {
                return Some((index, pred));
            }
        }
        None
    }

    /// 评估值是否满足谓词条件
    fn value_matches(&self, value: &Value, pred: &PropertyPredicate) -> bool {
        match pred.op {
            PredicateOp::Eq => value == &pred.value,
            PredicateOp::Ne => value != &pred.value,
            PredicateOp::Lt => Self::compare_values(value, &pred.value).map(|c| c < 0).unwrap_or(false),
            PredicateOp::Le => Self::compare_values(value, &pred.value).map(|c| c <= 0).unwrap_or(false),
            PredicateOp::Gt => Self::compare_values(value, &pred.value).map(|c| c > 0).unwrap_or(false),
            PredicateOp::Ge => Self::compare_values(value, &pred.value).map(|c| c >= 0).unwrap_or(false),
            PredicateOp::In => {
                // IN 操作需要值是列表
                matches!(&pred.value, Value::List(list) if list.contains(value))
            }
            PredicateOp::StartsWith => {
                if let (Value::String(s1), Value::String(s2)) = (value, &pred.value) {
                    s1.starts_with(s2)
                } else {
                    false
                }
            }
        }
    }

    /// 比较两个值
    fn compare_values(left: &Value, right: &Value) -> Option<i32> {
        match (left, right) {
            (Value::Int(i1), Value::Int(i2)) => Some(i1.cmp(i2) as i32),
            (Value::Float(f1), Value::Float(f2)) => f1.partial_cmp(f2).map(|c| c as i32),
            (Value::Int(i), Value::Float(f)) => (*i as f64).partial_cmp(f).map(|c| c as i32),
            (Value::Float(f), Value::Int(i)) => f.partial_cmp(&(*i as f64)).map(|c| c as i32),
            (Value::String(s1), Value::String(s2)) => Some(s1.cmp(s2) as i32),
            _ => None,
        }
    }
}

impl SeekStrategy for PropIndexSeek {
    fn execute<S: StorageClient>(
        &self,
        storage: &S,
        context: &SeekStrategyContext,
    ) -> Result<SeekResult, StorageError> {
        let mut vertex_ids = Vec::new();
        let mut rows_scanned = 0;

        // 查找最佳索引
        if let Some((index_info, primary_pred)) = self.find_best_index(context) {
            // 获取标签对应的顶点
            let space_name = "default"; // 实际应从 context 获取
            let vertices = storage.scan_vertices_by_tag(space_name, &index_info.target_name)?;
            rows_scanned = vertices.len();

            // 过滤满足所有谓词的顶点
            for vertex in vertices {
                let mut matches_all = true;
                
                // 检查主谓词
                if let Some(prop_value) = vertex.get_property_any(&primary_pred.property) {
                    if !self.value_matches(prop_value, primary_pred) {
                        matches_all = false;
                    }
                } else {
                    matches_all = false;
                }

                // 检查其他谓词
                if matches_all {
                    for pred in &self.predicates {
                        if pred.property != primary_pred.property {
                            if let Some(prop_value) = vertex.get_property_any(&pred.property) {
                                if !self.value_matches(prop_value, pred) {
                                    matches_all = false;
                                    break;
                                }
                            } else {
                                matches_all = false;
                                break;
                            }
                        }
                    }
                }

                if matches_all {
                    vertex_ids.push(vertex.vid().clone());
                }
            }
        }

        Ok(SeekResult {
            vertex_ids,
            strategy_used: SeekStrategyType::PropIndexSeek,
            rows_scanned,
        })
    }

    fn supports(&self, _context: &SeekStrategyContext) -> bool {
        // 只要有属性谓词就支持
        !self.predicates.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;

    #[test]
    fn test_predicate_op_from_str() {
        assert_eq!(PredicateOp::from_str("="), Some(PredicateOp::Eq));
        assert_eq!(PredicateOp::from_str("<"), Some(PredicateOp::Lt));
        assert_eq!(PredicateOp::from_str(">="), Some(PredicateOp::Ge));
        assert_eq!(PredicateOp::from_str("IN"), Some(PredicateOp::In));
        assert_eq!(PredicateOp::from_str("STARTS WITH"), Some(PredicateOp::StartsWith));
        assert_eq!(PredicateOp::from_str("unknown"), None);
    }

    #[test]
    fn test_extract_predicate_eq() {
        let expr = Expression::binary(
            Expression::property(Expression::variable("v"), "age"),
            crate::core::BinaryOperator::Equal,
            Expression::literal(18),
        );

        let pred = PropIndexSeek::extract_predicate(&expr);
        assert!(pred.is_some());
        
        let pred = pred.expect("Failed to extract predicate");
        assert_eq!(pred.property, "age");
        assert_eq!(pred.op, PredicateOp::Eq);
        assert_eq!(pred.value, Value::Int(18));
    }

    #[test]
    fn test_value_matches() {
        let seek = PropIndexSeek::new(vec![]);
        
        let pred = PropertyPredicate {
            property: "age".to_string(),
            op: PredicateOp::Gt,
            value: Value::Int(18),
        };
        
        assert!(seek.value_matches(&Value::Int(20), &pred));
        assert!(!seek.value_matches(&Value::Int(18), &pred));
        assert!(!seek.value_matches(&Value::Int(15), &pred));
    }
}
