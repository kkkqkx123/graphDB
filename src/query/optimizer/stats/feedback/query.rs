//! 查询反馈结构模块
//!
//! 提供查询执行反馈的数据结构，包括算子反馈和查询执行反馈。

use std::time::Instant;

/// 算子执行反馈
///
/// 记录单个算子的执行统计信息。
/// 参考PostgreSQL的EXPLAIN ANALYZE输出格式。
///
/// # 字段说明
/// - `operator_id`: 算子唯一标识
/// - `operator_type`: 算子类型（如Scan, Filter, Join等）
/// - `estimated_rows`: 优化器估计的输出行数
/// - `actual_rows`: 实际输出行数
/// - `estimated_time_us`: 估计执行时间（微秒）
/// - `actual_time_us`: 实际执行时间（微秒）
/// - `execution_loops`: 执行次数（如Nested Loop内层扫描的执行次数）
///
/// # 示例
/// ```
/// use graphdb::query::optimizer::stats::feedback::query::OperatorFeedback;
///
/// let feedback = OperatorFeedback {
///     operator_id: "scan_1".to_string(),
///     operator_type: "IndexScan".to_string(),
///     estimated_rows: 100,
///     actual_rows: 150,
///     estimated_time_us: 1000,
///     actual_time_us: 1200,
///     execution_loops: 1,
/// };
///
/// assert_eq!(feedback.row_estimation_error(), 0.5); // (150-100)/100
/// ```
#[derive(Debug, Clone)]
pub struct OperatorFeedback {
    /// 算子ID
    pub operator_id: String,
    /// 算子类型
    pub operator_type: String,
    /// 估计输出行数
    pub estimated_rows: u64,
    /// 实际输出行数
    pub actual_rows: u64,
    /// 估计执行时间（微秒）
    pub estimated_time_us: u64,
    /// 实际执行时间（微秒）
    pub actual_time_us: u64,
    /// 执行次数（如Nested Loop内层扫描的执行次数）
    pub execution_loops: u64,
}

impl OperatorFeedback {
    /// 创建新的算子反馈
    pub fn new(
        operator_id: String,
        operator_type: String,
        estimated_rows: u64,
        actual_rows: u64,
    ) -> Self {
        Self {
            operator_id,
            operator_type,
            estimated_rows,
            actual_rows,
            estimated_time_us: 0,
            actual_time_us: 0,
            execution_loops: 1,
        }
    }

    /// 计算行数估计误差
    ///
    /// 返回相对误差：(实际-估计)/估计
    /// 如果估计为0，返回1.0
    pub fn row_estimation_error(&self) -> f64 {
        if self.estimated_rows == 0 {
            return 1.0;
        }
        let estimated = self.estimated_rows as f64;
        let actual = self.actual_rows as f64;
        ((actual - estimated).abs() / estimated).min(10.0)
    }

    /// 计算时间估计误差
    ///
    /// 返回相对误差：(实际-估计)/估计
    pub fn time_estimation_error(&self) -> f64 {
        if self.estimated_time_us == 0 {
            return 1.0;
        }
        let estimated = self.estimated_time_us as f64;
        let actual = self.actual_time_us as f64;
        ((actual - estimated).abs() / estimated).min(10.0)
    }

    /// 获取每次执行的平均实际行数
    ///
    /// 对于多次执行的算子（如Nested Loop内层），
    /// 返回每次执行的平均行数。
    pub fn avg_rows_per_loop(&self) -> f64 {
        if self.execution_loops == 0 {
            return 0.0;
        }
        self.actual_rows as f64 / self.execution_loops as f64
    }

    /// 获取每次执行的平均实际时间（微秒）
    pub fn avg_time_us_per_loop(&self) -> f64 {
        if self.execution_loops == 0 {
            return 0.0;
        }
        self.actual_time_us as f64 / self.execution_loops as f64
    }
}

/// 查询执行反馈
///
/// 记录单次查询执行的完整反馈信息。
///
/// # 示例
/// ```
/// use graphdb::query::optimizer::stats::feedback::query::QueryExecutionFeedback;
///
/// let mut feedback = QueryExecutionFeedback::new("query_fp_123".to_string());
/// feedback.estimated_rows = 1000;
/// feedback.actual_rows = 1200;
/// feedback.estimated_time_us = 5000;
/// feedback.actual_time_us = 6000;
///
/// assert!(feedback.row_estimation_error() > 0.0);
/// ```
#[derive(Debug, Clone)]
pub struct QueryExecutionFeedback {
    /// 查询指纹
    pub query_fingerprint: String,
    /// 估计输出行数
    pub estimated_rows: u64,
    /// 实际输出行数
    pub actual_rows: u64,
    /// 估计执行时间（微秒）
    pub estimated_time_us: u64,
    /// 实际执行时间（微秒）
    pub actual_time_us: u64,
    /// 执行时间戳
    pub execution_timestamp: Instant,
    /// 各算子的反馈
    pub operator_feedbacks: Vec<OperatorFeedback>,
}

