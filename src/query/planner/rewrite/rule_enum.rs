//! 重写规则枚举 - 静态分发实现
//!
//! 该模块使用枚举实现静态分发，避免动态分发的开销。
//! 所有规则都作为枚举变体，通过 match 进行分发。
//!
//! # 优势
//!
//! - 无动态分发开销（无虚函数表查找）
//! - 无堆分配（规则存储在栈上）
//! - 更好的缓存局部性
//! - 编译器可以内联优化
//!
//! # 使用示例
//!
//! ```rust
//! use crate::query::planner::rewrite::rule_enum::{RewriteRule, RuleRegistry};
//!
//! // 创建规则注册表
//! let registry = RuleRegistry::default();
//!
//! // 应用规则
//! for rule in registry.iter() {
//!     if let Some(result) = rule.apply(ctx, node)? {
//!         // 处理结果
//!     }
//! }
//! ```

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::RewriteRule as RewriteRuleTrait;
use crate::query::planner::rewrite::elimination;
use crate::query::planner::rewrite::merge;
use crate::query::planner::rewrite::predicate_pushdown;
use crate::query::planner::rewrite::projection_pushdown;
use crate::query::planner::rewrite::limit_pushdown;
use crate::query::planner::rewrite::aggregate;

macro_rules! define_rewrite_rules {
    (
        $(#[$enum_meta:meta])*
        pub enum $enum_name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant_name:ident($rule_type:ty)
            ),+ $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        #[derive(Debug)]
        pub enum $enum_name {
            $(
                $(#[$variant_meta])*
                $variant_name($rule_type),
            )+
        }

        impl $enum_name {
            pub fn name(&self) -> &'static str {
                match self {
                    $(
                        $enum_name::$variant_name(_) => {
                            let type_name = stringify!($rule_type);
                            if let Some(pos) = type_name.rfind("::") {
                                &type_name[pos + 2..]
                            } else {
                                type_name
                            }
                        }
                    )+
                }
            }

            pub fn pattern(&self) -> Pattern {
                match self {
                    $(
                        $enum_name::$variant_name(rule) => rule.pattern(),
                    )+
                }
            }

            pub fn apply(
                &self,
                ctx: &mut RewriteContext,
                node: &PlanNodeEnum,
            ) -> RewriteResult<Option<TransformResult>> {
                match self {
                    $(
                        $enum_name::$variant_name(rule) => rule.apply(ctx, node),
                    )+
                }
            }

            pub fn matches(&self, node: &PlanNodeEnum) -> bool {
                self.pattern().matches(node)
            }
        }

        impl RewriteRuleTrait for $enum_name {
            fn name(&self) -> &'static str {
                self.name()
            }

            fn pattern(&self) -> Pattern {
                self.pattern()
            }

            fn apply(
                &self,
                ctx: &mut RewriteContext,
                node: &PlanNodeEnum,
            ) -> RewriteResult<Option<TransformResult>> {
                self.apply(ctx, node)
            }
        }
    };
}

