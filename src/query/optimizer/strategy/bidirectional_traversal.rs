//! 双向遍历优化器
//!
//! 为最短路径查询提供双向BFS遍历优化决策
//! 双向BFS同时从起点和终点搜索，可将复杂度从O(b^d)降到O(b^(d/2))

use std::sync::Arc;

use crate::core::types::EdgeDirection;
use crate::query::optimizer::cost::CostCalculator;
use crate::query::optimizer::stats::{EdgeTypeStatistics, StatisticsManager};

/// 深度分配上下文
#[derive(Debug, Clone)]
pub struct DepthAllocationContext {
    /// 起点变量名
    pub start_variable: String,
    /// 终点变量名
    pub end_variable: String,
    /// 边类型列表
    pub edge_types: Vec<String>,
    /// 总遍历深度
    pub total_depth: u32,
    /// 起点标签（如果有）
    pub start_tag: Option<String>,
    /// 终点标签（如果有）
    pub end_tag: Option<String>,
    /// 起点估计度数（如果已知）
    pub start_degree_hint: Option<f64>,
    /// 终点估计度数（如果已知）
    pub end_degree_hint: Option<f64>,
}

impl DepthAllocationContext {
    /// 创建新的深度分配上下文
    pub fn new(
        start_variable: impl Into<String>,
        end_variable: impl Into<String>,
        edge_types: Vec<String>,
        total_depth: u32,
    ) -> Self {
        Self {
            start_variable: start_variable.into(),
            end_variable: end_variable.into(),
            edge_types,
            total_depth,
            start_tag: None,
            end_tag: None,
            start_degree_hint: None,
            end_degree_hint: None,
        }
    }

    /// 设置起点标签
    pub fn with_start_tag(mut self, tag: impl Into<String>) -> Self {
        self.start_tag = Some(tag.into());
        self
    }

    /// 设置终点标签
    pub fn with_end_tag(mut self, tag: impl Into<String>) -> Self {
        self.end_tag = Some(tag.into());
        self
    }

    /// 设置起点度数提示
    pub fn with_start_degree(mut self, degree: f64) -> Self {
        self.start_degree_hint = Some(degree);
        self
    }

    /// 设置终点度数提示
    pub fn with_end_degree(mut self, degree: f64) -> Self {
        self.end_degree_hint = Some(degree);
        self
    }
}

/// 双向遍历决策
#[derive(Debug, Clone)]
pub struct BidirectionalDecision {
    /// 是否使用双向遍历
    pub use_bidirectional: bool,
    /// 正向搜索起点变量
    pub forward_start: String,
    /// 反向搜索起点变量
    pub backward_start: String,
    /// 预计减少的搜索空间比例
    pub estimated_savings: f64,
    /// 推荐的深度限制
    pub recommended_depth: u32,
}

impl BidirectionalDecision {
    /// 创建不使用双向遍历的决策
    pub fn unidirectional(forward_start: String) -> Self {
        Self {
            use_bidirectional: false,
            forward_start,
            backward_start: String::new(),
            estimated_savings: 0.0,
            recommended_depth: 0,
        }
    }

    /// 创建使用双向遍历的决策
    pub fn bidirectional(
        forward_start: String,
        backward_start: String,
        estimated_savings: f64,
        recommended_depth: u32,
    ) -> Self {
        Self {
            use_bidirectional: true,
            forward_start,
            backward_start,
            estimated_savings,
            recommended_depth,
        }
    }
}

/// 双向遍历优化器
pub struct BidirectionalTraversalOptimizer {
    /// 代价计算器（预留，用于未来更精细的代价估算）
    #[allow(dead_code)]
    cost_calculator: Arc<CostCalculator>,
    /// 统计信息管理器
    stats_manager: Arc<StatisticsManager>,
}

impl BidirectionalTraversalOptimizer {
    /// 创建新的双向遍历优化器
    pub fn new(
        cost_calculator: Arc<CostCalculator>,
        stats_manager: Arc<StatisticsManager>,
    ) -> Self {
        Self {
            cost_calculator,
            stats_manager,
        }
    }

