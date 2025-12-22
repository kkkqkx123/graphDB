//! 表达式求值器特征定义
//!
//! 定义表达式求值器的核心接口和特征

use crate::core::types::expression::Expression;
use crate::core::Value;
use crate::core::ExpressionError;

/// 表达式求值器核心特征
pub trait Evaluator {
    /// 求值表达式
    fn evaluate(&self, expr: &Expression, context: &dyn ExpressionContext) -> Result<Value, ExpressionError>;
    
    /// 批量求值表达式
    fn evaluate_batch(&self, expressions: &[Expression], context: &dyn ExpressionContext) -> Result<Vec<Value>, ExpressionError> {
        let mut results = Vec::with_capacity(expressions.len());
        for expr in expressions {
            results.push(self.evaluate(expr, context)?);
        }
        Ok(results)
    }
    
    /// 检查表达式是否可以求值
    fn can_evaluate(&self, expr: &Expression, context: &dyn ExpressionContext) -> bool {
        true // 默认实现：所有表达式都可以求值
    }
    
    /// 获取求值器名称
    fn name(&self) -> &str;
    
    /// 获取求值器描述
    fn description(&self) -> &str;
    
    /// 获取求值器版本
    fn version(&self) -> &str;
}

/// 表达式上下文特征
pub trait ExpressionContext {
    /// 获取变量值
    fn get_variable(&self, name: &str) -> Option<Value>;
    
    /// 设置变量值
    fn set_variable(&mut self, name: String, value: Value);
    
    /// 获取所有变量名
    fn get_variable_names(&self) -> Vec<&str>;
    
    /// 检查变量是否存在
    fn has_variable(&self, name: &str) -> bool {
        self.get_variable(name).is_some()
    }
    
    /// 获取上下文深度
    fn get_depth(&self) -> usize {
        0 // 默认实现
    }
}