//! DeduceAliasTypeVisitor - 用于推导表达式别名类型的访问器
//!
//! 主要功能：
//! - 推导表达式的别名类型（Vertex/Edge/Path/Runtime）
//! - 识别顶点表达式
//! - 识别边表达式
//! - 识别路径构建表达式
//! - 识别函数调用的返回类型

use crate::core::{
    expression_visitor::{ExpressionVisitor, ExpressionVisitorState},
    BinaryOperator, DataType, Expression, UnaryOperator, Value,
};
use crate::core::types::operators::AggregateFunction;
use crate::query::parser::ast::expr::*;

/// 别名类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AliasType {
    /// 顶点类型
    Vertex,
    /// 边类型
    Edge,
    /// 路径类型
    Path,
    /// 运行时类型（无法在编译时确定）
    Runtime,
}

impl Default for AliasType {
    fn default() -> Self {
        AliasType::Runtime
    }
}

/// 别名类型推导访问器
///
/// 用于推导表达式的别名类型，支持类型检查和优化
#[derive(Debug)]
pub struct DeduceAliasTypeVisitor {
    /// 输入类型
    input_type: AliasType,
    /// 输出类型
    output_type: AliasType,
    /// 错误状态
    error: Option<String>,
    /// 访问者状态
    state: ExpressionVisitorState,
}

impl DeduceAliasTypeVisitor {
    /// 创建新的别名类型推导访问器
    pub fn new() -> Self {
        Self {
            input_type: AliasType::Runtime,
            output_type: AliasType::Runtime,
            error: None,
            state: ExpressionVisitorState::new(),
        }
    }

    /// 创建带有输入类型的访问器
    pub fn with_input_type(input_type: AliasType) -> Self {
        Self {
            input_type,
            output_type: input_type,
            error: None,
            state: ExpressionVisitorState::new(),
        }
    }

    /// 推导表达式的别名类型
    pub fn deduce(&mut self, expr: &Expression) -> Result<AliasType, String> {
        self.output_type = self.input_type;
        self.error = None;

        self.visit_expression(expr)?;

        if let Some(err) = &self.error {
            Err(err.clone())
        } else {
            Ok(self.output_type)
        }
    }

    /// 获取输出类型
    pub fn output_type(&self) -> AliasType {
        self.output_type
    }

    /// 设置输出类型
    fn set_output_type(&mut self, output_type: AliasType) {
        self.output_type = output_type;
    }

    /// 设置错误信息
    fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    /// 检查函数是否返回特定类型
    fn check_function_return_type(&self, name: &str) -> Option<AliasType> {
        let name_upper = name.to_uppercase();

        match name_upper.as_str() {
            "ID" => Some(AliasType::Vertex),
            "SRC" | "DST" => Some(AliasType::Vertex),
            "TYPE" => Some(AliasType::Edge),
            "RANK" => Some(AliasType::Edge),
            "PROPERTIES" => Some(AliasType::Runtime),
            _ => None,
        }
    }
}

impl Default for DeduceAliasTypeVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionVisitor for DeduceAliasTypeVisitor {
    type Result = Result<(), String>;

