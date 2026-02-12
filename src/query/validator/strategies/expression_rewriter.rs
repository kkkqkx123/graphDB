//! 表达式重写工具
//!
//! 对应 nebula-graph 的 ExpressionUtils.h/.cpp 的功能
//! 提供表达式重写功能，将用户友好的语法转换为内部表示

use crate::core::Expression;
use std::collections::HashMap;

/// 表达式重写器
///
/// 用于将表达式从一种形式转换为另一种形式
pub struct ExpressionRewriter {
    alias_type_map: HashMap<String, AliasType>,
}

/// 别名类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AliasType {
    Vertex,
    Edge,
    Path,
    Unknown,
}

impl ExpressionRewriter {
    pub fn new() -> Self {
        Self {
            alias_type_map: HashMap::new(),
        }
    }

    /// 设置别名类型映射
    pub fn set_alias_type_map(&mut self, map: HashMap<String, AliasType>) {
        self.alias_type_map = map;
    }

    /// 重写标签属性表达式为属性表达式
    ///
    /// 类似于 nebula-graph 的 rewriteLabelAttr2PropExpr
    pub fn rewrite_label_attr_to_prop_expr(&self, expr: &Expression, is_edge: bool) -> Expression {
        if is_edge {
            self.rewrite_label_attr_to_edge_prop(expr)
        } else {
            self.rewrite_label_attr_to_tag_prop(expr)
        }
    }

    /// 重写标签属性表达式为边属性表达式
    ///
    /// 类似于 nebula-graph 的 rewriteLabelAttr2EdgeProp
    /// 将 LabelAttribute 转换为 EdgeProperty
    pub fn rewrite_label_attr_to_edge_prop(&self, expr: &Expression) -> Expression {
        match expr {
            Expression::LabelTagProperty { tag, property } => {
                Expression::EdgeProperty {
                    edge_name: self.extract_label_name(tag),
                    property: property.clone(),
                }
            }
            Expression::Binary { left, op, right } => {
                Expression::Binary {
                    left: Box::new(self.rewrite_label_attr_to_edge_prop(left)),
                    op: op.clone(),
                    right: Box::new(self.rewrite_label_attr_to_edge_prop(right)),
                }
            }
            Expression::Unary { op, operand } => {
                Expression::Unary {
                    op: op.clone(),
                    operand: Box::new(self.rewrite_label_attr_to_edge_prop(operand)),
                }
            }
            Expression::Function { name, args } => {
                let rewritten_args: Vec<Expression> = args
                    .iter()
                    .map(|arg| self.rewrite_label_attr_to_edge_prop(arg))
                    .collect();
                Expression::Function {
                    name: name.clone(),
                    args: rewritten_args,
                }
            }
            Expression::Aggregate { func, arg, distinct } => {
                Expression::Aggregate {
                    func: func.clone(),
                    arg: Box::new(self.rewrite_label_attr_to_edge_prop(arg)),
                    distinct: *distinct,
                }
            }
            Expression::List(items) => {
                let rewritten_items: Vec<Expression> = items
                    .iter()
                    .map(|item| self.rewrite_label_attr_to_edge_prop(item))
                    .collect();
                Expression::List(rewritten_items)
            }
            Expression::Map(pairs) => {
                let rewritten_pairs: Vec<(String, Expression)> = pairs
                    .iter()
                    .map(|(key, value)| {
                        (key.clone(), self.rewrite_label_attr_to_edge_prop(value))
                    })
                    .collect();
                Expression::Map(rewritten_pairs)
            }
            Expression::Case {
                test_expr,
                conditions,
                default,
            } => {
                let rewritten_test_expr = test_expr
                    .as_ref()
                    .map(|e| Box::new(self.rewrite_label_attr_to_edge_prop(e)));
                let rewritten_conditions: Vec<(Expression, Expression)> = conditions
                    .iter()
                    .map(|(cond, result)| {
                        (
                            self.rewrite_label_attr_to_edge_prop(cond),
                            self.rewrite_label_attr_to_edge_prop(result),
                        )
                    })
                    .collect();
                let rewritten_default = default
                    .as_ref()
                    .map(|e| Box::new(self.rewrite_label_attr_to_edge_prop(e)));
                Expression::Case {
                    test_expr: rewritten_test_expr,
                    conditions: rewritten_conditions,
                    default: rewritten_default,
                }
            }
            Expression::TypeCast {
                expression,
                target_type,
            } => Expression::TypeCast {
                expression: Box::new(self.rewrite_label_attr_to_edge_prop(expression)),
                target_type: target_type.clone(),
            },
            Expression::Subscript { collection, index } => Expression::Subscript {
                collection: Box::new(self.rewrite_label_attr_to_edge_prop(collection)),
                index: Box::new(self.rewrite_label_attr_to_edge_prop(index)),
            },
            Expression::Range {
                collection,
                start,
                end,
            } => Expression::Range {
                collection: Box::new(self.rewrite_label_attr_to_edge_prop(collection)),
                start: start.as_ref().map(|e| Box::new(self.rewrite_label_attr_to_edge_prop(e))),
                end: end.as_ref().map(|e| Box::new(self.rewrite_label_attr_to_edge_prop(e))),
            },
            Expression::Path(items) => {
                let rewritten_items: Vec<Expression> = items
                    .iter()
                    .map(|item| self.rewrite_label_attr_to_edge_prop(item))
                    .collect();
                Expression::Path(rewritten_items)
            }
            Expression::ListComprehension {
                variable,
                source,
                filter,
                map,
            } => Expression::ListComprehension {
                variable: variable.clone(),
                source: Box::new(self.rewrite_label_attr_to_edge_prop(source)),
                filter: filter.as_ref().map(|e| Box::new(self.rewrite_label_attr_to_edge_prop(e))),
                map: map.as_ref().map(|e| Box::new(self.rewrite_label_attr_to_edge_prop(e))),
            },
            Expression::Predicate { func, args } => {
                let rewritten_args: Vec<Expression> = args
                    .iter()
                    .map(|arg| self.rewrite_label_attr_to_edge_prop(arg))
                    .collect();
                Expression::Predicate {
                    func: func.clone(),
                    args: rewritten_args,
                }
            }
            Expression::Reduce {
                accumulator,
                initial,
                variable,
                source,
                mapping,
            } => Expression::Reduce {
                accumulator: accumulator.clone(),
                initial: Box::new(self.rewrite_label_attr_to_edge_prop(initial)),
                variable: variable.clone(),
                source: Box::new(self.rewrite_label_attr_to_edge_prop(source)),
                mapping: Box::new(self.rewrite_label_attr_to_edge_prop(mapping)),
            },
            Expression::PathBuild(exprs) => {
                let rewritten_exprs: Vec<Expression> = exprs
                    .iter()
                    .map(|e| self.rewrite_label_attr_to_edge_prop(e))
                    .collect();
                Expression::PathBuild(rewritten_exprs)
            }
            _ => expr.clone(),
        }
    }

