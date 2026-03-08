//! 优化器错误类型
//!
//! 定义查询优化器相关的错误类型，包括：
//! - 代价计算错误
//! - 计划优化错误
//! - 统计信息错误
//! - 规则应用错误

use thiserror::Error;

/// 优化器错误类型
#[derive(Error, Debug, Clone)]
pub enum OptimizeError {
    /// 不支持的节点类型
    #[error("Unsupported node type: {0}")]
    UnsupportedNodeType(String),

    /// 缺少统计信息
    #[error("Missing statistics: {0}")]
    MissingStatistics(String),

    /// 代价计算错误
    #[error("Calculation error: {0}")]
    CalculationError(String),

    /// 计划优化错误
    #[error("Plan optimization error: {0}")]
    PlanOptimizationError(String),

    /// 规则应用错误
    #[error("Rule application error: {0}")]
    RuleApplicationError(String),

    /// 统计信息错误
    #[error("Statistics error: {0}")]
    StatisticsError(String),

    /// 索引选择错误
    #[error("Index selection error: {0}")]
    IndexSelectionError(String),

    /// 连接顺序优化错误
    #[error("Join order optimization error: {0}")]
    JoinOrderError(String),

    /// 表达式转换错误
    #[error("Expression transform error: {0}")]
    ExpressionTransformError(String),

    /// 内部优化错误
    #[error("Internal optimization error: {0}")]
    InternalError(String),
}

/// 优化器结果类型
pub type OptimizeResult<T> = Result<T, OptimizeError>;

/// Cost calculation related errors
#[derive(Error, Debug, Clone)]
pub enum CostError {
    /// Unsupported node type
    #[error("Unsupported node type: {0}")]
    UnsupportedNodeType(String),

    /// Missing statistics
    #[error("Missing statistics: {0}")]
    MissingStatistics(String),

    /// Calculation error
    #[error("Calculation error: {0}")]
    CalculationError(String),
}

/// 代价计算结果类型
pub type CostResult<T> = Result<T, CostError>;

impl From<CostError> for OptimizeError {
    fn from(err: CostError) -> Self {
        match err {
            CostError::UnsupportedNodeType(msg) => OptimizeError::UnsupportedNodeType(msg),
            CostError::MissingStatistics(msg) => OptimizeError::MissingStatistics(msg),
            CostError::CalculationError(msg) => OptimizeError::CalculationError(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimize_error_display() {
        let err = OptimizeError::UnsupportedNodeType("UnknownNode".to_string());
        assert!(err.to_string().contains("Unsupported node type"));

        let err = OptimizeError::MissingStatistics("vertex_count".to_string());
        assert!(err.to_string().contains("Missing statistics"));
    }

    #[test]
    fn test_cost_error_conversion() {
        let cost_err = CostError::CalculationError("division by zero".to_string());
        let opt_err: OptimizeError = cost_err.into();
        assert!(matches!(opt_err, OptimizeError::CalculationError(_)));
    }
}
