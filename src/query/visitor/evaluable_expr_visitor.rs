//! EvaluableExprVisitor - 用于判断表达式是否可求值的访问器
//! 对应 NebulaGraph EvaluableExprVisitor.h/.cpp 的功能

use crate::graph::expression::expr_type::Expression;

pub struct EvaluableExprVisitor {
    /// 表达式是否可求值
    evaluable: bool,
    /// 错误信息
    error: Option<String>,
}

impl EvaluableExprVisitor {
    pub fn new() -> Self {
        Self {
            evaluable: true,
            error: None,
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
            Expression::Constant(_) => Ok(()),

            // 变量表达式依赖于上下文，可能不可求值
            Expression::Property(_name) => {
                // 在当前实现中，如果表达式包含变量，则可能是不可求值的
                // 在实际实现中，需要检查该变量是否在当前上下文中被定义
                self.evaluable = false;
                Ok(())
            }

            // 算术表达式 - 如果所有子表达式都可求值，则该表达式可求值
            Expression::UnaryOp(_, operand) => self.visit(operand),

            Expression::BinaryOp(left, _, right) => {
                self.visit(left)?;
                self.visit(right)
            }

            // 函数调用 - 内置函数如果参数可求值则可求值
            Expression::Function(_, args) => {
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