define_rewrite_rules! {
    pub enum RewriteRule {
        // ==================== 消除规则 ====================
        EliminateFilter(elimination::EliminateFilterRule),
        RemoveNoopProject(elimination::RemoveNoopProjectRule),
        EliminateAppendVertices(elimination::EliminateAppendVerticesRule),
        RemoveAppendVerticesBelowJoin(elimination::RemoveAppendVerticesBelowJoinRule),
        EliminateRowCollect(elimination::EliminateRowCollectRule),
        EliminateEmptySetOperation(elimination::EliminateEmptySetOperationRule),
        DedupElimination(elimination::DedupEliminationRule),
        EliminateSort(elimination::EliminateSortRule),

        // ==================== 合并规则 ====================
        CombineFilter(merge::CombineFilterRule),
        CollapseProject(merge::CollapseProjectRule),
        CollapseConsecutiveProject(merge::CollapseConsecutiveProjectRule),
        MergeGetVerticesAndProject(merge::MergeGetVerticesAndProjectRule),
        MergeGetVerticesAndDedup(merge::MergeGetVerticesAndDedupRule),
        MergeGetNbrsAndProject(merge::MergeGetNbrsAndProjectRule),
        MergeGetNbrsAndDedup(merge::MergeGetNbrsAndDedupRule),

        // ==================== 谓词下推规则 ====================
        PushFilterDownTraverse(predicate_pushdown::PushFilterDownTraverseRule),
        PushFilterDownExpandAll(predicate_pushdown::PushFilterDownExpandAllRule),
        PushFilterDownNode(predicate_pushdown::PushFilterDownNodeRule),
        PushEFilterDown(predicate_pushdown::PushEFilterDownRule),
        PushVFilterDownScanVertices(predicate_pushdown::PushVFilterDownScanVerticesRule),
        PushFilterDownInnerJoin(predicate_pushdown::PushFilterDownInnerJoinRule),
        PushFilterDownHashInnerJoin(predicate_pushdown::PushFilterDownHashInnerJoinRule),
        PushFilterDownHashLeftJoin(predicate_pushdown::PushFilterDownHashLeftJoinRule),
        PushFilterDownCrossJoin(predicate_pushdown::PushFilterDownCrossJoinRule),
        PushFilterDownGetNbrs(predicate_pushdown::PushFilterDownGetNbrsRule),
        PushFilterDownAllPaths(predicate_pushdown::PushFilterDownAllPathsRule),

        // ==================== 投影下推规则 ====================
        ProjectionPushDown(projection_pushdown::ProjectionPushDownRule),
        PushProjectDown(projection_pushdown::PushProjectDownRule),

        // ==================== LIMIT下推规则 ====================
        PushLimitDownGetVertices(limit_pushdown::PushLimitDownGetVerticesRule),
        PushLimitDownGetEdges(limit_pushdown::PushLimitDownGetEdgesRule),
        PushLimitDownScanVertices(limit_pushdown::PushLimitDownScanVerticesRule),
        PushLimitDownScanEdges(limit_pushdown::PushLimitDownScanEdgesRule),
        PushLimitDownIndexScan(limit_pushdown::PushLimitDownIndexScanRule),
        PushTopNDownIndexScan(limit_pushdown::PushTopNDownIndexScanRule),

        // ==================== 聚合优化规则 ====================
        PushFilterDownAggregate(aggregate::PushFilterDownAggregateRule),
    }
}

#[derive(Debug)]
pub struct RuleRegistry {
    rules: Vec<RewriteRule>,
}

impl RuleRegistry {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add(&mut self, rule: RewriteRule) {
        self.rules.push(rule);
    }

