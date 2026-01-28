//! FindVisitor - 用于查找表达式中特定类型子表达式的访问器
//! 对应 NebulaGraph FindVisitor.h/.cpp 的功能

use crate::core::types::expression::visitor::{ExpressionVisitor, ExpressionVisitorState};
use crate::core::types::expression::DataType;
use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
use crate::core::Value;
use crate::core::Expression;

#[derive(Debug)]
pub struct FindVisitor {
    /// 找到的表达式列表
    found_exprs: Vec<Expression>,
    /// 访问者状态
    state: ExpressionVisitorState,
}

impl FindVisitor {
    pub fn new() -> Self {
        Self {
            found_exprs: Vec::new(),
            state: ExpressionVisitorState::new(),
        }
    }

    /// 搜索表达式中匹配类型的所有子表达式
    pub fn find(&mut self, expression: &Expression) -> Vec<Expression> {
        self.found_exprs.clear();
        let _ = self.visit_expression(expression);
        self.found_exprs.clone()
    }

    /// 检查表达式中是否存在匹配类型的子表达式
    pub fn exist(&mut self, expression: &Expression) -> bool {
        self.found_exprs.clear();
        let _ = self.visit_expression(expression);
        !self.found_exprs.is_empty()
    }

    /// 搜索表达式中匹配特定条件的子表达式
    pub fn find_if<F>(&mut self, expression: &Expression, predicate: F) -> Vec<Expression>
    where
        F: Fn(&Expression) -> bool,
    {
        let mut results = Vec::new();
        self.visit_with_predicate(expression, &predicate, &mut results);
        results
    }

    fn visit_with_predicate<F>(
        &self,
        expression: &Expression,
        predicate: &F,
        results: &mut Vec<Expression>,
    ) where
        F: Fn(&Expression) -> bool,
    {
        if predicate(expression) {
            results.push(expression.clone());
        }

        for child in expression.children() {
            self.visit_with_predicate(child, predicate, results);
        }
    }
}

