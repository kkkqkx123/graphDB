//! 优化决策类型定义
//!
//! 定义从 AST 到物理执行计划的中间表示——优化决策。
//! 这些决策是基于代价的优化选择，但不包含具体的计划树结构。

use std::time::Instant;

/// 完整的优化决策
#[derive(Debug, Clone, PartialEq)]
pub struct OptimizationDecision {
    /// 遍历起点选择决策
    pub traversal_start: TraversalStartDecision,
    /// 索引选择决策
    pub index_selection: IndexSelectionDecision,
    /// 连接顺序决策
    pub join_order: JoinOrderDecision,
    /// 适用的重写规则序列
    pub rewrite_rules: Vec<RewriteRuleId>,
    /// 决策时的统计信息版本
    pub stats_version: u64,
    /// 决策时的索引版本
    pub index_version: u64,
    /// 决策时间戳
    pub created_at: Instant,
}

impl OptimizationDecision {
    /// 创建新的优化决策
    pub fn new(
        traversal_start: TraversalStartDecision,
        index_selection: IndexSelectionDecision,
        join_order: JoinOrderDecision,
        stats_version: u64,
        index_version: u64,
    ) -> Self {
        Self {
            traversal_start,
            index_selection,
            join_order,
            rewrite_rules: Vec::new(),
            stats_version,
            index_version,
            created_at: Instant::now(),
        }
    }

    /// 检查决策是否仍然有效
    pub fn is_valid(&self, current_stats_version: u64, current_index_version: u64) -> bool {
        self.stats_version == current_stats_version && self.index_version == current_index_version
    }

    /// 获取决策年龄（秒）
    pub fn age_secs(&self) -> u64 {
        self.created_at.elapsed().as_secs()
    }
}

/// 遍历起点选择决策
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraversalStartDecision {
    /// 起始节点变量名
    pub start_variable: String,
    /// 访问路径类型
    pub access_path: AccessPath,
    /// 估计的选择性（以整数表示，避免浮点精度问题）
    pub estimated_selectivity_scaled: u64, // 实际值 = 此值 / 1_000_000
    /// 估计的代价（以整数表示）
    pub estimated_cost_scaled: u64, // 实际值 = 此值 / 1_000_000
}

impl TraversalStartDecision {
    /// 创建新的遍历起点决策
    pub fn new(
        start_variable: String,
        access_path: AccessPath,
        estimated_selectivity: f64,
        estimated_cost: f64,
    ) -> Self {
        Self {
            start_variable,
            access_path,
            estimated_selectivity_scaled: (estimated_selectivity * 1_000_000.0) as u64,
            estimated_cost_scaled: (estimated_cost * 1_000_000.0) as u64,
        }
    }

    /// 获取估计选择性
    pub fn estimated_selectivity(&self) -> f64 {
        self.estimated_selectivity_scaled as f64 / 1_000_000.0
    }

    /// 获取估计代价
    pub fn estimated_cost(&self) -> f64 {
        self.estimated_cost_scaled as f64 / 1_000_000.0
    }
}

/// 访问路径类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AccessPath {
    /// 显式VID指定
    ExplicitVid {
        /// VID表达式描述（简化表示）
        vid_description: String,
    },
    /// 索引扫描
    IndexScan {
        /// 索引名称
        index_name: String,
        /// 属性名称
        property_name: String,
        /// 谓词描述
        predicate_description: String,
    },
    /// 标签索引
    TagIndex {
        /// 标签名称
        tag_name: String,
    },
    /// 全表扫描
    FullScan {
        /// 实体类型
        entity_type: EntityType,
    },
    /// 变量绑定
    VariableBinding {
        /// 源变量名
        source_variable: String,
    },
}

/// 实体类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntityType {
    /// 顶点
    Vertex {
        /// 标签名称（可选）
        tag_name: Option<String>,
    },
    /// 边
    Edge {
        /// 边类型（可选）
        edge_type: Option<String>,
    },
}

/// 索引选择决策
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndexSelectionDecision {
    /// 每个实体类型的索引选择
    pub entity_indexes: Vec<EntityIndexChoice>,
}

