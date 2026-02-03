//! AST 转换器（AstTransformer）
//! 实现 StmtTransformer trait，用于深度优先遍历和转换语句 AST
//! 支持在遍历过程中修改和转换语句

use crate::core::Expression;
use crate::core::types::Expression::*;
use crate::query::visitor::stmt_transformer::StmtTransformer;

pub trait AstTransformer: StmtTransformer {
    fn transform_expression(&mut self, expr: &Expression) -> Expression {
        match expr {
            Literal(value) => Literal(value.clone()),
            Variable(name) => Variable(name.clone()),
            Property { object, property } => {
                let transformed = self.transform_expression(object);
                Property {
                    object: Box::new(transformed),
                    property: property.clone(),
                }
            }
            Binary { left, op, right } => {
                let transformed_left = self.transform_expression(left);
                let transformed_right = self.transform_expression(right);
                Binary {
                    left: Box::new(transformed_left),
                    op: op.clone(),
                    right: Box::new(transformed_right),
                }
            }
            Unary { op, operand } => {
                let transformed = self.transform_expression(operand);
                Unary {
                    op: op.clone(),
                    operand: Box::new(transformed),
                }
            }
            Function { name, args } => {
                let transformed_args = args
                    .iter()
                    .map(|arg| self.transform_expression(arg))
                    .collect();
                Function {
                    name: name.clone(),
                    args: transformed_args,
                }
            }
            Aggregate { func, arg, distinct } => {
                let transformed_arg = self.transform_expression(arg);
                Aggregate {
                    func: func.clone(),
                    arg: Box::new(transformed_arg),
                    distinct: *distinct,
                }
            }
            List(items) => {
                let transformed = items
                    .iter()
                    .map(|item| self.transform_expression(item))
                    .collect();
                List(transformed)
            }
            Map(items) => {
                let transformed = items
                    .iter()
                    .map(|(key, value)| (key.clone(), self.transform_expression(value)))
                    .collect();
                Map(transformed)
            }
            Case { test_expr, conditions, default } => {
                let transformed_test_expr = test_expr
                    .as_ref()
                    .map(|expr| Box::new(self.transform_expression(expr)));
                let transformed_conditions = conditions
                    .iter()
                    .map(|(when, then)| {
                        (
                            self.transform_expression(when),
                            self.transform_expression(then),
                        )
                    })
                    .collect();
                let transformed_default = default
                    .as_ref()
                    .map(|expr| Box::new(self.transform_expression(expr)));
                Case {
                    test_expr: transformed_test_expr,
                    conditions: transformed_conditions,
                    default: transformed_default,
                }
            }
            TypeCast { expression, target_type } => {
                let transformed = self.transform_expression(expression);
                TypeCast {
                    expression: Box::new(transformed),
                    target_type: target_type.clone(),
                }
            }
            Subscript { collection, index } => {
                let transformed_collection = self.transform_expression(collection);
                let transformed_index = self.transform_expression(index);
                Subscript {
                    collection: Box::new(transformed_collection),
                    index: Box::new(transformed_index),
                }
            }
            Range { collection, start, end } => {
                let transformed_collection = self.transform_expression(collection);
                let transformed_start = start
                    .as_ref()
                    .map(|expr| Box::new(self.transform_expression(expr)));
                let transformed_end = end
                    .as_ref()
                    .map(|expr| Box::new(self.transform_expression(expr)));
                Range {
                    collection: Box::new(transformed_collection),
                    start: transformed_start,
                    end: transformed_end,
                }
            }
            Path(exprs) => {
                let transformed = exprs
                    .iter()
                    .map(|expr| self.transform_expression(expr))
                    .collect();
                Path(transformed)
            }
            Label(label) => Label(label.clone()),
            ListComprehension {
                variable,
                source,
                filter,
                map,
            } => {
                let transformed_source = self.transform_expression(source);
                let transformed_filter = filter
                    .as_ref()
                    .map(|expr| Box::new(self.transform_expression(expr)));
                let transformed_map = map
                    .as_ref()
                    .map(|expr| Box::new(self.transform_expression(expr)));
                ListComprehension {
                    variable: variable.clone(),
                    source: Box::new(transformed_source),
                    filter: transformed_filter,
                    map: transformed_map,
                }
            }
            LabelTagProperty { tag, property } => {
                let transformed_tag = self.transform_expression(tag);
                LabelTagProperty {
                    tag: Box::new(transformed_tag),
                    property: property.clone(),
                }
            }
            TagProperty { tag_name, property } => TagProperty {
                tag_name: tag_name.clone(),
                property: property.clone(),
            },
            EdgeProperty { edge_name, property } => EdgeProperty {
                edge_name: edge_name.clone(),
                property: property.clone(),
            },
            Predicate { func, args } => {
                let transformed_args = args
                    .iter()
                    .map(|arg| self.transform_expression(arg))
                    .collect();
                Predicate {
                    func: func.clone(),
                    args: transformed_args,
                }
            }
            Reduce {
                accumulator,
                initial,
                variable,
                source,
                mapping,
            } => {
                let transformed_initial = self.transform_expression(initial);
                let transformed_source = self.transform_expression(source);
                let transformed_mapping = self.transform_expression(mapping);
                Reduce {
                    accumulator: accumulator.clone(),
                    initial: Box::new(transformed_initial),
                    variable: variable.clone(),
                    source: Box::new(transformed_source),
                    mapping: Box::new(transformed_mapping),
                }
            }
            PathBuild(exprs) => {
                let transformed = exprs
                    .iter()
                    .map(|expr| self.transform_expression(expr))
                    .collect();
                PathBuild(transformed)
            }
        }
    }

    fn transform_expressions(&mut self, exprs: &[Expression]) -> Vec<Expression> {
        exprs
            .iter()
            .map(|expr| self.transform_expression(expr))
            .collect()
    }
}
