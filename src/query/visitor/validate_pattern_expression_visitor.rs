//! ValidatePatternExpressionVisitor - 用于验证模式表达式的访问器=
//!
//! 主要功能：
//! - 验证列表推导表达式的变量作用域
//! - 验证路径模式表达式的合法性
//! - 处理局部变量定义
//! - 检查变量冲突

use crate::core::{
    expression_visitor::{ExpressionVisitor, ExpressionVisitorState},
    BinaryOperator, DataType, Expression, UnaryOperator, Value,
};
use crate::core::types::operators::AggregateFunction;
use crate::query::parser::ast::expr::*;

/// 模式表达式验证访问器
///
/// 用于验证模式表达式的合法性，处理变量作用域
#[derive(Debug)]
pub struct ValidatePatternExpressionVisitor {
    /// 局部变量集合
    local_variables: Vec<String>,
    /// 错误状态
    error: Option<String>,
    /// 访问者状态
    state: ExpressionVisitorState,
}

impl ValidatePatternExpressionVisitor {
    /// 创建新的模式表达式验证访问器
    pub fn new() -> Self {
        Self {
            local_variables: Vec::new(),
            error: None,
            state: ExpressionVisitorState::new(),
        }
    }

    /// 验证表达式
    pub fn validate(&mut self, expr: &Expression) -> Result<(), String> {
        self.local_variables.clear();
        self.error = None;

        self.visit_expression(expr)?;

        if let Some(err) = &self.error {
            Err(err.clone())
        } else {
            Ok(())
        }
    }

    /// 获取错误信息
    pub fn get_error(&self) -> Option<&String> {
        self.error.as_ref()
    }

    /// 设置错误信息
    fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    /// 添加局部变量
    fn add_local_variable(&mut self, var: &str) {
        if !self.local_variables.contains(&var.to_string()) {
            self.local_variables.push(var.to_string());
        }
    }

    /// 移除局部变量
    fn remove_local_variable(&mut self, var: &str) {
        if let Some(pos) = self.local_variables.iter().position(|v| v == var) {
            self.local_variables.remove(pos);
        }
    }

    /// 检查变量是否为局部变量
    fn is_local_variable(&self, var: &str) -> bool {
        self.local_variables.contains(&var.to_string())
    }

    /// 将多个表达式用 AND 连接
    fn and_all(&self, exprs: &[Expression]) -> Option<Expression> {
        if exprs.is_empty() {
            return None;
        }

        if exprs.len() == 1 {
            return Some(exprs[0].clone());
        }

        let mut result = exprs[0].clone();
        for expr in &exprs[1..] {
            result = Expression::Binary {
                left: Box::new(result),
                op: BinaryOperator::And,
                right: Box::new(expr.clone()),
            };
        }

        Some(result)
    }
}

impl Default for ValidatePatternExpressionVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionVisitor for ValidatePatternExpressionVisitor {
    type Result = Result<(), String>;

    fn visit_literal(&mut self, _value: &Value) -> Self::Result {
        Ok(())
    }

    fn visit_variable(&mut self, _name: &str) -> Self::Result {
        Ok(())
    }

    fn visit_property(&mut self, object: &Expression, _property: &str) -> Self::Result {
        self.visit_expression(object)
    }

    fn visit_binary(
        &mut self,
        left: &Expression,
        _op: &BinaryOperator,
        right: &Expression,
    ) -> Self::Result {
        self.visit_expression(left)?;
        self.visit_expression(right)
    }

    fn visit_unary(&mut self, _op: &UnaryOperator, operand: &Expression) -> Self::Result {
        self.visit_expression(operand)
    }

    fn visit_function(&mut self, name: &str, args: &[Expression]) -> Self::Result {
        let name_upper = name.to_uppercase();

        match name_upper.as_str() {
            "HAS" | "HASLABEL" | "HASTAG" => {
                if args.len() >= 1 {
                    if let Expression::Variable(var) = &args[0] {
                        if self.is_local_variable(var) {
                            self.set_error(format!(
                                "函数 {} 的参数不能是局部变量: {}",
                                name, var
                            ));
                            return Err(self.error.clone().unwrap());
                        }
                    }
                }
            }
            _ => {}
        }

        for arg in args {
            self.visit_expression(arg)?;
        }
        Ok(())
    }

    fn visit_aggregate(
        &mut self,
        _func: &AggregateFunction,
        arg: &Expression,
        _distinct: bool,
    ) -> Self::Result {
        self.visit_expression(arg)
    }

