//! ExtractFilterExprVisitor - 用于提取过滤表达式的访问器
//! 对应 NebulaGraph ExtractFilterExprVisitor.h/.cpp 的功能

use crate::core::{
    AggregateFunction, BinaryOperator, DataType, Expression, UnaryOperator,
};
use crate::core::Value;
use crate::core::visitor::{Visitor, VisitorState};

#[derive(Debug)]
pub struct ExtractFilterExprVisitor {
    /// 提取到的过滤表达式
    filter_exprs: Vec<Expression>,
    /// 是否只提取顶层的过滤条件
    top_level_only: bool,
    /// 当前是否在顶层
    is_top_level: bool,
}

impl Clone for ExtractFilterExprVisitor {
    fn clone(&self) -> Self {
        Self {
            filter_exprs: self.filter_exprs.clone(),
            top_level_only: self.top_level_only,
            is_top_level: self.is_top_level,
        }
    }
}

impl ExtractFilterExprVisitor {
    pub fn new(top_level_only: bool) -> Self {
        Self {
            filter_exprs: Vec::new(),
            top_level_only: top_level_only,
            is_top_level: true,
        }
    }

    pub fn extract(&mut self, expr: &Expression) -> Result<Vec<Expression>, String> {
        self.filter_exprs.clear();
        self.is_top_level = true;
        self.visit(expr)?;
        Ok(self.filter_exprs.clone())
    }

    fn visit(&mut self, expr: &Expression) -> Result<(), String> {
        match expr {
            Expression::Binary { left, op: _, right } => {
                if self.is_top_level || !self.top_level_only {
                    self.visit_with_updated_level(left)?;
                    self.visit_with_updated_level(right)?;
                } else {
                    self.filter_exprs.push(expr.clone());
                }
                Ok(())
            }

            Expression::Function { name, args: _ } => {
                if is_filter_function(name) {
                    if self.is_top_level || !self.top_level_only {
                        self.filter_exprs.push(expr.clone());
                    }
                }
                Ok(())
            }

            _ => {
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
            _ => Ok(()),
        }
    }

    pub fn get_filter_exprs(&self) -> &Vec<Expression> {
        &self.filter_exprs
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

impl<'a> Visitor<Expression> for ExtractFilterExprVisitor {
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
        static EMPTY_STATE: VisitorState = VisitorState::new();
        &EMPTY_STATE
    }

    fn state_mut(&mut self) -> &mut VisitorState {
        static mut MUTABLE_STATE: VisitorState = VisitorState::new();
        unsafe { &mut MUTABLE_STATE }
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
