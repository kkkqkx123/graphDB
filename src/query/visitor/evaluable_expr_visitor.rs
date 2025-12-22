//! EvaluableExprVisitor - 用于判断表达式是否可求值的访问器
//! 对应 NebulaGraph EvaluableExprVisitor.h/.cpp 的功能

use crate::core::visitor::{VisitorCore, VisitorContext, VisitorResult};
use crate::core::{Expression, ExpressionVisitor, LiteralValue, BinaryOperator, UnaryOperator, AggregateFunction, DataType};
use crate::query::visitor::QueryVisitor;

#[derive(Debug)]
pub struct EvaluableExprVisitor {
    /// 表达式是否可求值
    evaluable: bool,
    /// 错误信息
    error: Option<String>,
    /// 访问器上下文
    context: VisitorContext,
    /// 访问器状态
    state: crate::core::visitor::visitor_state_enum::VisitorStateEnum,
}

impl EvaluableExprVisitor {
    pub fn new() -> Self {
        Self {
            evaluable: true,
            error: None,
            context: VisitorContext::new(crate::core::visitor::VisitorConfig::new()),
            state: crate::core::visitor::visitor_state_enum::VisitorStateEnum::new(),
        }
    }

    /// 创建带初始深度的 EvaluableExprVisitor
    pub fn with_depth(depth: usize) -> Self {
        Self {
            evaluable: true,
            error: None,
            context: VisitorContext::new(crate::core::visitor::VisitorConfig::new()),
            state: crate::core::visitor::visitor_state_enum::VisitorStateEnum::with_depth(depth),
        }
    }

    /// 创建带配置的 EvaluableExprVisitor
    pub fn with_config(config: crate::core::visitor::VisitorConfig) -> Self {
        Self {
            evaluable: true,
            error: None,
            context: VisitorContext::new(config),
            state: crate::core::visitor::visitor_state_enum::VisitorStateEnum::new(),
        }
    }

