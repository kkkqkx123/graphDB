//! 表达式求值模块
//!
//! 提供表达式求值过程中的选项配置和统计信息

use serde::{Deserialize, Serialize};

/// 表达式求值选项
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationOptions {
    /// 是否启用严格模式
    pub strict_mode: bool,
    /// 是否允许隐式类型转换
    pub allow_implicit_conversion: bool,
    /// 最大递归深度
    pub max_recursion_depth: usize,
    /// 超时时间（毫秒）
    pub timeout_ms: Option<u64>,
}

impl Default for EvaluationOptions {
    fn default() -> Self {
        Self {
            strict_mode: false,
            allow_implicit_conversion: true,
            max_recursion_depth: 1000,
            timeout_ms: Some(30000), // 30秒
        }
    }
}

/// 表达式求值统计
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationStatistics {
    /// 求值的表达式数量
    pub expressions_evaluated: usize,
    /// 函数调用次数
    pub function_calls: usize,
    /// 变量访问次数
    pub variable_accesses: usize,
    /// 总求值时间（微秒）
    pub total_evaluation_time_us: u64,
    /// 平均求值时间（微秒）
    pub average_evaluation_time_us: f64,
    /// 最大递归深度
    pub max_recursion_depth: usize,
}

impl EvaluationStatistics {
    /// 创建新的求值统计
    pub fn new() -> Self {
        Self {
            expressions_evaluated: 0,
            function_calls: 0,
            variable_accesses: 0,
            total_evaluation_time_us: 0,
            average_evaluation_time_us: 0.0,
            max_recursion_depth: 0,
        }
    }

    /// 记录表达式求值
    pub fn record_expression_evaluation(&mut self, evaluation_time_us: u64) {
        self.expressions_evaluated += 1;
        self.total_evaluation_time_us += evaluation_time_us;
        self.average_evaluation_time_us =
            self.total_evaluation_time_us as f64 / self.expressions_evaluated as f64;
    }

    /// 记录函数调用
    pub fn record_function_call(&mut self) {
        self.function_calls += 1;
    }

    /// 记录变量访问
    pub fn record_variable_access(&mut self) {
        self.variable_accesses += 1;
    }

    /// 更新最大递归深度
    pub fn update_max_recursion_depth(&mut self, depth: usize) {
        if depth > self.max_recursion_depth {
            self.max_recursion_depth = depth;
        }
    }
}

impl Default for EvaluationStatistics {
    fn default() -> Self {
        Self::new()
    }
}
