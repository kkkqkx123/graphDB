//! ExtractGroupSuiteVisitor - 用于提取分组套件的访问器
//!
//! 主要功能：
//! - 从表达式中提取分组套件（用于 GROUP BY 优化）
//! - 识别可用于分组的表达式
//! - 构建分组键集合
//! - 支持聚合函数识别

use crate::core::types::expression::Expression;
use crate::core::{
    expression_visitor::{ExpressionVisitor, ExpressionVisitorState},
    BinaryOperator, DataType, UnaryOperator, Value,
};
use crate::core::types::operators::AggregateFunction;
use crate::query::parser::ast::expression::*;

/// 分组套件
#[derive(Debug, Clone, Default)]
pub struct GroupSuite {
    /// 分组键集合
    pub group_keys: Vec<Expression>,
    /// 分组项集合
    pub group_items: Vec<Expression>,
    /// 聚合函数集合
    pub aggregates: Vec<Expression>,
}

impl GroupSuite {
    /// 创建新的分组套件
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加分组键
    pub fn add_group_key(&mut self, expression: Expression) {
        if !self.group_keys.contains(&expression) {
            self.group_keys.push(expression);
        }
    }

    /// 添加分组项
    pub fn add_group_item(&mut self, expression: Expression) {
        if !self.group_items.contains(&expression) {
            self.group_items.push(expression);
        }
    }

    /// 添加聚合函数
    pub fn add_aggregate(&mut self, expression: Expression) {
        if !self.aggregates.contains(&expression) {
            self.aggregates.push(expression);
        }
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.group_keys.is_empty()
            && self.group_items.is_empty()
            && self.aggregates.is_empty()
    }

    /// 合并另一个 GroupSuite
    pub fn union(&mut self, other: &GroupSuite) {
        for key in &other.group_keys {
            self.add_group_key(key.clone());
        }

        for item in &other.group_items {
            self.add_group_item(item.clone());
        }

        for agg in &other.aggregates {
            self.add_aggregate(agg.clone());
        }
    }
}

/// 分组套件提取访问器
///
/// 用于从表达式中提取分组套件，支持 GROUP BY 优化
#[derive(Debug)]
pub struct ExtractGroupSuiteVisitor {
    /// 提取到的分组套件
    group_suite: GroupSuite,
    /// 错误状态
    error: Option<String>,
    /// 访问者状态
    state: ExpressionVisitorState,
}

impl ExtractGroupSuiteVisitor {
    /// 创建新的分组套件提取访问器
    pub fn new() -> Self {
        Self {
            group_suite: GroupSuite::new(),
            error: None,
            state: ExpressionVisitorState::new(),
        }
    }

    /// 提取分组套件
    pub fn extract(&mut self, expression: &Expression) -> Result<GroupSuite, String> {
        self.group_suite = GroupSuite::new();
        self.error = None;

        self.visit_expression(expression)?;

        if let Some(err) = &self.error {
            Err(err.clone())
        } else {
            Ok(self.group_suite.clone())
        }
    }

    /// 获取提取到的分组套件
    pub fn get_group_suite(&self) -> &GroupSuite {
        &self.group_suite
    }

    /// 设置错误信息
    fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    /// 检查表达式是否为聚合函数
    fn is_aggregate_function(&self, expression: &Expression) -> bool {
        matches!(expression, Expression::Aggregate { .. })
    }

    /// 检查表达式是否为可分组的表达式
    fn is_groupable(&self, expression: &Expression) -> bool {
        match expression {
            Expression::Literal(_) => true,
            Expression::Variable(_) => true,
            Expression::Property { .. } => true,
            Expression::Function { name, args } => {
                let name_upper = name.to_uppercase();
                match name_upper.as_str() {
                    "ID" | "SRC" | "DST" => args.len() == 1,
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

impl Default for ExtractGroupSuiteVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionVisitor for ExtractGroupSuiteVisitor {
    type Result = Result<(), String>;

    fn visit_literal(&mut self, value: &Value) -> Self::Result {
        self.group_suite
            .add_group_key(Expression::Literal(value.clone()));
        Ok(())
    }

    fn visit_variable(&mut self, name: &str) -> Self::Result {
        self.group_suite
            .add_group_key(Expression::Variable(name.to_string()));
        Ok(())
    }

    fn visit_property(&mut self, object: &Expression, property: &str) -> Self::Result {
        let prop_expression = Expression::Property {
            object: Box::new(object.clone()),
            property: property.to_string(),
        };
        self.group_suite.add_group_key(prop_expression);
        self.visit_expression(object)
    }

    fn visit_binary(
        &mut self,
        left: &Expression,
        _op: &BinaryOperator,
        right: &Expression,
    ) -> Self::Result {
        if self.is_groupable(left) {
            self.group_suite.add_group_key(left.clone());
        }
        if self.is_groupable(right) {
            self.group_suite.add_group_key(right.clone());
        }
        self.visit_expression(left)?;
        self.visit_expression(right)
    }

    fn visit_unary(&mut self, _op: &UnaryOperator, operand: &Expression) -> Self::Result {
        if self.is_groupable(operand) {
            self.group_suite.add_group_key(operand.clone());
        }
        self.visit_expression(operand)
    }

    fn visit_function(&mut self, name: &str, args: &[Expression]) -> Self::Result {
        let name_upper = name.to_uppercase();

        match name_upper.as_str() {
            "ID" | "SRC" | "DST" => {
                if args.len() == 1 {
                    let func_expression = Expression::Function {
                        name: name.to_string(),
                        args: args.to_vec(),
                    };
                    self.group_suite.add_group_key(func_expression);
                }
            }
            _ => {
                for arg in args {
                    self.visit_expression(arg)?;
                }
            }
        }
        Ok(())
    }

    fn visit_aggregate(
        &mut self,
        func: &AggregateFunction,
        arg: &Expression,
        _distinct: bool,
    ) -> Self::Result {
        let agg_expression = Expression::Aggregate {
            func: func.clone(),
            arg: Box::new(arg.clone()),
            distinct: false,
        };
        self.group_suite.add_aggregate(agg_expression);
        self.visit_expression(arg)
    }

    fn visit_list(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit_expression(item)?;
        }
        Ok(())
    }

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
        for (_, expression) in pairs {
            self.visit_expression(expression)?;
        }
        Ok(())
    }

    fn visit_case(
        &mut self,
        conditions: &[(Expression, Expression)],
        default: &Option<Box<Expression>>,
    ) -> Self::Result {
        for (cond, expression) in conditions {
            self.visit_expression(cond)?;
            self.visit_expression(expression)?;
        }
        if let Some(default_expression) = default {
            self.visit_expression(default_expression)?;
        }
        Ok(())
    }

    fn visit_type_cast(&mut self, expression: &Expression, _target_type: &DataType) -> Self::Result {
        self.visit_expression(expression)
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
        if let Some(start_expression) = start {
            self.visit_expression(start_expression)?;
        }
        if let Some(end_expression) = end {
            self.visit_expression(end_expression)?;
        }
        Ok(())
    }

    fn visit_path(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit_expression(item)?;
        }
        Ok(())
    }

    fn visit_label(&mut self, name: &str) -> Self::Result {
        self.group_suite.add_group_key(Expression::Label(name.to_string()));
        Ok(())
    }

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }
}
