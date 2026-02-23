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
        PushFilterDownJoin(predicate_pushdown::PushFilterDownJoinRule),
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
        registry.add(RewriteRule::EliminateFilter(elimination::EliminateFilterRule));
        registry.add(RewriteRule::RemoveNoopProject(elimination::RemoveNoopProjectRule));
        registry.add(RewriteRule::EliminateAppendVertices(elimination::EliminateAppendVerticesRule));
        registry.add(RewriteRule::RemoveAppendVerticesBelowJoin(elimination::RemoveAppendVerticesBelowJoinRule));
        registry.add(RewriteRule::EliminateRowCollect(elimination::EliminateRowCollectRule));
        registry.add(RewriteRule::EliminateEmptySetOperation(elimination::EliminateEmptySetOperationRule));
        registry.add(RewriteRule::DedupElimination(elimination::DedupEliminationRule));
        registry.add(RewriteRule::CombineFilter(merge::CombineFilterRule));
        registry.add(RewriteRule::CollapseProject(merge::CollapseProjectRule));
        registry.add(RewriteRule::CollapseConsecutiveProject(merge::CollapseConsecutiveProjectRule));
        registry.add(RewriteRule::MergeGetVerticesAndProject(merge::MergeGetVerticesAndProjectRule));
        registry.add(RewriteRule::MergeGetVerticesAndDedup(merge::MergeGetVerticesAndDedupRule));
        registry.add(RewriteRule::MergeGetNbrsAndProject(merge::MergeGetNbrsAndProjectRule));
        registry.add(RewriteRule::MergeGetNbrsAndDedup(merge::MergeGetNbrsAndDedupRule));
        registry.add(RewriteRule::PushFilterDownTraverse(predicate_pushdown::PushFilterDownTraverseRule));
        registry.add(RewriteRule::PushFilterDownExpandAll(predicate_pushdown::PushFilterDownExpandAllRule));
        registry.add(RewriteRule::PushFilterDownJoin(predicate_pushdown::PushFilterDownJoinRule));
        registry.add(RewriteRule::PushFilterDownNode(predicate_pushdown::PushFilterDownNodeRule));
        registry.add(RewriteRule::PushEFilterDown(predicate_pushdown::PushEFilterDownRule));
        registry.add(RewriteRule::PushVFilterDownScanVertices(predicate_pushdown::PushVFilterDownScanVerticesRule));
        registry.add(RewriteRule::PushFilterDownInnerJoin(predicate_pushdown::PushFilterDownInnerJoinRule));
        registry.add(RewriteRule::PushFilterDownHashInnerJoin(predicate_pushdown::PushFilterDownHashInnerJoinRule));
        registry.add(RewriteRule::PushFilterDownHashLeftJoin(predicate_pushdown::PushFilterDownHashLeftJoinRule));
        registry.add(RewriteRule::PushFilterDownCrossJoin(predicate_pushdown::PushFilterDownCrossJoinRule));
        registry.add(RewriteRule::PushFilterDownGetNbrs(predicate_pushdown::PushFilterDownGetNbrsRule));
        registry.add(RewriteRule::PushFilterDownAllPaths(predicate_pushdown::PushFilterDownAllPathsRule));
        registry.add(RewriteRule::ProjectionPushDown(projection_pushdown::ProjectionPushDownRule));
        registry.add(RewriteRule::PushProjectDown(projection_pushdown::PushProjectDownRule));
        registry.add(RewriteRule::PushLimitDownGetVertices(limit_pushdown::PushLimitDownGetVerticesRule));
        registry.add(RewriteRule::PushLimitDownGetEdges(limit_pushdown::PushLimitDownGetEdgesRule));
        registry.add(RewriteRule::PushLimitDownScanVertices(limit_pushdown::PushLimitDownScanVerticesRule));
        registry.add(RewriteRule::PushLimitDownScanEdges(limit_pushdown::PushLimitDownScanEdgesRule));
        registry.add(RewriteRule::PushLimitDownIndexScan(limit_pushdown::PushLimitDownIndexScanRule));
        registry.add(RewriteRule::PushTopNDownIndexScan(limit_pushdown::PushTopNDownIndexScanRule));
        registry.add(RewriteRule::PushFilterDownAggregate(aggregate::PushFilterDownAggregateRule));
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_registry_default() {
        let registry = RuleRegistry::default();
        assert_eq!(registry.len(), 35);
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
