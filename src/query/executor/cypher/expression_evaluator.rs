//! Cypher表达式评估器
//!
//! 在query层提供Cypher表达式评估功能，避免core层依赖query层

use crate::core::value::Value;
use crate::expression::ExpressionContext;
use crate::expression::ExpressionError;

/// Cypher表达式评估器
#[derive(Debug)]
pub struct CypherExpressionEvaluator;

impl CypherExpressionEvaluator {
    /// 评估Cypher表达式
    pub fn evaluate_cypher(
        &self,
        cypher_expr: &crate::query::parser::cypher::ast::expressions::Expression,
        context: &mut dyn ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        // 使用我们在parser/cypher中创建的评估器
        crate::query::parser::cypher::CypherEvaluator::evaluate_cypher(cypher_expr, context)
    }

    /// 批量评估Cypher表达式
    pub fn evaluate_cypher_batch(
        &self,
        cypher_exprs: &[crate::query::parser::cypher::ast::expressions::Expression],
        context: &mut dyn ExpressionContext,
    ) -> Result<Vec<Value>, ExpressionError> {
        let mut results = Vec::new();
        for expr in cypher_exprs {
            results.push(self.evaluate_cypher(expr, context)?);
        }
        Ok(results)
    }

    /// 检查Cypher表达式是否为常量
    pub fn is_cypher_constant(
        &self,
        cypher_expr: &crate::query::parser::cypher::ast::expressions::Expression,
    ) -> bool {
        // 一个Cypher表达式是常量如果它只包含字面量和常量运算
        match cypher_expr {
            crate::query::parser::cypher::ast::expressions::Expression::Literal(_) => true,
            crate::query::parser::cypher::ast::expressions::Expression::Binary(bin_expr) => {
                self.is_cypher_constant(&bin_expr.left) && self.is_cypher_constant(&bin_expr.right)
            }
            crate::query::parser::cypher::ast::expressions::Expression::Unary(unary_expr) => {
                self.is_cypher_constant(&unary_expr.expression)
            }
            _ => false, // 变量、函数调用等不是常量
        }
    }

    /// 获取Cypher表达式中使用的所有变量
    pub fn get_cypher_variables(
        &self,
        cypher_expr: &crate::query::parser::cypher::ast::expressions::Expression,
    ) -> Vec<String> {
        let mut variables = Vec::new();
        self.collect_cypher_variables(cypher_expr, &mut variables);
        variables.sort();
        variables.dedup();
        variables
    }

    /// 递归收集Cypher表达式中的变量
    fn collect_cypher_variables(
        &self,
        cypher_expr: &crate::query::parser::cypher::ast::expressions::Expression,
        variables: &mut Vec<String>,
    ) {
        match cypher_expr {
            crate::query::parser::cypher::ast::expressions::Expression::Variable(name) => {
                variables.push(name.clone());
            }
            crate::query::parser::cypher::ast::expressions::Expression::Property(prop_expr) => {
                self.collect_cypher_variables(&prop_expr.expression, variables);
            }
            crate::query::parser::cypher::ast::expressions::Expression::FunctionCall(func_call) => {
                for arg in &func_call.arguments {
                    self.collect_cypher_variables(arg, variables);
                }
            }
            crate::query::parser::cypher::ast::expressions::Expression::Binary(bin_expr) => {
                self.collect_cypher_variables(&bin_expr.left, variables);
                self.collect_cypher_variables(&bin_expr.right, variables);
            }
            crate::query::parser::cypher::ast::expressions::Expression::Unary(unary_expr) => {
                self.collect_cypher_variables(&unary_expr.expression, variables);
            }
            crate::query::parser::cypher::ast::expressions::Expression::List(list_expr) => {
                for element in &list_expr.elements {
                    self.collect_cypher_variables(element, variables);
                }
            }
            crate::query::parser::cypher::ast::expressions::Expression::Map(map_expr) => {
                for (_, value) in &map_expr.properties {
                    self.collect_cypher_variables(value, variables);
                }
            }
            crate::query::parser::cypher::ast::expressions::Expression::Case(case_expr) => {
                for alternative in &case_expr.alternatives {
                    self.collect_cypher_variables(&alternative.when_expression, variables);
                    self.collect_cypher_variables(&alternative.then_expression, variables);
                }
                if let Some(default_expr) = &case_expr.default_alternative {
                    self.collect_cypher_variables(default_expr, variables);
                }
            }
            crate::query::parser::cypher::ast::expressions::Expression::PatternExpression(_) => {
                // 模式表达式可能包含变量，但目前不处理
            }
            _ => {}
        }
    }