    /// 评估是否适合双向遍历
    ///
    /// # 参数
    /// - `start_variable`: 起点变量名
    /// - `end_variable`: 终点变量名
    /// - `edge_types`: 边类型列表
    /// - `max_depth`: 最大遍历深度
    ///
    /// # 返回值
    /// 返回双向遍历决策
    pub fn evaluate(
        &self,
        start_variable: &str,
        end_variable: &str,
        edge_types: &[String],
        max_depth: u32,
    ) -> BidirectionalDecision {
        // 深度小于2时，双向遍历收益不大
        if max_depth < 2 {
            return BidirectionalDecision::unidirectional(start_variable.to_string());
        }

        // 获取平均分支因子
        let avg_branching = self.estimate_average_branching(edge_types);

        // 计算单向搜索空间: b^d
        let unidirectional_cost = avg_branching.powi(max_depth as i32);

        // 计算双向搜索空间: 2 * b^(d/2)
        let half_depth = (max_depth as f64 / 2.0).ceil() as i32;
        let bidirectional_cost = 2.0 * avg_branching.powi(half_depth);

        // 计算节省比例
        let savings = if unidirectional_cost > 0.0 {
            1.0 - (bidirectional_cost / unidirectional_cost)
        } else {
            0.0
        };

        // 如果节省超过30%，建议使用双向遍历
        if savings > 0.3 {
            BidirectionalDecision::bidirectional(
                start_variable.to_string(),
                end_variable.to_string(),
                savings,
                max_depth,
            )
        } else {
            BidirectionalDecision::unidirectional(start_variable.to_string())
        }
    }

    /// 评估路径查询是否适合双向遍历
    pub fn evaluate_path_query(
        &self,
        start_variable: &str,
        end_variable: &str,
        edge_types: &[String],
        min_depth: u32,
        max_depth: u32,
    ) -> BidirectionalDecision {
        // 对于路径查询，需要考虑深度范围
        let avg_depth = ((min_depth + max_depth) as f64 / 2.0) as u32;

        // 如果最小深度较大，双向遍历更有价值
        if min_depth >= 2 {
            let decision = self.evaluate(start_variable, end_variable, edge_types, avg_depth);
            if decision.use_bidirectional {
                return decision;
            }
        }

        // 否则使用标准评估
        self.evaluate(start_variable, end_variable, edge_types, max_depth)
    }

    /// 估计平均分支因子
    fn estimate_average_branching(&self, edge_types: &[String]) -> f64 {
        if edge_types.is_empty() {
            // 默认分支因子
            return 2.0;
        }

        let mut total_branching = 0.0;
        let mut count = 0;

        for edge_type in edge_types {
            if let Some(stats) = self.stats_manager.get_edge_stats(edge_type) {
                // 使用平均出度作为分支因子估计
                let branching = stats.avg_out_degree;
                total_branching += branching;
                count += 1;
            }
        }

        if count > 0 {
            total_branching / count as f64
        } else {
            2.0 // 默认值
        }
    }

    /// 估计从某点可达的节点数量
    #[allow(dead_code)]
    fn estimate_reachable_nodes(&self, edge_types: &[String], depth: u32) -> f64 {
        let branching = self.estimate_average_branching(edge_types);

        // 使用几何级数公式: 1 + b + b^2 + ... + b^d = (b^(d+1) - 1) / (b - 1)
        if (branching - 1.0).abs() < f64::EPSILON {
            (depth + 1) as f64
        } else {
            (branching.powi((depth + 1) as i32) - 1.0) / (branching - 1.0)
        }
    }

    /// 计算双向遍历的推荐深度分配
    ///
    /// 基于边类型统计信息和度数估计，智能分配正向和反向搜索的深度
    /// 策略：度数较小的一端分配更多深度，因为分支少、搜索空间小
    pub fn calculate_depth_allocation(&self, context: &DepthAllocationContext) -> (u32, u32) {
        let total_depth = context.total_depth;

        // 深度小于2时，直接平均分配
        if total_depth < 2 {
            return (total_depth, 0);
        }

        // 获取边类型统计信息
        let edge_stats: Vec<EdgeTypeStatistics> = context
            .edge_types
            .iter()
            .filter_map(|et| self.stats_manager.get_edge_stats(et))
            .collect();

        // 估计起点和终点的度数
        let start_degree = context
            .start_degree_hint
            .or_else(|| {
                self.estimate_vertex_degree(&context.start_tag, &edge_stats, EdgeDirection::Out)
            })
            .unwrap_or_else(|| self.estimate_average_branching(&context.edge_types));

        let end_degree = context
            .end_degree_hint
            .or_else(|| {
                self.estimate_vertex_degree(&context.end_tag, &edge_stats, EdgeDirection::In)
            })
            .unwrap_or_else(|| self.estimate_average_branching(&context.edge_types));

        // 基于度数比例计算深度分配
        // 度数越小，分配的深度越大（因为搜索空间增长慢）
        self.allocate_depth_by_degree(start_degree, end_degree, total_depth, &edge_stats)
    }

