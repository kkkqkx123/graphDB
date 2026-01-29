//! 谓词下推 - 支持将过滤条件下推到存储层
//!
//! 提供谓词表达式和下推优化：
//! - Predicate: 谓词 trait
//! - Expression: 表达式类型
//! - SimplePredicate: 简单谓词实现
//! - PredicateOptimizer: 谓词优化器

use crate::core::Value;
use std::any::Any;
use std::collections::HashMap;
use std::fmt;

/// 比较操作符
#[derive(Debug, Clone, PartialEq)]
pub enum CompareOp {
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    In,
    NotIn,
    Like,
    IsNull,
    IsNotNull,
}

/// 逻辑操作符
#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOp {
    And,
    Or,
    Not,
}

/// 二元表达式
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

/// 一元表达式
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// 表达式类型
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// 字面量值
    Literal(Value),
    /// 列引用
    Column(String),
    /// 参数引用
    Parameter(usize),
    /// 二元运算
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    /// 一元运算
    Unary {
        op: UnaryOp,
        expr: Box<Expression>,
    },
    /// 函数调用
    Function {
        name: String,
        args: Vec<Expression>,
    },
    /// 聚合函数
    Aggregate {
        func: String,
        expr: Box<Expression>,
        distinct: bool,
    },
    /// 条件表达式
    Case {
        cond: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Option<Box<Expression>>,
    },
}

impl Expression {
    pub fn column(name: &str) -> Self {
        Expression::Column(name.to_string())
    }

    pub fn literal(value: Value) -> Self {
        Expression::Literal(value)
    }

    pub fn eq(left: Expression, right: Expression) -> Self {
        Expression::Function {
            name: "Equal".to_string(),
            args: vec![left, right],
        }
    }

    pub fn and(left: Expression, right: Expression) -> Self {
        Expression::Function {
            name: "And".to_string(),
            args: vec![left, right],
        }
    }

    pub fn or(left: Expression, right: Expression) -> Self {
        Expression::Function {
            name: "Or".to_string(),
            args: vec![left, right],
        }
    }
}

/// 谓词 trait
pub trait Predicate: Send + Sync + fmt::Debug {
    fn evaluate(&self, row: &[Value]) -> bool;
    fn to_expression(&self) -> Expression;
    fn can_pushdown(&self) -> bool;
    fn pushdown_cost(&self) -> f64;
    fn as_predicate(&self) -> &dyn Predicate;
    fn box_clone(&self) -> Box<dyn Predicate>;
}

/// 简单谓词 - 基于单个条件的过滤
#[derive(Debug, Clone, PartialEq)]
pub struct SimplePredicate {
    column: String,
    op: CompareOp,
    value: Value,
}

impl SimplePredicate {
    pub fn new(column: &str, op: CompareOp, value: Value) -> Self {
        Self {
            column: column.to_string(),
            op,
            value,
        }
    }
}

impl Predicate for SimplePredicate {
    fn evaluate(&self, row: &[Value]) -> bool {
        let parts: Vec<&str> = self.column.split('.').collect();
        let col_name = parts.last().map(|s| *s).unwrap_or(&self.column);

        for (idx, val) in row.iter().enumerate() {
            let row_col_name = format!("col_{}", idx);
            if row_col_name == self.column || self.column == col_name {
                return self.evaluate_value(val);
            }
        }
        false
    }

    fn to_expression(&self) -> Expression {
        Expression::Function {
            name: format!("{:?}", self.op),
            args: vec![
                Expression::Column(self.column.clone()),
                Expression::Literal(self.value.clone()),
            ],
        }
    }

    fn can_pushdown(&self) -> bool {
        matches!(
            self.op,
            CompareOp::Equal
                | CompareOp::Greater
                | CompareOp::GreaterEqual
                | CompareOp::Less
                | CompareOp::LessEqual
                | CompareOp::In
        )
    }

    fn pushdown_cost(&self) -> f64 {
        1.0
    }

    fn as_predicate(&self) -> &dyn Predicate {
        self
    }

    fn box_clone(&self) -> Box<dyn Predicate> {
        Box::new(self.clone())
    }
}