impl ExpressionVisitor for FindVisitor {
    type Result = ();

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }

    fn visit_literal(&mut self, value: &Value) -> Self::Result {
        self.found_exprs.push(Expression::Literal(value.clone()));
    }

    fn visit_variable(&mut self, name: &str) -> Self::Result {
        self.found_exprs.push(Expression::Variable(name.to_string()));
    }

    fn visit_property(&mut self, object: &Expression, property: &str) -> Self::Result {
        self.found_exprs.push(Expression::Property {
            object: Box::new(object.clone()),
            property: property.to_string(),
        });
        self.visit_expression(object);
    }

    fn visit_binary(
        &mut self,
        left: &Expression,
        _op: &BinaryOperator,
        right: &Expression,
    ) -> Self::Result {
        self.found_exprs.push(Expression::Binary {
            left: Box::new(left.clone()),
            op: *_op,
            right: Box::new(right.clone()),
        });
        self.visit_expression(left);
        self.visit_expression(right);
    }

    fn visit_unary(
        &mut self,
        op: &UnaryOperator,
        operand: &Expression,
    ) -> Self::Result {
        self.found_exprs.push(Expression::Unary {
            op: *op,
            operand: Box::new(operand.clone()),
        });
        self.visit_expression(operand);
    }

    fn visit_function(&mut self, name: &str, args: &[Expression]) -> Self::Result {
        self.found_exprs.push(Expression::Function {
            name: name.to_string(),
            args: args.to_vec(),
        });
        for arg in args {
            self.visit_expression(arg);
        }
    }

    fn visit_aggregate(
        &mut self,
        func: &AggregateFunction,
        arg: &Expression,
        distinct: bool,
    ) -> Self::Result {
        self.found_exprs.push(Expression::Aggregate {
            func: func.clone(),
            arg: Box::new(arg.clone()),
            distinct,
        });
        self.visit_expression(arg);
    }

    fn visit_list(&mut self, items: &[Expression]) -> Self::Result {
        self.found_exprs.push(Expression::List(items.to_vec()));
        for item in items {
            self.visit_expression(item);
        }
    }

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
        self.found_exprs.push(Expression::Map(pairs.to_vec()));
        for (_, value) in pairs {
            self.visit_expression(value);
        }
    }

    fn visit_case(
        &mut self,
        conditions: &[(Expression, Expression)],
        default: Option<&Expression>,
    ) -> Self::Result {
        self.found_exprs.push(Expression::Case {
            conditions: conditions.to_vec(),
            default: default.map(|e| Box::new(e.clone())),
        });
        for (cond, expression) in conditions {
            self.visit_expression(cond);
            self.visit_expression(expression);
        }
        if let Some(default_expression) = default {
            self.visit_expression(default_expression);
        }
    }

    fn visit_type_cast(&mut self, expression: &Expression, target_type: &DataType) -> Self::Result {
        self.found_exprs.push(Expression::TypeCast {
            expression: Box::new(expression.clone()),
            target_type: target_type.clone(),
        });
        self.visit_expression(expression);
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result {
        self.found_exprs.push(Expression::Subscript {
            collection: Box::new(collection.clone()),
            index: Box::new(index.clone()),
        });
        self.visit_expression(collection);
        self.visit_expression(index);
    }

    fn visit_range(
        &mut self,
        collection: &Expression,
        start: Option<&Expression>,
        end: Option<&Expression>,
    ) -> Self::Result {
        self.found_exprs.push(Expression::Range {
            collection: Box::new(collection.clone()),
            start: start.map(|e| Box::new(e.clone())),
            end: end.map(|e| Box::new(e.clone())),
        });
        self.visit_expression(collection);
        if let Some(start_expression) = start {
            self.visit_expression(start_expression);
        }
        if let Some(end_expression) = end {
            self.visit_expression(end_expression);
        }
    }

    fn visit_path(&mut self, items: &[Expression]) -> Self::Result {
        self.found_exprs.push(Expression::Path(items.to_vec()));
        for item in items {
            self.visit_expression(item);
        }
    }

    fn visit_label(&mut self, name: &str) -> Self::Result {
        self.found_exprs.push(Expression::Label(name.to_string()));
    }

    fn visit_list_comprehension(
        &mut self,
        variable: &str,
        source: &Expression,
        filter: Option<&Expression>,
        map: Option<&Expression>,
    ) -> Self::Result {
        self.found_exprs.push(Expression::ListComprehension {
            variable: variable.to_string(),
            source: Box::new(source.clone()),
            filter: filter.map(|e| Box::new(e.clone())),
            map: map.map(|e| Box::new(e.clone())),
        });
        self.visit_expression(source);
        if let Some(f) = filter {
            self.visit_expression(f);
        }
        if let Some(m) = map {
            self.visit_expression(m);
        }
    }

    fn visit_label_tag_property(&mut self, tag: &Expression, property: &str) -> Self::Result {
        self.found_exprs.push(Expression::LabelTagProperty {
            tag: Box::new(tag.clone()),
            property: property.to_string(),
        });
        self.visit_expression(tag);
    }

    fn visit_tag_property(&mut self, tag_name: &str, property: &str) -> Self::Result {
        self.found_exprs.push(Expression::TagProperty {
            tag_name: tag_name.to_string(),
            property: property.to_string(),
        });
    }

    fn visit_edge_property(&mut self, edge_name: &str, property: &str) -> Self::Result {
        self.found_exprs.push(Expression::EdgeProperty {
            edge_name: edge_name.to_string(),
            property: property.to_string(),
        });
    }

    fn visit_predicate(&mut self, func: &str, args: &[Expression]) -> Self::Result {
        self.found_exprs.push(Expression::Predicate {
            func: func.to_string(),
            args: args.to_vec(),
        });
        for arg in args {
            self.visit_expression(arg);
        }
    }

    fn visit_reduce(
        &mut self,
        accumulator: &str,
        initial: &Expression,
        variable: &str,
        source: &Expression,
        mapping: &Expression,
    ) -> Self::Result {
        self.found_exprs.push(Expression::Reduce {
            accumulator: accumulator.to_string(),
            initial: Box::new(initial.clone()),
            variable: variable.to_string(),
            source: Box::new(source.clone()),
            mapping: Box::new(mapping.clone()),
        });
        self.visit_expression(initial);
        self.visit_expression(source);
        self.visit_expression(mapping);
    }

    fn visit_path_build(&mut self, exprs: &[Expression]) -> Self::Result {
        self.found_exprs.push(Expression::PathBuild(exprs.to_vec()));
        for expr in exprs {
            self.visit_expression(expr);
        }
    }
}