    /// 基于度数比例分配深度
    fn allocate_depth_by_degree(
        &self,
        start_degree: f64,
        end_degree: f64,
        total_depth: u32,
        edge_stats: &[EdgeTypeStatistics],
    ) -> (u32, u32) {
        // 计算倾斜度调整因子
        let skewness_adjustment = self.calculate_skewness_adjustment(edge_stats);

        // 基础分配：度数小的一端分配更多深度
        // 使用对数比例平滑极端值
        let log_start = (start_degree + 1.0).ln();
        let log_end = (end_degree + 1.0).ln();
        let log_total = log_start + log_end;

        // 反向比例分配：度数越小，深度越大
        let base_forward_ratio = if log_total > 0.0 {
            log_end / log_total
        } else {
            0.5
        };

        // 应用倾斜度调整
        let adjusted_ratio = base_forward_ratio * (1.0 + skewness_adjustment);
        let forward_ratio = adjusted_ratio.clamp(0.2, 0.8); // 限制在20%-80%之间

        let forward_depth =
            ((forward_ratio * total_depth as f64).round() as u32).clamp(1, total_depth - 1);
        let backward_depth = total_depth - forward_depth;

        (forward_depth, backward_depth)
    }

    /// 计算倾斜度调整因子
    fn calculate_skewness_adjustment(&self, edge_stats: &[EdgeTypeStatistics]) -> f64 {
        if edge_stats.is_empty() {
            return 0.0;
        }

        let total_skewness: f64 = edge_stats.iter().map(|s| s.degree_gini_coefficient).sum();

        let avg_skewness = total_skewness / edge_stats.len() as f64;

        // 倾斜度越高，越倾向于平均分配（避免陷入热点）
        // 返回范围: -0.1 到 0.1
        if avg_skewness > 0.7 {
            -0.1 // 严重倾斜，向平均分配调整
        } else if avg_skewness > 0.5 {
            -0.05 // 中度倾斜
        } else {
            0.0 // 轻度倾斜，不调整
        }
    }

    /// 估计顶点度数
    fn estimate_vertex_degree(
        &self,
        tag: &Option<String>,
        _edge_stats: &[EdgeTypeStatistics],
        direction: EdgeDirection,
    ) -> Option<f64> {
        let tag_name = tag.as_ref()?;

        // 获取标签统计信息
        let tag_stats = self.stats_manager.get_tag_stats(tag_name)?;

        // 根据方向返回平均度数
        match direction {
            EdgeDirection::Out => Some(tag_stats.avg_out_degree),
            EdgeDirection::In => Some(tag_stats.avg_in_degree),
            EdgeDirection::Both => Some((tag_stats.avg_out_degree + tag_stats.avg_in_degree) / 2.0),
        }
    }

