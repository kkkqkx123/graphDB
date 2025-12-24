//! ExtractFilterExprVisitor - 用于提取过滤表达式的访问器
//! 对应 NebulaGraph ExtractFilterExprVisitor.h/.cpp 的功能

use crate::core::{
    AggregateFunction, BinaryOperator, DataType, Expression, LiteralValue, UnaryOperator,
};
use crate::expression::ExpressionVisitor;
use crate::query::visitor::QueryVisitor;

#[derive(Debug)]
pub struct ExtractFilterExprVisitor {
    /// 提取到的过滤表达式
    filter_exprs: Vec<Expression>,
    /// 是否只提取顶层的过滤条件
    top_level_only: bool,
    /// 当前是否在顶层
    is_top_level: bool,
}

impl Clone for ExtractFilterExprVisitor {
    fn clone(&self) -> Self {
        Self {
            filter_exprs: self.filter_exprs.clone(),
            top_level_only: self.top_level_only,
            is_top_level: self.is_top_level,
        }
    }
}

impl ExtractFilterExprVisitor {
    pub fn new(top_level_only: bool) -> Self {
        Self {
            filter_exprs: Vec::new(),
            top_level_only: top_level_only,
            is_top_level: true,
        }
    }

    pub fn extract(&mut self, expr: &Expression) -> Result<Vec<Expression>, String> {
        self.filter_exprs.clear();
        self.is_top_level = true;
        self.visit(expr)?;
        Ok(self.filter_exprs.clone())
    }

    fn visit(&mut self, expr: &Expression) -> Result<(), String> {
        match expr {
            Expression::Binary { left, op: _, right } => {
                if self.is_top_level || !self.top_level_only {
                    self.visit_with_updated_level(left)?;
                    self.visit_with_updated_level(right)?;
                } else {
                    self.filter_exprs.push(expr.clone());
                }
                Ok(())
            }

            Expression::Function { name, args: _ } => {
                if is_filter_function(name) {
                    if self.is_top_level || !self.top_level_only {
                        self.filter_exprs.push(expr.clone());
                    }
                }
                Ok(())
            }

            _ => {
                if self.is_top_level || !self.top_level_only {
                    if is_filter_expression(expr) {
                        self.filter_exprs.push(expr.clone());
                    }
                }
                self.visit_children(expr)
            }
        }
    }

    fn visit_with_updated_level(&mut self, expr: &Expression) -> Result<(), String> {
        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        let result = self.visit(expr);
        self.is_top_level = old_top_level;
        result
    }

