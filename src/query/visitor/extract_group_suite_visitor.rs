//! ExtractGroupSuiteVisitor - 用于提取分组套件的访问器
//!
//! 主要功能：
//! - 从表达式中提取分组套件（用于 GROUP BY 优化）
//! - 识别可用于分组的表达式
//! - 构建分组键集合
//! - 支持聚合函数识别

use crate::core::{
    expression_visitor::{ExpressionVisitor, ExpressionVisitorState},
    BinaryOperator, DataType, Expression, UnaryOperator, Value,
};
use crate::core::types::operators::AggregateFunction;
use crate::expression::Expr;
use crate::query::parser::ast::expr::*;

/// 分组套件
#[derive(Debug, Clone, Default)]
pub struct GroupSuite {
    /// 分组键集合
    pub group_keys: Vec<Expr>,
    /// 分组项集合
    pub group_items: Vec<Expr>,
    /// 聚合函数集合
    pub aggregates: Vec<Expr>,
}

impl GroupSuite {
    /// 创建新的分组套件
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加分组键
    pub fn add_group_key(&mut self, expr: Expression) {
        if !self.group_keys.contains(&expr) {
            self.group_keys.push(expr);
        }
    }

    /// 添加分组项
    pub fn add_group_item(&mut self, expr: Expression) {
        if !self.group_items.contains(&expr) {
            self.group_items.push(expr);
        }
    }

    /// 添加聚合函数
    pub fn add_aggregate(&mut self, expr: Expression) {
        if !self.aggregates.contains(&expr) {
            self.aggregates.push(expr);
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
    pub fn extract(&mut self, expr: &Expr) -> Result<GroupSuite, String> {
        self.group_suite = GroupSuite::new();
        self.error = None;

        self.visit_expression(expr)?;

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
    fn is_aggregate_function(&self, expr: &Expr) -> bool {
        matches!(expr, Expr::Aggregate { .. })
    }

    /// 检查表达式是否为可分组的表达式
    fn is_groupable(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Literal(_) => true,
            Expr::Variable(_) => true,
            Expr::Property { .. } => true,
            Expr::Function { name, args } => {
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
            .add_group_key(Expr::Literal(value.clone()));
        Ok(())
    }

    fn visit_variable(&mut self, name: &str) -> Self::Result {
        self.group_suite
            .add_group_key(Expr::Variable(name.to_string()));
        Ok(())
    }

    fn visit_property(&mut self, object: &Expr, property: &str) -> Self::Result {
        let prop_expr = Expr::Property {
            object: Box::new(object.clone()),
            property: property.to_string(),
        };
        self.group_suite.add_group_key(prop_expr);
        self.visit_expression(object)
    }

    fn visit_binary(
        &mut self,
        left: &Expr,
        _op: &BinaryOperator,
        right: &Expr,
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

    fn visit_unary(&mut self, _op: &UnaryOperator, operand: &Expr) -> Self::Result {
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
                    let func_expr = Expr::Function {
                        name: name.to_string(),
                        args: args.to_vec(),
                    };
                    self.group_suite.add_group_key(func_expr);
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
        arg: &Expr,
        _distinct: bool,
    ) -> Self::Result {
        let agg_expr = Expr::Aggregate {
            func: func.clone(),
            arg: Box::new(arg.clone()),
            distinct: false,
        };
        self.group_suite.add_aggregate(agg_expr);
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
        conditions: &[(Expr, Expr)],
        default: &Option<Box<Expr>>,
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

    fn visit_type_cast(&mut self, expr: &Expr, _target_type: &DataType) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_subscript(&mut self, collection: &Expr, index: &Expr) -> Self::Result {
        self.visit_expression(collection)?;
        self.visit_expression(index)
    }

    fn visit_range(
        &mut self,
        collection: &Expr,
        start: &Option<Box<Expr>>,
        end: &Option<Box<Expr>>,
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

    fn visit_label(&mut self, name: &str) -> Self::Result {
        self.group_suite.add_group_key(Expr::Label(name.to_string()));
        Ok(())
    }

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }
}
