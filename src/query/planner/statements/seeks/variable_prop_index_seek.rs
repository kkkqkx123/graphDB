//! 变量属性索引查找策略
//!
//! 基于变量属性的索引查找，用于运行时变量值确定的情况
//!
//! 适用场景:
//! - MATCH (v:Person) WHERE v.name = $varName
//! - MATCH (v:Person) WHERE v.age > $minAge
//! - 参数化查询中的变量绑定

use super::seek_strategy::SeekStrategy;
use super::seek_strategy_base::{IndexInfo, SeekResult, SeekStrategyContext, SeekStrategyType};
use crate::core::{StorageError, Value};
use crate::storage::StorageClient;

/// 变量属性谓词
#[derive(Debug, Clone, PartialEq)]
pub struct VariablePropertyPredicate {
    pub property: String,
    pub op: VariablePredicateOp,
    pub variable_name: String,
}

/// 变量谓词操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariablePredicateOp {
    Eq,      // =
    Ne,      // !=
    Lt,      // <
    Le,      // <=
    Gt,      // >
    Ge,      // >=
    In,      // IN
}

impl VariablePredicateOp {
    /// 从字符串解析操作符
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "=" | "==" => Some(VariablePredicateOp::Eq),
            "!=" | "<>" => Some(VariablePredicateOp::Ne),
            "<" => Some(VariablePredicateOp::Lt),
            "<=" => Some(VariablePredicateOp::Le),
            ">" => Some(VariablePredicateOp::Gt),
            ">=" => Some(VariablePredicateOp::Ge),
            "IN" => Some(VariablePredicateOp::In),
            _ => None,
        }
    }

    /// 转换为普通谓词操作
    pub fn to_predicate_op(&self) -> super::prop_index_seek::PredicateOp {
        match self {
            VariablePredicateOp::Eq => super::prop_index_seek::PredicateOp::Eq,
            VariablePredicateOp::Ne => super::prop_index_seek::PredicateOp::Ne,
            VariablePredicateOp::Lt => super::prop_index_seek::PredicateOp::Lt,
            VariablePredicateOp::Le => super::prop_index_seek::PredicateOp::Le,
            VariablePredicateOp::Gt => super::prop_index_seek::PredicateOp::Gt,
            VariablePredicateOp::Ge => super::prop_index_seek::PredicateOp::Ge,
            VariablePredicateOp::In => super::prop_index_seek::PredicateOp::In,
        }
    }
}

/// 变量属性索引查找策略
#[derive(Debug, Clone)]
pub struct VariablePropIndexSeek {
    predicates: Vec<VariablePropertyPredicate>,
    variable_values: std::collections::HashMap<String, Value>,
}

impl VariablePropIndexSeek {
    pub fn new(predicates: Vec<VariablePropertyPredicate>) -> Self {
        Self {
            predicates,
            variable_values: std::collections::HashMap::new(),
        }
    }

    /// 绑定变量值
    pub fn bind_variable(&mut self, name: &str, value: Value) {
        self.variable_values.insert(name.to_string(), value);
    }

    /// 批量绑定变量值
    pub fn bind_variables(&mut self, values: std::collections::HashMap<String, Value>) {
        self.variable_values.extend(values);
    }

    /// 检查所有变量是否已绑定
    pub fn all_variables_bound(&self) -> bool {
        self.predicates.iter().all(|pred| {
            self.variable_values.contains_key(&pred.variable_name)
        })
    }

    /// 从表达式列表提取变量属性谓词
    pub fn extract_predicates(expressions: &[crate::core::Expression]) -> Vec<VariablePropertyPredicate> {
        let mut predicates = Vec::new();
        
        for expr in expressions {
            if let Some(pred) = Self::extract_predicate(expr) {
                predicates.push(pred);
            }
        }
        
        predicates
    }