    /// 检查Cypher表达式是否包含聚合函数
    pub fn contains_cypher_aggregate(
        &self,
        cypher_expr: &crate::query::parser::cypher::ast::expressions::Expression,
    ) -> bool {
        // 检查是否是聚合函数调用
        match cypher_expr {
            crate::query::parser::cypher::ast::expressions::Expression::FunctionCall(func_call) => {
                // 检查函数名是否为聚合函数
                let agg_functions = [
                    "count",
                    "sum",
                    "avg",
                    "min",
                    "max",
                    "collect",
                    "collect_distinct",
                ];
                agg_functions.contains(&func_call.function_name.to_lowercase().as_str())
            }
            crate::query::parser::cypher::ast::expressions::Expression::Binary(bin_expr) => {
                self.contains_cypher_aggregate(&bin_expr.left)
                    || self.contains_cypher_aggregate(&bin_expr.right)
            }
            crate::query::parser::cypher::ast::expressions::Expression::Unary(unary_expr) => {
                self.contains_cypher_aggregate(&unary_expr.expression)
            }
            crate::query::parser::cypher::ast::expressions::Expression::List(list_expr) => {
                list_expr
                    .elements
                    .iter()
                    .any(|e| self.contains_cypher_aggregate(e))
            }
            crate::query::parser::cypher::ast::expressions::Expression::Map(map_expr) => map_expr
                .properties
                .values()
                .any(|v| self.contains_cypher_aggregate(v)),
            crate::query::parser::cypher::ast::expressions::Expression::Case(case_expr) => {
                case_expr.alternatives.iter().any(|alt| {
                    self.contains_cypher_aggregate(&alt.when_expression)
                        || self.contains_cypher_aggregate(&alt.then_expression)
                }) || case_expr
                    .default_alternative
                    .as_ref()
                    .map_or(false, |e| self.contains_cypher_aggregate(e))
            }
            _ => false,
        }
    }

    /// 优化Cypher表达式
    pub fn optimize_cypher_expression(
        &self,
        cypher_expr: &crate::query::parser::cypher::ast::expressions::Expression,
    ) -> crate::query::parser::cypher::ast::expressions::Expression {
        // 使用我们在parser/cypher中创建的优化器
        crate::query::parser::cypher::CypherExpressionOptimizer::optimize_cypher_expression(
            cypher_expr,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression::BasicExpressionContext;
    use crate::query::parser::cypher::ast::expressions::{
        Expression as CypherExpression, Literal as CypherLiteral,
    };

    #[test]
    fn test_evaluate_cypher_literal() {
        let mut context = BasicExpressionContext::default();
        let evaluator = CypherExpressionEvaluator;
        let cypher_expr = CypherExpression::Literal(CypherLiteral::Integer(42));

        let result = evaluator
            .evaluate_cypher(&cypher_expr, &mut context)
            .expect("Cypher evaluation should succeed for literals");
        assert_eq!(result, crate::core::value::Value::Int(42));
    }

    #[test]
    fn test_is_cypher_constant() {
        let evaluator = CypherExpressionEvaluator;

        // 字面量是常量
        let literal_expr = CypherExpression::Literal(CypherLiteral::Integer(42));
        assert!(evaluator.is_cypher_constant(&literal_expr));

        // 变量不是常量
        let var_expr = CypherExpression::Variable("x".to_string());
        assert!(!evaluator.is_cypher_constant(&var_expr));
    }
}
