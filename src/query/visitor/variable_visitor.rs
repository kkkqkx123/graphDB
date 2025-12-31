//! VariableVisitor - 用于收集表达式中变量的访问器
//! 对应 NebulaGraph VariableVisitor.h/.cpp 的功能

use crate::core::{
    visitor::{Visitor, VisitorState},
    Value,
};
use crate::expression::Expression;
use std::collections::HashSet;

#[derive(Debug)]
pub struct VariableVisitor {
    /// 收集到的变量名集合
    variables: HashSet<String>,
    /// 访问者状态
    state: VisitorState,
}

impl VariableVisitor {
    pub fn new() -> Self {
        Self {
            variables: HashSet::new(),
            state: VisitorState::new(),
        }
    }

    /// 收集表达式中使用的所有变量
    pub fn collect_variables(&mut self, expr: &Expression) -> HashSet<String> {
        self.variables.clear();
        self.visit(expr);
        self.variables.clone()
    }

    /// 检查表达式中是否包含变量
    pub fn has_variables(&mut self, expr: &Expression) -> bool {
        self.variables.clear();
        self.visit(expr);
        !self.variables.is_empty()
    }

    /// 获取收集到的变量列表
    pub fn get_variables(&self) -> Vec<String> {
        self.variables.iter().cloned().collect()
    }

    /// 清空收集到的变量
    pub fn clear(&mut self) {
        self.variables.clear();
    }
}

impl Visitor<Expression> for VariableVisitor {
    type Result = ();

    fn visit(&mut self, target: &Expression) -> Self::Result {
        match target {
            Expression::Variable(name) => {
                // 收集变量名
                self.variables.insert(name.to_string());
            }
            Expression::Property { object, property: _ } => {
                // 递归访问对象表达式
                self.visit(object);
            }
            Expression::Binary { left, op: _, right } => {
                // 递归访问左右操作数
                self.visit(left);
                self.visit(right);
            }
            Expression::Unary { op: _, operand } => {
                // 递归访问操作数
                self.visit(operand);
            }
            Expression::Function { name: _, args } => {
                // 递归访问所有参数
                for arg in args {
                    self.visit(arg);
                }
            }
            Expression::Aggregate { func: _, arg, distinct: _ } => {
                // 递归访问参数
                self.visit(arg);
            }
            Expression::List(items) => {
                // 递归访问列表项
                for item in items {
                    self.visit(item);
                }
            }
            Expression::Map(pairs) => {
                // 递归访问映射值
                for (_, value) in pairs {
                    self.visit(value);
                }
            }
            Expression::Case { conditions, default } => {
                // 递归访问条件和默认值
                for (condition, value) in conditions {
                    self.visit(condition);
                    self.visit(value);
                }
                if let Some(expr) = default {
                    self.visit(expr);
                }
            }
            Expression::TypeCast { expr, target_type: _ } => {
                // 递归访问表达式
                self.visit(expr);
            }
            Expression::Subscript { collection, index } => {
                // 递归访问集合和索引
                self.visit(collection);
                self.visit(index);
            }
            Expression::Range { collection, start, end } => {
                // 递归访问集合和范围
                self.visit(collection);
                if let Some(expr) = start {
                    self.visit(expr);
                }
                if let Some(expr) = end {
                    self.visit(expr);
                }
            }
            Expression::Path(items) => {
                // 递归访问路径项
                for item in items {
                    self.visit(item);
                }
            }
            Expression::TagProperty { tag: _, prop: _ } => {
                // 标签属性不包含变量
            }
            Expression::EdgeProperty { edge: _, prop: _ } => {
                // 边属性不包含变量
            }
            Expression::InputProperty(_) => {
                // 输入属性不包含变量
            }
            Expression::VariableProperty { var, prop: _ } => {
                // 变量属性包含变量
                self.variables.insert(var.to_string());
            }
            Expression::SourceProperty { tag: _, prop: _ } => {
                // 源属性不包含变量
            }
            Expression::DestinationProperty { tag: _, prop: _ } => {
                // 目标属性不包含变量
            }
            _ => {
                // 其他表达式类型不包含变量
            }
        }
    }

    fn state(&self) -> &VisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut VisitorState {
        &mut self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::operators::BinaryOperator;

    #[test]
    fn test_collect_variables() {
        let mut visitor = VariableVisitor::new();

        // 测试简单变量表达式
        let expr = Expression::Variable("x".to_string());
        let variables = visitor.collect_variables(&expr);
        assert_eq!(variables.len(), 1);
        assert!(variables.contains("x"));

        // 测试复杂表达式中的变量
        let expr = Expression::Binary {
            left: Box::new(Expression::Variable("a".to_string())),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Binary {
                left: Box::new(Expression::Variable("b".to_string())),
                op: BinaryOperator::Multiply,
                right: Box::new(Expression::Literal(Value::Int(2))),
            }),
        };

        let variables = visitor.collect_variables(&expr);
        assert_eq!(variables.len(), 2);
        assert!(variables.contains("a"));
        assert!(variables.contains("b"));
    }

    #[test]
    fn test_has_variables() {
        let mut visitor = VariableVisitor::new();

        // 测试包含变量的表达式
        let expr = Expression::Variable("x".to_string());
        assert!(visitor.has_variables(&expr));

        // 测试不包含变量的表达式
        let expr = Expression::Literal(Value::Int(42));
        assert!(!visitor.has_variables(&expr));

        // 测试混合表达式
        let expr = Expression::Binary {
            left: Box::new(Expression::Variable("a".to_string())),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Literal(Value::Int(1))),
        };

        assert!(visitor.has_variables(&expr));
    }

    #[test]
    fn test_get_variables() {
        let mut visitor = VariableVisitor::new();

        let expr = Expression::Binary {
            left: Box::new(Expression::Variable("var1".to_string())),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Variable("var2".to_string())),
        };

        visitor.collect_variables(&expr);
        let variables = visitor.get_variables();
        
        assert_eq!(variables.len(), 2);
        assert!(variables.contains(&"var1".to_string()));
        assert!(variables.contains(&"var2".to_string()));
    }

    #[test]
    fn test_clear() {
        let mut visitor = VariableVisitor::new();

        let expr = Expression::Variable("x".to_string());
        visitor.collect_variables(&expr);
        
        assert!(!visitor.get_variables().is_empty());
        
        visitor.clear();
        assert!(visitor.get_variables().is_empty());
    }
}