impl SimplePredicate {
    fn evaluate_value(&self, val: &Value) -> bool {
        match (&self.op, val, &self.value) {
            (CompareOp::Equal, Value::Int(a), Value::Int(b)) => a == b,
            (CompareOp::Equal, Value::Float(a), Value::Float(b)) => (a - b).abs() < 1e-9,
            (CompareOp::Equal, Value::String(a), Value::String(b)) => a == b,
            (CompareOp::NotEqual, Value::Int(a), Value::Int(b)) => a != b,
            (CompareOp::NotEqual, Value::Float(a), Value::Float(b)) => (a - b).abs() >= 1e-9,
            (CompareOp::NotEqual, Value::String(a), Value::String(b)) => a != b,
            (CompareOp::Greater, Value::Int(a), Value::Int(b)) => a > b,
            (CompareOp::Greater, Value::Float(a), Value::Float(b)) => a > b,
            (CompareOp::GreaterEqual, Value::Int(a), Value::Int(b)) => a >= b,
            (CompareOp::GreaterEqual, Value::Float(a), Value::Float(b)) => a >= b,
            (CompareOp::Less, Value::Int(a), Value::Int(b)) => a < b,
            (CompareOp::Less, Value::Float(a), Value::Float(b)) => a < b,
            (CompareOp::LessEqual, Value::Int(a), Value::Int(b)) => a <= b,
            (CompareOp::LessEqual, Value::Float(a), Value::Float(b)) => a <= b,
            (CompareOp::IsNull, _, _) => matches!(val, Value::Empty),
            (CompareOp::IsNotNull, _, _) => !matches!(val, Value::Empty),
            _ => false,
        }
    }
}

/// 组合谓词 - 多个谓词的逻辑组合
#[derive(Debug)]
pub struct CompoundPredicate {
    op: LogicalOp,
    predicates: Vec<Box<dyn Predicate>>,
}

impl Clone for CompoundPredicate {
    fn clone(&self) -> Self {
        Self {
            op: self.op.clone(),
            predicates: self.predicates.iter().map(|p| p.box_clone()).collect(),
        }
    }
}

impl CompoundPredicate {
    pub fn new(op: LogicalOp, predicates: Vec<Box<dyn Predicate>>) -> Self {
        Self { op, predicates }
    }

    pub fn and(predicates: Vec<Box<dyn Predicate>>) -> Self {
        Self::new(LogicalOp::And, predicates)
    }

    pub fn or(predicates: Vec<Box<dyn Predicate>>) -> Self {
        Self::new(LogicalOp::Or, predicates)
    }
}

impl Predicate for CompoundPredicate {
    fn evaluate(&self, row: &[Value]) -> bool {
        match self.op {
            LogicalOp::And => self.predicates.iter().all(|p| p.evaluate(row)),
            LogicalOp::Or => self.predicates.iter().any(|p| p.evaluate(row)),
            LogicalOp::Not => !self.predicates.iter().all(|p| p.evaluate(row)),
        }
    }

    fn to_expression(&self) -> Expression {
        let exprs: Vec<Expression> = self
            .predicates
            .iter()
            .map(|p| p.to_expression())
            .collect();

        Expression::Function {
            name: format!("{:?}", self.op),
            args: exprs,
        }
    }

    fn can_pushdown(&self) -> bool {
        match self.op {
            LogicalOp::And => self.predicates.iter().all(|p| p.can_pushdown()),
            LogicalOp::Or => self.predicates.iter().any(|p| p.can_pushdown()),
            LogicalOp::Not => false,
        }
    }

    fn pushdown_cost(&self) -> f64 {
        match self.op {
            LogicalOp::And => self.predicates.iter().map(|p| p.pushdown_cost()).sum(),
            LogicalOp::Or => self
                .predicates
                .iter()
                .map(|p| p.pushdown_cost())
                .fold(0.0, |a, b| a.min(b)),
            LogicalOp::Not => 100.0,
        }
    }

    fn as_predicate(&self) -> &dyn Predicate {
        self
    }

    fn box_clone(&self) -> Box<dyn Predicate> {
        Box::new(self.clone())
    }
}

/// 谓词下推优化器
#[derive(Debug, Default)]
pub struct PredicateOptimizer {
    pushdown_candidates: Vec<Box<dyn Predicate>>,
    filter_candidates: Vec<Box<dyn Predicate>>,
}

impl PredicateOptimizer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn analyze(&mut self, predicate: &dyn Predicate) {
        if predicate.can_pushdown() {
            self.pushdown_candidates
                .push(predicate.box_clone());
        } else {
            self.filter_candidates
                .push(predicate.box_clone());
        }
    }

    pub fn get_pushdown_predicates(&self) -> &[Box<dyn Predicate>] {
        &self.pushdown_candidates
    }

    pub fn get_filter_predicates(&self) -> &[Box<dyn Predicate>] {
        &self.filter_candidates
    }

    pub fn optimize(&self, predicate: &dyn Predicate) -> (Vec<Box<dyn Predicate>>, Vec<Box<dyn Predicate>>) {
        let mut pushdown = Vec::new();
        let mut filter = Vec::new();
        let predicate = predicate.box_clone();
        self.classify_predicate(predicate, &mut pushdown, &mut filter);
        (pushdown, filter)
    }

    fn classify_predicate(
        &self,
        predicate: Box<dyn Predicate>,
        pushdown: &mut Vec<Box<dyn Predicate>>,
        filter: &mut Vec<Box<dyn Predicate>>,
    ) {
        if let Some(compound) = predicate.as_any().downcast_ref::<CompoundPredicate>() {
            for p in &compound.predicates {
                self.classify_predicate(p.box_clone(), pushdown, filter);
            }
        } else if predicate.can_pushdown() {
            pushdown.push(predicate);
        } else {
            filter.push(predicate);
        }
    }
}

