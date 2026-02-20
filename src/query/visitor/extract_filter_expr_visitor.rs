//! ExtractFilterExprVisitor - 用于提取过滤表达式的访问器

use crate::core::types::expression::Expression;
use crate::core::types::expression::visitor::{ExpressionVisitor, ExpressionVisitorState};
use crate::core::Value;
use crate::core::{AggregateFunction, BinaryOperator, DataType, UnaryOperator};

/// 推送目标类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PushTarget {
    /// 推送到 GetVertices
    GetVertices,
    /// 推送到 GetNeighbors
    GetNeighbors,
    /// 不推送（仅提取）
    None,
}

#[derive(Debug)]
pub struct ExtractFilterExprVisitor {
    /// 提取到的过滤表达式
    filter_exprs: Vec<Expression>,
    /// 剩余的不能下推的表达式
    remained_expr: Option<Expression>,
    /// 是否只提取顶层的过滤条件
    top_level_only: bool,
    /// 当前是否在顶层
    is_top_level: bool,
    /// 推送目标
    push_target: PushTarget,
    /// 访问者状态
    state: ExpressionVisitorState,
    /// 是否成功
    ok: bool,
}

impl Clone for ExtractFilterExprVisitor {
    fn clone(&self) -> Self {
        Self {
            filter_exprs: self.filter_exprs.clone(),
            remained_expr: self.remained_expr.clone(),
            top_level_only: self.top_level_only,
            is_top_level: self.is_top_level,
            push_target: self.push_target,
            state: self.state.clone(),
            ok: self.ok,
        }
    }
}

impl ExtractFilterExprVisitor {
    pub fn new(top_level_only: bool) -> Self {
        Self {
            filter_exprs: Vec::new(),
            remained_expr: None,
            top_level_only,
            is_top_level: true,
            push_target: PushTarget::None,
            state: ExpressionVisitorState::new(),
            ok: true,
        }

    }

    pub fn make_push_get_vertices() -> Self {
        Self {
            filter_exprs: Vec::new(),
            remained_expr: None,
            top_level_only: false,
            is_top_level: true,
            push_target: PushTarget::GetVertices,
            state: ExpressionVisitorState::new(),
            ok: true,
        }
    }

    pub fn make_push_get_neighbors() -> Self {
        Self {
            filter_exprs: Vec::new(),
            remained_expr: None,
            top_level_only: false,
            is_top_level: true,
            push_target: PushTarget::GetNeighbors,
            state: ExpressionVisitorState::new(),
            ok: true,
        }
    }

    pub fn extract(&mut self, expression: &Expression) -> Result<Vec<Expression>, String> {
        self.filter_exprs.clear();
        self.remained_expr = None;
        self.is_top_level = true;
        self.ok = true;
        let result = self.visit_expression(expression);
        result?;
        Ok(self.filter_exprs.clone())
    }

    fn visit_with_updated_level(&mut self, expression: &Expression) -> Result<(), String> {
        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        let result = self.visit_expression(expression);
        self.is_top_level = old_top_level;
        result
    }

    pub fn get_filter_exprs(&self) -> &Vec<Expression> {
        &self.filter_exprs
    }

    pub fn remained_expr(&self) -> Option<Expression> {
        self.remained_expr.clone()
    }

    pub fn ok(&self) -> bool {
        self.ok
    }

    fn can_push(&self, expr: &Expression) -> bool {
        match self.push_target {
            PushTarget::GetVertices => self.can_push_to_get_vertices(expr),
            PushTarget::GetNeighbors => self.can_push_to_get_neighbors(expr),
            PushTarget::None => false,
        }
    }