    /// 创建带配置和初始深度的 EvaluableExprVisitor
    pub fn with_config_and_depth(config: crate::core::visitor::VisitorConfig, depth: usize) -> Self {
        Self {
            evaluable: true,
            error: None,
            context: VisitorContext::new(config),
            state: crate::core::visitor::visitor_state_enum::VisitorStateEnum::with_depth(depth),
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

    fn visit(&mut self, expr: &Expression) -> Result<(), String> {
        match expr {
            // 常量表达式是可求值的
            Expression::Literal(_) => Ok(()),

            // 变量表达式依赖于上下文，可能不可求值
            Expression::Property {
                object: _,
                property: _,
            } => {
                // 在当前实现中，如果表达式包含变量，则可能是不可求值的
                // 在实际实现中，需要检查该变量是否在当前上下文中被定义
                self.evaluable = false;
                Ok(())
            }

            // 算术表达式 - 如果所有子表达式都可求值，则该表达式可求值
            Expression::Unary { op: _, operand } => self.visit(operand),

            Expression::Binary { left, op: _, right } => {
                self.visit(left)?;
                self.visit(right)
            }

            // 函数调用 - 内置函数如果参数可求值则可求值
            Expression::Function { name: _, args } => {
                for arg in args {
                    self.visit(arg)?;
                }
                Ok(())
            }

            // 其他表达式类型，默认不可求值
            _ => {
                self.evaluable = false;
                Ok(())
            }
        }
    }
}

impl VisitorCore<Expression> for EvaluableExprVisitor {
    type Result = Result<(), String>;

    fn visit(&mut self, target: &Expression) -> Self::Result {
        // 使用表达式访问器模式进行访问
        match target {
            Expression::Literal(value) => self.visit_literal(value),
            Expression::Variable(name) => self.visit_variable(name),
            Expression::Property { object, property } => self.visit_property(object, property),
            Expression::Binary { left, op, right } => self.visit_binary(left, op, right),
            Expression::Unary { op, operand } => self.visit_unary(op, operand),
            Expression::Function { name, args } => self.visit_function(name, args),
            Expression::Aggregate { func, arg, distinct } => self.visit_aggregate(func, arg, *distinct),
            Expression::List(items) => self.visit_list(items),
            Expression::Map(pairs) => self.visit_map(pairs),
            Expression::Case { conditions, default } => {
                let default_cloned = default.as_ref().map(|b| (**b).clone());
                self.visit_case(conditions, &default_cloned)
            }
            Expression::TypeCast { expr, target_type } => self.visit_type_cast(expr, target_type),
            Expression::Subscript { collection, index } => self.visit_subscript(collection, index),
            Expression::Range { collection, start, end } => {
                let start_cloned = start.as_ref().map(|b| (**b).clone());
                let end_cloned = end.as_ref().map(|b| (**b).clone());
                self.visit_range(collection, &start_cloned, &end_cloned)
            }
            Expression::Path(items) => self.visit_path(items),
            Expression::Label(name) => self.visit_label(name),
            Expression::TagProperty { tag, prop } => self.visit_tag_property(tag, prop),
            Expression::EdgeProperty { edge, prop } => self.visit_edge_property(edge, prop),
            Expression::InputProperty(prop) => self.visit_input_property(prop),
            Expression::VariableProperty { var, prop } => self.visit_variable_property(var, prop),
            Expression::SourceProperty { tag, prop } => self.visit_source_property(tag, prop),
            Expression::DestinationProperty { tag, prop } => self.visit_destination_property(tag, prop),
            
            // 处理新增的表达式类型
            Expression::UnaryPlus(expr) => self.visit_unary(&UnaryOperator::Plus, expr),
            Expression::UnaryNegate(expr) => self.visit_unary(&UnaryOperator::Minus, expr),
            Expression::UnaryNot(expr) => self.visit_unary(&UnaryOperator::Not, expr),
            Expression::UnaryIncr(expr) => self.visit_unary(&UnaryOperator::Increment, expr),
            Expression::UnaryDecr(expr) => self.visit_unary(&UnaryOperator::Decrement, expr),
            Expression::IsNull(expr) => self.visit_unary(&UnaryOperator::IsNull, expr),
            Expression::IsNotNull(expr) => self.visit_unary(&UnaryOperator::IsNotNull, expr),
            Expression::IsEmpty(expr) => self.visit_unary(&UnaryOperator::IsEmpty, expr),
            Expression::IsNotEmpty(expr) => self.visit_unary(&UnaryOperator::IsNotEmpty, expr),
            
            Expression::TypeCasting { expr, .. } => self.visit_type_cast(expr, &DataType::String),
            Expression::ListComprehension { generator, condition } => {
                // 简化为函数调用
                let cond_expr = condition
                    .as_ref()
                    .map(|c| (**c).clone())
                    .unwrap_or(Expression::bool(true));
                self.visit_function(
                    "list_comprehension",
                    &[(**generator).clone(), cond_expr],
                )
            }
            Expression::Predicate { list, condition } => {
                self.visit_function("predicate", &[(**list).clone(), (**condition).clone()])
            }
            Expression::Reduce { list, initial, expr, .. } => {
                self.visit_function("reduce", &[(**list).clone(), (**initial).clone(), (**expr).clone()])
            }
            Expression::PathBuild(items) => self.visit_path(items),
            Expression::ESQuery(query) => self.visit_function("es_query", &[Expression::string(query)]),
            Expression::UUID => self.visit_function("uuid", &[]),
            Expression::SubscriptRange { collection, start, end } => {
                let start_cloned = start.as_ref().map(|b| (**b).clone());
                let end_cloned = end.as_ref().map(|b| (**b).clone());
                self.visit_range(collection, &start_cloned, &end_cloned)
            }
            Expression::MatchPathPattern { patterns, .. } => self.visit_list(patterns),
        }
    }

    fn context(&self) -> &VisitorContext {
        &self.context
    }

    fn context_mut(&mut self) -> &mut VisitorContext {
        &mut self.context
    }

    fn state(&self) -> &crate::core::visitor::visitor_state_enum::VisitorStateEnum {
        &self.state
    }

    fn state_mut(&mut self) -> &mut crate::core::visitor::visitor_state_enum::VisitorStateEnum {
        &mut self.state
    }
}

impl ExpressionVisitor for EvaluableExprVisitor {
    fn visit_literal(&mut self, _value: &LiteralValue) -> Self::Result {
        // 常量表达式是可求值的
        Ok(())
    }

    fn visit_variable(&mut self, _name: &str) -> Self::Result {
        // 变量表达式依赖于上下文，可能不可求值
        self.evaluable = false;
        Ok(())
    }

    fn visit_property(&mut self, _object: &Expression, _property: &str) -> Self::Result {
        // 属性表达式依赖于上下文，可能不可求值
        self.evaluable = false;
        Ok(())
    }

    fn visit_binary(&mut self, left: &Expression, _op: &BinaryOperator, right: &Expression) -> Self::Result {
        self.visit(left)?;
        self.visit(right)?;
        Ok(())
    }

    fn visit_unary(&mut self, _op: &UnaryOperator, operand: &Expression) -> Self::Result {
        self.visit(operand)?;
        Ok(())
    }

    fn visit_function(&mut self, _name: &str, args: &[Expression]) -> Self::Result {
        for arg in args {
            self.visit(arg)?;
        }
        Ok(())
    }

    fn visit_aggregate(&mut self, _func: &AggregateFunction, arg: &Expression, _distinct: bool) -> Self::Result {
        self.visit(arg)?;
        Ok(())
    }

    fn visit_list(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit(item)?;
        }
        Ok(())
    }

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
        for (_, value) in pairs {
            self.visit(value)?;
        }
        Ok(())
    }

