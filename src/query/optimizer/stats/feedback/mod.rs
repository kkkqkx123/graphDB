//! 运行时统计反馈模块
//!
//! 提供轻量级的执行反馈收集机制，用于动态调整选择性估计模型。
//! 使用指数加权移动平均(EWMA)算法校正选择性估计。
//!
//! ## 模块结构
//!
//! - `fingerprint` - 查询指纹生成和归一化
//! - `collector` - 执行反馈收集器
//! - `trigger` - 自动反馈触发机制
//! - `selectivity` - 选择性校正和管理
//! - `query` - 查询执行反馈结构
//! - `history` - 查询反馈历史管理

pub mod collector;
pub mod fingerprint;
pub mod history;
pub mod query;
pub mod selectivity;
pub mod trigger;

// 重新导出主要类型，保持向后兼容
pub use collector::ExecutionFeedbackCollector;
pub use fingerprint::{generate_query_fingerprint, normalize_query};
pub use history::QueryFeedbackHistory;
pub use query::{OperatorFeedback, QueryExecutionFeedback};
pub use selectivity::{FeedbackDrivenSelectivity, SelectivityFeedbackManager};
pub use trigger::{AutoFeedbackConfig, AutoFeedbackTrigger};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_integration() {
        // 测试模块集成
        let collector = ExecutionFeedbackCollector::new();
        collector.start();
        collector.record_rows(100);
        collector.finish();

        let mut selectivity = FeedbackDrivenSelectivity::new(0.1);
        selectivity.update_with_feedback(0.15);

        let query_feedback = QueryExecutionFeedback::new("fp_123".to_string());

        let history = QueryFeedbackHistory::new(10);
        // 添加反馈来验证history工作正常
        history.add_feedback(query_feedback.clone());

        let config = AutoFeedbackConfig::new();

        // 所有模块都能正常工作
        assert_eq!(collector.get_actual_rows(), 100);
        assert!(selectivity.corrected_selectivity() > 0.0);
        assert_eq!(query_feedback.query_fingerprint, "fp_123");
        assert_eq!(history.total_feedback_count(), 1);
        assert!(config.enabled);
    }
}
