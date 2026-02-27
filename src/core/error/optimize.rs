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
    #[error("不支持的节点类型: {0}")]
    UnsupportedNodeType(String),

    /// 缺少统计信息
    #[error("缺少统计信息: {0}")]
    MissingStatistics(String),

    /// 代价计算错误
    #[error("代价计算错误: {0}")]
    CalculationError(String),

    /// 计划优化错误
    #[error("计划优化错误: {0}")]
    PlanOptimizationError(String),

    /// 规则应用错误
    #[error("规则应用错误: {0}")]
    RuleApplicationError(String),

    /// 统计信息错误
    #[error("统计信息错误: {0}")]
    StatisticsError(String),

    /// 索引选择错误
    #[error("索引选择错误: {0}")]
    IndexSelectionError(String),

    /// 连接顺序优化错误
    #[error("连接顺序优化错误: {0}")]
    JoinOrderError(String),

    /// 表达式转换错误
    #[error("表达式转换错误: {0}")]
    ExpressionTransformError(String),

    /// 内部优化错误
    #[error("内部优化错误: {0}")]
    InternalError(String),
}

/// 优化器结果类型
pub type OptimizeResult<T> = Result<T, OptimizeError>;

/// 代价计算相关错误
#[derive(Error, Debug, Clone)]
pub enum CostError {
    /// 不支持的节点类型
    #[error("不支持的节点类型: {0}")]
    UnsupportedNodeType(String),

    /// 缺少统计信息
    #[error("缺少统计信息: {0}")]
    MissingStatistics(String),

    /// 计算错误
    #[error("计算错误: {0}")]
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
        assert!(err.to_string().contains("不支持的节点类型"));

        let err = OptimizeError::MissingStatistics("vertex_count".to_string());
        assert!(err.to_string().contains("缺少统计信息"));
    }

    #[test]
    fn test_cost_error_conversion() {
        let cost_err = CostError::CalculationError("division by zero".to_string());
        let opt_err: OptimizeError = cost_err.into();
        assert!(matches!(opt_err, OptimizeError::CalculationError(_)));
    }
}
