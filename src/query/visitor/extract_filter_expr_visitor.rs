//! ExtractFilterExprVisitor - 用于提取过滤表达式的访问器
//! 对应 NebulaGraph ExtractFilterExprVisitor.h/.cpp 的功能

use crate::expression::Expression;

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

fn is_filter_expression(expr: &Expression) -> bool {
    // 检查表达式是否为过滤表达式
    // 通常关系表达式和函数调用是过滤表达式
    matches!(
        expr,
        Expression::Binary { .. } | Expression::Function { .. }
    )
}