    pub fn iter(&self) -> impl Iterator<Item = &RewriteRule> {
        self.rules.iter()
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub fn clear(&mut self) {
        self.rules.clear();
    }

    pub fn into_vec(self) -> Vec<RewriteRule> {
        self.rules
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.add(RewriteRule::EliminateFilter(elimination::EliminateFilterRule::new()));
        registry.add(RewriteRule::RemoveNoopProject(elimination::RemoveNoopProjectRule::new()));
        registry.add(RewriteRule::EliminateAppendVertices(elimination::EliminateAppendVerticesRule::new()));
        registry.add(RewriteRule::RemoveAppendVerticesBelowJoin(elimination::RemoveAppendVerticesBelowJoinRule::new()));
        registry.add(RewriteRule::EliminateRowCollect(elimination::EliminateRowCollectRule::new()));
        registry.add(RewriteRule::EliminateEmptySetOperation(elimination::EliminateEmptySetOperationRule::new()));
        registry.add(RewriteRule::DedupElimination(elimination::DedupEliminationRule::new()));
        registry.add(RewriteRule::EliminateSort(elimination::EliminateSortRule::new()));
        registry.add(RewriteRule::CombineFilter(merge::CombineFilterRule::new()));
        registry.add(RewriteRule::CollapseProject(merge::CollapseProjectRule::new()));
        registry.add(RewriteRule::CollapseConsecutiveProject(merge::CollapseConsecutiveProjectRule::new()));
        registry.add(RewriteRule::MergeGetVerticesAndProject(merge::MergeGetVerticesAndProjectRule::new()));
        registry.add(RewriteRule::MergeGetVerticesAndDedup(merge::MergeGetVerticesAndDedupRule::new()));
        registry.add(RewriteRule::MergeGetNbrsAndProject(merge::MergeGetNbrsAndProjectRule::new()));
        registry.add(RewriteRule::MergeGetNbrsAndDedup(merge::MergeGetNbrsAndDedupRule::new()));
        registry.add(RewriteRule::PushFilterDownTraverse(predicate_pushdown::PushFilterDownTraverseRule::new()));
        registry.add(RewriteRule::PushFilterDownExpandAll(predicate_pushdown::PushFilterDownExpandAllRule::new()));
        registry.add(RewriteRule::PushFilterDownNode(predicate_pushdown::PushFilterDownNodeRule::new()));
        registry.add(RewriteRule::PushEFilterDown(predicate_pushdown::PushEFilterDownRule::new()));
        registry.add(RewriteRule::PushVFilterDownScanVertices(predicate_pushdown::PushVFilterDownScanVerticesRule::new()));
        registry.add(RewriteRule::PushFilterDownInnerJoin(predicate_pushdown::PushFilterDownInnerJoinRule::new()));
        registry.add(RewriteRule::PushFilterDownHashInnerJoin(predicate_pushdown::PushFilterDownHashInnerJoinRule::new()));
        registry.add(RewriteRule::PushFilterDownHashLeftJoin(predicate_pushdown::PushFilterDownHashLeftJoinRule::new()));
        registry.add(RewriteRule::PushFilterDownCrossJoin(predicate_pushdown::PushFilterDownCrossJoinRule::new()));
        registry.add(RewriteRule::PushFilterDownGetNbrs(predicate_pushdown::PushFilterDownGetNbrsRule::new()));
        registry.add(RewriteRule::PushFilterDownAllPaths(predicate_pushdown::PushFilterDownAllPathsRule::new()));
        registry.add(RewriteRule::ProjectionPushDown(projection_pushdown::ProjectionPushDownRule::new()));
        registry.add(RewriteRule::PushProjectDown(projection_pushdown::PushProjectDownRule::new()));
        registry.add(RewriteRule::PushLimitDownGetVertices(limit_pushdown::PushLimitDownGetVerticesRule::new()));
        registry.add(RewriteRule::PushLimitDownGetEdges(limit_pushdown::PushLimitDownGetEdgesRule::new()));
        registry.add(RewriteRule::PushLimitDownScanVertices(limit_pushdown::PushLimitDownScanVerticesRule::new()));
        registry.add(RewriteRule::PushLimitDownScanEdges(limit_pushdown::PushLimitDownScanEdgesRule::new()));
        registry.add(RewriteRule::PushLimitDownIndexScan(limit_pushdown::PushLimitDownIndexScanRule::new()));
        registry.add(RewriteRule::PushTopNDownIndexScan(limit_pushdown::PushTopNDownIndexScanRule::new()));
        registry.add(RewriteRule::PushFilterDownAggregate(aggregate::PushFilterDownAggregateRule::new()));
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_registry_default() {
        let registry = RuleRegistry::default();
        assert_eq!(registry.len(), 34);
    }

    #[test]
    fn test_rule_names() {
        let registry = RuleRegistry::default();
        for rule in registry.iter() {
            let name = rule.name();
            assert!(!name.is_empty());
            assert!(name.ends_with("Rule"));
        }
    }
}