trait AsAny {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: std::any::Any> AsAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 谓词下推结果
#[derive(Debug)]
pub struct PushdownResult {
    pub pushed_predicates: Vec<Box<dyn Predicate>>,
    pub remaining_predicates: Vec<Box<dyn Predicate>>,
    pub estimated_cost_reduction: f64,
}

impl Clone for PushdownResult {
    fn clone(&self) -> Self {
        Self {
            pushed_predicates: self.pushed_predicates.iter().map(|p| p.box_clone()).collect(),
            remaining_predicates: self.remaining_predicates.iter().map(|p| p.box_clone()).collect(),
            estimated_cost_reduction: self.estimated_cost_reduction,
        }
    }
}

impl PushdownResult {
    pub fn new(
        pushed: Vec<Box<dyn Predicate>>,
        remaining: Vec<Box<dyn Predicate>>,
        reduction: f64,
    ) -> Self {
        Self {
            pushed_predicates: pushed,
            remaining_predicates: remaining,
            estimated_cost_reduction: reduction,
        }
    }

    pub fn empty() -> Self {
        Self {
            pushed_predicates: Vec::new(),
            remaining_predicates: Vec::new(),
            estimated_cost_reduction: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_predicate_eq() {
        let pred = SimplePredicate::new(
            "age",
            CompareOp::Equal,
            Value::Int(25),
        );

        let row = vec![Value::String("Alice".to_string()), Value::Int(25)];
        assert!(pred.evaluate(&row));

        let row2 = vec![Value::String("Bob".to_string()), Value::Int(30)];
        assert!(!pred.evaluate(&row2));
    }

    #[test]
    fn test_simple_predicate_greater() {
        let pred = SimplePredicate::new(
            "age",
            CompareOp::Greater,
            Value::Int(30),
        );

        let row = vec![Value::String("Alice".to_string()), Value::Int(35)];
        assert!(pred.evaluate(&row));

        let row2 = vec![Value::String("Bob".to_string()), Value::Int(25)];
        assert!(!pred.evaluate(&row2));
    }

    #[test]
    fn test_compound_predicate_and() {
        let pred1 = SimplePredicate::new("age", CompareOp::Greater, Value::Int(20));
        let pred2 = SimplePredicate::new("age", CompareOp::Less, Value::Int(40));
        let compound = CompoundPredicate::and(vec![Box::new(pred1), Box::new(pred2)]);

        let row = vec![Value::String("Alice".to_string()), Value::Int(30)];
        assert!(compound.evaluate(&row));

        let row2 = vec![Value::String("Bob".to_string()), Value::Int(50)];
        assert!(!compound.evaluate(&row2));
    }

    #[test]
    fn test_compound_predicate_or() {
        let pred1 = SimplePredicate::new("name", CompareOp::Equal, Value::String("Alice".to_string()));
        let pred2 = SimplePredicate::new("name", CompareOp::Equal, Value::String("Bob".to_string()));
        let compound = CompoundPredicate::or(vec![Box::new(pred1), Box::new(pred2)]);

        let row = vec![Value::String("Alice".to_string()), Value::Int(25)];
        assert!(compound.evaluate(&row));

        let row2 = vec![Value::String("Bob".to_string()), Value::Int(30)];
        assert!(compound.evaluate(&row2));

        let row3 = vec![Value::String("Charlie".to_string()), Value::Int(35)];
        assert!(!compound.evaluate(&row3));
    }

    #[test]
    fn test_predicate_can_pushdown() {
        let pred = SimplePredicate::new("age", CompareOp::Equal, Value::Int(25));
        assert!(pred.can_pushdown());

        let not_pred = SimplePredicate::new("age", CompareOp::Like, Value::String("%".to_string()));
        assert!(!not_pred.can_pushdown());
    }

    #[test]
    fn test_optimizer() {
        let pred1 = SimplePredicate::new("age", CompareOp::Equal, Value::Int(25));
        let pred2 = SimplePredicate::new("name", CompareOp::Like, Value::String("%".to_string()));
        let optimizer = PredicateOptimizer::new();

        let (pushdown, filter) = optimizer.optimize(&pred1);
        assert_eq!(pushdown.len(), 1);
        assert_eq!(filter.len(), 0);
    }
}
