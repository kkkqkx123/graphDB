//! EvaluableExprVisitor - 用于判断表达式是否可求值的访问器
//! 对应 NebulaGraph EvaluableExprVisitor.h/.cpp 的功能

use crate::core::{
    AggregateFunction, BinaryOperator, DataType, Expression, UnaryOperator,
};
use crate::core::Value;
use crate::core::visitor::{Visitor, VisitorState};

#[derive(Debug)]
pub struct EvaluableExprVisitor {
    /// 表达式是否可求值
    evaluable: bool,
    /// 错误信息
    error: Option<String>,
    /// 访问者状态
    state: VisitorState,
}

impl EvaluableExprVisitor {
    pub fn new() -> Self {
        Self {
            evaluable: true,
            error: None,
            state: VisitorState::new(),
        }
    }

    pub fn is_evaluable(&mut self, expr: &Expression) -> bool {
        self.evaluable = true;
        self.error = None;

        if let Err(e) = self.visit(expr) {
            self.evaluable = false;
            self.error = Some(e);
        }

        self.evaluable
    }

    pub fn get_error(&self) -> Option<&String> {
        self.error.as_ref()
    }
}

impl EvaluableExprVisitor {
    fn visit_literal(&mut self, _value: &Value) -> Result<(), String> {
        Ok(())
    }

    fn visit_variable(&mut self, _name: &str) -> Result<(), String> {
        self.evaluable = false;
        Ok(())
    }

    fn visit_property(&mut self, _object: &Expression, _property: &str) -> Result<(), String> {
        self.evaluable = false;
        Ok(())
    }

    fn visit_binary(
        &mut self,
        left: &Expression,
        _op: &BinaryOperator,
        right: &Expression,
    ) -> Result<(), String> {
        self.visit(left)?;
        self.visit(right)?;
        Ok(())
    }

    fn visit_unary(&mut self, _op: &UnaryOperator, operand: &Expression) -> Result<(), String> {
        self.visit(operand)?;
        Ok(())
    }

    fn visit_function(&mut self, _name: &str, args: &[Expression]) -> Result<(), String> {
        for arg in args {
            self.visit(arg)?;
        }
        Ok(())
    }

    fn visit_aggregate(
        &mut self,
        _func: &AggregateFunction,
        arg: &Expression,
        _distinct: bool,
    ) -> Result<(), String> {
        self.visit(arg)?;
        Ok(())
    }

    fn visit_list(&mut self, items: &[Expression]) -> Result<(), String> {
        for item in items {
            self.visit(item)?;
        }
        Ok(())
    }

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Result<(), String> {
        for (_, value) in pairs {
            self.visit(value)?;
        }
        Ok(())
    }

    fn visit_case(
        &mut self,
        conditions: &[(Expression, Expression)],
        default: &Option<Box<Expression>>,
    ) -> Result<(), String> {
        for (condition, value) in conditions {
            self.visit(condition)?;
            self.visit(value)?;
        }
        if let Some(default_expr) = default {
            self.visit(default_expr)?;
        }
        Ok(())
    }