    /// 重写标签属性表达式为标签属性表达式
    ///
    /// 类似于 nebula-graph 的 rewriteLabelAttr2TagProp
    /// 将 LabelAttribute 转换为 TagProperty
    pub fn rewrite_label_attr_to_tag_prop(&self, expr: &Expression) -> Expression {
        match expr {
            Expression::LabelTagProperty { tag, property } => {
                Expression::TagProperty {
                    tag_name: self.extract_label_name(tag),
                    property: property.clone(),
                }
            }
            Expression::Binary { left, op, right } => {
                Expression::Binary {
                    left: Box::new(self.rewrite_label_attr_to_tag_prop(left)),
                    op: op.clone(),
                    right: Box::new(self.rewrite_label_attr_to_tag_prop(right)),
                }
            }
            Expression::Unary { op, operand } => {
                Expression::Unary {
                    op: op.clone(),
                    operand: Box::new(self.rewrite_label_attr_to_tag_prop(operand)),
                }
            }
            Expression::Function { name, args } => {
                let rewritten_args: Vec<Expression> = args
                    .iter()
                    .map(|arg| self.rewrite_label_attr_to_tag_prop(arg))
                    .collect();
                Expression::Function {
                    name: name.clone(),
                    args: rewritten_args,
                }
            }
            Expression::Aggregate { func, arg, distinct } => {
                Expression::Aggregate {
                    func: func.clone(),
                    arg: Box::new(self.rewrite_label_attr_to_tag_prop(arg)),
                    distinct: *distinct,
                }
            }
            Expression::List(items) => {
                let rewritten_items: Vec<Expression> = items
                    .iter()
                    .map(|item| self.rewrite_label_attr_to_tag_prop(item))
                    .collect();
                Expression::List(rewritten_items)
            }
            Expression::Map(pairs) => {
                let rewritten_pairs: Vec<(String, Expression)> = pairs
                    .iter()
                    .map(|(key, value)| {
                        (key.clone(), self.rewrite_label_attr_to_tag_prop(value))
                    })
                    .collect();
                Expression::Map(rewritten_pairs)
            }
            Expression::Case {
                test_expr,
                conditions,
                default,
            } => {
                let rewritten_test_expr = test_expr
                    .as_ref()
                    .map(|e| Box::new(self.rewrite_label_attr_to_tag_prop(e)));
                let rewritten_conditions: Vec<(Expression, Expression)> = conditions
                    .iter()
                    .map(|(cond, result)| {
                        (
                            self.rewrite_label_attr_to_tag_prop(cond),
                            self.rewrite_label_attr_to_tag_prop(result),
                        )
                    })
                    .collect();
                let rewritten_default = default
                    .as_ref()
                    .map(|e| Box::new(self.rewrite_label_attr_to_tag_prop(e)));
                Expression::Case {
                    test_expr: rewritten_test_expr,
                    conditions: rewritten_conditions,
                    default: rewritten_default,
                }
            }
            Expression::TypeCast {
                expression,
                target_type,
            } => Expression::TypeCast {
                expression: Box::new(self.rewrite_label_attr_to_tag_prop(expression)),
                target_type: target_type.clone(),
            },
            Expression::Subscript { collection, index } => Expression::Subscript {
                collection: Box::new(self.rewrite_label_attr_to_tag_prop(collection)),
                index: Box::new(self.rewrite_label_attr_to_tag_prop(index)),
            },
            Expression::Range {
                collection,
                start,
                end,
            } => Expression::Range {
                collection: Box::new(self.rewrite_label_attr_to_tag_prop(collection)),
                start: start.as_ref().map(|e| Box::new(self.rewrite_label_attr_to_tag_prop(e))),
                end: end.as_ref().map(|e| Box::new(self.rewrite_label_attr_to_tag_prop(e))),
            },
            Expression::Path(items) => {
                let rewritten_items: Vec<Expression> = items
                    .iter()
                    .map(|item| self.rewrite_label_attr_to_tag_prop(item))
                    .collect();
                Expression::Path(rewritten_items)
            }
            Expression::ListComprehension {
                variable,
                source,
                filter,
                map,
            } => Expression::ListComprehension {
                variable: variable.clone(),
                source: Box::new(self.rewrite_label_attr_to_tag_prop(source)),
                filter: filter.as_ref().map(|e| Box::new(self.rewrite_label_attr_to_tag_prop(e))),
                map: map.as_ref().map(|e| Box::new(self.rewrite_label_attr_to_tag_prop(e))),
            },
            Expression::Predicate { func, args } => {
                let rewritten_args: Vec<Expression> = args
                    .iter()
                    .map(|arg| self.rewrite_label_attr_to_tag_prop(arg))
                    .collect();
                Expression::Predicate {
                    func: func.clone(),
                    args: rewritten_args,
                }
            }
            Expression::Reduce {
                accumulator,
                initial,
                variable,
                source,
                mapping,
            } => Expression::Reduce {
                accumulator: accumulator.clone(),
                initial: Box::new(self.rewrite_label_attr_to_tag_prop(initial)),
                variable: variable.clone(),
                source: Box::new(self.rewrite_label_attr_to_tag_prop(source)),
                mapping: Box::new(self.rewrite_label_attr_to_tag_prop(mapping)),
            },
            Expression::PathBuild(exprs) => {
                let rewritten_exprs: Vec<Expression> = exprs
                    .iter()
                    .map(|e| self.rewrite_label_attr_to_tag_prop(e))
                    .collect();
                Expression::PathBuild(rewritten_exprs)
            }
            _ => expr.clone(),
        }
    }

