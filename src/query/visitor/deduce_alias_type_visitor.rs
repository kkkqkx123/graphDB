//! DeduceAliasTypeVisitor - 用于推导表达式别名类型的访问器
//!
//! 主要功能：
//! - 推导表达式的别名类型（Vertex/Edge/Path/Runtime）
//! - 识别顶点表达式
//! - 识别边表达式
//! - 识别路径构建表达式
//! - 识别函数调用的返回类型

use crate::core::types::expression::Expression;
use crate::core::types::expression::visitor::{ExpressionVisitor, ExpressionVisitorState};
use crate::core::{
    BinaryOperator, DataType, UnaryOperator, Value,
};
use crate::core::types::operators::AggregateFunction;

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
    pub fn deduce(&mut self, expression: &Expression) -> Result<AliasType, String> {
        self.output_type = self.input_type;
        self.error = None;

        self.visit_expression(expression)?;

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
        for (_, expression) in pairs {
            self.visit_expression(expression)?;
        }
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_case(
        &mut self,
        conditions: &[(Expression, Expression)],
        default: Option<&Expression>,
    ) -> Self::Result {
        for (cond, expression) in conditions {
            self.visit_expression(cond)?;
            self.visit_expression(expression)?;
        }
        if let Some(default_expression) = default {
            self.visit_expression(default_expression)?;
        }
        self.set_output_type(AliasType::Runtime);
        Ok(())
    }

    fn visit_type_cast(&mut self, expression: &Expression, _target_type: &DataType) -> Self::Result {
        self.visit_expression(expression)?;
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
        start: Option<&Expression>,
        end: Option<&Expression>,
    ) -> Self::Result {
        self.visit_expression(collection)?;
        if let Some(start_expression) = start {
            self.visit_expression(start_expression)?;
        }
        if let Some(end_expression) = end {
            self.visit_expression(end_expression)?;
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

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }
}