impl IndexSelectionDecision {
    /// 创建空的索引选择决策
    pub fn empty() -> Self {
        Self {
            entity_indexes: Vec::new(),
        }
    }

    /// 添加实体索引选择
    pub fn add_choice(&mut self, choice: EntityIndexChoice) {
        self.entity_indexes.push(choice);
    }
}

/// 实体索引选择
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityIndexChoice {
    /// 实体类型（标签或边类型）
    pub entity_name: String,
    /// 选择的索引
    pub selected_index: IndexChoice,
    /// 估计的选择性（缩放值）
    pub selectivity_scaled: u64,
}

impl EntityIndexChoice {
    /// 创建新的实体索引选择
    pub fn new(entity_name: String, selected_index: IndexChoice, selectivity: f64) -> Self {
        Self {
            entity_name,
            selected_index,
            selectivity_scaled: (selectivity * 1_000_000.0) as u64,
        }
    }

    /// 获取选择性
    pub fn selectivity(&self) -> f64 {
        self.selectivity_scaled as f64 / 1_000_000.0
    }
}

/// 索引选择
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IndexChoice {
    /// 主键索引
    PrimaryKey,
    /// 属性索引
    PropertyIndex {
        /// 属性名称
        property_name: String,
        /// 索引名称
        index_name: String,
    },
    /// 复合索引
    CompositeIndex {
        /// 属性名称列表
        property_names: Vec<String>,
        /// 索引名称
        index_name: String,
    },
    /// 无可用索引
    None,
}

/// 连接顺序决策
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JoinOrderDecision {
    /// 连接顺序（变量名序列）
    pub join_order: Vec<String>,
    /// 每个连接的算法选择
    pub join_algorithms: Vec<JoinAlgorithm>,
}

impl JoinOrderDecision {
    /// 创建空的连接顺序决策
    pub fn empty() -> Self {
        Self {
            join_order: Vec::new(),
            join_algorithms: Vec::new(),
        }
    }

    /// 添加连接步骤
    pub fn add_join_step(&mut self, variable: String, algorithm: JoinAlgorithm) {
        self.join_order.push(variable);
        self.join_algorithms.push(algorithm);
    }
}

/// 连接算法
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum JoinAlgorithm {
    /// 哈希连接
    HashJoin {
        /// 构建侧变量名
        build_side: String,
        /// 探测侧变量名
        probe_side: String,
    },
    /// 嵌套循环连接
    NestedLoopJoin {
        /// 外表变量名
        outer: String,
        /// 内表变量名
        inner: String,
    },
    /// 索引连接
    IndexJoin {
        /// 有索引的一侧变量名
        indexed_side: String,
    },
}

/// 重写规则ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RewriteRuleId {
    /// 谓词下推
    PushFilterDown,
    /// 投影下推
    PushProjectDown,
    /// LIMIT下推
    PushLimitDown,
    /// 操作合并
    MergeOperations,
    /// 冗余消除
    EliminateRedundancy,
    /// 聚合优化
    AggregateOptimization,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traversal_start_decision() {
        let decision = TraversalStartDecision::new(
            "n".to_string(),
            AccessPath::TagIndex {
                tag_name: "Person".to_string(),
            },
            0.1,
            100.0,
        );

        assert_eq!(decision.start_variable, "n");
        assert!((decision.estimated_selectivity() - 0.1).abs() < 0.0001);
        assert!((decision.estimated_cost() - 100.0).abs() < 0.0001);
    }

    #[test]
    fn test_optimization_decision_validity() {
        let decision = OptimizationDecision::new(
            TraversalStartDecision::new(
                "n".to_string(),
                AccessPath::FullScan {
                    entity_type: EntityType::Vertex { tag_name: None },
                },
                1.0,
                1000.0,
            ),
            IndexSelectionDecision::empty(),
            JoinOrderDecision::empty(),
            1,
            1,
        );

        assert!(decision.is_valid(1, 1));
        assert!(!decision.is_valid(2, 1));
        assert!(!decision.is_valid(1, 2));
    }
}
