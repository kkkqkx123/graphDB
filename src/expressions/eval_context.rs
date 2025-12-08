use std::collections::HashMap;
use crate::graph::expression::Expression;
use crate::core::Value;
use crate::expressions::base::{EvaluationError, ExpressionContext as BaseExpressionContext, DefaultExpressionContext};

/// 简化的表达式上下文实现，仅包含变量存储
/// 这个版本适用于只需要变量访问而不需要图数据库特定功能的场景
/// 它实现了与基础 ExpressionContext trait 兼容的接口
#[derive(Default, Debug, Clone)]
pub struct SimpleExpressionContext {
    variables: HashMap<String, Value>,
}

impl SimpleExpressionContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// 设置变量值
    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    /// 获取变量值
    pub fn get_variable_direct(&self, name: &str) -> Result<Value, EvaluationError> {
        self.variables.get(name).cloned()
            .ok_or_else(|| EvaluationError::UndefinedVariable(name.to_string()))
    }

    /// 批量设置变量
    pub fn set_variables(&mut self, vars: HashMap<String, Value>) {
        self.variables = vars;
    }

    /// 检查变量是否存在
    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// 获取所有变量名
    pub fn variable_names(&self) -> Vec<String> {
        self.variables.keys().cloned().collect()
    }

    /// 清除所有变量
    pub fn clear(&mut self) {
        self.variables.clear();
    }

    /// 从 DefaultExpressionContext 创建 SimpleExpressionContext
    /// 注意：这只转换变量部分，不包含图数据库特定功能
    pub fn from_default_context(base_context: &DefaultExpressionContext) -> Self {
        // 由于 DefaultExpressionContext 的 variables 字段是私有的，
        // 这里提供一个空的实现，实际使用时可能需要在 DefaultExpressionContext 中添加获取变量的方法
        Self::new()
    }
}

/// 实现基础的 ExpressionContext trait，使其与现有表达式求值系统兼容
impl BaseExpressionContext for SimpleExpressionContext {
    fn get_variable(&self, name: &str) -> Result<Value, EvaluationError> {
        self.get_variable_direct(name)
    }

    // 对于图数据库特定的方法，返回默认错误
    fn get_tag_property(&self, _tag: &str, _property: &str) -> Result<Value, EvaluationError> {
        Err(EvaluationError::Other("SimpleExpressionContext does not support tag properties".to_string()))
    }

    fn get_edge_property(&self, _edge: &str, _property: &str) -> Result<Value, EvaluationError> {
        Err(EvaluationError::Other("SimpleExpressionContext does not support edge properties".to_string()))
    }

    fn get_src_vertex(&self) -> Result<Value, EvaluationError> {
        Err(EvaluationError::Other("SimpleExpressionContext does not support vertex operations".to_string()))
    }

    fn get_dst_vertex(&self) -> Result<Value, EvaluationError> {
        Err(EvaluationError::Other("SimpleExpressionContext does not support vertex operations".to_string()))
    }

    fn get_current_vertex(&self) -> Result<Value, EvaluationError> {
        Err(EvaluationError::Other("SimpleExpressionContext does not support vertex operations".to_string()))
    }

    fn get_current_edge(&self) -> Result<Value, EvaluationError> {
        Err(EvaluationError::Other("SimpleExpressionContext does not support edge operations".to_string()))
    }
}

/// 表达式求值器，提供表达式求值的高级接口
/// 使用 SimpleExpressionContext 作为底层实现
pub struct SimpleExpressionEvaluator {
    context: SimpleExpressionContext,
}

impl SimpleExpressionEvaluator {
    /// 创建新的表达式求值器
    pub fn new() -> Self {
        Self {
            context: SimpleExpressionContext::new(),
        }
    }

    /// 使用给定的上下文创建表达式求值器
    pub fn with_context(context: SimpleExpressionContext) -> Self {
        Self { context }
    }

    /// 设置变量值
    pub fn set_variable(&mut self, name: String, value: Value) {
        self.context.set_variable(name, value);
    }

    /// 求值表达式
    pub fn evaluate(&self, expression: &Expression) -> Result<Value, EvaluationError> {
        expression.eval(&self.context)
    }

    /// 获取上下文的引用
    pub fn context(&self) -> &SimpleExpressionContext {
        &self.context
    }

    /// 获取上下文的可变引用
    pub fn context_mut(&mut self) -> &mut SimpleExpressionContext {
        &mut self.context
    }
}

impl Default for SimpleExpressionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

// 为了向后兼容，提供一个类型别名
pub type ExpressionEvaluator = SimpleExpressionEvaluator;

// 重新导出基础模块中的类型，以保持向后兼容性
pub use crate::expressions::base::ExpressionContext;