    fn visit_type_cast(&mut self, expr: &Expression, _target_type: &DataType) -> Result<(), String> {
        self.visit(expr)?;
        Ok(())
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Result<(), String> {
        self.visit(collection)?;
        self.visit(index)?;
        Ok(())
    }

    fn visit_range(
        &mut self,
        collection: &Expression,
        start: &Option<Box<Expression>>,
        end: &Option<Box<Expression>>,
    ) -> Result<(), String> {
        self.visit(collection)?;
        if let Some(start_expr) = start {
            self.visit(start_expr)?;
        }
        if let Some(end_expr) = end {
            self.visit(end_expr)?;
        }
        Ok(())
    }

    fn visit_path(&mut self, items: &[Expression]) -> Result<(), String> {
        for item in items {
            self.visit(item)?;
        }
        Ok(())
    }

    fn visit_label(&mut self, _name: &str) -> Result<(), String> {
        Ok(())
    }

    fn visit_tag_property(&mut self, _tag: &str, _prop: &str) -> Result<(), String> {
        self.evaluable = false;
        Ok(())
    }

    fn visit_edge_property(&mut self, _edge: &str, _prop: &str) -> Result<(), String> {
        self.evaluable = false;
        Ok(())
    }

    fn visit_input_property(&mut self, _prop: &str) -> Result<(), String> {
        self.evaluable = false;
        Ok(())
    }

    fn visit_variable_property(&mut self, _var: &str, _prop: &str) -> Result<(), String> {
        self.evaluable = false;
        Ok(())
    }

    fn visit_source_property(&mut self, _tag: &str, _prop: &str) -> Result<(), String> {
        self.evaluable = false;
        Ok(())
    }

    fn visit_destination_property(&mut self, _tag: &str, _prop: &str) -> Result<(), String> {
        self.evaluable = false;
        Ok(())
    }

    fn visit_unary_plus(&mut self, expr: &Expression) -> Result<(), String> {
        self.visit(expr)?;
        Ok(())
    }

    fn visit_unary_negate(&mut self, expr: &Expression) -> Result<(), String> {
        self.visit(expr)?;
        Ok(())
    }

    fn visit_unary_not(&mut self, expr: &Expression) -> Result<(), String> {
        self.visit(expr)?;
        Ok(())
    }

    fn visit_unary_incr(&mut self, expr: &Expression) -> Result<(), String> {
        self.visit(expr)?;
        Ok(())
    }

    fn visit_unary_decr(&mut self, expr: &Expression) -> Result<(), String> {
        self.visit(expr)?;
        Ok(())
    }

    fn visit_is_null(&mut self, expr: &Expression) -> Result<(), String> {
        self.visit(expr)?;
        Ok(())
    }

    fn visit_is_not_null(&mut self, expr: &Expression) -> Result<(), String> {
        self.visit(expr)?;
        Ok(())
    }

    fn visit_is_empty(&mut self, expr: &Expression) -> Result<(), String> {
        self.visit(expr)?;
        Ok(())
    }

    fn visit_is_not_empty(&mut self, expr: &Expression) -> Result<(), String> {
        self.visit(expr)?;
        Ok(())
    }

    fn visit_type_casting(&mut self, expr: &Expression, _target_type: &str) -> Result<(), String> {
        self.visit(expr)?;
        Ok(())
    }

    fn visit_list_comprehension(
        &mut self,
        generator: &Expression,
        condition: &Option<Box<Expression>>,
    ) -> Result<(), String> {
        self.visit(generator)?;
        if let Some(cond) = condition {
            self.visit(cond)?;
        }
        Ok(())
    }

    fn visit_predicate(&mut self, list: &Expression, condition: &Expression) -> Result<(), String> {
        self.visit(list)?;
        self.visit(condition)?;
        Ok(())
    }

    fn visit_reduce(
        &mut self,
        list: &Expression,
        _var: &str,
        initial: &Expression,
        expr: &Expression,
    ) -> Result<(), String> {
        self.visit(list)?;
        self.visit(initial)?;
        self.visit(expr)?;
        Ok(())
    }

    fn visit_path_build(&mut self, items: &[Expression]) -> Result<(), String> {
        for item in items {
            self.visit(item)?;
        }
        Ok(())
    }

    fn visit_es_query(&mut self, _query: &str) -> Result<(), String> {
        self.evaluable = false;
        Ok(())
    }

    fn visit_uuid(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn visit_subscript_range(
        &mut self,
        collection: &Expression,
        start: &Option<Box<Expression>>,
        end: &Option<Box<Expression>>,
    ) -> Result<(), String> {
        self.visit(collection)?;
        if let Some(start_expr) = start {
            self.visit(start_expr)?;
        }
        if let Some(end_expr) = end {
            self.visit(end_expr)?;
        }
        Ok(())
    }

    fn visit_match_path_pattern(&mut self, _path_alias: &str, patterns: &[Expression]) -> Result<(), String> {
        for pattern in patterns {
            self.visit(pattern)?;
        }
        Ok(())
    }
}



impl<'a> Visitor<Expression> for EvaluableExprVisitor {
    type Result = Result<(), String>;

    fn visit(&mut self, target: &Expression) -> <Self as Visitor<Expression>>::Result {
        // 避免递归调用，直接调用内部方法
        match target {
            Expression::Literal(value) => self.visit_literal(value),
            Expression::Variable(name) => self.visit_variable(name),
            Expression::Property {
                object,
                property,
            } => self.visit_property(object, property),
            Expression::Binary { left, op, right } => self.visit_binary(left, op, right),
            Expression::Unary { op, operand } => self.visit_unary(op, operand),
            Expression::Function { name, args } => self.visit_function(name, args),
            Expression::Aggregate { func, arg, distinct } => {
                self.visit_aggregate(func, arg, *distinct)
            }
            Expression::List(items) => self.visit_list(items),
            Expression::Map(pairs) => self.visit_map(pairs),
            Expression::Case { conditions, default } => self.visit_case(conditions, default),
            Expression::TypeCast { expr, target_type } => self.visit_type_cast(expr, target_type),
            Expression::Subscript { collection, index } => self.visit_subscript(collection, index),
            Expression::Range { collection, start, end } => self.visit_range(collection, start, end),
            Expression::Path(items) => self.visit_path(items),
            Expression::Label(name) => self.visit_label(name),
            Expression::TagProperty { tag, prop } => self.visit_tag_property(tag, prop),
            Expression::EdgeProperty { edge, prop } => self.visit_edge_property(edge, prop),
            Expression::InputProperty(prop) => self.visit_input_property(prop),
            Expression::VariableProperty { var, prop } => self.visit_variable_property(var, prop),
            Expression::SourceProperty { tag, prop } => self.visit_source_property(tag, prop),
            Expression::DestinationProperty { tag, prop } => {
                self.visit_destination_property(tag, prop)
            }
            Expression::UnaryPlus(expr) => self.visit_unary_plus(expr),
            Expression::UnaryNegate(expr) => self.visit_unary_negate(expr),
            Expression::UnaryNot(expr) => self.visit_unary_not(expr),
            Expression::UnaryIncr(expr) => self.visit_unary_incr(expr),
            Expression::UnaryDecr(expr) => self.visit_unary_decr(expr),
            Expression::IsNull(expr) => self.visit_is_null(expr),
            Expression::IsNotNull(expr) => self.visit_is_not_null(expr),
            Expression::IsEmpty(expr) => self.visit_is_empty(expr),
            Expression::IsNotEmpty(expr) => self.visit_is_not_empty(expr),
            Expression::TypeCasting { expr, target_type } => {
                self.visit_type_casting(expr, target_type)
            }
            Expression::ListComprehension { generator, condition } => {
                self.visit_list_comprehension(generator, condition)
            }
            Expression::Predicate { list, condition } => self.visit_predicate(list, condition),
            Expression::Reduce { list, var, initial, expr } => {
                self.visit_reduce(list, var, initial, expr)
            }
            Expression::PathBuild(items) => self.visit_path_build(items),
            Expression::ESQuery(query) => self.visit_es_query(query),
            Expression::UUID => self.visit_uuid(),
            Expression::SubscriptRange { collection, start, end } => {
                self.visit_subscript_range(collection, start, end)
            }
            Expression::MatchPathPattern { path_alias, patterns } => {
                self.visit_match_path_pattern(path_alias, patterns)
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
