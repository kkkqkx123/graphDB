//! 优化规则枚举定义
//! 使用枚举替代字符串匹配规则，提高类型安全性和可维护性
//!
//! # 说明
//!
//! 该枚举仅包含基于代价的优化规则（CBO）。
//! 启发式规则（如谓词下推、投影下推等）已迁移到 planner/rewrite 模块，
//! 在计划生成阶段直接应用，不再通过优化器枚举管理。

use std::rc::Rc;

use crate::query::optimizer::core::OptimizationPhase;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptimizationRule {
    // 逻辑优化规则（基于代价）
    TopN,
    OptimizeSetOperationInputOrder,

    // 物理优化规则（基于代价）
    JoinOptimization,
    ScanWithFilterOptimization,
    IndexFullScan,
    IndexScan,
    EdgeIndexFullScan,
    TagIndexFullScan,
    UnionAllEdgeIndexScan,
    UnionAllTagIndexScan,

    // 索引覆盖扫描规则
    IndexCoveringScan,
}

impl OptimizationRule {
    pub fn phase(&self) -> OptimizationPhase {
        match self {
            Self::TopN | Self::OptimizeSetOperationInputOrder => OptimizationPhase::Logical,

            Self::JoinOptimization
            | Self::ScanWithFilterOptimization
            | Self::IndexFullScan
            | Self::IndexScan
            | Self::EdgeIndexFullScan
            | Self::TagIndexFullScan
            | Self::UnionAllEdgeIndexScan
            | Self::UnionAllTagIndexScan
            | Self::IndexCoveringScan => OptimizationPhase::Physical,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::TopN => "TopNRule",
            Self::OptimizeSetOperationInputOrder => "OptimizeSetOperationInputOrderRule",
            Self::JoinOptimization => "JoinOptimizationRule",
            Self::ScanWithFilterOptimization => "ScanWithFilterOptimizationRule",
            Self::IndexFullScan => "IndexFullScanRule",
            Self::IndexScan => "IndexScanRule",
            Self::EdgeIndexFullScan => "EdgeIndexFullScanRule",
            Self::TagIndexFullScan => "TagIndexFullScanRule",
            Self::UnionAllEdgeIndexScan => "UnionAllEdgeIndexScanRule",
            Self::UnionAllTagIndexScan => "UnionAllTagIndexScanRule",
            Self::IndexCoveringScan => "IndexCoveringScanRule",
        }
    }

    pub fn create_instance(&self) -> Option<Rc<dyn super::OptRule>> {
        match self {
            Self::TopN => Some(Rc::new(super::TopNRule)),
            Self::OptimizeSetOperationInputOrder => Some(Rc::new(super::OptimizeSetOperationInputOrderRule)),
            Self::JoinOptimization => Some(Rc::new(super::JoinOptimizationRule)),
            Self::ScanWithFilterOptimization => Some(Rc::new(super::ScanWithFilterOptimizationRule)),
            Self::IndexFullScan => Some(Rc::new(super::IndexFullScanRule)),
            Self::IndexScan => Some(Rc::new(super::IndexScanRule)),
            Self::EdgeIndexFullScan => Some(Rc::new(super::EdgeIndexFullScanRule)),
            Self::TagIndexFullScan => Some(Rc::new(super::TagIndexFullScanRule)),
            Self::UnionAllEdgeIndexScan => Some(Rc::new(super::UnionAllEdgeIndexScanRule)),
            Self::UnionAllTagIndexScan => Some(Rc::new(super::UnionAllTagIndexScanRule)),
            Self::IndexCoveringScan => Some(Rc::new(super::IndexCoveringScanRule)),
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "TopNRule" => Some(Self::TopN),
            "OptimizeSetOperationInputOrderRule" => Some(Self::OptimizeSetOperationInputOrder),
            "JoinOptimizationRule" => Some(Self::JoinOptimization),
            "ScanWithFilterOptimizationRule" => Some(Self::ScanWithFilterOptimization),
            "IndexFullScanRule" => Some(Self::IndexFullScan),
            "IndexScanRule" => Some(Self::IndexScan),
            "EdgeIndexFullScanRule" => Some(Self::EdgeIndexFullScan),
            "TagIndexFullScanRule" => Some(Self::TagIndexFullScan),
            "UnionAllEdgeIndexScanRule" => Some(Self::UnionAllEdgeIndexScan),
            "UnionAllTagIndexScanRule" => Some(Self::UnionAllTagIndexScan),
            "IndexCoveringScanRule" => Some(Self::IndexCoveringScan),
            _ => None,
        }
    }

    /// 获取所有基于代价的优化规则
    pub fn all_cbo_rules() -> Vec<Self> {
        vec![
            Self::TopN,
            Self::OptimizeSetOperationInputOrder,
            Self::JoinOptimization,
            Self::ScanWithFilterOptimization,
            Self::IndexFullScan,
            Self::IndexScan,
            Self::EdgeIndexFullScan,
            Self::TagIndexFullScan,
            Self::UnionAllEdgeIndexScan,
            Self::UnionAllTagIndexScan,
            Self::IndexCoveringScan,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_phase() {
        assert_eq!(OptimizationRule::TopN.phase(), OptimizationPhase::Logical);
        assert_eq!(
            OptimizationRule::JoinOptimization.phase(),
            OptimizationPhase::Physical
        );
    }

    #[test]
    fn test_rule_name_roundtrip() {
        for rule in OptimizationRule::all_cbo_rules() {
            let name = rule.name();
            let parsed = OptimizationRule::from_name(name);
            assert_eq!(parsed, Some(rule), "Failed to roundtrip rule: {}", name);
        }
    }
}