    /// 从单个表达式提取变量属性谓词
    fn extract_predicate(expr: &crate::core::Expression) -> Option<VariablePropertyPredicate> {
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
                
                // 尝试提取属性名和变量名: v.name = $var
                if let (Some(prop), Some(var_name)) = (Self::extract_property(left), Self::extract_variable(right)) {
                    if let Some(pred_op) = VariablePredicateOp::from_str(op_str) {
                        return Some(VariablePropertyPredicate {
                            property: prop,
                            op: pred_op,
                            variable_name: var_name,
                        });
                    }
                }
                
                // 交换左右尝试: $var = v.name
                if let (Some(prop), Some(var_name)) = (Self::extract_property(right), Self::extract_variable(left)) {
                    let swapped_op = match op_str {
                        "<" => VariablePredicateOp::Gt,
                        "<=" => VariablePredicateOp::Ge,
                        ">" => VariablePredicateOp::Lt,
                        ">=" => VariablePredicateOp::Le,
                        _ => VariablePredicateOp::from_str(op_str)?,
                    };
                    return Some(VariablePropertyPredicate {
                        property: prop,
                        op: swapped_op,
                        variable_name: var_name,
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

    /// 从表达式提取变量名
    fn extract_variable(expr: &crate::core::Expression) -> Option<String> {
        match expr {
            // 变量以 $ 开头表示参数
            crate::core::Expression::Variable(name) if name.starts_with('$') => {
                Some(name[1..].to_string())
            }
            _ => None,
        }
    }

    /// 查找适合变量属性谓词的索引
    fn find_best_index<'a>(&'a self, context: &'a SeekStrategyContext) -> Option<(&'a IndexInfo, &'a VariablePropertyPredicate)> {
        for pred in &self.predicates {
            if let Some(index) = context.get_index_for_property(&pred.property) {
                return Some((index, pred));
            }
        }
        None
    }

    /// 评估值是否满足谓词条件
    fn value_matches(&self, value: &Value, pred: &VariablePropertyPredicate) -> bool {
        // 获取变量值
        let var_value = match self.variable_values.get(&pred.variable_name) {
            Some(v) => v,
            None => return false, // 变量未绑定，无法匹配
        };

        match pred.op {
            VariablePredicateOp::Eq => value == var_value,
            VariablePredicateOp::Ne => value != var_value,
            VariablePredicateOp::Lt => Self::compare_values(value, var_value).map(|c| c < 0).unwrap_or(false),
            VariablePredicateOp::Le => Self::compare_values(value, var_value).map(|c| c <= 0).unwrap_or(false),
            VariablePredicateOp::Gt => Self::compare_values(value, var_value).map(|c| c > 0).unwrap_or(false),
            VariablePredicateOp::Ge => Self::compare_values(value, var_value).map(|c| c >= 0).unwrap_or(false),
            VariablePredicateOp::In => {
                // IN 操作需要变量值是列表
                matches!(var_value, Value::List(list) if list.contains(value))
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

impl SeekStrategy for VariablePropIndexSeek {
    fn execute(
        &self,
        storage: &dyn StorageClient,
        context: &SeekStrategyContext,
    ) -> Result<SeekResult, StorageError> {
        // 检查变量是否已绑定
        if !self.all_variables_bound() {
            return Err(StorageError::InvalidInput(
                "变量属性查找需要所有变量已绑定".to_string()
            ));
        }

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
            strategy_used: SeekStrategyType::VariablePropIndexSeek,
            rows_scanned,
        })
    }

    fn estimated_cost(&self, _context: &SeekStrategyContext) -> f64 {
        if self.all_variables_bound() {
            5.0 // 变量已绑定，成本较低
        } else {
            100.0 // 变量未绑定，需要延迟执行
        }
    }

    fn supports(&self, _context: &SeekStrategyContext) -> bool {
        // 只要有变量属性谓词就支持
        !self.predicates.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;

    #[test]
    fn test_variable_predicate_op_from_str() {
        assert_eq!(VariablePredicateOp::from_str("="), Some(VariablePredicateOp::Eq));
        assert_eq!(VariablePredicateOp::from_str("<"), Some(VariablePredicateOp::Lt));
        assert_eq!(VariablePredicateOp::from_str(">="), Some(VariablePredicateOp::Ge));
        assert_eq!(VariablePredicateOp::from_str("IN"), Some(VariablePredicateOp::In));
        assert_eq!(VariablePredicateOp::from_str("unknown"), None);
    }

    #[test]
    fn test_extract_variable_predicate() {
        let expr = Expression::binary(
            Expression::property(Expression::variable("v"), "name"),
            crate::core::BinaryOperator::Equal,
            Expression::variable("$varName"),
        );

        let pred = VariablePropIndexSeek::extract_predicate(&expr);
        assert!(pred.is_some());

        let pred = pred.unwrap();
        assert_eq!(pred.property, "name");
        assert_eq!(pred.op, VariablePredicateOp::Eq);
        assert_eq!(pred.variable_name, "varName");
    }

    #[test]
    fn test_variable_binding() {
        let mut seek = VariablePropIndexSeek::new(vec![
            VariablePropertyPredicate {
                property: "age".to_string(),
                op: VariablePredicateOp::Gt,
                variable_name: "minAge".to_string(),
            }
        ]);
        
        assert!(!seek.all_variables_bound());
        
        seek.bind_variable("minAge", Value::Int(18));
        
        assert!(seek.all_variables_bound());
    }
}