    fn visit_list(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit_expression(item)?;
        }
        Ok(())
    }

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
        for (_, expr) in pairs {
            self.visit_expression(expr)?;
        }
        Ok(())
    }

    fn visit_case(
        &mut self,
        conditions: &[(Expression, Expression)],
        default: &Option<Box<Expression>>,
    ) -> Self::Result {
        for (cond, expr) in conditions {
            self.visit_expression(cond)?;
            self.visit_expression(expr)?;
        }
        if let Some(default_expr) = default {
            self.visit_expression(default_expr)?;
        }
        Ok(())
    }

    fn visit_type_cast(&mut self, expr: &Expression, _target_type: &DataType) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result {
        self.visit_expression(collection)?;
        self.visit_expression(index)
    }

    fn visit_range(
        &mut self,
        collection: &Expression,
        start: &Option<Box<Expression>>,
        end: &Option<Box<Expression>>,
    ) -> Self::Result {
        self.visit_expression(collection)?;
        if let Some(start_expr) = start {
            self.visit_expression(start_expr)?;
        }
        if let Some(end_expr) = end {
            self.visit_expression(end_expr)?;
        }
        Ok(())
    }

    fn visit_path(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit_expression(item)?;
        }
        Ok(())
    }

    fn visit_label(&mut self, _name: &str) -> Self::Result {
        Ok(())
    }

    fn visit_tag_property(&mut self, _tag: &str, _prop: &str) -> Self::Result {
        Ok(())
    }

    fn visit_edge_property(&mut self, _edge: &str, _prop: &str) -> Self::Result {
        Ok(())
    }

    fn visit_input_property(&mut self, _prop: &str) -> Self::Result {
        Ok(())
    }

    fn visit_variable_property(&mut self, _var: &str, _prop: &str) -> Self::Result {
        Ok(())
    }

    fn visit_source_property(&mut self, _tag: &str, _prop: &str) -> Self::Result {
        Ok(())
    }

    fn visit_destination_property(&mut self, _tag: &str, _prop: &str) -> Self::Result {
        Ok(())
    }

    fn visit_unary_plus(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_unary_negate(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_unary_not(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_unary_incr(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_unary_decr(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_is_null(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_is_not_null(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_is_empty(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_is_not_empty(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_list_comprehension(
        &mut self,
        generator: &Expression,
        condition: &Option<Box<Expression>>,
    ) -> Self::Result {
        if let Expression::Variable(var) = generator {
            self.add_local_variable(var);
        }

        self.visit_expression(generator)?;

        if let Some(cond) = condition {
            self.visit_expression(cond)?;
        }

        if let Expression::Variable(var) = generator {
            self.remove_local_variable(var);
        }

        Ok(())
    }

    fn visit_predicate(&mut self, list: &Expression, condition: &Expression) -> Self::Result {
        self.visit_expression(list)?;
        self.visit_expression(condition)
    }

    fn visit_reduce(
        &mut self,
        list: &Expression,
        var: &str,
        initial: &Expression,
        expr: &Expression,
    ) -> Self::Result {
        self.add_local_variable(var);

        self.visit_expression(list)?;
        self.visit_expression(initial)?;
        self.visit_expression(expr)?;

        self.remove_local_variable(var);
        Ok(())
    }

    fn visit_es_query(&mut self, _query: &str) -> Self::Result {
        Ok(())
    }

    fn visit_uuid(&mut self) -> Self::Result {
        Ok(())
    }

    fn visit_match_path_pattern(
        &mut self,
        _path_alias: &str,
        patterns: &[Expression],
    ) -> Self::Result {
        for pattern in patterns {
            self.visit_expression(pattern)?;
        }
        Ok(())
    }

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }

    fn visit_type_casting(&mut self, expr: &Expression, _target_type: &str) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_path_build(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit_expression(item)?;
        }
        Ok(())
    }

    fn visit_subscript_range(
        &mut self,
        collection: &Expression,
        start: &Option<Box<Expression>>,
        end: &Option<Box<Expression>>,
    ) -> Self::Result {
        self.visit_expression(collection)?;
        if let Some(start_expr) = start {
            self.visit_expression(start_expr)?;
        }
        if let Some(end_expr) = end {
            self.visit_expression(end_expr)?;
        }
        Ok(())
    }

    fn visit_constant_expr(&mut self, _expr: &ConstantExpr) -> Self::Result {
        Ok(())
    }

    fn visit_variable_expr(&mut self, _expr: &VariableExpr) -> Self::Result {
        Ok(())
    }

    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> Self::Result {
        self.visit_expr(expr.left.as_ref())?;
        self.visit_expr(expr.right.as_ref())?;
        Ok(())
    }

    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> Self::Result {
        self.visit_expr(expr.operand.as_ref())?;
        Ok(())
    }

    fn visit_function_call_expr(&mut self, expr: &FunctionCallExpr) -> Self::Result {
        for arg in &expr.args {
            self.visit_expr(arg)?;
        }
        Ok(())
    }

    fn visit_property_access_expr(&mut self, expr: &PropertyAccessExpr) -> Self::Result {
        self.visit_expr(expr.object.as_ref())?;
        Ok(())
    }

    fn visit_list_expr(&mut self, expr: &ListExpr) -> Self::Result {
        for element in &expr.elements {
            self.visit_expr(element)?;
        }
        Ok(())
    }

    fn visit_map_expr(&mut self, expr: &MapExpr) -> Self::Result {
        for (_key, value) in &expr.pairs {
            self.visit_expr(value)?;
        }
        Ok(())
    }

    fn visit_case_expr(&mut self, expr: &CaseExpr) -> Self::Result {
        for (when_expr, then_expr) in &expr.when_then_pairs {
            self.visit_expr(when_expr)?;
            self.visit_expr(then_expr)?;
        }
        if let Some(default_expr) = &expr.default {
            self.visit_expr(default_expr.as_ref())?;
        }
        Ok(())
    }

    fn visit_subscript_expr(&mut self, expr: &SubscriptExpr) -> Self::Result {
        self.visit_expr(expr.collection.as_ref())?;
        self.visit_expr(expr.index.as_ref())?;
        Ok(())
    }

    fn visit_predicate_expr(&mut self, expr: &PredicateExpr) -> Self::Result {
        self.visit_expr(expr.list.as_ref())?;
        self.visit_expr(expr.condition.as_ref())?;
        Ok(())
    }

    fn visit_tag_property_expr(&mut self, _expr: &TagPropertyExpr) -> Self::Result {
        Ok(())
    }

    fn visit_edge_property_expr(&mut self, _expr: &EdgePropertyExpr) -> Self::Result {
        Ok(())
    }

    fn visit_input_property_expr(&mut self, _expr: &InputPropertyExpr) -> Self::Result {
        Ok(())
    }

    fn visit_variable_property_expr(&mut self, _expr: &VariablePropertyExpr) -> Self::Result {
        Ok(())
    }

    fn visit_source_property_expr(&mut self, _expr: &SourcePropertyExpr) -> Self::Result {
        Ok(())
    }

    fn visit_destination_property_expr(&mut self, _expr: &DestinationPropertyExpr) -> Self::Result {
        Ok(())
    }

    fn visit_type_cast_expr(&mut self, expr: &TypeCastExpr) -> Self::Result {
        self.visit_expr(expr.expr.as_ref())
    }

    fn visit_range_expr(&mut self, expr: &RangeExpr) -> Self::Result {
        self.visit_expr(expr.collection.as_ref())?;
        if let Some(start_expr) = &expr.start {
            self.visit_expr(start_expr.as_ref())?;
        }
        if let Some(end_expr) = &expr.end {
            self.visit_expr(end_expr.as_ref())?;
        }
        Ok(())
    }

    fn visit_path_expr(&mut self, expr: &PathExpr) -> Self::Result {
        for element in &expr.elements {
            self.visit_expr(element)?;
        }
        Ok(())
    }

    fn visit_label_expr(&mut self, _expr: &LabelExpr) -> Self::Result {
        Ok(())
    }

    fn visit_reduce_expr(&mut self, expr: &ReduceExpr) -> Self::Result {
        self.visit_expr(expr.list.as_ref())?;
        self.visit_expr(expr.initial.as_ref())?;
        self.visit_expr(expr.expr.as_ref())
    }

    fn visit_list_comprehension_expr(&mut self, expr: &ListComprehensionExpr) -> Self::Result {
        self.visit_expr(expr.generator.as_ref())?;
        if let Some(condition_expr) = &expr.condition {
            self.visit_expr(condition_expr.as_ref())?;
        }
        Ok(())
    }
}
