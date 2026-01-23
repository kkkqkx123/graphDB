//! 表达式工具类
//!
//! 提供表达式分析和转换的实用函数，类似于 nebula-graph 的 ExpressionUtils

use crate::core::types::expression::Expr;
use crate::core::types::operators::BinaryOperator;
use crate::core::{Expression, Value};

pub struct ExpressionUtils;

impl ExpressionUtils {
    pub fn is_one_step_edge_prop(edge_alias: &str, expr: &Expr) -> bool {
        if let Expr::Property { object, .. } = expr {
            if let Expr::Variable(name) = object.as_ref() {
                return name == edge_alias;
            }
        }
        false
    }

    pub fn split_filter(
        filter: &Expr,
        picker: impl Fn(&Expr) -> bool,
    ) -> (Option<Expr>, Option<Expr>) {
        let mut picked_exprs = Vec::new();
        let mut unpicked_exprs = Vec::new();

        Self::split_filter_recursive(filter, &picker, &mut picked_exprs, &mut unpicked_exprs);

        let picked = if picked_exprs.is_empty() {
            None
        } else {
            Some(Self::and_all(picked_exprs))
        };

        let unpicked = if unpicked_exprs.is_empty() {
            None
        } else {
            Some(Self::and_all(unpicked_exprs))
        };

        (picked, unpicked)
    }

    fn split_filter_recursive(
        expr: &Expr,
        picker: &impl Fn(&Expr) -> bool,
        picked: &mut Vec<Expr>,
        unpicked: &mut Vec<Expr>,
    ) {
        match expr {
            Expr::Binary {
                left,
                op: BinaryOperator::And,
                right,
            } => {
                Self::split_filter_recursive(left, picker, picked, unpicked);
                Self::split_filter_recursive(right, picker, picked, unpicked);
            }
            _ => {
                if picker(expr) {
                    picked.push(expr.clone());
                } else {
                    unpicked.push(expr.clone());
                }
            }
        }
    }

    pub fn rewrite_edge_property_filter(
        _edge_alias: &str,
        filter: Expr,
    ) -> Expr {
        filter
    }

    pub fn rewrite_tag_property_filter(_tag: &str, filter: Expr) -> Expr {
        filter
    }

    fn and_all(mut exprs: Vec<Expr>) -> Expr {
        match exprs.len() {
            0 => Expr::Literal(crate::core::Value::Bool(true)),
            1 => exprs.pop().expect("Should have one element"),
            _ => {
                let mut result = exprs.pop().expect("Should have elements");
                while let Some(expr) = exprs.pop() {
                    result = Expr::Binary {
                        left: Box::new(expr),
                        op: BinaryOperator::And,
                        right: Box::new(result),
                    };
                }
                result
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_one_step_edge_prop() {
        let expr = Expression::Property {
            object: Box::new(Expression::Variable("e".to_string())),
            property: "name".to_string(),
        };
        assert!(ExpressionUtils::is_one_step_edge_prop("e", &expr));
        assert!(!ExpressionUtils::is_one_step_edge_prop("e2", &expr));
    }

    #[test]
    fn test_split_filter() {
        let expr = Expression::Binary {
            left: Box::new(Expression::Property {
                object: Box::new(Expression::Variable("e".to_string())),
                property: "name".to_string(),
            }),
            op: BinaryOperator::And,
            right: Box::new(Expression::Property {
                object: Box::new(Expression::Variable("v".to_string())),
                property: "age".to_string(),
            }),
        };

        let (picked, unpicked) = ExpressionUtils::split_filter(&expr, |e| {
            ExpressionUtils::is_one_step_edge_prop("e", e)
        });

        assert!(picked.is_some());
        assert!(unpicked.is_some());
    }
}