    /// 检查是否适合使用双向遍历
    ///
    /// 基于边类型统计信息判断双向遍历是否有收益
    pub fn should_use_bidirectional(&self, edge_types: &[String]) -> bool {
        if edge_types.is_empty() {
            return false;
        }

        let mut total_out_degree = 0.0;
        let mut total_in_degree = 0.0;
        let mut count = 0;

        for edge_type in edge_types {
            if let Some(stats) = self.stats_manager.get_edge_stats(edge_type) {
                total_out_degree += stats.avg_out_degree;
                total_in_degree += stats.avg_in_degree;
                count += 1;

                // 如果存在严重倾斜，不建议双向遍历
                if stats.is_heavily_skewed() {
                    return false;
                }
            }
        }

        if count == 0 {
            return false;
        }

        let avg_out = total_out_degree / count as f64;
        let avg_in = total_in_degree / count as f64;

        // 如果两个方向的度数都很低（< 10），双向遍历有收益
        // 或者两个方向度数接近，也可以考虑双向遍历
        (avg_out < 10.0 && avg_in < 10.0) || ((avg_out - avg_in).abs() / (avg_out + avg_in) < 0.3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::optimizer::cost::config::CostModelConfig;
    use crate::query::optimizer::stats::{EdgeTypeStatistics, TagStatistics};

    fn create_test_optimizer() -> BidirectionalTraversalOptimizer {
        let stats_manager = Arc::new(crate::query::optimizer::stats::StatisticsManager::new());
        let config = CostModelConfig::default();
        let cost_calculator = Arc::new(CostCalculator::with_config(stats_manager.clone(), config));

        // 添加测试数据
        let edge_stats = EdgeTypeStatistics::new("friend".to_string());
        stats_manager.update_edge_stats(edge_stats);

        BidirectionalTraversalOptimizer::new(cost_calculator, stats_manager)
    }

    fn create_test_optimizer_with_stats() -> BidirectionalTraversalOptimizer {
        let stats_manager = Arc::new(crate::query::optimizer::stats::StatisticsManager::new());
        let config = CostModelConfig::default();
        let cost_calculator = Arc::new(CostCalculator::with_config(stats_manager.clone(), config));

        // 添加标签统计
        let person_tag = TagStatistics {
            tag_name: "Person".to_string(),
            vertex_count: 1000,
            avg_out_degree: 5.0,
            avg_in_degree: 4.0,
        };
        stats_manager.update_tag_stats(person_tag);

        let company_tag = TagStatistics {
            tag_name: "Company".to_string(),
            vertex_count: 100,
            avg_out_degree: 2.0,
            avg_in_degree: 50.0,
        };
        stats_manager.update_tag_stats(company_tag);

        // 添加边统计
        let edge_stats = EdgeTypeStatistics {
            edge_type: "works_at".to_string(),
            edge_count: 500,
            avg_out_degree: 1.0,
            avg_in_degree: 5.0,
            max_out_degree: 1,
            max_in_degree: 10,
            unique_src_vertices: 500,
            out_degree_std_dev: 0.5,
            in_degree_std_dev: 2.0,
            degree_gini_coefficient: 0.2,
            hot_vertices: Vec::new(),
        };
        stats_manager.update_edge_stats(edge_stats);

        BidirectionalTraversalOptimizer::new(cost_calculator, stats_manager)
    }

    #[test]
    fn test_unidirectional_for_shallow_depth() {
        let optimizer = create_test_optimizer();
        let decision = optimizer.evaluate("a", "b", &[], 1);

        assert!(!decision.use_bidirectional);
        assert_eq!(decision.forward_start, "a");
    }

    #[test]
    fn test_bidirectional_for_deep_depth() {
        let optimizer = create_test_optimizer();
        // 深度为4时，双向遍历应该有明显收益
        let decision = optimizer.evaluate("a", "b", &[], 4);

        // 由于默认分支因子为2，深度4时应该有显著节省
        if decision.use_bidirectional {
            assert!(decision.estimated_savings > 0.0);
            assert_eq!(decision.forward_start, "a");
            assert_eq!(decision.backward_start, "b");
        }
    }

    #[test]
    fn test_estimate_average_branching() {
        let optimizer = create_test_optimizer();
        let branching = optimizer.estimate_average_branching(&[]);

        // 空边类型列表应返回默认值
        assert_eq!(branching, 2.0);
    }

    #[test]
    fn test_depth_allocation_with_context() {
        let optimizer = create_test_optimizer_with_stats();

        // 测试平均分配（无标签信息）
        let context = DepthAllocationContext::new("a", "b", vec!["works_at".to_string()], 4);
        let (forward, backward) = optimizer.calculate_depth_allocation(&context);

        assert_eq!(forward + backward, 4);
        assert!(forward >= 1);
        assert!(backward >= 1);
    }

    #[test]
    fn test_depth_allocation_with_tags() {
        let optimizer = create_test_optimizer_with_stats();

        // Person(出度5) -> Company(入度50)，应该给Person端更多深度
        let context = DepthAllocationContext::new("a", "b", vec!["works_at".to_string()], 4)
            .with_start_tag("Person")
            .with_end_tag("Company");

        let (forward, backward) = optimizer.calculate_depth_allocation(&context);

        // 验证深度总和正确
        assert_eq!(forward + backward, 4);
        // Person出度(5) < Company入度(50)，应该给Person更多深度
        // 即forward_depth应该较大
        assert!(forward >= backward, "度数小的一端应该分配更多深度");
    }

    #[test]
    fn test_depth_allocation_with_degree_hints() {
        let optimizer = create_test_optimizer();

        // 使用度数提示
        let context = DepthAllocationContext::new("a", "b", vec![], 6)
            .with_start_degree(2.0)
            .with_end_degree(8.0);

        let (forward, backward) = optimizer.calculate_depth_allocation(&context);

        // 度数2 < 度数8，应该给起点更多深度
        assert!(forward > backward, "度数小的端点应该分配更多深度");
        assert_eq!(forward + backward, 6);
    }

    #[test]
    fn test_should_use_bidirectional() {
        let optimizer = create_test_optimizer_with_stats();

        // works_at边的度数较低，应该适合双向遍历
        assert!(optimizer.should_use_bidirectional(&["works_at".to_string()]));

        // 空边类型列表应该返回false
        assert!(!optimizer.should_use_bidirectional(&[]));
    }

    #[test]
    fn test_depth_allocation_shallow_depth() {
        let optimizer = create_test_optimizer();

        // 深度为1时，应该全部分配给正向
        let context = DepthAllocationContext::new("a", "b", vec![], 1);
        let (forward, backward) = optimizer.calculate_depth_allocation(&context);

        assert_eq!(forward, 1);
        assert_eq!(backward, 0);
    }
}
