//! ExtractFilterExprVisitor - 用于提取过滤表达式的访问器
//! 对应 NebulaGraph ExtractFilterExprVisitor.h/.cpp 的功能

use crate::core::visitor::{VisitorCore, VisitorContext, VisitorResult};
use crate::core::{Expression, ExpressionVisitor, LiteralValue, BinaryOperator, UnaryOperator, AggregateFunction, DataType};
use crate::query::visitor::QueryVisitor;

#[derive(Debug)]
pub struct ExtractFilterExprVisitor {
    /// 提取到的过滤表达式
    filter_exprs: Vec<Expression>,
    /// 是否只提取顶层的过滤条件
    top_level_only: bool,
    /// 当前是否在顶层
    is_top_level: bool,
    /// 访问器上下文
    context: VisitorContext,
    /// 访问器状态
    state: crate::core::visitor::visitor_state_enum::VisitorStateEnum,
}

impl Clone for ExtractFilterExprVisitor {
    fn clone(&self) -> Self {
        Self {
            filter_exprs: self.filter_exprs.clone(),
            top_level_only: self.top_level_only,
            is_top_level: self.is_top_level,
            context: self.context.clone(),
            state: crate::core::visitor::visitor_state_enum::VisitorStateEnum::new(),
        }
    }
}

impl ExtractFilterExprVisitor {
    pub fn new(top_level_only: bool) -> Self {
        Self {
            filter_exprs: Vec::new(),
            top_level_only: top_level_only,
            is_top_level: true,
            context: VisitorContext::new(crate::core::visitor::VisitorConfig::new()),
            state: crate::core::visitor::visitor_state_enum::VisitorStateEnum::new(),
        }
    }

    /// 创建带初始深度的 ExtractFilterExprVisitor
    pub fn with_depth(top_level_only: bool, depth: usize) -> Self {
        Self {
            filter_exprs: Vec::new(),
            top_level_only: top_level_only,
            is_top_level: true,
            context: VisitorContext::new(crate::core::visitor::VisitorConfig::new()),
            state: crate::core::visitor::visitor_state_enum::VisitorStateEnum::with_depth(depth),
        }
    }

    /// 创建带配置的 ExtractFilterExprVisitor
    pub fn with_config(top_level_only: bool, config: crate::core::visitor::VisitorConfig) -> Self {
        Self {
            filter_exprs: Vec::new(),
            top_level_only: top_level_only,
            is_top_level: true,
            context: VisitorContext::new(config),
            state: crate::core::visitor::visitor_state_enum::VisitorStateEnum::new(),
        }
    }

    /// 创建带配置和初始深度的 ExtractFilterExprVisitor
    pub fn with_config_and_depth(
        top_level_only: bool,
        config: crate::core::visitor::VisitorConfig,
        depth: usize
    ) -> Self {
        Self {
            filter_exprs: Vec::new(),
            top_level_only: top_level_only,
            is_top_level: true,
            context: VisitorContext::new(config),
            state: crate::core::visitor::visitor_state_enum::VisitorStateEnum::with_depth(depth),
        }
    }

    pub fn extract(&mut self, expr: &Expression) -> Result<Vec<Expression>, String> {
        self.filter_exprs.clear();
        self.is_top_level = true;
        self.visit(expr)?;
        Ok(self.filter_exprs.clone())
    }

    fn visit(&mut self, expr: &Expression) -> Result<(), String> {
        // 简化实现：将所有二元操作符表达式视为过滤表达式
        match expr {
            // AND操作通常包含多个过滤条件
            Expression::Binary { left, op: _, right } => {
                if self.is_top_level || !self.top_level_only {
                    // 如果在顶层，或者不只提取顶层，则继续遍历子表达式
                    self.visit_with_updated_level(left)?;
                    self.visit_with_updated_level(right)?;
                } else {
                    // 如果不在顶层且只提取顶层，则将整个表达式作为一个过滤条件
                    self.filter_exprs.push(expr.clone());
                }
                Ok(())
            }

            // 函数调用，检查是否是过滤相关的函数
            Expression::Function { name, args: _ } => {
                // 某些函数可能用于过滤，如 is_empty, is_null 等
                if is_filter_function(name) {
                    if self.is_top_level || !self.top_level_only {
                        self.filter_exprs.push(expr.clone());
                    }
                }
                Ok(())
            }

            // 处理其他可能的过滤表达式
            _ => {
                // 检查是否为其他类型的过滤表达式
                if self.is_top_level || !self.top_level_only {
                    if is_filter_expression(expr) {
                        self.filter_exprs.push(expr.clone());
                    }
                }
                self.visit_children(expr)
            }
        }
    }

