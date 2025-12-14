//! 表达式 AST 定义
//!
//! 定义所有表达式类型的 AST 节点，支持访问者模式和类型检查。

use crate::core::Value;
use super::{AstNode, Expression, Span, ExpressionType, node::*};
use std::fmt;

// 重新导出基础表达式类型
pub use super::node::{
    ConstantExpr, VariableExpr, BinaryExpr, UnaryExpr, FunctionCallExpr, 
    PropertyAccessExpr, ListExpr, MapExpr, CaseExpr, SubscriptExpr, PredicateExpr,
    BinaryOp, UnaryOp, PredicateType,
};

/// 表达式工厂 - 用于创建表达式节点
pub struct ExpressionFactory;

impl ExpressionFactory {
    /// 创建常量表达式
    pub fn constant(value: Value, span: Span) -> Box<dyn Expression> {
        Box::new(ConstantExpr::new(value, span))
    }
    
    /// 创建变量表达式
    pub fn variable(name: String, span: Span) -> Box<dyn Expression> {
        Box::new(VariableExpr::new(name, span))
    }
    
    /// 创建二元表达式
    pub fn binary(
        left: Box<dyn Expression>, 
        op: BinaryOp, 
        right: Box<dyn Expression>, 
        span: Span
    ) -> Box<dyn Expression> {
        Box::new(BinaryExpr::new(left, op, right, span))
    }
    
    /// 创建一元表达式
    pub fn unary(op: UnaryOp, operand: Box<dyn Expression>, span: Span) -> Box<dyn Expression> {
        Box::new(UnaryExpr::new(op, operand, span))
    }
    
    /// 创建函数调用表达式
    pub fn function_call(
        name: String, 
        args: Vec<Box<dyn Expression>>, 
        distinct: bool, 
        span: Span
    ) -> Box<dyn Expression> {
        Box::new(FunctionCallExpr::new(name, args, distinct, span))
    }
    
    /// 创建属性访问表达式
    pub fn property_access(
        object: Box<dyn Expression>, 
        property: String, 
        span: Span
    ) -> Box<dyn Expression> {
        Box::new(PropertyAccessExpr::new(object, property, span))
    }
    
    /// 创建列表表达式
    pub fn list(elements: Vec<Box<dyn Expression>>, span: Span) -> Box<dyn Expression> {
        Box::new(ListExpr::new(elements, span))
    }
    
    /// 创建映射表达式
    pub fn map(
        pairs: Vec<(String, Box<dyn Expression>)>, 
        span: Span
    ) -> Box<dyn Expression> {
        Box::new(MapExpr::new(pairs, span))
    }
    
    /// 创建 CASE 表达式
    pub fn case(
        match_expr: Option<Box<dyn Expression>>,
        when_then_pairs: Vec<(Box<dyn Expression>, Box<dyn Expression>)>,
        default: Option<Box<dyn Expression>>,
        span: Span
    ) -> Box<dyn Expression> {
        Box::new(CaseExpr::new(match_expr, when_then_pairs, default, span))
    }
    
    /// 创建下标表达式
    pub fn subscript(
        collection: Box<dyn Expression>,
        index: Box<dyn Expression>,
        span: Span
    ) -> Box<dyn Expression> {
        Box::new(SubscriptExpr::new(collection, index, span))
    }
    
    /// 创建谓词表达式
    pub fn predicate(
        predicate: PredicateType,
        list: Box<dyn Expression>,
        condition: Box<dyn Expression>,
        span: Span
    ) -> Box<dyn Expression> {
        Box::new(PredicateExpr::new(predicate, list, condition, span))
    }
}

/// 表达式工具函数
pub struct ExpressionUtils;

impl ExpressionUtils {
    /// 检查表达式是否为常量
    pub fn is_constant(expr: &dyn Expression) -> bool {
        expr.is_constant()
    }
    
    /// 获取表达式的所有子表达式
    pub fn collect_expressions(expr: &dyn Expression) -> Vec<Box<dyn Expression>> {
        let mut result = Vec::new();
        Self::collect_recursive(expr, &mut result);
        result
    }
    
    fn collect_recursive(expr: &dyn Expression, result: &mut Vec<Box<dyn Expression>>) {
        result.push(super::Expression::clone_box(expr));
        
        for child in expr.children() {
            Self::collect_recursive(child.as_ref(), result);
        }
    }
    
    /// 查找表达式中的变量
    pub fn find_variables(expr: &dyn Expression) -> Vec<String> {
        let mut variables = Vec::new();
        Self::find_variables_recursive(expr, &mut variables);
        variables
    }
    
    fn find_variables_recursive(expr: &dyn Expression, variables: &mut Vec<String>) {
        match expr.expr_type() {
            ExpressionType::Variable => {
                if let Some(var_expr) = expr.as_any().downcast_ref::<VariableExpr>() {
                    variables.push(var_expr.name.clone());
                }
            }
            _ => {
                for child in expr.children() {
                    Self::find_variables_recursive(child.as_ref(), variables);
                }
            }
        }
    }
    
    /// 检查表达式是否包含聚合函数
    pub fn contains_aggregate(expr: &dyn Expression) -> bool {
        Self::contains_aggregate_recursive(expr)
    }
    
    fn contains_aggregate_recursive(expr: &dyn Expression) -> bool {
        match expr.expr_type() {
            ExpressionType::FunctionCall => {
                if let Some(func_expr) = expr.as_any().downcast_ref::<FunctionCallExpr>() {
                    let func_name = func_expr.name.to_uppercase();
                    matches!(func_name.as_str(), 
                        "COUNT" | "SUM" | "AVG" | "MIN" | "MAX" |
                        "COLLECT" | "AGGREGATE"
                    )
                } else {
                    false
                }
            }
            _ => {
                expr.children().iter().any(|child| Self::contains_aggregate_recursive(child.as_ref()))
            }
        }
    }
    
    /// 简化常量表达式
    pub fn simplify(expr: Box<dyn Expression>) -> Box<dyn Expression> {
        // 这里可以实现常量折叠等优化
        // 目前只是返回原表达式
        expr
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_expression_factory() {
        let span = Span::default();
        
        // 测试常量表达式
        let const_expr = ExpressionFactory::constant(Value::Int(42), span);
        assert_eq!(const_expr.expr_type(), ExpressionType::Constant);
        assert!(const_expr.is_constant());
        
        // 测试变量表达式
        let var_expr = ExpressionFactory::variable("x".to_string(), span);
        assert_eq!(var_expr.expr_type(), ExpressionType::Variable);
        assert!(!var_expr.is_constant());
        
        // 测试二元表达式
        let left = ExpressionFactory::constant(Value::Int(5), span);
        let right = ExpressionFactory::constant(Value::Int(3), span);
        let binary_expr = ExpressionFactory::binary(left, BinaryOp::Add, right, span);
        assert_eq!(binary_expr.expr_type(), ExpressionType::Binary);
        assert!(binary_expr.is_constant());
    }
    
    #[test]
    fn test_expression_utils() {
        let span = Span::default();
        
        // 测试变量查找
        let var_expr = ExpressionFactory::variable("test_var".to_string(), span);
        let variables = ExpressionUtils::find_variables(var_expr.as_ref());
        assert_eq!(variables, vec!["test_var"]);
        
        // 测试聚合函数检查
        let func_expr = ExpressionFactory::function_call(
            "COUNT".to_string(),
            vec![ExpressionFactory::variable("x".to_string(), span)],
            false,
            span
        );
        assert!(ExpressionUtils::contains_aggregate(func_expr.as_ref()));
    }
}