//! 查找策略模块
//! 定义查找策略的公共trait和接口

use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::path_structs::NodeInfo;

/// 查找策略trait
///
/// 所有查找策略都应该实现这个trait，提供统一的接口
pub trait SeekStrategy {
    /// 构建查找计划
    ///
    /// 根据节点信息构建相应的查找计划
    fn build_plan(&self) -> Result<SubPlan, PlannerError>;

    /// 检查是否可以使用该查找策略
    ///
    /// 根据节点信息判断是否可以使用该查找策略
    fn match_node(&self) -> bool;

    /// 获取查找策略的名称
    ///
    /// 返回查找策略的名称，用于调试和日志
    fn name(&self) -> &'static str;

    /// 估算查找成本
    ///
    /// 返回该查找策略的估算成本，用于选择最优策略
    fn estimate_cost(&self) -> f64;
}

/// 查找策略类型枚举
///
/// 用于标识不同的查找策略类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekStrategyType {
    /// 扫描查找
    Scan,
    /// 标签索引查找
    LabelIndex,
    /// 属性索引查找
    PropIndex,
    /// 顶点ID查找
    VertexId,
    /// 可变顶点ID查找
    VariableVertexId,
    /// 可变属性索引查找
    VariablePropIndex,
}

impl SeekStrategyType {
    /// 获取查找策略的优先级
    ///
    /// 返回查找策略的优先级，数值越小优先级越高
    pub fn priority(&self) -> u8 {
        match self {
            SeekStrategyType::VertexId => 1,          // 顶点ID查找优先级最高
            SeekStrategyType::VariableVertexId => 2,  // 可变顶点ID查找次之
            SeekStrategyType::PropIndex => 3,         // 属性索引查找
            SeekStrategyType::VariablePropIndex => 4, // 可变属性索引查找
            SeekStrategyType::LabelIndex => 5,        // 标签索引查找
            SeekStrategyType::Scan => 6,              // 扫描查找优先级最低
        }
    }

    /// 获取查找策略的默认成本
    ///
    /// 返回查找策略的默认成本估算
    pub fn default_cost(&self) -> f64 {
        match self {
            SeekStrategyType::VertexId => 1.0,           // 顶点ID查找成本最低
            SeekStrategyType::VariableVertexId => 5.0,   // 可变顶点ID查找成本较低
            SeekStrategyType::PropIndex => 10.0,         // 属性索引查找成本中等
            SeekStrategyType::VariablePropIndex => 20.0, // 可变属性索引查找成本较高
            SeekStrategyType::LabelIndex => 50.0,        // 标签索引查找成本较高
            SeekStrategyType::Scan => 1000.0,            // 扫描查找成本最高
        }
    }
}

/// 查找策略选择器
///
/// 用于根据节点信息选择最优的查找策略
pub struct SeekStrategySelector;

impl SeekStrategySelector {
    /// 选择最优的查找策略
    ///
    /// 根据节点信息和可用策略选择最优的查找策略
    pub fn select_best_strategy(
        _node_info: &NodeInfo,
        strategies: &[Box<dyn SeekStrategy>],
    ) -> Option<usize> {
        let mut best_index = None;
        let mut best_cost = f64::MAX;

        for (index, strategy) in strategies.iter().enumerate() {
            if strategy.match_node() {
                let cost = strategy.estimate_cost();
                if cost < best_cost {
                    best_cost = cost;
                    best_index = Some(index);
                }
            }
        }

        best_index
    }

    /// 按优先级排序策略
    ///
    /// 根据策略类型和优先级对策略进行排序
    pub fn sort_strategies_by_priority(strategies: &mut [Box<dyn SeekStrategy>]) {
        strategies.sort_by(|a, b| {
            let cost_a = a.estimate_cost();
            let cost_b = b.estimate_cost();
            cost_a
                .partial_cmp(&cost_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试用的查找策略实现
    struct TestSeekStrategy {
        strategy_type: SeekStrategyType,
        can_match: bool,
    }

    impl SeekStrategy for TestSeekStrategy {
        fn build_plan(&self) -> Result<SubPlan, PlannerError> {
            Err(PlannerError::UnsupportedOperation(
                "Test strategy".to_string(),
            ))
        }

        fn match_node(&self) -> bool {
            self.can_match
        }

        fn name(&self) -> &'static str {
            "TestSeekStrategy"
        }

        fn estimate_cost(&self) -> f64 {
            self.strategy_type.default_cost()
        }
    }

    #[test]
    fn test_seek_strategy_type_priority() {
        assert_eq!(SeekStrategyType::VertexId.priority(), 1);
        assert_eq!(SeekStrategyType::Scan.priority(), 6);
    }

    #[test]
    fn test_seek_strategy_type_default_cost() {
        assert_eq!(SeekStrategyType::VertexId.default_cost(), 1.0);
        assert_eq!(SeekStrategyType::Scan.default_cost(), 1000.0);
    }

    #[test]
    fn test_select_best_strategy() {
        let strategies: Vec<Box<dyn SeekStrategy>> = vec![
            Box::new(TestSeekStrategy {
                strategy_type: SeekStrategyType::Scan,
                can_match: true,
            }),
            Box::new(TestSeekStrategy {
                strategy_type: SeekStrategyType::VertexId,
                can_match: true,
            }),
        ];

        let best_index =
            SeekStrategySelector::select_best_strategy(&NodeInfo::default(), &strategies);
        assert_eq!(best_index, Some(1)); // VertexId策略应该被选中
    }

    #[test]
    fn test_select_best_strategy_no_match() {
        let strategies: Vec<Box<dyn SeekStrategy>> = vec![
            Box::new(TestSeekStrategy {
                strategy_type: SeekStrategyType::Scan,
                can_match: false,
            }),
            Box::new(TestSeekStrategy {
                strategy_type: SeekStrategyType::VertexId,
                can_match: false,
            }),
        ];

        let best_index =
            SeekStrategySelector::select_best_strategy(&NodeInfo::default(), &strategies);
        assert_eq!(best_index, None); // 没有匹配的策略
    }
}
