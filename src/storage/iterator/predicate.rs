//! Predicate pushdown – Supports the pushing of filtering conditions down to the storage layer.
//!
//! Provide predicate expressions and push-down optimization techniques:
//! Predicate: Verb phrase or noun phrase that describes the characteristics or properties of something
//! Type of expression
//! SimplePredicate: Implementation of a simple predicate
//! PredicateOptimizer: A predicate optimizer
//!
//! Implement static distribution using PredicateEnum to avoid the overhead associated with the dynamic distribution of Box<dyn Predicate>.

use crate::core::Value;

/// Comparison operator
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

/// Logical operators
#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOp {
    And,
    Or,
    Not,
}

/// Binary expression
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

/// Monomial expression
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// Expression type
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Literal values
    Literal(Value),
    /// List of references
    Column(String),
    /// Parameter reference
    Parameter(usize),
    /// Binary operation
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    /// Unary operation
    Unary { op: UnaryOp, expr: Box<Expression> },
    /// Function call
    Function { name: String, args: Vec<Expression> },
    /// Aggregate functions
    Aggregate {
        func: String,
        expr: Box<Expression>,
        distinct: bool,
    },
    /// Conditional expressions
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

/// Predicate enumeration: Using static distribution in place of `Box<dyn Predicate>`
#[derive(Debug, Clone, PartialEq)]
pub enum PredicateEnum {
    /// simple predicate
    Simple(SimplePredicate),
    /// Composite predicate
    Compound(CompoundPredicate),
}

impl PredicateEnum {
    /// Evaluating predicates
    pub fn evaluate(&self, row: &[Value]) -> bool {
        match self {
            PredicateEnum::Simple(p) => p.evaluate(row),
            PredicateEnum::Compound(p) => p.evaluate(row),
        }
    }

    /// Translate the following text into an expression:
    pub fn to_expression(&self) -> Expression {
        match self {
            PredicateEnum::Simple(p) => p.to_expression(),
            PredicateEnum::Compound(p) => p.to_expression(),
        }
    }

    /// Is it possible to push it down?
    pub fn can_pushdown(&self) -> bool {
        match self {
            PredicateEnum::Simple(p) => p.can_pushdown(),
            PredicateEnum::Compound(p) => p.can_pushdown(),
        }
    }

    /// Obtain the cost of push notifications
    pub fn pushdown_cost(&self) -> f64 {
        match self {
            PredicateEnum::Simple(p) => p.pushdown_cost(),
            PredicateEnum::Compound(p) => p.pushdown_cost(),
        }
    }

    /// Create a new simple predicate.
    pub fn simple(column: &str, op: CompareOp, value: Value) -> Self {
        PredicateEnum::Simple(SimplePredicate::new(column, op, value))
    }

    /// Creating an AND-combined predicate
    pub fn and(predicates: Vec<PredicateEnum>) -> Self {
        PredicateEnum::Compound(CompoundPredicate::and(predicates))
    }

    /// Creating an OR-combined predicate
    pub fn or(predicates: Vec<PredicateEnum>) -> Self {
        PredicateEnum::Compound(CompoundPredicate::or(predicates))
    }

    /// Obtaining references for simple predicates
    pub fn as_simple(&self) -> Option<&SimplePredicate> {
        match self {
            PredicateEnum::Simple(p) => Some(p),
            _ => None,
        }
    }

    /// Obtaining references for composite predicates
    pub fn as_compound(&self) -> Option<&CompoundPredicate> {
        match self {
            PredicateEnum::Compound(p) => Some(p),
            _ => None,
        }
    }
}

