//! FindVisitor - 用于查找表达式中特定类型子表达式的访问器
//! 对应 NebulaGraph FindVisitor.h/.cpp 的功能

use crate::core::types::expression::DataType;
use crate::core::{
    visitor::{Visitor, VisitorState},
    Value,
};
use crate::expression::{Expression, ExpressionType};
use std::collections::HashSet;

#[derive(Debug)]
pub struct FindVisitor {
    /// 要查找的表达式类型集合
    target_types: HashSet<ExpressionType>,
    /// 找到的表达式列表
    found_exprs: Vec<Expression>,
    /// 访问者状态
    state: VisitorState,
}

impl FindVisitor {
    pub fn new() -> Self {
        Self {
            target_types: HashSet::new(),
            found_exprs: Vec::new(),
            state: VisitorState::new(),
        }
    }

    /// 设置要查找的表达式类型
    pub fn set_target_types(&mut self, types: Vec<ExpressionType>) -> &mut Self {
        self.target_types.clear();
        for expr_type in types {
            self.target_types.insert(expr_type);
        }
        self
    }

    /// 添加要查找的表达式类型
    pub fn add_target_type(&mut self, expr_type: ExpressionType) -> &mut Self {
        self.target_types.insert(expr_type);
        self
    }

    /// 搜索表达式中匹配类型的所有子表达式
    pub fn find(&mut self, expr: &Expression) -> Vec<Expression> {
        self.found_exprs.clear();
        self.visit(expr);
        self.found_exprs.clone()
    }

    /// 检查表达式中是否存在匹配类型的子表达式
    pub fn exist(&mut self, expr: &Expression) -> bool {
        self.found_exprs.clear();
        self.visit(expr);
        !self.found_exprs.is_empty()
    }

    /// 搜索表达式中匹配特定条件的子表达式
    pub fn find_if<F>(&mut self, expr: &Expression, predicate: F) -> Vec<Expression>
    where
        F: Fn(&Expression) -> bool,
    {
        let mut results = Vec::new();
        self.visit_with_predicate(expr, &predicate, &mut results);
        results
    }

    fn visit_with_predicate<F>(
        &self,
        expr: &Expression,
        predicate: &F,
        results: &mut Vec<Expression>,
    ) where
        F: Fn(&Expression) -> bool,
    {
        if predicate(expr) {
            results.push(expr.clone());
        }

        for child in expr.children() {
            self.visit_with_predicate(child, predicate, results);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::operators::BinaryOperator;

    #[test]
    fn test_find_literals() {
        let mut visitor = FindVisitor::new();

        let expr = Expression::Binary {
            left: Box::new(Expression::Literal(Value::Int(1))),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Binary {
                left: Box::new(Expression::Literal(Value::Int(2))),
                op: BinaryOperator::Multiply,
                right: Box::new(Expression::Literal(Value::Int(3))),
            }),
        };

        let literals = visitor.add_target_type(ExpressionType::Literal).find(&expr);

        assert_eq!(literals.len(), 3);
    }

    #[test]
    fn test_find_with_predicate() {
        let mut visitor = FindVisitor::new();

        let expr = Expression::Binary {
            left: Box::new(Expression::Literal(Value::Int(1))),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Binary {
                left: Box::new(Expression::Literal(Value::Int(2))),
                op: BinaryOperator::Multiply,
                right: Box::new(Expression::Literal(Value::Int(3))),
            }),
        };

        let literals = visitor.find_if(&expr, |e| matches!(e, Expression::Literal(Value::Int(_))));

        assert_eq!(literals.len(), 3);
    }
}

impl Visitor<Expression> for FindVisitor {
    type Result = ();

