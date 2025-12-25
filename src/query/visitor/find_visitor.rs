//! FindVisitor - 用于查找表达式中特定类型子表达式的访问器
//! 对应 NebulaGraph FindVisitor.h/.cpp 的功能

use crate::core::types::expression::DataType;
use crate::core::Value;
use crate::expression::{Expression, ExpressionType, ExpressionVisitor};
use crate::query::visitor::QueryVisitor;
use std::collections::HashSet;

#[derive(Debug)]
pub struct FindVisitor {
    /// 要查找的表达式类型集合
    target_types: HashSet<ExpressionType>,
    /// 找到的表达式列表
    found_exprs: Vec<Expression>,
}

impl FindVisitor {
    pub fn new() -> Self {
        Self {
            target_types: HashSet::new(),
            found_exprs: Vec::new(),
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

impl ExpressionVisitor for FindVisitor {
    type Result = ();

    fn visit_literal(&mut self, value: &Value) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Literal) {
            self.found_exprs.push(Expression::Literal(value.clone()));
        }
    }

    fn visit_variable(&mut self, name: &str) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Variable) {
            self.found_exprs
                .push(Expression::Variable(name.to_string()));
        }
    }

    fn visit_property(&mut self, object: &Expression, property: &str) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Property) {
            self.found_exprs.push(Expression::Property {
                object: Box::new(object.clone()),
                property: property.to_string(),
            });
        }
        self.visit(object);
    }

    fn visit_binary(
        &mut self,
        left: &Expression,
        op: &crate::core::types::operators::BinaryOperator,
        right: &Expression,
    ) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Binary) {
            self.found_exprs.push(Expression::Binary {
                left: Box::new(left.clone()),
                op: op.clone(),
                right: Box::new(right.clone()),
            });
        }
        self.visit(left);
        self.visit(right);
    }

    fn visit_unary(
        &mut self,
        op: &crate::core::types::operators::UnaryOperator,
        operand: &Expression,
    ) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Unary) {
            self.found_exprs.push(Expression::Unary {
                op: op.clone(),
                operand: Box::new(operand.clone()),
            });
        }
        self.visit(operand);
    }

    fn visit_function(&mut self, name: &str, args: &[Expression]) -> Self::Result {
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

    fn visit_aggregate(
        &mut self,
        func: &crate::core::types::operators::AggregateFunction,
        arg: &Expression,
        _distinct: bool,
    ) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Aggregate) {
            self.found_exprs.push(Expression::Aggregate {
                func: func.clone(),
                arg: Box::new(arg.clone()),
                distinct: false,
            });
        }
        self.visit(arg);
    }

    fn visit_list(&mut self, items: &[Expression]) -> Self::Result {
        if self.target_types.contains(&ExpressionType::List) {
            self.found_exprs.push(Expression::List(items.to_vec()));
        }
        for item in items {
            self.visit(item);
        }
    }

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Map) {
            self.found_exprs.push(Expression::Map(pairs.to_vec()));
        }
        for (_, value) in pairs {
            self.visit(value);
        }
    }

    fn visit_case(
        &mut self,
        conditions: &[(Expression, Expression)],
        default: &Option<Box<Expression>>,
    ) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Case) {
            self.found_exprs.push(Expression::Case {
                conditions: conditions.to_vec(),
                default: default.as_ref().map(|e| Box::new(e.as_ref().clone())),
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

    fn visit_type_cast(&mut self, expr: &Expression, target_type: &DataType) -> Self::Result {
        if self.target_types.contains(&ExpressionType::TypeCast) {
            self.found_exprs.push(Expression::TypeCast {
                expr: Box::new(expr.clone()),
                target_type: target_type.clone(),
            });
        }
        self.visit(expr);
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Subscript) {
            self.found_exprs.push(Expression::Subscript {
                collection: Box::new(collection.clone()),
                index: Box::new(index.clone()),
            });
        }
        self.visit(collection);
        self.visit(index);
    }

    fn visit_range(
        &mut self,
        collection: &Expression,
        start: &Option<Box<Expression>>,
        end: &Option<Box<Expression>>,
    ) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Range) {
            self.found_exprs.push(Expression::Range {
                collection: Box::new(collection.clone()),
                start: start.as_ref().map(|e| Box::new(e.as_ref().clone())),
                end: end.as_ref().map(|e| Box::new(e.as_ref().clone())),
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

    fn visit_path(&mut self, items: &[Expression]) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Path) {
            self.found_exprs.push(Expression::Path(items.to_vec()));
        }
        for item in items {
            self.visit(item);
        }
    }

    fn visit_label(&mut self, name: &str) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Label) {
            self.found_exprs.push(Expression::Label(name.to_string()));
        }
    }

    fn visit_tag_property(&mut self, tag: &str, prop: &str) -> Self::Result {
        if self.target_types.contains(&ExpressionType::TagProperty) {
            self.found_exprs.push(Expression::TagProperty {
                tag: tag.to_string(),
                prop: prop.to_string(),
            });
        }
    }

    fn visit_edge_property(&mut self, edge: &str, prop: &str) -> Self::Result {
        if self.target_types.contains(&ExpressionType::EdgeProperty) {
            self.found_exprs.push(Expression::EdgeProperty {
                edge: edge.to_string(),
                prop: prop.to_string(),
            });
        }
    }

    fn visit_input_property(&mut self, prop: &str) -> Self::Result {
        if self.target_types.contains(&ExpressionType::InputProperty) {
            self.found_exprs
                .push(Expression::InputProperty(prop.to_string()));
        }
    }

    fn visit_variable_property(&mut self, var: &str, prop: &str) -> Self::Result {
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

    fn visit_source_property(&mut self, tag: &str, prop: &str) -> Self::Result {
        if self.target_types.contains(&ExpressionType::SourceProperty) {
            self.found_exprs.push(Expression::SourceProperty {
                tag: tag.to_string(),
                prop: prop.to_string(),
            });
        }
    }

    fn visit_destination_property(&mut self, tag: &str, prop: &str) -> Self::Result {
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

    fn visit_unary_plus(&mut self, expr: &Expression) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Unary) {
            self.found_exprs.push(Expression::UnaryPlus(Box::new(expr.clone())));
        }
        self.visit(expr);
    }

    fn visit_unary_negate(&mut self, expr: &Expression) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Unary) {
            self.found_exprs.push(Expression::UnaryNegate(Box::new(expr.clone())));
        }
        self.visit(expr);
    }

    fn visit_unary_not(&mut self, expr: &Expression) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Unary) {
            self.found_exprs.push(Expression::UnaryNot(Box::new(expr.clone())));
        }
        self.visit(expr);
    }

    fn visit_unary_incr(&mut self, expr: &Expression) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Unary) {
            self.found_exprs.push(Expression::UnaryIncr(Box::new(expr.clone())));
        }
        self.visit(expr);
    }

    fn visit_unary_decr(&mut self, expr: &Expression) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Unary) {
            self.found_exprs.push(Expression::UnaryDecr(Box::new(expr.clone())));
        }
        self.visit(expr);
    }

    fn visit_is_null(&mut self, expr: &Expression) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Unary) {
            self.found_exprs.push(Expression::IsNull(Box::new(expr.clone())));
        }
        self.visit(expr);
    }

    fn visit_is_not_null(&mut self, expr: &Expression) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Unary) {
            self.found_exprs.push(Expression::IsNotNull(Box::new(expr.clone())));
        }
        self.visit(expr);
    }

    fn visit_is_empty(&mut self, expr: &Expression) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Unary) {
            self.found_exprs.push(Expression::IsEmpty(Box::new(expr.clone())));
        }
        self.visit(expr);
    }

    fn visit_is_not_empty(&mut self, expr: &Expression) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Unary) {
            self.found_exprs.push(Expression::IsNotEmpty(Box::new(expr.clone())));
        }
        self.visit(expr);
    }

    fn visit_type_casting(&mut self, expr: &Expression, target_type: &str) -> Self::Result {
        if self.target_types.contains(&ExpressionType::TypeCast) {
            self.found_exprs.push(Expression::TypeCasting {
                expr: Box::new(expr.clone()),
                target_type: target_type.to_string(),
            });
        }
        self.visit(expr);
    }

    fn visit_list_comprehension(
        &mut self,
        generator: &Expression,
        condition: &Option<Box<Expression>>,
    ) -> Self::Result {
        if self.target_types.contains(&ExpressionType::List) {
            self.found_exprs.push(Expression::ListComprehension {
                generator: Box::new(generator.clone()),
                condition: condition.clone(),
            });
        }
        self.visit(generator);
        if let Some(cond) = condition {
            self.visit(cond);
        }
    }

    fn visit_predicate(&mut self, list: &Expression, condition: &Expression) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Property) {
            self.found_exprs.push(Expression::Predicate {
                list: Box::new(list.clone()),
                condition: Box::new(condition.clone()),
            });
        }
        self.visit(list);
        self.visit(condition);
    }

    fn visit_reduce(
        &mut self,
        list: &Expression,
        var: &str,
        initial: &Expression,
        expr: &Expression,
    ) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Aggregate) {
            self.found_exprs.push(Expression::Reduce {
                list: Box::new(list.clone()),
                var: var.to_string(),
                initial: Box::new(initial.clone()),
                expr: Box::new(expr.clone()),
            });
        }
        self.visit(list);
        self.visit(initial);
        self.visit(expr);
    }

    fn visit_path_build(&mut self, items: &[Expression]) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Path) {
            self.found_exprs.push(Expression::PathBuild(items.to_vec()));
        }
        for item in items {
            self.visit(item);
        }
    }

    fn visit_es_query(&mut self, query: &str) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Function) {
            self.found_exprs.push(Expression::ESQuery(query.to_string()));
        }
    }

    fn visit_uuid(&mut self) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Literal) {
            self.found_exprs.push(Expression::UUID);
        }
    }

    fn visit_subscript_range(
        &mut self,
        collection: &Expression,
        start: &Option<Box<Expression>>,
        end: &Option<Box<Expression>>,
    ) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Subscript) {
            self.found_exprs.push(Expression::SubscriptRange {
                collection: Box::new(collection.clone()),
                start: start.as_ref().map(|e| Box::new(e.as_ref().clone())),
                end: end.as_ref().map(|e| Box::new(e.as_ref().clone())),
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

    fn visit_match_path_pattern(&mut self, path_alias: &str, patterns: &[Expression]) -> Self::Result {
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

impl QueryVisitor for FindVisitor {
    type QueryResult = Vec<Expression>;

    fn get_result(&self) -> Self::QueryResult {
        self.found_exprs.clone()
    }

    fn reset(&mut self) {
        self.found_exprs.clear();
    }

    fn is_success(&self) -> bool {
        true
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

        let literals = visitor.find_if(&expr, |e| {
            matches!(e, Expression::Literal(Value::Int(_)))
        });

        assert_eq!(literals.len(), 3);
    }
}