    fn visit_with_updated_level(&mut self, expr: &Expression) -> Result<(), String> {
        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        let result = self.visit(expr);
        self.is_top_level = old_top_level;
        result
    }

    fn visit_children(&mut self, expr: &Expression) -> Result<(), String> {
        match expr {
            Expression::Unary { op: _, operand } => self.visit(operand),
            Expression::Binary { left, op: _, right } => {
                self.visit(left)?;
                self.visit(right)
            }
            Expression::Function { name: _, args } => {
                for arg in args {
                    self.visit(arg)?;
                }
                Ok(())
            }
            // 其他表达式类型，通常不需要进一步访问子节点
            _ => Ok(()),
        }
    }

    pub fn get_filter_exprs(&self) -> &Vec<Expression> {
        &self.filter_exprs
    }
}

fn is_filter_function(func_name: &str) -> bool {
    // 检查函数名是否为过滤相关函数
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

impl VisitorCore<Expression> for ExtractFilterExprVisitor {
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
                let default_cloned = default.map(|b| (**b).clone());
                self.visit_case(conditions, &default_cloned)
            }
            Expression::TypeCast { expr, target_type } => self.visit_type_cast(expr, target_type),
            Expression::Subscript { collection, index } => self.visit_subscript(collection, index),
            Expression::Range { collection, start, end } => {
                let start_cloned = start.map(|b| (**b).clone());
                let end_cloned = end.map(|b| (**b).clone());
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
                let start_cloned = start.map(|b| (**b).clone());
                let end_cloned = end.map(|b| (**b).clone());
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

impl ExpressionVisitor for ExtractFilterExprVisitor {
    fn visit_literal(&mut self, _value: &LiteralValue) -> Self::Result {
        // 字面量不是过滤表达式
        Ok(())
    }

    fn visit_variable(&mut self, _name: &str) -> Self::Result {
        // 变量不是过滤表达式
        Ok(())
    }

    fn visit_property(&mut self, object: &Expression, property: &str) -> Self::Result {
        // 属性访问可能是过滤表达式的一部分
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::Property {
                object: Box::new(object.clone()),
                property: property.to_string(),
            });
        }
        self.visit(object)?;
        Ok(())
    }

    fn visit_binary(&mut self, left: &Expression, op: &BinaryOperator, right: &Expression) -> Self::Result {
        // 二元操作通常是过滤表达式
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::Binary {
                left: Box::new(left.clone()),
                op: op.clone(),
                right: Box::new(right.clone()),
            });
        }
        
        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(left)?;
        self.visit(right)?;
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_unary(&mut self, op: &UnaryOperator, operand: &Expression) -> Self::Result {
        // 一元操作可能是过滤表达式的一部分
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::Unary {
                op: op.clone(),
                operand: Box::new(operand.clone()),
            });
        }
        
        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(operand)?;
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_function(&mut self, name: &str, args: &[Expression]) -> Self::Result {
        // 某些函数可能是过滤表达式
        if is_filter_function(name) && (self.is_top_level || !self.top_level_only) {
            self.filter_exprs.push(Expression::Function {
                name: name.to_string(),
                args: args.to_vec(),
            });
        }
        
        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        for arg in args {
            self.visit(arg)?;
        }
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_aggregate(&mut self, func: &AggregateFunction, arg: &Expression, _distinct: bool) -> Self::Result {
        // 聚合函数通常不是过滤表达式
        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(arg)?;
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_list(&mut self, items: &[Expression]) -> Self::Result {
        // 列表不是过滤表达式
        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        for item in items {
            self.visit(item)?;
        }
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
        // 映射不是过滤表达式
        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        for (_, value) in pairs {
            self.visit(value)?;
        }
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_case(&mut self, conditions: &[(Expression, Expression)], default: &Option<Expression>) -> Self::Result {
        // CASE表达式可能是过滤表达式
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::Case {
                conditions: conditions.to_vec(),
                default: default.as_ref().map(|e| Box::new(e.clone())),
            });
        }
        
        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        for (condition, value) in conditions {
            self.visit(condition)?;
            self.visit(value)?;
        }
        if let Some(default_expr) = default {
            self.visit(default_expr)?;
        }
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_type_cast(&mut self, expr: &Expression, target_type: &DataType) -> Self::Result {
        // 类型转换可能是过滤表达式的一部分
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::TypeCast {
                expr: Box::new(expr.clone()),
                target_type: target_type.clone(),
            });
        }
        
        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(expr)?;
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result {
        // 下标访问可能是过滤表达式的一部分
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::Subscript {
                collection: Box::new(collection.clone()),
                index: Box::new(index.clone()),
            });
        }
        
        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(collection)?;
        self.visit(index)?;
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_range(&mut self, collection: &Expression, start: &Option<Expression>, end: &Option<Expression>) -> Self::Result {
        // 范围访问可能是过滤表达式的一部分
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::Range {
                collection: Box::new(collection.clone()),
                start: start.as_ref().map(|e| Box::new(e.clone())),
                end: end.as_ref().map(|e| Box::new(e.clone())),
            });
        }
        
        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        self.visit(collection)?;
        if let Some(start_expr) = start {
            self.visit(start_expr)?;
        }
        if let Some(end_expr) = end {
            self.visit(end_expr)?;
        }
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_path(&mut self, items: &[Expression]) -> Self::Result {
        // 路径表达式可能是过滤表达式的一部分
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::Path(items.to_vec()));
        }
        
        let old_top_level = self.is_top_level;
        self.is_top_level = false;
        for item in items {
            self.visit(item)?;
        }
        self.is_top_level = old_top_level;
        Ok(())
    }

    fn visit_label(&mut self, name: &str) -> Self::Result {
        // 标签不是过滤表达式
        Ok(())
    }

    fn visit_tag_property(&mut self, tag: &str, prop: &str) -> Self::Result {
        // 标签属性可能是过滤表达式的一部分
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::TagProperty {
                tag: tag.to_string(),
                prop: prop.to_string(),
            });
        }
        Ok(())
    }

    fn visit_edge_property(&mut self, edge: &str, prop: &str) -> Self::Result {
        // 边属性可能是过滤表达式的一部分
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::EdgeProperty {
                edge: edge.to_string(),
                prop: prop.to_string(),
            });
        }
        Ok(())
    }

    fn visit_input_property(&mut self, prop: &str) -> Self::Result {
        // 输入属性可能是过滤表达式的一部分
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::InputProperty(prop.to_string()));
        }
        Ok(())
    }

    fn visit_variable_property(&mut self, var: &str, prop: &str) -> Self::Result {
        // 变量属性可能是过滤表达式的一部分
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::VariableProperty {
                var: var.to_string(),
                prop: prop.to_string(),
            });
        }
        Ok(())
    }

    fn visit_source_property(&mut self, tag: &str, prop: &str) -> Self::Result {
        // 源属性可能是过滤表达式的一部分
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::SourceProperty {
                tag: tag.to_string(),
                prop: prop.to_string(),
            });
        }
        Ok(())
    }

    fn visit_destination_property(&mut self, tag: &str, prop: &str) -> Self::Result {
        // 目标属性可能是过滤表达式的一部分
        if self.is_top_level || !self.top_level_only {
            self.filter_exprs.push(Expression::DestinationProperty {
                tag: tag.to_string(),
                prop: prop.to_string(),
            });
        }
        Ok(())
    }
}

impl QueryVisitor for ExtractFilterExprVisitor {
    type QueryResult = Vec<Expression>;

    fn get_result(&self) -> Self::QueryResult {
        self.filter_exprs.clone()
    }
    
    fn reset(&mut self) {
        self.filter_exprs.clear();
        self.is_top_level = true;
    }
    
    fn is_success(&self) -> bool {
        true // ExtractFilterExprVisitor 总是成功，即使没有找到任何过滤表达式
    }
}

fn is_filter_expression(expr: &Expression) -> bool {
    // 检查表达式是否为过滤表达式
    // 通常关系表达式和函数调用是过滤表达式
    matches!(
        expr,
        Expression::Binary { .. } | Expression::Function { .. }
    )
}