    fn can_push_to_get_vertices(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Property { object, .. } => {
                matches!(object.as_ref(), Expression::Variable(_))
            }
            Expression::Binary { left, op: _, right } => {
                self.can_push_to_get_vertices(left) && self.can_push_to_get_vertices(right)
            }
            Expression::Unary { operand, .. } => self.can_push_to_get_vertices(operand),
            Expression::Function { name, args } => {
                is_filter_function(name) && args.iter().all(|a| self.can_push_to_get_vertices(a))
            }
            _ => false,
        }
    }

    fn can_push_to_get_neighbors(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Property { object, .. } => {
                matches!(object.as_ref(), Expression::Variable(_))
            }
            Expression::Binary { left, op: _, right } => {
                self.can_push_to_get_neighbors(left) && self.can_push_to_get_neighbors(right)
            }
            Expression::Unary { operand, .. } => self.can_push_to_get_neighbors(operand),
            Expression::Function { name, args } => {
                is_filter_function(name) && args.iter().all(|a| self.can_push_to_get_neighbors(a))
            }
            _ => false,
        }
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
        op: &BinaryOperator,
        right: &Expression,
    ) -> Self::Result {
        if self.push_target == PushTarget::None {
            if self.is_top_level || !self.top_level_only {
                self.visit_with_updated_level(left)?;
                self.visit_with_updated_level(right)?;
            } else {
                self.filter_exprs.push(Expression::Binary {
                    left: Box::new(left.clone()),
                    op: op.clone(),
                    right: Box::new(right.clone()),
                });
            }
        } else {
            let left_can_push = self.can_push(left);
            let right_can_push = self.can_push(right);

            if left_can_push && right_can_push {
                self.filter_exprs.push(Expression::Binary {
                    left: Box::new(left.clone()),
                    op: op.clone(),
                    right: Box::new(right.clone()),
                });
            } else if left_can_push {
                self.filter_exprs.push(left.clone());
                if self.remained_expr.is_none() {
                    self.remained_expr = Some(right.clone());
                } else {
                    let current = match self.remained_expr.take() {
                        Some(expr) => expr,
                        None => return Ok(()),
                    };
                    self.remained_expr = Some(Expression::Binary {
                        left: Box::new(current),
                        op: op.clone(),
                        right: Box::new(right.clone()),
                    });
                }
            } else if right_can_push {
                self.filter_exprs.push(right.clone());
                if self.remained_expr.is_none() {
                    self.remained_expr = Some(left.clone());
                } else {
                    let current = match self.remained_expr.take() {
                        Some(expr) => expr,
                        None => return Ok(()),
                    };
                    self.remained_expr = Some(Expression::Binary {
                        left: Box::new(current),
                        op: op.clone(),
                        right: Box::new(left.clone()),
                    });
                }
            } else {
                if self.remained_expr.is_none() {
                    self.remained_expr = Some(Expression::Binary {
                        left: Box::new(left.clone()),
                        op: op.clone(),
                        right: Box::new(right.clone()),
                    });
                } else {
                    let current = match self.remained_expr.take() {
                        Some(expr) => expr,
                        None => return Ok(()),
                    };
                    self.remained_expr = Some(Expression::Binary {
                        left: Box::new(current),
                        op: op.clone(),
                        right: Box::new(Expression::Binary {
                            left: Box::new(left.clone()),
                            op: op.clone(),
                            right: Box::new(right.clone()),
                        }),
                    });
                }
            }
        }
        Ok(())
    }

    fn visit_unary(&mut self, _op: &UnaryOperator, operand: &Expression) -> Self::Result {
        self.visit_expression(operand)
    }

    fn visit_function(&mut self, name: &str, args: &[Expression]) -> Self::Result {
        if is_filter_function(name) {
            if self.is_top_level || !self.top_level_only {
                self.filter_exprs.push(Expression::Function {
                    name: name.to_string(),
                    args: args.to_vec(),
                });
            }
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
        for (_, expression) in pairs {
            self.visit_expression(expression)?;
        }
        Ok(())
    }

    fn visit_case(
        &mut self,
        test_expr: Option<&Expression>,
        conditions: &[(Expression, Expression)],
        default: Option<&Expression>,
    ) -> Self::Result {
        if let Some(test) = test_expr {
            self.visit_expression(test)?;
        }
        for (cond, expression) in conditions {
            self.visit_expression(cond)?;
            self.visit_expression(expression)?;
        }
        if let Some(d) = default {
            self.visit_expression(d)?;
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
        start: Option<&Expression>,
        end: Option<&Expression>,
    ) -> Self::Result {
        self.visit_expression(collection)?;
        if let Some(s) = start {
            self.visit_expression(s)?;
        }
        if let Some(e) = end {
            self.visit_expression(e)?;
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

    fn visit_list_comprehension(
        &mut self,
        _variable: &str,
        source: &Expression,
        filter: Option<&Expression>,
        map: Option<&Expression>,
    ) -> Self::Result {
        self.visit_expression(source)?;
        if let Some(f) = filter {
            self.visit_expression(f)?;
        }
        if let Some(m) = map {
            self.visit_expression(m)?;
        }
        Ok(())
    }

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }

    fn visit_label_tag_property(&mut self, tag: &Expression, _property: &str) -> Self::Result {
        self.visit_expression(tag)
    }

    fn visit_tag_property(&mut self, _tag_name: &str, _property: &str) -> Self::Result {
        Ok(())
    }

    fn visit_edge_property(&mut self, _edge_name: &str, _property: &str) -> Self::Result {
        Ok(())
    }

    fn visit_predicate(&mut self, _func: &str, args: &[Expression]) -> Self::Result {
        for arg in args {
            self.visit_expression(arg)?;
        }
        Ok(())
    }

    fn visit_reduce(
        &mut self,
        _accumulator: &str,
        initial: &Expression,
        _variable: &str,
        source: &Expression,
        mapping: &Expression,
    ) -> Self::Result {
        self.visit_expression(initial)?;
        self.visit_expression(source)?;
        self.visit_expression(mapping)
    }

    fn visit_path_build(&mut self, exprs: &[Expression]) -> Self::Result {
        for expr in exprs {
            self.visit_expression(expr)?;
        }
        Ok(())
    }

    fn visit_parameter(&mut self, _name: &str) -> Self::Result {
        Ok(())
    }
}
