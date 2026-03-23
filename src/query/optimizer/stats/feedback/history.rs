//! 查询反馈历史模块
//!
//! 提供查询反馈历史记录的管理功能，包括存储、检索和清理。

use crate::query::optimizer::stats::feedback::query::QueryExecutionFeedback;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::time::Instant;

/// 查询反馈历史
///
/// 管理查询执行反馈的历史记录，按查询指纹分组存储。
///
/// # 示例
/// ```
/// use graphdb::query::optimizer::stats::feedback::history::QueryFeedbackHistory;
/// use graphdb::query::optimizer::stats::feedback::query::QueryExecutionFeedback;
///
/// let history = QueryFeedbackHistory::new(100);
/// let feedback = QueryExecutionFeedback::new("fp_123".to_string());
/// history.add_feedback(feedback);
///
/// let feedbacks = history.get_feedback_for_query("fp_123");
/// assert_eq!(feedbacks.len(), 1);
/// ```
#[derive(Debug)]
pub struct QueryFeedbackHistory {
    /// 查询指纹到反馈列表的映射
    feedbacks: RwLock<HashMap<String, Vec<QueryExecutionFeedback>>>,
    /// 最大历史记录数（每个查询）
    max_history_per_query: usize,
}

impl QueryFeedbackHistory {
    /// 创建新的查询反馈历史
    ///
    /// # 参数
    /// - `max_history_per_query`: 每个查询保留的最大历史记录数
    pub fn new(max_history_per_query: usize) -> Self {
        Self {
            feedbacks: RwLock::new(HashMap::new()),
            max_history_per_query: max_history_per_query.max(1),
        }
    }

    /// 添加查询执行反馈
    pub fn add_feedback(&self, feedback: QueryExecutionFeedback) {
        let mut feedbacks = self.feedbacks.write();
        let entry = feedbacks
            .entry(feedback.query_fingerprint.clone())
            .or_insert_with(Vec::new);

        entry.push(feedback);

        // 限制历史记录数量
        if entry.len() > self.max_history_per_query {
            entry.remove(0);
        }
    }

    /// 获取特定查询的所有反馈
    pub fn get_feedback_for_query(&self, fingerprint: &str) -> Vec<QueryExecutionFeedback> {
        self.feedbacks
            .read()
            .get(fingerprint)
            .cloned()
            .unwrap_or_default()
    }

