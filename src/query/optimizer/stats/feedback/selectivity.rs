//! 选择性校正模块
//!
//! 提供基于历史执行反馈动态调整选择性估计的功能。
//! 使用指数加权移动平均(EWMA)算法校正选择性估计。

use parking_lot::RwLock;
use std::collections::HashMap;

/// 反馈驱动的选择性校正
///
/// 基于历史执行反馈动态调整选择性估计。
/// 使用指数加权移动平均(EWMA)算法。
///
/// # 示例
/// ```
/// use graphdb::query::optimizer::stats::feedback::selectivity::FeedbackDrivenSelectivity;
///
/// let mut feedback = FeedbackDrivenSelectivity::new(0.1);
/// assert_eq!(feedback.estimated_selectivity(), 0.1);
///
/// // 更新反馈
/// feedback.update_with_feedback(0.15);
/// feedback.update_with_feedback(0.12);
///
/// // 获取校正后的选择性
/// let corrected = feedback.corrected_selectivity();
/// assert!(corrected > 0.0 && corrected <= 0.99);
/// ```
#[derive(Debug, Clone)]
pub struct FeedbackDrivenSelectivity {
    /// 原始估计选择性
    estimated_selectivity: f64,
    /// 历史实际选择性（滑动窗口平均）
    actual_selectivity_ewma: f64,
    /// 校正因子
    correction_factor: f64,
    /// 反馈次数
    feedback_count: u64,
    /// EWMA平滑因子
    alpha: f64,
    /// 最小校正因子
    min_correction: f64,
    /// 最大校正因子
    max_correction: f64,
    /// 选择性上限（默认0.99）
    selectivity_cap: f64,
    /// 累积估计误差（用于计算误差统计）
    cumulative_estimation_error: f64,
    /// 误差平方和（用于计算标准差）
    error_sum_squares: f64,
}

impl FeedbackDrivenSelectivity {
    /// 创建新的反馈驱动选择性估计
    pub fn new(estimated_selectivity: f64) -> Self {
        Self {
            estimated_selectivity,
            actual_selectivity_ewma: estimated_selectivity,
            correction_factor: 1.0,
            feedback_count: 0,
            alpha: 0.3,
            min_correction: 0.1,
            max_correction: 10.0,
            selectivity_cap: 0.99,
            cumulative_estimation_error: 0.0,
            error_sum_squares: 0.0,
        }
    }

    /// 使用自定义参数创建
    pub fn with_params(
        estimated_selectivity: f64,
        alpha: f64,
        min_correction: f64,
        max_correction: f64,
    ) -> Self {
        Self {
            estimated_selectivity,
            actual_selectivity_ewma: estimated_selectivity,
            correction_factor: 1.0,
            feedback_count: 0,
            alpha,
            min_correction,
            max_correction,
            selectivity_cap: 0.99,
            cumulative_estimation_error: 0.0,
            error_sum_squares: 0.0,
        }
    }

    /// 获取原始估计选择性
    pub fn estimated_selectivity(&self) -> f64 {
        self.estimated_selectivity
    }

    /// 获取校正后的选择性
    pub fn corrected_selectivity(&self) -> f64 {
        (self.estimated_selectivity * self.correction_factor)
            .clamp(self.min_correction * self.estimated_selectivity, self.selectivity_cap)
    }

    /// 获取校正因子
    pub fn correction_factor(&self) -> f64 {
        self.correction_factor
    }

    /// 获取反馈次数
    pub fn feedback_count(&self) -> u64 {
        self.feedback_count
    }

    /// 获取选择性估计的置信度
    ///
    /// 基于反馈次数计算置信度，反馈越多置信度越高。
    /// 返回值范围：0.0 - 1.0
    pub fn estimation_confidence(&self) -> f64 {
        // 使用sigmoid函数计算置信度
        // 反馈次数达到100时，置信度接近0.9
        let x = self.feedback_count as f64 * 0.1;
        1.0 / (1.0 + (-x).exp())
    }

    /// 获取平均估计误差
    pub fn avg_estimation_error(&self) -> f64 {
        if self.feedback_count == 0 {
            return 1.0;
        }
        self.cumulative_estimation_error / self.feedback_count as f64
    }

    /// 获取估计误差的标准差
    pub fn error_std_dev(&self) -> f64 {
        if self.feedback_count < 2 {
            return 0.0;
        }
        let n = self.feedback_count as f64;
        let mean = self.cumulative_estimation_error / n;
        let variance = (self.error_sum_squares / n) - (mean * mean);
        variance.max(0.0).sqrt()
    }