impl QueryExecutionFeedback {
    /// 创建新的查询执行反馈
    pub fn new(query_fingerprint: String) -> Self {
        Self {
            query_fingerprint,
            estimated_rows: 0,
            actual_rows: 0,
            estimated_time_us: 0,
            actual_time_us: 0,
            execution_timestamp: Instant::now(),
            operator_feedbacks: Vec::new(),
        }
    }

    /// 计算行数估计误差
    pub fn row_estimation_error(&self) -> f64 {
        if self.estimated_rows == 0 {
            return 1.0;
        }
        let estimated = self.estimated_rows as f64;
        let actual = self.actual_rows as f64;
        ((actual - estimated).abs() / estimated).min(10.0)
    }

    /// 计算时间估计误差
    pub fn time_estimation_error(&self) -> f64 {
        if self.estimated_time_us == 0 {
            return 1.0;
        }
        let estimated = self.estimated_time_us as f64;
        let actual = self.actual_time_us as f64;
        ((actual - estimated).abs() / estimated).min(10.0)
    }

    /// 添加算子反馈
    pub fn add_operator_feedback(&mut self, feedback: OperatorFeedback) {
        self.operator_feedbacks.push(feedback);
    }

    /// 获取算子反馈数量
    pub fn operator_feedback_count(&self) -> usize {
        self.operator_feedbacks.len()
    }

    /// 获取特定算子的反馈
    pub fn get_operator_feedback(&self, operator_id: &str) -> Option<&OperatorFeedback> {
        self.operator_feedbacks
            .iter()
            .find(|f| f.operator_id == operator_id)
    }

    /// 计算所有算子的平均行数估计误差
    pub fn avg_operator_row_error(&self) -> f64 {
        if self.operator_feedbacks.is_empty() {
            return 0.0;
        }
        let total_error: f64 = self
            .operator_feedbacks
            .iter()
            .map(|f| f.row_estimation_error())
            .sum();
        total_error / self.operator_feedbacks.len() as f64
    }

    /// 计算所有算子的平均时间估计误差
    pub fn avg_operator_time_error(&self) -> f64 {
        if self.operator_feedbacks.is_empty() {
            return 0.0;
        }
        let total_error: f64 = self
            .operator_feedbacks
            .iter()
            .map(|f| f.time_estimation_error())
            .sum();
        total_error / self.operator_feedbacks.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_feedback() {
        let feedback = OperatorFeedback {
            operator_id: "scan_1".to_string(),
            operator_type: "IndexScan".to_string(),
            estimated_rows: 100,
            actual_rows: 150,
            estimated_time_us: 1000,
            actual_time_us: 1200,
            execution_loops: 1,
        };

        assert_eq!(feedback.row_estimation_error(), 0.5); // (150-100)/100
        assert_eq!(feedback.time_estimation_error(), 0.2); // (1200-1000)/1000
    }

    #[test]
    fn test_operator_feedback_loops() {
        let feedback = OperatorFeedback {
            operator_id: "nested_loop_inner".to_string(),
            operator_type: "IndexScan".to_string(),
            estimated_rows: 100,
            actual_rows: 500, // 总共500行，执行了10次
            estimated_time_us: 1000,
            actual_time_us: 5000,
            execution_loops: 10,
        };

        assert_eq!(feedback.avg_rows_per_loop(), 50.0); // 500/10
        assert_eq!(feedback.avg_time_us_per_loop(), 500.0); // 5000/10
    }

    #[test]
    fn test_query_execution_feedback() {
        let mut feedback = QueryExecutionFeedback::new("fp_123".to_string());
        feedback.estimated_rows = 1000;
        feedback.actual_rows = 1200;
        feedback.estimated_time_us = 5000;
        feedback.actual_time_us = 6000;

        // 添加算子反馈
        let op_feedback =
            OperatorFeedback::new("scan_1".to_string(), "SeqScan".to_string(), 1000, 1200);
        feedback.add_operator_feedback(op_feedback);

        assert_eq!(feedback.operator_feedback_count(), 1);
        assert!(feedback.row_estimation_error() > 0.0);
        assert!(feedback.time_estimation_error() > 0.0);
    }

    #[test]
    fn test_avg_operator_errors() {
        let mut feedback = QueryExecutionFeedback::new("fp_123".to_string());

        // 添加两个算子反馈
        feedback.add_operator_feedback(OperatorFeedback {
            operator_id: "op1".to_string(),
            operator_type: "Scan".to_string(),
            estimated_rows: 100,
            actual_rows: 110,
            estimated_time_us: 1000,
            actual_time_us: 1100,
            execution_loops: 1,
        });

        feedback.add_operator_feedback(OperatorFeedback {
            operator_id: "op2".to_string(),
            operator_type: "Filter".to_string(),
            estimated_rows: 100,
            actual_rows: 90,
            estimated_time_us: 500,
            actual_time_us: 450,
            execution_loops: 1,
        });

        let avg_row_error = feedback.avg_operator_row_error();
        let avg_time_error = feedback.avg_operator_time_error();

        assert!(avg_row_error > 0.0);
        assert!(avg_time_error > 0.0);
    }
}