    fn visit_case(&mut self, conditions: &[(Expression, Expression)], default: &Option<Expression>) -> Self::Result {
        for (condition, value) in conditions {
            self.visit(condition)?;
            self.visit(value)?;
        }
        if let Some(default_expr) = default {
            self.visit(default_expr)?;
        }
        Ok(())
    }

    fn visit_type_cast(&mut self, expr: &Expression, _target_type: &DataType) -> Self::Result {
        self.visit(expr)?;
        Ok(())
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result {
        self.visit(collection)?;
        self.visit(index)?;
        Ok(())
    }

    fn visit_range(&mut self, collection: &Expression, start: &Option<Expression>, end: &Option<Expression>) -> Self::Result {
        self.visit(collection)?;
        if let Some(start_expr) = start {
            self.visit(start_expr)?;
        }
        if let Some(end_expr) = end {
            self.visit(end_expr)?;
        }
        Ok(())
    }

    fn visit_path(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit(item)?;
        }
        Ok(())
    }

    fn visit_label(&mut self, _name: &str) -> Self::Result {
        // 标签表达式是可求值的
        Ok(())
    }

    fn visit_tag_property(&mut self, _tag: &str, _prop: &str) -> Self::Result {
        // 标签属性依赖于上下文，可能不可求值
        self.evaluable = false;
        Ok(())
    }

    fn visit_edge_property(&mut self, _edge: &str, _prop: &str) -> Self::Result {
        // 边属性依赖于上下文，可能不可求值
        self.evaluable = false;
        Ok(())
    }

    fn visit_input_property(&mut self, _prop: &str) -> Self::Result {
        // 输入属性依赖于上下文，可能不可求值
        self.evaluable = false;
        Ok(())
    }

    fn visit_variable_property(&mut self, _var: &str, _prop: &str) -> Self::Result {
        // 变量属性依赖于上下文，可能不可求值
        self.evaluable = false;
        Ok(())
    }

    fn visit_source_property(&mut self, _tag: &str, _prop: &str) -> Self::Result {
        // 源属性依赖于上下文，可能不可求值
        self.evaluable = false;
        Ok(())
    }

    fn visit_destination_property(&mut self, _tag: &str, _prop: &str) -> Self::Result {
        // 目标属性依赖于上下文，可能不可求值
        self.evaluable = false;
        Ok(())
    }
}

impl QueryVisitor for EvaluableExprVisitor {
    type QueryResult = bool;

    fn get_result(&self) -> Self::QueryResult {
        self.evaluable
    }
    
    fn reset(&mut self) {
        self.evaluable = true;
        self.error = None;
    }
    
    fn is_success(&self) -> bool {
        self.error.is_none()
    }
}