    /// 更新校正因子（根据新反馈）
    ///
    /// 使用指数加权移动平均(EWMA)算法。
    pub fn update_with_feedback(&mut self, actual_selectivity: f64) {
        if self.estimated_selectivity <= 0.0 {
            return;
        }

        let ratio = actual_selectivity / self.estimated_selectivity;

        // 计算当前估计误差
        let estimated = self.corrected_selectivity();
        let error = (actual_selectivity - estimated).abs();
        self.cumulative_estimation_error += error;
        self.error_sum_squares += error * error;

        // 使用EWMA更新校正因子
        self.correction_factor =
            (1.0 - self.alpha) * self.correction_factor + self.alpha * ratio;

        // 限制校正因子范围，避免过度校正
        self.correction_factor = self
            .correction_factor
            .clamp(self.min_correction, self.max_correction);

        // 更新实际选择性EWMA
        self.actual_selectivity_ewma =
            (1.0 - self.alpha) * self.actual_selectivity_ewma + self.alpha * actual_selectivity;

        self.feedback_count += 1;
    }

    /// 批量更新反馈
    pub fn update_with_batch(&mut self, actual_selectivities: &[f64]) {
        for &selectivity in actual_selectivities {
            self.update_with_feedback(selectivity);
        }
    }

    /// 重置校正因子
    pub fn reset_correction(&mut self) {
        self.correction_factor = 1.0;
        self.actual_selectivity_ewma = self.estimated_selectivity;
        self.feedback_count = 0;
        self.cumulative_estimation_error = 0.0;
        self.error_sum_squares = 0.0;
    }

    /// 设置EWMA平滑因子
    pub fn set_alpha(&mut self, alpha: f64) {
        self.alpha = alpha.clamp(0.0, 1.0);
    }

    /// 设置校正范围
    pub fn set_correction_range(&mut self, min: f64, max: f64) {
        self.min_correction = min.max(0.01);
        self.max_correction = max.max(self.min_correction);
    }

    /// 设置选择性上限
    pub fn set_selectivity_cap(&mut self, cap: f64) {
        self.selectivity_cap = cap.clamp(0.5, 1.0);
    }
}

impl Default for FeedbackDrivenSelectivity {
    fn default() -> Self {
        Self::new(0.1)
    }
}

/// 选择性反馈管理器
///
/// 管理多个条件的选择性反馈。
///
/// # 示例
/// ```
/// use graphdb::query::optimizer::stats::feedback::selectivity::SelectivityFeedbackManager;
///
/// let manager = SelectivityFeedbackManager::new();
/// manager.register_condition("age > 25".to_string(), 0.3);
///
/// let corrected = manager.get_corrected_selectivity("age > 25");
/// assert!(corrected.is_some());
/// ```
#[derive(Debug)]
pub struct SelectivityFeedbackManager {
    /// 条件键到选择性校正的映射
    feedbacks: RwLock<HashMap<String, FeedbackDrivenSelectivity>>,
    /// 默认EWMA平滑因子
    default_alpha: f64,
    /// 默认最小校正因子
    default_min_correction: f64,
    /// 默认最大校正因子
    default_max_correction: f64,
}

impl SelectivityFeedbackManager {
    /// 创建新的反馈管理器
    pub fn new() -> Self {
        Self {
            feedbacks: RwLock::new(HashMap::new()),
            default_alpha: 0.3,
            default_min_correction: 0.1,
            default_max_correction: 10.0,
        }
    }

    /// 使用自定义参数创建
    pub fn with_params(alpha: f64, min_correction: f64, max_correction: f64) -> Self {
        Self {
            feedbacks: RwLock::new(HashMap::new()),
            default_alpha: alpha,
            default_min_correction: min_correction,
            default_max_correction: max_correction,
        }
    }

    /// 注册条件的选择性估计
    pub fn register_condition(&self, key: String, estimated_selectivity: f64) {
        let feedback = FeedbackDrivenSelectivity::with_params(
            estimated_selectivity,
            self.default_alpha,
            self.default_min_correction,
            self.default_max_correction,
        );
        self.feedbacks.write().insert(key, feedback);
    }

    /// 获取校正后的选择性
    pub fn get_corrected_selectivity(&self, key: &str) -> Option<f64> {
        self.feedbacks
            .read()
            .get(key)
            .map(|f| f.corrected_selectivity())
    }

    /// 更新反馈
    pub fn update_feedback(&self, key: &str, actual_selectivity: f64) -> bool {
        let mut feedbacks = self.feedbacks.write();
        if let Some(feedback) = feedbacks.get_mut(key) {
            feedback.update_with_feedback(actual_selectivity);
            true
        } else {
            false
        }
    }