    /// 获取特定查询的反馈数量
    pub fn get_feedback_count(&self, fingerprint: &str) -> usize {
        self.feedbacks
            .read()
            .get(fingerprint)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// 获取所有查询指纹
    pub fn get_all_fingerprints(&self) -> Vec<String> {
        self.feedbacks.read().keys().cloned().collect()
    }

    /// 清除特定查询的历史
    pub fn clear_query_history(&self, fingerprint: &str) -> bool {
        self.feedbacks.write().remove(fingerprint).is_some()
    }

    /// 清除所有历史
    pub fn clear_all(&self) {
        self.feedbacks.write().clear();
    }

    /// 获取历史记录总数
    pub fn total_feedback_count(&self) -> usize {
        self.feedbacks.read().values().map(|v| v.len()).sum()
    }

    /// 获取查询数量
    pub fn query_count(&self) -> usize {
        self.feedbacks.read().len()
    }

    /// 获取最近N条反馈
    pub fn get_recent_feedbacks(&self, n: usize) -> Vec<QueryExecutionFeedback> {
        let feedbacks = self.feedbacks.read();
        let mut all_feedbacks: Vec<_> =
            feedbacks.values().flat_map(|v| v.iter().cloned()).collect();

        // 按时间戳排序（最新的在前）
        all_feedbacks.sort_by(|a, b| {
            b.execution_timestamp
                .elapsed()
                .cmp(&a.execution_timestamp.elapsed())
        });

        all_feedbacks.into_iter().take(n).collect()
    }

    /// 获取查询的平均行数估计误差
    pub fn get_avg_row_error(&self, fingerprint: &str) -> Option<f64> {
        let feedbacks = self.feedbacks.read();
        let query_feedbacks = feedbacks.get(fingerprint)?;

        if query_feedbacks.is_empty() {
            return None;
        }

        let total_error: f64 = query_feedbacks
            .iter()
            .map(|f| f.row_estimation_error())
            .sum();
        Some(total_error / query_feedbacks.len() as f64)
    }

    /// 获取查询的平均时间估计误差
    pub fn get_avg_time_error(&self, fingerprint: &str) -> Option<f64> {
        let feedbacks = self.feedbacks.read();
        let query_feedbacks = feedbacks.get(fingerprint)?;

        if query_feedbacks.is_empty() {
            return None;
        }

        let total_error: f64 = query_feedbacks
            .iter()
            .map(|f| f.time_estimation_error())
            .sum();
        Some(total_error / query_feedbacks.len() as f64)
    }

    /// 清理过期历史（基于时间）
    pub fn cleanup_old_feedbacks(&self, max_age: std::time::Duration) {
        let mut feedbacks = self.feedbacks.write();
        let now = Instant::now();

        for query_feedbacks in feedbacks.values_mut() {
            query_feedbacks.retain(|f| now.duration_since(f.execution_timestamp) < max_age);
        }

        // 移除空的条目
        feedbacks.retain(|_, v| !v.is_empty());
    }

    /// 设置最大历史记录数
    pub fn set_max_history(&self, max_history: usize) {
        let max_history = max_history.max(1);
        let mut feedbacks = self.feedbacks.write();

        for query_feedbacks in feedbacks.values_mut() {
            while query_feedbacks.len() > max_history {
                query_feedbacks.remove(0);
            }
        }
    }
}

impl Default for QueryFeedbackHistory {
    fn default() -> Self {
        Self::new(100)
    }
}

impl Clone for QueryFeedbackHistory {
    fn clone(&self) -> Self {
        Self {
            feedbacks: RwLock::new(self.feedbacks.read().clone()),
            max_history_per_query: self.max_history_per_query,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::optimizer::stats::feedback::query::QueryExecutionFeedback;

    #[test]
    fn test_query_feedback_history() {
        let history = QueryFeedbackHistory::new(10);

        // 添加反馈
        let feedback1 = QueryExecutionFeedback::new("fp_123".to_string());
        history.add_feedback(feedback1);

        let feedbacks = history.get_feedback_for_query("fp_123");
        assert_eq!(feedbacks.len(), 1);

        // 添加更多反馈
        let feedback2 = QueryExecutionFeedback::new("fp_123".to_string());
        history.add_feedback(feedback2);

        assert_eq!(history.get_feedback_count("fp_123"), 2);
        assert_eq!(history.total_feedback_count(), 2);
        assert_eq!(history.query_count(), 1);
    }

    #[test]
    fn test_history_limit() {
        let history = QueryFeedbackHistory::new(3);

        // 添加4条反馈（超过限制）
        for i in 0..4 {
            let mut feedback = QueryExecutionFeedback::new("fp_123".to_string());
            feedback.actual_rows = i as u64 * 100;
            history.add_feedback(feedback);
        }

        // 应该只保留3条
        assert_eq!(history.get_feedback_count("fp_123"), 3);
    }

    #[test]
    fn test_clear_history() {
        let history = QueryFeedbackHistory::new(10);

        let feedback = QueryExecutionFeedback::new("fp_123".to_string());
        history.add_feedback(feedback);

        assert!(history.clear_query_history("fp_123"));
        assert_eq!(history.get_feedback_count("fp_123"), 0);
        assert!(!history.clear_query_history("nonexistent"));
    }

    #[test]
    fn test_multiple_queries() {
        let history = QueryFeedbackHistory::new(10);

        history.add_feedback(QueryExecutionFeedback::new("fp_1".to_string()));
        history.add_feedback(QueryExecutionFeedback::new("fp_2".to_string()));
        history.add_feedback(QueryExecutionFeedback::new("fp_1".to_string()));

        assert_eq!(history.query_count(), 2);
        assert_eq!(history.total_feedback_count(), 3);

        let fingerprints = history.get_all_fingerprints();
        assert_eq!(fingerprints.len(), 2);
    }

    #[test]
    fn test_avg_errors() {
        let history = QueryFeedbackHistory::new(10);

        // 添加两条有估计误差的反馈
        let mut feedback1 = QueryExecutionFeedback::new("fp_123".to_string());
        feedback1.estimated_rows = 100;
        feedback1.actual_rows = 110; // 10% 误差
        history.add_feedback(feedback1);

        let mut feedback2 = QueryExecutionFeedback::new("fp_123".to_string());
        feedback2.estimated_rows = 100;
        feedback2.actual_rows = 90; // 10% 误差
        history.add_feedback(feedback2);

        let avg_error = history.get_avg_row_error("fp_123").unwrap();
        assert!((avg_error - 0.1).abs() < 0.01); // 平均误差应该接近0.1
    }

    #[test]
    fn test_nonexistent_query() {
        let history = QueryFeedbackHistory::new(10);

        assert_eq!(history.get_feedback_count("nonexistent"), 0);
        assert!(history.get_feedback_for_query("nonexistent").is_empty());
        assert!(history.get_avg_row_error("nonexistent").is_none());
    }
}