    fn visit_literal(&mut self, _value: &Value) -> Self::Result {
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_variable(&mut self, _name: &str) -> Self::Result {
        self.set_output_type(AliasType::Runtime);
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
        self.visit_expression(right)?;
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_unary(&mut self, _op: &UnaryOperator, operand: &Expression) -> Self::Result {
        self.visit_expression(operand)?;
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_function(&mut self, name: &str, args: &[Expression]) -> Self::Result {
        if let Some(return_type) = self.check_function_return_type(name) {
            self.set_output_type(return_type);
        } else {
            for arg in args {
                self.visit_expression(arg)?;
            }
            self.set_output_type(AliasType::Runtime);
        }
        Ok(())
    }

    fn visit_aggregate(
        &mut self,
        _func: &AggregateFunction,
        arg: &Expression,
        _distinct: bool,
    ) -> Self::Result {
        self.visit_expression(arg)?;
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_list(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit_expression(item)?;
        }
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
        for (_, expr) in pairs {
            self.visit_expression(expr)?;
        }
        self.set_output_type(AliasType::Runtime);
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
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_type_cast(&mut self, expr: &Expression, _target_type: &DataType) -> Self::Result {
        self.visit_expression(expr)?;
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result {
        self.visit_expression(collection)?;
        self.visit_expression(index)?;
        self.set_output_type(AliasType::Runtime);
        Ok(())
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
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_path(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit_expression(item)?;
        }
        self.set_output_type(AliasType::Path);
        Ok(())
    }

    fn visit_label(&mut self, _name: &str) -> Self::Result {
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_tag_property(&mut self, _tag: &str, _prop: &str) -> Self::Result {
        self.set_output_type(AliasType::Vertex);
        Ok(())
    }

    fn visit_edge_property(&mut self, _edge: &str, _prop: &str) -> Self::Result {
        self.set_output_type(AliasType::Edge);
        Ok(())
    }

    fn visit_input_property(&mut self, _prop: &str) -> Self::Result {
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_variable_property(&mut self, _var: &str, _prop: &str) -> Self::Result {
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_source_property(&mut self, _tag: &str, _prop: &str) -> Self::Result {
        self.set_output_type(AliasType::Vertex);
        Ok(())
    }

    fn visit_destination_property(&mut self, _tag: &str, _prop: &str) -> Self::Result {
        self.set_output_type(AliasType::Vertex);
        Ok(())
    }

    fn visit_unary_plus(&mut self, expr: &Expression) -> Self::Result {
        self.visit_unary(&UnaryOperator::Plus, expr)
    }

    fn visit_unary_negate(&mut self, expr: &Expression) -> Self::Result {
        self.visit_unary(&UnaryOperator::Minus, expr)
    }

    fn visit_unary_not(&mut self, expr: &Expression) -> Self::Result {
        self.visit_unary(&UnaryOperator::Not, expr)
    }

    fn visit_unary_incr(&mut self, expr: &Expression) -> Self::Result {
        self.visit_unary(&UnaryOperator::Increment, expr)
    }

    fn visit_unary_decr(&mut self, expr: &Expression) -> Self::Result {
        self.visit_unary(&UnaryOperator::Decrement, expr)
    }

    fn visit_is_null(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr)?;
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_is_not_null(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr)?;
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_is_empty(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr)?;
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_is_not_empty(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr)?;
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_list_comprehension(
        &mut self,
        generator: &Expression,
        condition: &Option<Box<Expression>>,
    ) -> Self::Result {
        self.visit_expression(generator)?;
        if let Some(cond) = condition {
            self.visit_expression(cond)?;
        }
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_predicate(&mut self, list: &Expression, condition: &Expression) -> Self::Result {
        self.visit_expression(list)?;
        self.visit_expression(condition)?;
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_reduce(
        &mut self,
        list: &Expression,
        _var: &str,
        initial: &Expression,
        expr: &Expression,
    ) -> Self::Result {
        self.visit_expression(list)?;
        self.visit_expression(initial)?;
        self.visit_expression(expr)?;
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_es_query(&mut self, _query: &str) -> Self::Result {
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_uuid(&mut self) -> Self::Result {
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_match_path_pattern(
        &mut self,
        _path_alias: &str,
        _patterns: &[Expression],
    ) -> Self::Result {
        self.set_output_type(AliasType::Path);
        Ok(())
    }

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }

    fn visit_type_casting(&mut self, expr: &Expression, _target_type: &str) -> Self::Result {
        self.visit_expression(expr)?;
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_path_build(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit_expression(item)?;
        }
        self.set_output_type(AliasType::Path);
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
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_constant_expr(&mut self, _expr: &ConstantExpr) -> Self::Result {
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_variable_expr(&mut self, _expr: &VariableExpr) -> Self::Result {
        self.set_output_type(AliasType::Runtime);
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

    fn visit_tag_property_expr(&mut self, expr: &TagPropertyExpr) -> Self::Result {
        self.visit_tag_property(&expr.tag, &expr.prop)
    }

    fn visit_edge_property_expr(&mut self, expr: &EdgePropertyExpr) -> Self::Result {
        self.visit_edge_property(&expr.edge, &expr.prop)
    }

    fn visit_input_property_expr(&mut self, expr: &InputPropertyExpr) -> Self::Result {
        self.visit_input_property(&expr.prop)
    }

    fn visit_variable_property_expr(&mut self, expr: &VariablePropertyExpr) -> Self::Result {
        self.visit_variable_property(&expr.var, &expr.prop)
    }

    fn visit_source_property_expr(&mut self, expr: &SourcePropertyExpr) -> Self::Result {
        self.visit_source_property(&expr.tag, &expr.prop)
    }

    fn visit_destination_property_expr(&mut self, expr: &DestinationPropertyExpr) -> Self::Result {
        self.visit_destination_property(&expr.tag, &expr.prop)
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
        self.set_output_type(AliasType::Runtime);
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