    /// 重写边属性函数为标签属性表达式
    ///
    /// 类似于 nebula-graph 的 rewriteEdgePropFunc2LabelAttribute
    /// 将 rank(e) 转换为 e._rank
    pub fn rewrite_edge_prop_func_to_label_attr(&self, expr: &Expression) -> Expression {
        match expr {
            Expression::Function { name, args } => {
                let func_name = name.to_uppercase();
                match func_name.as_str() {
                    "RANK" => {
                        if let Some(arg) = args.first() {
                            Expression::LabelTagProperty {
                                tag: Box::new(arg.clone()),
                                property: "_rank".to_string(),
                            }
                        } else {
                            expr.clone()
                        }
                    }
                    "SRC" => {
                        if let Some(arg) = args.first() {
                            Expression::LabelTagProperty {
                                tag: Box::new(arg.clone()),
                                property: "_src".to_string(),
                            }
                        } else {
                            expr.clone()
                        }
                    }
                    "DST" => {
                        if let Some(arg) = args.first() {
                            Expression::LabelTagProperty {
                                tag: Box::new(arg.clone()),
                                property: "_dst".to_string(),
                            }
                        } else {
                            expr.clone()
                        }
                    }
                    _ => {
                        let rewritten_args: Vec<Expression> = args
                            .iter()
                            .map(|arg| self.rewrite_edge_prop_func_to_label_attr(arg))
                            .collect();
                        Expression::Function {
                            name: name.clone(),
                            args: rewritten_args,
                        }
                    }
                }
            }
            Expression::Binary { left, op, right } => {
                Expression::Binary {
                    left: Box::new(self.rewrite_edge_prop_func_to_label_attr(left)),
                    op: op.clone(),
                    right: Box::new(self.rewrite_edge_prop_func_to_label_attr(right)),
                }
            }
            Expression::Unary { op, operand } => {
                Expression::Unary {
                    op: op.clone(),
                    operand: Box::new(self.rewrite_edge_prop_func_to_label_attr(operand)),
                }
            }
            Expression::Aggregate { func, arg, distinct } => {
                Expression::Aggregate {
                    func: func.clone(),
                    arg: Box::new(self.rewrite_edge_prop_func_to_label_attr(arg)),
                    distinct: *distinct,
                }
            }
            Expression::List(items) => {
                let rewritten_items: Vec<Expression> = items
                    .iter()
                    .map(|item| self.rewrite_edge_prop_func_to_label_attr(item))
                    .collect();
                Expression::List(rewritten_items)
            }
            Expression::Map(pairs) => {
                let rewritten_pairs: Vec<(String, Expression)> = pairs
                    .iter()
                    .map(|(key, value)| {
                        (key.clone(), self.rewrite_edge_prop_func_to_label_attr(value))
                    })
                    .collect();
                Expression::Map(rewritten_pairs)
            }
            Expression::Case {
                test_expr,
                conditions,
                default,
            } => {
                let rewritten_test_expr = test_expr
                    .as_ref()
                    .map(|e| Box::new(self.rewrite_edge_prop_func_to_label_attr(e)));
                let rewritten_conditions: Vec<(Expression, Expression)> = conditions
                    .iter()
                    .map(|(cond, result)| {
                        (
                            self.rewrite_edge_prop_func_to_label_attr(cond),
                            self.rewrite_edge_prop_func_to_label_attr(result),
                        )
                    })
                    .collect();
                let rewritten_default = default
                    .as_ref()
                    .map(|e| Box::new(self.rewrite_edge_prop_func_to_label_attr(e)));
                Expression::Case {
                    test_expr: rewritten_test_expr,
                    conditions: rewritten_conditions,
                    default: rewritten_default,
                }
            }
            Expression::TypeCast {
                expression,
                target_type,
            } => Expression::TypeCast {
                expression: Box::new(self.rewrite_edge_prop_func_to_label_attr(expression)),
                target_type: target_type.clone(),
            },
            Expression::Subscript { collection, index } => Expression::Subscript {
                collection: Box::new(self.rewrite_edge_prop_func_to_label_attr(collection)),
                index: Box::new(self.rewrite_edge_prop_func_to_label_attr(index)),
            },
            Expression::Range {
                collection,
                start,
                end,
            } => Expression::Range {
                collection: Box::new(self.rewrite_edge_prop_func_to_label_attr(collection)),
                start: start.as_ref().map(|e| Box::new(self.rewrite_edge_prop_func_to_label_attr(e))),
                end: end.as_ref().map(|e| Box::new(self.rewrite_edge_prop_func_to_label_attr(e))),
            },
            Expression::Path(items) => {
                let rewritten_items: Vec<Expression> = items
                    .iter()
                    .map(|item| self.rewrite_edge_prop_func_to_label_attr(item))
                    .collect();
                Expression::Path(rewritten_items)
            }
            Expression::ListComprehension {
                variable,
                source,
                filter,
                map,
            } => Expression::ListComprehension {
                variable: variable.clone(),
                source: Box::new(self.rewrite_edge_prop_func_to_label_attr(source)),
                filter: filter.as_ref().map(|e| Box::new(self.rewrite_edge_prop_func_to_label_attr(e))),
                map: map.as_ref().map(|e| Box::new(self.rewrite_edge_prop_func_to_label_attr(e))),
            },
            Expression::Predicate { func, args } => {
                let rewritten_args: Vec<Expression> = args
                    .iter()
                    .map(|arg| self.rewrite_edge_prop_func_to_label_attr(arg))
                    .collect();
                Expression::Predicate {
                    func: func.clone(),
                    args: rewritten_args,
                }
            }
            Expression::Reduce {
                accumulator,
                initial,
                variable,
                source,
                mapping,
            } => Expression::Reduce {
                accumulator: accumulator.clone(),
                initial: Box::new(self.rewrite_edge_prop_func_to_label_attr(initial)),
                variable: variable.clone(),
                source: Box::new(self.rewrite_edge_prop_func_to_label_attr(source)),
                mapping: Box::new(self.rewrite_edge_prop_func_to_label_attr(mapping)),
            },
            Expression::PathBuild(exprs) => {
                let rewritten_exprs: Vec<Expression> = exprs
                    .iter()
                    .map(|e| self.rewrite_edge_prop_func_to_label_attr(e))
                    .collect();
                Expression::PathBuild(rewritten_exprs)
            }
            _ => expr.clone(),
        }
    }