    /// 批量更新反馈
    pub fn update_feedback_batch(&self, updates: &[(String, f64)]) {
        let mut feedbacks = self.feedbacks.write();
        for (key, actual_selectivity) in updates {
            if let Some(feedback) = feedbacks.get_mut(key) {
                feedback.update_with_feedback(*actual_selectivity);
            }
        }
    }

    /// 获取反馈信息
    pub fn get_feedback(&self, key: &str) -> Option<FeedbackDrivenSelectivity> {
        self.feedbacks.read().get(key).cloned()
    }

    /// 获取所有反馈键
    pub fn get_all_keys(&self) -> Vec<String> {
        self.feedbacks.read().keys().cloned().collect()
    }

    /// 清除所有反馈
    pub fn clear_all(&self) {
        self.feedbacks.write().clear();
    }

    /// 移除特定条件的反馈
    pub fn remove_feedback(&self, key: &str) -> Option<FeedbackDrivenSelectivity> {
        self.feedbacks.write().remove(key)
    }

    /// 获取反馈数量
    pub fn feedback_count(&self) -> usize {
        self.feedbacks.read().len()
    }

    /// 设置默认参数
    pub fn set_default_params(&mut self, alpha: f64, min_correction: f64, max_correction: f64) {
        self.default_alpha = alpha.clamp(0.0, 1.0);
        self.default_min_correction = min_correction.max(0.01);
        self.default_max_correction = max_correction.max(self.default_min_correction);
    }
}

impl Default for SelectivityFeedbackManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SelectivityFeedbackManager {
    fn clone(&self) -> Self {
        Self {
            feedbacks: RwLock::new(self.feedbacks.read().clone()),
            default_alpha: self.default_alpha,
            default_min_correction: self.default_min_correction,
            default_max_correction: self.default_max_correction,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_driven_selectivity() {
        let mut feedback = FeedbackDrivenSelectivity::new(0.1);
        assert_eq!(feedback.estimated_selectivity(), 0.1);

        // 更新反馈
        feedback.update_with_feedback(0.15);
        feedback.update_with_feedback(0.12);

        // 校正后的选择性应该在合理范围内
        let corrected = feedback.corrected_selectivity();
        assert!(corrected > 0.0 && corrected <= 0.99);

        // 反馈次数应该为2
        assert_eq!(feedback.feedback_count(), 2);
    }

    #[test]
    fn test_feedback_correction_range() {
        let mut feedback = FeedbackDrivenSelectivity::new(0.5);

        // 大量更新，测试校正因子限制
        for _ in 0..100 {
            feedback.update_with_feedback(0.01); // 远低于估计值
        }

        // 校正因子应该被限制在最小值
        assert!(feedback.correction_factor() >= 0.1);
    }

    #[test]
    fn test_estimation_confidence() {
        let mut feedback = FeedbackDrivenSelectivity::new(0.1);
        // 初始置信度应该是0.5（sigmoid(0) = 0.5）
        let initial_confidence = feedback.estimation_confidence();
        assert!(initial_confidence < 0.55 && initial_confidence > 0.45);

        // 添加多次反馈
        for i in 0..100 {
            feedback.update_with_feedback(0.1 + (i as f64 * 0.001));
        }

        assert!(feedback.estimation_confidence() > 0.9); // 反馈后置信度高
    }

    #[test]
    fn test_avg_estimation_error() {
        let mut feedback = FeedbackDrivenSelectivity::new(0.5);

        // 无反馈时误差为1.0
        assert_eq!(feedback.avg_estimation_error(), 1.0);

        // 添加反馈
        feedback.update_with_feedback(0.5); // 无误差
        feedback.update_with_feedback(0.6); // 有误差

        assert!(feedback.avg_estimation_error() < 1.0);
    }

    #[test]
    fn test_selectivity_feedback_manager() {
        let manager = SelectivityFeedbackManager::new();
        manager.register_condition("age > 25".to_string(), 0.3);
        manager.register_condition("salary > 5000".to_string(), 0.2);

        assert_eq!(manager.feedback_count(), 2);

        // 更新反馈
        manager.update_feedback("age > 25", 0.35);
        manager.update_feedback("salary > 5000", 0.18);

        // 获取校正后的选择性
        let corrected_age = manager.get_corrected_selectivity("age > 25");
        assert!(corrected_age.is_some());

        // 获取不存在的条件
        assert!(manager.get_corrected_selectivity("unknown").is_none());
    }

    #[test]
    fn test_selectivity_cap() {
        let mut feedback = FeedbackDrivenSelectivity::new(0.9);
        feedback.set_selectivity_cap(0.95);

        // 大量更新，试图使选择性超过上限
        for _ in 0..50 {
            feedback.update_with_feedback(1.0);
        }

        // 校正后的选择性不应该超过上限
        assert!(feedback.corrected_selectivity() <= 0.95);
    }
}
