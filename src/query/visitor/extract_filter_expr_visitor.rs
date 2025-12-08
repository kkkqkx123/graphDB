//! ExtractFilterExprVisitor - 用于提取过滤表达式的访问器
//! 对应 NebulaGraph ExtractFilterExprVisitor.h/.cpp 的功能

use crate::expressions::{Expression, ExpressionKind};

#[derive(Debug, Clone)]
pub struct ExtractFilterExprVisitor {
    /// 提取到的过滤表达式
    filter_exprs: Vec<Expression>,
    /// 是否只提取顶层的过滤条件
    top_level_only: bool,
    /// 当前是否在顶层
    is_top_level: bool,
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
        match &expr.kind {
            // AND操作通常包含多个过滤条件
            ExpressionKind::Logical { op, operands } if op == "And" || op == "LogicalAnd" => {
                if self.is_top_level || !self.top_level_only {
                    // 如果在顶层，或者不只提取顶层，则继续遍历AND操作的子表达式
                    for operand in operands {
                        self.visit_with_updated_level(operand)?;
                    }
                } else {
                    // 如果不在顶层且只提取顶层，则将整个AND表达式作为一个过滤条件
                    self.filter_exprs.push(expr.clone());
                }
                Ok(())
            },
            // OR操作通常用于分支条件，可能不是纯粹的过滤条件
            ExpressionKind::Logical { op, .. } if op == "Or" || op == "LogicalOr" => {
                if !self.top_level_only {
                    // 除非只提取顶层，否则也要处理OR操作
                    // 但OR操作通常不被视为简单的过滤条件
                    self.filter_exprs.push(expr.clone());
                } else if self.is_top_level {
                    self.filter_exprs.push(expr.clone());
                }
                Ok(())
            },
            // 简单的关系表达式通常作为过滤条件
            ExpressionKind::Relational { .. } => {
                if self.is_top_level || !self.top_level_only {
                    self.filter_exprs.push(expr.clone());
                }
                Ok(())
            },
            // 函数调用，检查是否是过滤相关的函数
            ExpressionKind::FunctionCall { name, .. } => {
                // 某些函数可能用于过滤，如 is_empty, is_null 等
                if is_filter_function(name) {
                    if self.is_top_level || !self.top_level_only {
                        self.filter_exprs.push(expr.clone());
                    }
                }
                Ok(())
            },
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
        match &expr.kind {
            ExpressionKind::Unary { operand, .. } => {
                self.visit(operand)
            },
            ExpressionKind::Arithmetic { left, right, .. } => {
                self.visit(left)?;
                self.visit(right)
            },
            ExpressionKind::Relational { left, right, .. } => {
                self.visit(left)?;
                self.visit(right)
            },
            ExpressionKind::Logical { operands, .. } => {
                for operand in operands {
                    self.visit(operand)?;
                }
                Ok(())
            },
            ExpressionKind::Subscript { left, right } => {
                self.visit(left)?;
                self.visit(right)
            },
            ExpressionKind::Attribute { left, right } => {
                self.visit(left)?;
                self.visit(right)
            },
            ExpressionKind::FunctionCall { args, .. } => {
                for arg in args {
                    self.visit(arg)?;
                }
                Ok(())
            },
            ExpressionKind::Aggregate { arg, .. } => {
                self.visit(arg)
            },
            ExpressionKind::List(items) => {
                for item in items {
                    self.visit(item)?;
                }
                Ok(())
            },
            ExpressionKind::Set(items) => {
                for item in items {
                    self.visit(item)?;
                }
                Ok(())
            },
            ExpressionKind::Map(kvs) => {
                for (k, v) in kvs {
                    self.visit(k)?;
                    self.visit(v)?;
                }
                Ok(())
            },
            ExpressionKind::Case { .. } => {
                // Case表达式处理
                Ok(())
            },
            ExpressionKind::Reduce { .. } => {
                // Reduce表达式处理
                Ok(())
            },
            ExpressionKind::ListComprehension { .. } => {
                // 列表推导式处理
                Ok(())
            },
            // 其他表达式类型，通常不需要进一步访问子节点
            _ => Ok(()),
        }
    }

    pub fn get_filter_exprs(&self) -> &Vec<Expression> {
        &self.filter_exprs
    }
}

fn is_filter_function(func_name: &String) -> bool {
    // 检查函数名是否为过滤相关函数
    matches!(func_name.as_str().to_lowercase().as_str(), 
             "isempty" | "isnull" | "isnotnull" | "isnullorempty" | 
             "has" | "haslabel" | "hastag" | "contains")
}

fn is_filter_expression(expr: &Expression) -> bool {
    // 检查表达式是否为过滤表达式
    // 通常关系表达式和函数调用是过滤表达式
    matches!(expr.kind,
             ExpressionKind::Relational { .. } |
             ExpressionKind::FunctionCall { .. } |
             ExpressionKind::Predicate { .. } |
             ExpressionKind::Unary { op, .. } if op == "IsNull" || op == "IsNotNull" || op == "IsEmpty" || op == "IsNotEmpty")
}