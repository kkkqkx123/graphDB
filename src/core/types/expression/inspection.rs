//! 表达式检查方法
//!
//! 提供检查表达式属性和状态的方法。

use crate::core::types::expression::Expression;
use crate::core::Value;

impl Expression {
    /// 检查表达式是否为常量
    ///
    /// 常量表达式在编译时即可确定值，不需要运行时求值。
    pub fn is_constant(&self) -> bool {
        match self {
            Expression::Literal(_) => true,
            Expression::List(items) => items.iter().all(|e| e.is_constant()),
            Expression::Map(pairs) => pairs.iter().all(|(_, e)| e.is_constant()),
            Expression::TagProperty { .. } => false,
            Expression::EdgeProperty { .. } => false,
            Expression::LabelTagProperty { .. } => false,
            _ => false,
        }
    }

    /// 检查表达式是否包含聚合函数
    ///
    /// 用于识别需要在 GROUP BY 上下文中求值的表达式。
    pub fn contains_aggregate(&self) -> bool {
        match self {
            Expression::Aggregate { .. } => true,
            _ => self.children().iter().any(|e| e.contains_aggregate()),
        }
    }

    /// 获取表达式中所有变量名
    ///
    /// 返回去重后的变量名列表。
    pub fn get_variables(&self) -> Vec<String> {
        let mut variables = Vec::new();
        self.collect_variables(&mut variables);
        variables.sort();
        variables.dedup();
        variables
    }

    /// 递归收集变量的辅助方法
    fn collect_variables(&self, variables: &mut Vec<String>) {
        match self {
            Expression::Variable(name) => {
                if !variables.contains(name) {
                    variables.push(name.clone());
                }
            }
            _ => {
                for child in self.children() {
                    child.collect_variables(variables);
                }
            }
        }
    }

    /// 检查是否为字面量表达式
    pub fn is_literal(&self) -> bool {
        matches!(self, Expression::Literal(_))
    }

    /// 获取字面量值（如果是字面量）
    pub fn as_literal(&self) -> Option<&Value> {
        match self {
            Expression::Literal(v) => Some(v),
            _ => None,
        }
    }

    /// 检查是否为变量表达式
    pub fn is_variable(&self) -> bool {
        matches!(self, Expression::Variable(_))
    }

    /// 获取变量名（如果是变量）
    pub fn as_variable(&self) -> Option<&str> {
        match self {
            Expression::Variable(name) => Some(name),
            _ => None,
        }
    }

    /// 检查是否为聚合表达式
    pub fn is_aggregate(&self) -> bool {
        matches!(self, Expression::Aggregate { .. })
    }

    /// 检查是否为属性访问表达式
    pub fn is_property(&self) -> bool {
        matches!(self, Expression::Property { .. })
    }

    /// 检查是否为函数调用表达式
    pub fn is_function(&self) -> bool {
        matches!(self, Expression::Function { .. })
    }

    /// 检查是否为二元运算表达式
    pub fn is_binary(&self) -> bool {
        matches!(self, Expression::Binary { .. })
    }

    /// 检查是否为一元运算表达式
    pub fn is_unary(&self) -> bool {
        matches!(self, Expression::Unary { .. })
    }

    /// 检查是否为列表表达式
    pub fn is_list(&self) -> bool {
        matches!(self, Expression::List(_))
    }

    /// 检查是否为映射表达式
    pub fn is_map(&self) -> bool {
        matches!(self, Expression::Map(_))
    }

    /// 检查是否为路径表达式
    pub fn is_path(&self) -> bool {
        matches!(self, Expression::Path(_))
    }

    /// 检查是否为标签表达式
    pub fn is_label(&self) -> bool {
        matches!(self, Expression::Label(_))
    }

    /// 检查是否为参数表达式
    pub fn is_parameter(&self) -> bool {
        matches!(self, Expression::Parameter(_))
    }

    /// 获取参数名（如果是参数）
    pub fn as_parameter(&self) -> Option<&str> {
        match self {
            Expression::Parameter(name) => Some(name),
            _ => None,
        }
    }

    /// 检查是否为条件表达式
    pub fn is_case(&self) -> bool {
        matches!(self, Expression::Case { .. })
    }

    /// 检查是否为类型转换表达式
    pub fn is_cast(&self) -> bool {
        matches!(self, Expression::TypeCast { .. })
    }

    /// 检查是否为下标访问表达式
    pub fn is_subscript(&self) -> bool {
        matches!(self, Expression::Subscript { .. })
    }

    /// 检查是否为范围表达式
    pub fn is_range(&self) -> bool {
        matches!(self, Expression::Range { .. })
    }

    /// 获取函数名（如果是函数调用）
    pub fn function_name(&self) -> Option<&str> {
        match self {
            Expression::Function { name, .. } => Some(name),
            _ => None,
        }
    }

    /// 获取聚合函数名（如果是聚合表达式）
    pub fn aggregate_function_name(&self) -> Option<&str> {
        match self {
            Expression::Aggregate { func, .. } => Some(func.name()),
            _ => None,
        }
    }
}