/// Simple predicates – Filtering based on a single condition
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

    pub fn column(&self) -> &str {
        &self.column
    }

    pub fn op(&self) -> &CompareOp {
        &self.op
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn evaluate(&self, row: &[Value]) -> bool {
        if let Ok(idx) = self.column.parse::<usize>() {
            if idx < row.len() {
                return self.evaluate_value(&row[idx]);
            }
            return false;
        }

        for val in row.iter() {
            if self.evaluate_value(val) {
                return true;
            }
        }
        false
    }

    pub fn to_expression(&self) -> Expression {
        Expression::Function {
            name: format!("{:?}", self.op),
            args: vec![
                Expression::Column(self.column.clone()),
                Expression::Literal(self.value.clone()),
            ],
        }
    }

    pub fn can_pushdown(&self) -> bool {
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

    pub fn pushdown_cost(&self) -> f64 {
        1.0
    }

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

/// Composite predicates – The logical combination of multiple predicates
#[derive(Debug, Clone, PartialEq)]
pub struct CompoundPredicate {
    op: LogicalOp,
    predicates: Vec<PredicateEnum>,
}

impl CompoundPredicate {
    pub fn new(op: LogicalOp, predicates: Vec<PredicateEnum>) -> Self {
        Self { op, predicates }
    }

    pub fn and(predicates: Vec<PredicateEnum>) -> Self {
        Self::new(LogicalOp::And, predicates)
    }

    pub fn or(predicates: Vec<PredicateEnum>) -> Self {
        Self::new(LogicalOp::Or, predicates)
    }

    pub fn op(&self) -> &LogicalOp {
        &self.op
    }

    pub fn predicates(&self) -> &[PredicateEnum] {
        &self.predicates
    }

    pub fn evaluate(&self, row: &[Value]) -> bool {
        match self.op {
            LogicalOp::And => self.predicates.iter().all(|p| p.evaluate(row)),
            LogicalOp::Or => self.predicates.iter().any(|p| p.evaluate(row)),
            LogicalOp::Not => !self.predicates.iter().all(|p| p.evaluate(row)),
        }
    }

    pub fn to_expression(&self) -> Expression {
        let exprs: Vec<Expression> = self.predicates.iter().map(|p| p.to_expression()).collect();

        Expression::Function {
            name: format!("{:?}", self.op),
            args: exprs,
        }
    }

    pub fn can_pushdown(&self) -> bool {
        match self.op {
            LogicalOp::And => self.predicates.iter().all(|p| p.can_pushdown()),
            LogicalOp::Or => self.predicates.iter().any(|p| p.can_pushdown()),
            LogicalOp::Not => false,
        }
    }

    pub fn pushdown_cost(&self) -> f64 {
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
}

/// Predicate Pushdown Optimizer
#[derive(Debug, Default)]
pub struct PredicateOptimizer {
    pushdown_candidates: Vec<PredicateEnum>,
    filter_candidates: Vec<PredicateEnum>,
}

impl PredicateOptimizer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn analyze(&mut self, predicate: &PredicateEnum) {
        if predicate.can_pushdown() {
            self.pushdown_candidates.push(predicate.clone());
        } else {
            self.filter_candidates.push(predicate.clone());
        }
    }

    pub fn get_pushdown_predicates(&self) -> &[PredicateEnum] {
        &self.pushdown_candidates
    }

    pub fn get_filter_predicates(&self) -> &[PredicateEnum] {
        &self.filter_candidates
    }

    pub fn optimize(&self, predicate: &PredicateEnum) -> (Vec<PredicateEnum>, Vec<PredicateEnum>) {
        let mut pushdown = Vec::new();
        let mut filter = Vec::new();
        self.classify_predicate(predicate.clone(), &mut pushdown, &mut filter);
        (pushdown, filter)
    }

    fn classify_predicate(
        &self,
        predicate: PredicateEnum,
        pushdown: &mut Vec<PredicateEnum>,
        filter: &mut Vec<PredicateEnum>,
    ) {
        if let Some(compound) = predicate.as_compound() {
            for p in &compound.predicates {
                self.classify_predicate(p.clone(), pushdown, filter);
            }
        } else if predicate.can_pushdown() {
            pushdown.push(predicate);
        } else {
            filter.push(predicate);
        }
    }
}

/// Predicate evaluation result
#[derive(Debug, Clone)]
pub struct PushdownResult {
    pub pushed_predicates: Vec<PredicateEnum>,
    pub remaining_predicates: Vec<PredicateEnum>,
    pub estimated_cost_reduction: f64,
}

impl PushdownResult {
    pub fn new(pushed: Vec<PredicateEnum>, remaining: Vec<PredicateEnum>, reduction: f64) -> Self {
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
        let pred = SimplePredicate::new("age", CompareOp::Equal, Value::Int(25));

        let row = vec![Value::String("Alice".to_string()), Value::Int(25)];
        assert!(pred.evaluate(&row));

        let row2 = vec![Value::String("Bob".to_string()), Value::Int(30)];
        assert!(!pred.evaluate(&row2));
    }

    #[test]
    fn test_simple_predicate_greater() {
        let pred = SimplePredicate::new("age", CompareOp::Greater, Value::Int(30));

        let row = vec![Value::String("Alice".to_string()), Value::Int(35)];
        assert!(pred.evaluate(&row));

        let row2 = vec![Value::String("Bob".to_string()), Value::Int(25)];
        assert!(!pred.evaluate(&row2));
    }

    #[test]
    fn test_predicate_enum_simple() {
        let pred = PredicateEnum::simple("age", CompareOp::Equal, Value::Int(25));

        let row = vec![Value::String("Alice".to_string()), Value::Int(25)];
        assert!(pred.evaluate(&row));
        assert!(pred.can_pushdown());
    }

    #[test]
    fn test_predicate_enum_and() {
        let pred1 = PredicateEnum::simple("age", CompareOp::Greater, Value::Int(20));
        let pred2 = PredicateEnum::simple("age", CompareOp::Less, Value::Int(40));
        let compound = PredicateEnum::and(vec![pred1, pred2]);

        let row = vec![Value::String("Alice".to_string()), Value::Int(30)];
        assert!(compound.evaluate(&row));

        let row2 = vec![Value::String("Bob".to_string()), Value::Int(50)];
        assert!(!compound.evaluate(&row2));
    }

    #[test]
    fn test_predicate_enum_or() {
        let pred1 =
            PredicateEnum::simple("name", CompareOp::Equal, Value::String("Alice".to_string()));
        let pred2 =
            PredicateEnum::simple("name", CompareOp::Equal, Value::String("Bob".to_string()));
        let compound = PredicateEnum::or(vec![pred1, pred2]);

        let row = vec![Value::String("Alice".to_string()), Value::Int(25)];
        assert!(compound.evaluate(&row));

        let row2 = vec![Value::String("Bob".to_string()), Value::Int(30)];
        assert!(compound.evaluate(&row2));

        let row3 = vec![Value::String("Charlie".to_string()), Value::Int(35)];
        assert!(!compound.evaluate(&row3));
    }

    #[test]
    fn test_predicate_can_pushdown() {
        let pred = PredicateEnum::simple("age", CompareOp::Equal, Value::Int(25));
        assert!(pred.can_pushdown());

        let not_pred =
            PredicateEnum::simple("age", CompareOp::Like, Value::String("%".to_string()));
        assert!(!not_pred.can_pushdown());
    }

    #[test]
    fn test_optimizer() {
        let pred1 = PredicateEnum::simple("age", CompareOp::Equal, Value::Int(25));
        let optimizer = PredicateOptimizer::new();

        let (pushdown, filter) = optimizer.optimize(&pred1);
        assert_eq!(pushdown.len(), 1);
        assert_eq!(filter.len(), 0);
    }

    #[test]
    fn test_compound_predicate_and() {
        let pred1 = SimplePredicate::new("age", CompareOp::Greater, Value::Int(20));
        let pred2 = SimplePredicate::new("age", CompareOp::Less, Value::Int(40));
        let compound = CompoundPredicate::and(vec![
            PredicateEnum::Simple(pred1),
            PredicateEnum::Simple(pred2),
        ]);

        let row = vec![Value::String("Alice".to_string()), Value::Int(30)];
        assert!(compound.evaluate(&row));

        let row2 = vec![Value::String("Bob".to_string()), Value::Int(50)];
        assert!(!compound.evaluate(&row2));
    }

    #[test]
    fn test_compound_predicate_or() {
        let pred1 =
            SimplePredicate::new("name", CompareOp::Equal, Value::String("Alice".to_string()));
        let pred2 =
            SimplePredicate::new("name", CompareOp::Equal, Value::String("Bob".to_string()));
        let compound = CompoundPredicate::or(vec![
            PredicateEnum::Simple(pred1),
            PredicateEnum::Simple(pred2),
        ]);

        let row = vec![Value::String("Alice".to_string()), Value::Int(25)];
        assert!(compound.evaluate(&row));

        let row2 = vec![Value::String("Bob".to_string()), Value::Int(30)];
        assert!(compound.evaluate(&row2));

        let row3 = vec![Value::String("Charlie".to_string()), Value::Int(35)];
        assert!(!compound.evaluate(&row3));
    }
}