    /// 从表达式中提取标签名称
    fn extract_label_name(&self, expr: &Expression) -> String {
        match expr {
            Expression::Label(name) => name.clone(),
            Expression::Variable(name) => name.clone(),
            _ => String::new(),
        }
    }
}

impl Default for ExpressionRewriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rewrite_edge_prop_func_to_label_attr() {
        let rewriter = ExpressionRewriter::new();

        let label_expr = Expression::Label("e".to_string());
        let rank_func = Expression::Function {
            name: "rank".to_string(),
            args: vec![label_expr],
        };

        let rewritten = rewriter.rewrite_edge_prop_func_to_label_attr(&rank_func);

        match rewritten {
            Expression::LabelTagProperty { property, .. } => {
                assert_eq!(property, "_rank");
            }
            _ => panic!("Expected LabelTagProperty"),
        }
    }

    #[test]
    fn test_rewrite_label_attr_to_edge_prop() {
        let rewriter = ExpressionRewriter::new();

        let label_expr = Expression::Label("e".to_string());
        let label_tag_prop = Expression::LabelTagProperty {
            tag: Box::new(label_expr),
            property: "name".to_string(),
        };

        let rewritten = rewriter.rewrite_label_attr_to_edge_prop(&label_tag_prop);

        match rewritten {
            Expression::EdgeProperty { edge_name, property } => {
                assert_eq!(edge_name, "e");
                assert_eq!(property, "name");
            }
            _ => panic!("Expected EdgeProperty"),
        }
    }

    #[test]
    fn test_rewrite_label_attr_to_tag_prop() {
        let rewriter = ExpressionRewriter::new();

        let label_expr = Expression::Label("v".to_string());
        let label_tag_prop = Expression::LabelTagProperty {
            tag: Box::new(label_expr),
            property: "name".to_string(),
        };

        let rewritten = rewriter.rewrite_label_attr_to_tag_prop(&label_tag_prop);

        match rewritten {
            Expression::TagProperty { tag_name, property } => {
                assert_eq!(tag_name, "v");
                assert_eq!(property, "name");
            }
            _ => panic!("Expected TagProperty"),
        }
    }
}
