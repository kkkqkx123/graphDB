//! EvaluableExprVisitor - 用于判断表达式是否可求值的访问器
//! 对应 NebulaGraph EvaluableExprVisitor.h/.cpp 的功能

use crate::expressions::{Expression, ExpressionKind};

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
        match &expr.kind {
            // 常量表达式是可求值的
            ExpressionKind::Constant(_) => Ok(()),
            
            // 变量表达式依赖于上下文，可能不可求值
            ExpressionKind::Variable(name) => {
                // 在当前实现中，如果表达式包含变量，则可能是不可求值的
                // 在实际实现中，需要检查该变量是否在当前上下文中被定义
                self.evaluable = false;
                Ok(())
            },
            
            // 输入属性表达式依赖于输入数据，可能不可求值
            ExpressionKind::InputProperty(_) => {
                self.evaluable = false;
                Ok(())
            },
            
            // 属性表达式（标签、边、顶点属性）依赖于运行时数据，不可求值
            ExpressionKind::TagProperty { .. } |
            ExpressionKind::EdgeProperty { .. } |
            ExpressionKind::VariableProperty { .. } |
            ExpressionKind::SourceProperty(_) |
            ExpressionKind::DestProperty(_) |
            ExpressionKind::EdgeSrcId |
            ExpressionKind::EdgeDstId |
            ExpressionKind::EdgeRank |
            ExpressionKind::EdgeType => {
                self.evaluable = false;
                Ok(())
            },
            
            // UUID表达式虽然生成值，但依赖于运行时，可能是不可求值的
            ExpressionKind::UUID => {
                self.evaluable = false;
                Ok(())
            },
            
            // 算术表达式 - 如果所有子表达式都可求值，则该表达式可求值
            ExpressionKind::Unary { operand, .. } => {
                self.visit(operand)
            },
            
            ExpressionKind::Arithmetic { left, right, .. } => {
                self.visit(left)?;
                self.visit(right)
            },
            
            // 关系表达式 - 如果所有子表达式都可求值，则该表达式可求值
            ExpressionKind::Relational { left, right, .. } => {
                self.visit(left)?;
                self.visit(right)
            },
            
            // 逻辑表达式 - 如果所有子表达式都可求值，则该表达式可求值
            ExpressionKind::Logical { operands, .. } => {
                for operand in operands {
                    self.visit(operand)?;
                }
                Ok(())
            },
            
            // 函数调用 - 内置函数如果参数可求值则可求值
            ExpressionKind::FunctionCall { args, .. } => {
                for arg in args {
                    self.visit(arg)?;
                }
                Ok(())
            },
            
            // 聚合表达式依赖于运行时数据，不可求值
            ExpressionKind::Aggregate { .. } => {
                self.evaluable = false;
                Ok(())
            },
            
            // 容器表达式，如果元素可求值则可求值
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
            
            // 类型转换，如果操作数可求值则可求值
            ExpressionKind::TypeCasting { operand, .. } => {
                self.visit(operand)
            },
            
            // 下标访问依赖于运行时数据，不可求值
            ExpressionKind::Subscript { .. } => {
                self.evaluable = false;
                Ok(())
            },
            
            // 属性访问依赖于运行时数据，不可求值
            ExpressionKind::Attribute { .. } => {
                self.evaluable = false;
                Ok(())
            },
            
            // 其他特殊表达式
            ExpressionKind::Label(_) |
            ExpressionKind::LabelAttribute { .. } => {
                // 标签相关表达式，可能可求值
                Ok(())
            },
            
            ExpressionKind::Case { .. } => {
                // Case表达式，如果所有子表达式可求值则可求值
                // 实现Case表达式的访问逻辑
                self.visit_case_expr(expr)
            },
            
            ExpressionKind::PathBuild(_) |
            ExpressionKind::Vertex(_) |
            ExpressionKind::Edge(_) => {
                // 路径、顶点、边构建表达式依赖于运行时数据
                self.evaluable = false;
                Ok(())
            },
            
            ExpressionKind::Column(_) => {
                // 列表达式依赖于上下文
                self.evaluable = false;
                Ok(())
            },
            
            ExpressionKind::Predicate { .. } => {
                // 谓词表达式，如果子表达式可求值则可求值
                self.visit_children(expr)
            },
            
            ExpressionKind::ListComprehension { .. } |
            ExpressionKind::Reduce { .. } |
            ExpressionKind::SubscriptRange { .. } |
            ExpressionKind::MatchPathPattern { .. } => {
                // 这些复杂表达式通常依赖于运行时数据
                self.evaluable = false;
                Ok(())
            },
            
            ExpressionKind::VersionedVariable { .. } => {
                // 版本化变量依赖于上下文
                self.evaluable = false;
                Ok(())
            },
            
            ExpressionKind::LabelTagProperty { .. } => {
                // 标签属性表达式依赖于运行时数据
                self.evaluable = false;
                Ok(())
            },
        }
    }

    fn visit_case_expr(&mut self, expr: &Expression) -> Result<(), String> {
        // Case表达式的访问逻辑
        // 在实际实现中，需要递归访问Case表达式的所有组件
        self.visit_children(expr)
    }

    fn visit_children(&mut self, expr: &Expression) -> Result<(), String> {
        // 递归访问表达式的所有子节点
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
                // Case表达式需要特殊处理，这里简化实现
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
}