    fn visit(&mut self, target: &Expression) -> Self::Result {
        match target {
            Expression::Literal(value) => {
                if self.target_types.contains(&ExpressionType::Literal) {
                    self.found_exprs.push(Expression::Literal(value.clone()));
                }
            }
            Expression::Variable(name) => {
                if self.target_types.contains(&ExpressionType::Variable) {
                    self.found_exprs
                        .push(Expression::Variable(name.to_string()));
                }
            }
            Expression::Property { object, property } => {
                if self.target_types.contains(&ExpressionType::Property) {
                    self.found_exprs.push(Expression::Property {
                        object: object.clone(),
                        property: property.to_string(),
                    });
                }
                self.visit(object);
            }
            Expression::Binary { left, op, right } => {
                if self.target_types.contains(&ExpressionType::Binary) {
                    self.found_exprs.push(Expression::Binary {
                        left: left.clone(),
                        op: op.clone(),
                        right: right.clone(),
                    });
                }
                self.visit(left);
                self.visit(right);
            }
            Expression::Unary { op, operand } => {
                if self.target_types.contains(&ExpressionType::Unary) {
                    self.found_exprs.push(Expression::Unary {
                        op: op.clone(),
                        operand: operand.clone(),
                    });
                }
                self.visit(operand);
            }
            Expression::Function { name, args } => {
                if self.target_types.contains(&ExpressionType::Function) {
                    self.found_exprs.push(Expression::Function {
                        name: name.to_string(),
                        args: args.to_vec(),
                    });
                }
                for arg in args {
                    self.visit(arg);
                }
            }
            Expression::Aggregate {
                func,
                arg,
                distinct,
            } => {
                if self.target_types.contains(&ExpressionType::Aggregate) {
                    self.found_exprs.push(Expression::Aggregate {
                        func: func.clone(),
                        arg: arg.clone(),
                        distinct: *distinct,
                    });
                }
                self.visit(arg);
            }
            Expression::List(items) => {
                if self.target_types.contains(&ExpressionType::List) {
                    self.found_exprs.push(Expression::List(items.to_vec()));
                }
                for item in items {
                    self.visit(item);
                }
            }
            Expression::Map(pairs) => {
                if self.target_types.contains(&ExpressionType::Map) {
                    self.found_exprs.push(Expression::Map(pairs.to_vec()));
                }
                for (_, value) in pairs {
                    self.visit(value);
                }
            }
            Expression::Case {
                conditions,
                default,
            } => {
                if self.target_types.contains(&ExpressionType::Case) {
                    self.found_exprs.push(Expression::Case {
                        conditions: conditions.to_vec(),
                        default: default.as_ref().map(|e| e.clone()),
                    });
                }
                for (condition, value) in conditions {
                    self.visit(condition);
                    self.visit(value);
                }
                if let Some(default_expr) = default {
                    self.visit(default_expr);
                }
            }
            Expression::TypeCast { expr, target_type } => {
                if self.target_types.contains(&ExpressionType::TypeCast) {
                    self.found_exprs.push(Expression::TypeCast {
                        expr: expr.clone(),
                        target_type: target_type.clone(),
                    });
                }
                self.visit(expr);
            }
            Expression::Subscript { collection, index } => {
                if self.target_types.contains(&ExpressionType::Subscript) {
                    self.found_exprs.push(Expression::Subscript {
                        collection: collection.clone(),
                        index: index.clone(),
                    });
                }
                self.visit(collection);
                self.visit(index);
            }
            Expression::Range {
                collection,
                start,
                end,
            } => {
                if self.target_types.contains(&ExpressionType::Range) {
                    self.found_exprs.push(Expression::Range {
                        collection: collection.clone(),
                        start: start.as_ref().map(|e| e.clone()),
                        end: end.as_ref().map(|e| e.clone()),
                    });
                }
                self.visit(collection);
                if let Some(start_expr) = start {
                    self.visit(start_expr);
                }
                if let Some(end_expr) = end {
                    self.visit(end_expr);
                }
            }
            Expression::Path(items) => {
                if self.target_types.contains(&ExpressionType::Path) {
                    self.found_exprs.push(Expression::Path(items.to_vec()));
                }
                for item in items {
                    self.visit(item);
                }
            }
            Expression::Label(name) => {
                if self.target_types.contains(&ExpressionType::Label) {
                    self.found_exprs.push(Expression::Label(name.to_string()));
                }
            }
            Expression::TagProperty { tag, prop } => {
                if self.target_types.contains(&ExpressionType::TagProperty) {
                    self.found_exprs.push(Expression::TagProperty {
                        tag: tag.to_string(),
                        prop: prop.to_string(),
                    });
                }
            }
            Expression::EdgeProperty { edge, prop } => {
                if self.target_types.contains(&ExpressionType::EdgeProperty) {
                    self.found_exprs.push(Expression::EdgeProperty {
                        edge: edge.to_string(),
                        prop: prop.to_string(),
                    });
                }
            }
            Expression::InputProperty(prop) => {
                if self.target_types.contains(&ExpressionType::InputProperty) {
                    self.found_exprs
                        .push(Expression::InputProperty(prop.to_string()));
                }
            }
            Expression::VariableProperty { var, prop } => {
                if self
                    .target_types
                    .contains(&ExpressionType::VariableProperty)
                {
                    self.found_exprs.push(Expression::VariableProperty {
                        var: var.to_string(),
                        prop: prop.to_string(),
                    });
                }
            }
            Expression::SourceProperty { tag, prop } => {
                if self.target_types.contains(&ExpressionType::SourceProperty) {
                    self.found_exprs.push(Expression::SourceProperty {
                        tag: tag.to_string(),
                        prop: prop.to_string(),
                    });
                }
            }
            Expression::DestinationProperty { tag, prop } => {
                if self
                    .target_types
                    .contains(&ExpressionType::DestinationProperty)
                {
                    self.found_exprs.push(Expression::DestinationProperty {
                        tag: tag.to_string(),
                        prop: prop.to_string(),
                    });
                }
            }
            Expression::UnaryPlus(expr) => {
                if self.target_types.contains(&ExpressionType::Unary) {
                    self.found_exprs.push(Expression::UnaryPlus(expr.clone()));
                }
                self.visit(expr);
            }
            Expression::UnaryNegate(expr) => {
                if self.target_types.contains(&ExpressionType::Unary) {
                    self.found_exprs.push(Expression::UnaryNegate(expr.clone()));
                }
                self.visit(expr);
            }
            Expression::UnaryNot(expr) => {
                if self.target_types.contains(&ExpressionType::Unary) {
                    self.found_exprs.push(Expression::UnaryNot(expr.clone()));
                }
                self.visit(expr);
            }
            Expression::UnaryIncr(expr) => {
                if self.target_types.contains(&ExpressionType::Unary) {
                    self.found_exprs.push(Expression::UnaryIncr(expr.clone()));
                }
                self.visit(expr);
            }
            Expression::UnaryDecr(expr) => {
                if self.target_types.contains(&ExpressionType::Unary) {
                    self.found_exprs.push(Expression::UnaryDecr(expr.clone()));
                }
                self.visit(expr);
            }
            Expression::IsNull(expr) => {
                if self.target_types.contains(&ExpressionType::Unary) {
                    self.found_exprs.push(Expression::IsNull(expr.clone()));
                }
                self.visit(expr);
            }
            Expression::IsNotNull(expr) => {
                if self.target_types.contains(&ExpressionType::Unary) {
                    self.found_exprs.push(Expression::IsNotNull(expr.clone()));
                }
                self.visit(expr);
            }
            Expression::IsEmpty(expr) => {
                if self.target_types.contains(&ExpressionType::Unary) {
                    self.found_exprs.push(Expression::IsEmpty(expr.clone()));
                }
                self.visit(expr);
            }
            Expression::IsNotEmpty(expr) => {
                if self.target_types.contains(&ExpressionType::Unary) {
                    self.found_exprs.push(Expression::IsNotEmpty(expr.clone()));
                }
                self.visit(expr);
            }
            Expression::ListComprehension {
                generator,
                condition,
            } => {
                if self.target_types.contains(&ExpressionType::List) {
                    self.found_exprs.push(Expression::ListComprehension {
                        generator: generator.clone(),
                        condition: condition.clone(),
                    });
                }
                self.visit(generator);
                if let Some(cond) = condition {
                    self.visit(cond);
                }
            }
            Expression::Predicate { list, condition } => {
                if self.target_types.contains(&ExpressionType::Property) {
                    self.found_exprs.push(Expression::Predicate {
                        list: list.clone(),
                        condition: condition.clone(),
                    });
                }
                self.visit(list);
                self.visit(condition);
            }
            Expression::Reduce {
                list,
                var,
                initial,
                expr,
            } => {
                if self.target_types.contains(&ExpressionType::Aggregate) {
                    self.found_exprs.push(Expression::Reduce {
                        list: list.clone(),
                        var: var.to_string(),
                        initial: initial.clone(),
                        expr: expr.clone(),
                    });
                }
                self.visit(list);
                self.visit(initial);
                self.visit(expr);
            }
            Expression::ESQuery(query) => {
                if self.target_types.contains(&ExpressionType::Function) {
                    self.found_exprs
                        .push(Expression::ESQuery(query.to_string()));
                }
            }
            Expression::UUID => {
                if self.target_types.contains(&ExpressionType::Literal) {
                    self.found_exprs.push(Expression::UUID);
                }
            }
            Expression::MatchPathPattern {
                path_alias,
                patterns,
            } => {
                if self.target_types.contains(&ExpressionType::Path) {
                    self.found_exprs.push(Expression::MatchPathPattern {
                        path_alias: path_alias.to_string(),
                        patterns: patterns.to_vec(),
                    });
                }
                for pattern in patterns {
                    self.visit(pattern);
                }
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