    fn visit_children(&mut self, expr: &Expression) -> Result<(), String> {
        match expr {
            Expression::Unary { op: _, operand } => self.visit(operand),
            Expression::Binary { left, op: _, right } => {
                self.visit(left)?;
                self.visit(right)
            }
            Expression::Function { name: _, args } => {
                for arg in args {
                    self.visit(arg)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn get_filter_exprs(&self) -> &Vec<Expression> {
        &self.filter_exprs
    }
}

fn is_filter_function(func_name: &str) -> bool {
    matches!(
        func_name.to_lowercase().as_str(),
        "isempty"
            | "isnull"
            | "isnotnull"
            | "isnullorempty"
            | "has"
            | "haslabel"
            | "hastag"
            | "contains"
    )
}

impl ExpressionVisitor for ExtractFilterExprVisitor {
    type Result = Result<(), String>;

    fn visit_literal(&mut self, _value: &LiteralValue) -> Self::Result {
        Ok(())
    }

    fn visit_variable(&mut self, _name: &str) -> Self::Result {
        Ok(())
    }

    fn visit_property(&mut self, object: &Expression, property: &str) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::Property {
                object: Box::new(object.clone()),
                property: property.to_string(),
            });
        }
        self.visit(object)?;
        Ok(())
    }

    fn visit_binary(
        &mut self,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
    ) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::Binary {
                left: Box::new(left.clone()),
                op: op.clone(),
                right: Box::new(right.clone()),
            });
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(left)?;
        self.visit(right)?;
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_unary(&mut self, op: &UnaryOperator, operand: &Expression) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::Unary {
                op: op.clone(),
                operand: Box::new(operand.clone()),
            });
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(operand)?;
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_function(&mut self, name: &str, args: &[Expression]) -> Self::Result {
        if is_filter_function(name) && (self.is_top_level || !self.top_level_only) {
            self.filter_exprs.push(Expression::Function {
                name: name.to_string(),
                args: args.to_vec(),
            });
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        for arg in args {
            self.visit(arg)?;
        }
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_aggregate(
        &mut self,
        _func: &AggregateFunction,
        arg: &Expression,
        _distinct: bool,
    ) -> Self::Result {
        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(arg)?;
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_list(&mut self, items: &[Expression]) -> Self::Result {
        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        for item in items {
            self.visit(item)?;
        }
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        for (_, value) in pairs {
            self.visit(value)?;
        }
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_case(
        &mut self,
        conditions: &[(Expression, Expression)],
        default: &Option<Box<Expression>>,
    ) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::Case {
                conditions: conditions.to_vec(),
                default: default.as_ref().map(|e| Box::new(e.as_ref().clone())),
            });
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        for (condition, value) in conditions {
            self.visit(condition)?;
            self.visit(value)?;
        }
        if let Some(default_expr) = default {
            self.visit(default_expr)?;
        }
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_type_cast(&mut self, expr: &Expression, target_type: &DataType) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::TypeCast {
                expr: Box::new(expr.clone()),
                target_type: target_type.clone(),
            });
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(expr)?;
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::Subscript {
                collection: Box::new(collection.clone()),
                index: Box::new(index.clone()),
            });
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(collection)?;
        self.visit(index)?;
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_range(
        &mut self,
        collection: &Expression,
        start: &Option<Box<Expression>>,
        end: &Option<Box<Expression>>,
    ) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::Range {
                collection: Box::new(collection.clone()),
                start: start.as_ref().map(|e| Box::new(e.as_ref().clone())),
                end: end.as_ref().map(|e| Box::new(e.as_ref().clone())),
            });
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(collection)?;
        if let Some(start_expr) = start {
            self.visit(start_expr)?;
        }
        if let Some(end_expr) = end {
            self.visit(end_expr)?;
        }
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_path(&mut self, items: &[Expression]) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::Path(items.to_vec()));
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        for item in items {
            self.visit(item)?;
        }
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_label(&mut self, _name: &str) -> Self::Result {
        Ok(())
    }

    fn visit_tag_property(&mut self, tag: &str, prop: &str) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::TagProperty {
                tag: tag.to_string(),
                prop: prop.to_string(),
            });
        }
        Ok(())
    }

    fn visit_edge_property(&mut self, edge: &str, prop: &str) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::EdgeProperty {
                edge: edge.to_string(),
                prop: prop.to_string(),
            });
        }
        Ok(())
    }

    fn visit_input_property(&mut self, prop: &str) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs
                .push(Expression::InputProperty(prop.to_string()));
        }
        Ok(())
    }

    fn visit_variable_property(&mut self, var: &str, prop: &str) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::VariableProperty {
                var: var.to_string(),
                prop: prop.to_string(),
            });
        }
        Ok(())
    }

    fn visit_source_property(&mut self, tag: &str, prop: &str) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::SourceProperty {
                tag: tag.to_string(),
                prop: prop.to_string(),
            });
        }
        Ok(())
    }

    fn visit_destination_property(&mut self, tag: &str, prop: &str) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::DestinationProperty {
                tag: tag.to_string(),
                prop: prop.to_string(),
            });
        }
        Ok(())
    }

    fn visit_unary_plus(&mut self, expr: &Expression) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::UnaryPlus(Box::new(expr.clone())));
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(expr)?;
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_unary_negate(&mut self, expr: &Expression) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::UnaryNegate(Box::new(expr.clone())));
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(expr)?;
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_unary_not(&mut self, expr: &Expression) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::UnaryNot(Box::new(expr.clone())));
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(expr)?;
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_unary_incr(&mut self, expr: &Expression) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::UnaryIncr(Box::new(expr.clone())));
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(expr)?;
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_unary_decr(&mut self, expr: &Expression) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::UnaryDecr(Box::new(expr.clone())));
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(expr)?;
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_is_null(&mut self, expr: &Expression) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::IsNull(Box::new(expr.clone())));
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(expr)?;
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_is_not_null(&mut self, expr: &Expression) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::IsNotNull(Box::new(expr.clone())));
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(expr)?;
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_is_empty(&mut self, expr: &Expression) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::IsEmpty(Box::new(expr.clone())));
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(expr)?;
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_is_not_empty(&mut self, expr: &Expression) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::IsNotEmpty(Box::new(expr.clone())));
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(expr)?;
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_type_casting(&mut self, expr: &Expression, target_type: &str) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::TypeCasting {
                expr: Box::new(expr.clone()),
                target_type: target_type.to_string(),
            });
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(expr)?;
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_list_comprehension(
        &mut self,
        generator: &Expression,
        condition: &Option<Box<Expression>>,
    ) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::ListComprehension {
                generator: Box::new(generator.clone()),
                condition: condition.as_ref().map(|e| Box::new(e.as_ref().clone())),
            });
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(generator)?;
        if let Some(cond) = condition {
            self.visit(cond)?;
        }
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_predicate(&mut self, list: &Expression, condition: &Expression) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::Predicate {
                list: Box::new(list.clone()),
                condition: Box::new(condition.clone()),
            });
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(list)?;
        self.visit(condition)?;
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_reduce(
        &mut self,
        list: &Expression,
        var: &str,
        initial: &Expression,
        expr: &Expression,
    ) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::Reduce {
                list: Box::new(list.clone()),
                var: var.to_string(),
                initial: Box::new(initial.clone()),
                expr: Box::new(expr.clone()),
            });
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(list)?;
        self.visit(initial)?;
        self.visit(expr)?;
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_path_build(&mut self, items: &[Expression]) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::PathBuild(items.to_vec()));
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        for item in items {
            self.visit(item)?;
        }
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_es_query(&mut self, query: &str) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::ESQuery(query.to_string()));
        }
        Ok(())
    }

    fn visit_uuid(&mut self) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::UUID);
        }
        Ok(())
    }

    fn visit_subscript_range(
        &mut self,
        collection: &Expression,
        start: &Option<Box<Expression>>,
        end: &Option<Box<Expression>>,
    ) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::SubscriptRange {
                collection: Box::new(collection.clone()),
                start: start.as_ref().map(|e| Box::new(e.as_ref().clone())),
                end: end.as_ref().map(|e| Box::new(e.as_ref().clone())),
            });
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(collection)?;
        if let Some(start_expr) = start {
            self.visit(start_expr)?;
        }
        if let Some(end_expr) = end {
            self.visit(end_expr)?;
        }
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_match_path_pattern(&mut self, path_alias: &str, patterns: &[Expression]) -> Self::Result {
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::MatchPathPattern {
                path_alias: path_alias.to_string(),
                patterns: patterns.to_vec(),
            });
        }

        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        for pattern in patterns {
            self.visit(pattern)?;
        }
        self.is_top_level = old_top_level;
        Ok(())
    }
}

impl QueryVisitor for ExtractFilterExprVisitor {
    type QueryResult = Vec<Expression>;

    fn get_result(&self) -> Self::QueryResult {
        self.filter_exprs.clone()
    }

    fn reset(&mut self) {
        self.filter_exprs.clear();
        self.is_top_level = true;
    }

    fn is_success(&self) -> bool {
        true // ExtractFilterExprVisitor 总是成功，即使没有找到任何过滤表达式
    }
}

fn is_filter_expression(expr: &Expression) -> bool {
    // 检查表达式是否为过滤表达式
    // 通常关系表达式和函数调用是过滤表达式
    matches!(
        expr,
        Expression::Binary { .. } | Expression::Function { .. }
    )
}
