//! 计划重写模块集成测试
//!
//! 测试范围:
//! - rewrite::rule_enum - 静态分发规则枚举
//! - rewrite::plan_rewriter - 计划重写器
//! - rewrite::RuleRegistry - 规则注册表
//! - 各种重写规则的集成测试

mod common;

use graphdb::query::planner::rewrite::{
    RewriteRuleEnum,
    RuleRegistry,
    PlanRewriter,
    create_default_rewriter,
    EliminateFilterRule,
    RemoveNoopProjectRule,
    EliminateAppendVerticesRule,
    RemoveAppendVerticesBelowJoinRule,
    EliminateRowCollectRule,
    EliminateEmptySetOperationRule,
    DedupEliminationRule,
    CombineFilterRule,
    CollapseProjectRule,
    CollapseConsecutiveProjectRule,
    MergeGetVerticesAndProjectRule,
    MergeGetVerticesAndDedupRule,
    MergeGetNbrsAndProjectRule,
    MergeGetNbrsAndDedupRule,
    PushFilterDownTraverseRule,
    PushFilterDownExpandAllRule,
    PushFilterDownNodeRule,
    PushEFilterDownRule,
    PushVFilterDownScanVerticesRule,
    PushFilterDownInnerJoinRule,
    PushFilterDownHashInnerJoinRule,
    PushFilterDownHashLeftJoinRule,
    PushFilterDownCrossJoinRule,
    PushFilterDownGetNbrsRule,
    PushFilterDownAllPathsRule,
    ProjectionPushDownRule,
    PushProjectDownRule,
    PushLimitDownGetVerticesRule,
    PushLimitDownGetEdgesRule,
    PushLimitDownScanVerticesRule,
    PushLimitDownScanEdgesRule,
    PushLimitDownIndexScanRule,
    PushTopNDownIndexScanRule,
    PushFilterDownAggregateRule,
};
use graphdb::query::planner::rewrite::rule::RewriteRule;

// ==================== RuleRegistry 集成测试 ====================

#[test]
fn test_rule_registry_default() {
    let registry = RuleRegistry::default();
    assert_eq!(registry.len(), 34, "默认注册表应包含 34 个规则");
    assert!(!registry.is_empty(), "注册表不应为空");
}

#[test]
fn test_rule_registry_iter() {
    let registry = RuleRegistry::default();
    let mut count = 0;

    for rule in registry.iter() {
        count += 1;
        let name = rule.name();
        assert!(!name.is_empty(), "规则名称不应为空");
        assert!(name.ends_with("Rule"), "规则名称应以 'Rule' 结尾");
    }

    assert_eq!(count, 34, "应迭代所有 34 个规则");
}

#[test]
fn test_rule_registry_new() {
    let registry = RuleRegistry::new();
    assert_eq!(registry.len(), 0, "新注册表应为空");
    assert!(registry.is_empty(), "新注册表应为空");
}

#[test]
fn test_rule_registry_add() {
    let mut registry = RuleRegistry::new();

    registry.add(RewriteRuleEnum::EliminateFilter(
        EliminateFilterRule::new()
    ));

    assert_eq!(registry.len(), 1, "添加一个规则后长度应为 1");
    assert!(!registry.is_empty(), "添加规则后不应为空");
}

#[test]
fn test_rule_registry_clear() {
    let mut registry = RuleRegistry::default();
    assert_eq!(registry.len(), 34, "默认注册表应有 34 个规则");

    registry.clear();
    assert_eq!(registry.len(), 0, "清空后长度应为 0");
    assert!(registry.is_empty(), "清空后应为空");
}

#[test]
fn test_rule_registry_into_vec() {
    let registry = RuleRegistry::default();
    let rules = registry.into_vec();

    assert_eq!(rules.len(), 34, "转换后的 Vec 应包含 34 个规则");
}

// ==================== RewriteRule 集成测试 ====================

#[test]
fn test_rewrite_rule_names() {
    let registry = RuleRegistry::default();

    let expected_names = vec![
        // 消除规则
        "EliminateFilterRule",
        "RemoveNoopProjectRule",
        "EliminateAppendVerticesRule",
        "RemoveAppendVerticesBelowJoinRule",
        "EliminateRowCollectRule",
        "EliminateEmptySetOperationRule",
        "DedupEliminationRule",
        // 合并规则
        "CombineFilterRule",
        "CollapseProjectRule",
        "CollapseConsecutiveProjectRule",
        "MergeGetVerticesAndProjectRule",
        "MergeGetVerticesAndDedupRule",
        "MergeGetNbrsAndProjectRule",
        "MergeGetNbrsAndDedupRule",
        // 谓词下推规则
        "PushFilterDownTraverseRule",
        "PushFilterDownExpandAllRule",
        "PushFilterDownNodeRule",
        "PushEFilterDownRule",
        "PushVFilterDownScanVerticesRule",
        "PushFilterDownInnerJoinRule",
        "PushFilterDownHashInnerJoinRule",
        "PushFilterDownHashLeftJoinRule",
        "PushFilterDownCrossJoinRule",
        "PushFilterDownGetNbrsRule",
        "PushFilterDownAllPathsRule",
        // 投影下推规则
        "ProjectionPushDownRule",
        "PushProjectDownRule",
        // LIMIT 下推规则
        "PushLimitDownGetVerticesRule",
        "PushLimitDownGetEdgesRule",
        "PushLimitDownScanVerticesRule",
        "PushLimitDownScanEdgesRule",
        "PushLimitDownIndexScanRule",
        "PushTopNDownIndexScanRule",
        // 聚合优化规则
        "PushFilterDownAggregateRule",
    ];

    let mut actual_names: Vec<_> = registry.iter()
        .map(|rule| rule.name())
        .collect();

    actual_names.sort();
    let mut expected_sorted = expected_names.clone();
    expected_sorted.sort();

    assert_eq!(actual_names, expected_sorted, "规则名称列表应匹配");
}

#[test]
fn test_rewrite_rule_pattern() {
    let rule = RewriteRuleEnum::EliminateFilter(
        EliminateFilterRule::new()
    );

    let pattern = rule.pattern();
    // 验证 pattern 方法可以正常调用
    let _ = pattern;
}

#[test]
fn test_rewrite_rule_trait_methods() {
    let rule = RewriteRuleEnum::EliminateFilter(
        EliminateFilterRule::new()
    );

    // 测试 RewriteRule trait 方法
    let name = rule.name();
    assert_eq!(name, "EliminateFilterRule");

    let pattern = rule.pattern();
    let _ = pattern;
}

// ==================== PlanRewriter 集成测试 ====================

#[test]
fn test_plan_rewriter_new() {
    let rewriter = PlanRewriter::new();
    // 验证重写器可以创建
    let _ = rewriter;
}

#[test]
fn test_plan_rewriter_from_registry() {
    let registry = RuleRegistry::default();
    let rewriter = PlanRewriter::from_registry(registry);
    // 验证重写器可以从注册表创建
    let _ = rewriter;
}

#[test]
fn test_plan_rewriter_default() {
    let rewriter = PlanRewriter::default();
    // 验证重写器默认实现
    let _ = rewriter;
}

#[test]
fn test_create_default_rewriter() {
    let rewriter = create_default_rewriter();
    // 验证默认重写器创建函数
    let _ = rewriter;
}

// ==================== 静态分发性能测试 ====================

#[test]
fn test_static_dispatch_overhead() {
    let registry = RuleRegistry::default();
    let rules: Vec<_> = registry.iter().collect();

    // 测试静态分发的性能
    // 通过多次调用验证没有明显的性能问题
    let iterations = 1000;

    for _ in 0..iterations {
        for rule in &rules {
            let name = rule.name();
            assert!(!name.is_empty());
        }
    }
}

// ==================== 规则分类测试 ====================

#[test]
fn test_elimination_rules_count() {
    let registry = RuleRegistry::default();

    let elimination_rules: Vec<_> = registry.iter()
        .filter(|rule| matches!(rule,
            RewriteRuleEnum::EliminateFilter(_) |
            RewriteRuleEnum::RemoveNoopProject(_) |
            RewriteRuleEnum::EliminateAppendVertices(_) |
            RewriteRuleEnum::RemoveAppendVerticesBelowJoin(_) |
            RewriteRuleEnum::EliminateRowCollect(_) |
            RewriteRuleEnum::EliminateEmptySetOperation(_) |
            RewriteRuleEnum::DedupElimination(_)
        ))
        .collect();

    assert_eq!(elimination_rules.len(), 7, "应有 7 个消除规则");
}

#[test]
fn test_merge_rules_count() {
    let registry = RuleRegistry::default();

    let merge_rules: Vec<_> = registry.iter()
        .filter(|rule| matches!(rule,
            RewriteRuleEnum::CombineFilter(_) |
            RewriteRuleEnum::CollapseProject(_) |
            RewriteRuleEnum::CollapseConsecutiveProject(_) |
            RewriteRuleEnum::MergeGetVerticesAndProject(_) |
            RewriteRuleEnum::MergeGetVerticesAndDedup(_) |
            RewriteRuleEnum::MergeGetNbrsAndProject(_) |
            RewriteRuleEnum::MergeGetNbrsAndDedup(_)
        ))
        .collect();

    assert_eq!(merge_rules.len(), 7, "应有 7 个合并规则");
}

#[test]
fn test_predicate_pushdown_rules_count() {
    let registry = RuleRegistry::default();

    let predicate_pushdown_rules: Vec<_> = registry.iter()
        .filter(|rule| matches!(rule,
            RewriteRuleEnum::PushFilterDownTraverse(_) |
            RewriteRuleEnum::PushFilterDownExpandAll(_) |
            RewriteRuleEnum::PushFilterDownNode(_) |
            RewriteRuleEnum::PushEFilterDown(_) |
            RewriteRuleEnum::PushVFilterDownScanVertices(_) |
            RewriteRuleEnum::PushFilterDownInnerJoin(_) |
            RewriteRuleEnum::PushFilterDownHashInnerJoin(_) |
            RewriteRuleEnum::PushFilterDownHashLeftJoin(_) |
            RewriteRuleEnum::PushFilterDownCrossJoin(_) |
            RewriteRuleEnum::PushFilterDownGetNbrs(_) |
            RewriteRuleEnum::PushFilterDownAllPaths(_)
        ))
        .collect();

    assert_eq!(predicate_pushdown_rules.len(), 11, "应有 11 个谓词下推规则");
}

#[test]
fn test_projection_pushdown_rules_count() {
    let registry = RuleRegistry::default();

    let projection_pushdown_rules: Vec<_> = registry.iter()
        .filter(|rule| matches!(rule,
            RewriteRuleEnum::ProjectionPushDown(_) |
            RewriteRuleEnum::PushProjectDown(_)
        ))
        .collect();

    assert_eq!(projection_pushdown_rules.len(), 2, "应有 2 个投影下推规则");
}

#[test]
fn test_limit_pushdown_rules_count() {
    let registry = RuleRegistry::default();

    let limit_pushdown_rules: Vec<_> = registry.iter()
        .filter(|rule| matches!(rule,
            RewriteRuleEnum::PushLimitDownGetVertices(_) |
            RewriteRuleEnum::PushLimitDownGetEdges(_) |
            RewriteRuleEnum::PushLimitDownScanVertices(_) |
            RewriteRuleEnum::PushLimitDownScanEdges(_) |
            RewriteRuleEnum::PushLimitDownIndexScan(_) |
            RewriteRuleEnum::PushTopNDownIndexScan(_)
        ))
        .collect();

    assert_eq!(limit_pushdown_rules.len(), 6, "应有 6 个 LIMIT 下推规则");
}

#[test]
fn test_aggregate_rules_count() {
    let registry = RuleRegistry::default();

    let aggregate_rules: Vec<_> = registry.iter()
        .filter(|rule| matches!(rule,
            RewriteRuleEnum::PushFilterDownAggregate(_)
        ))
        .collect();

    assert_eq!(aggregate_rules.len(), 1, "应有 1 个聚合优化规则");
}

// ==================== 规则唯一性测试 ====================

#[test]
fn test_rule_names_unique() {
    let registry = RuleRegistry::default();

    let mut names: Vec<_> = registry.iter()
        .map(|rule| rule.name())
        .collect();

    names.sort();
    names.dedup();

    assert_eq!(names.len(), 34, "所有规则名称应唯一");
}

// ==================== 宏生成代码验证测试 ====================

#[test]
fn test_macro_generated_enum() {
    // 验证宏生成的枚举包含所有预期的变体
    let _ = RewriteRuleEnum::EliminateFilter(EliminateFilterRule::new());
    let _ = RewriteRuleEnum::RemoveNoopProject(RemoveNoopProjectRule::new());
    let _ = RewriteRuleEnum::CombineFilter(CombineFilterRule::new());
    let _ = RewriteRuleEnum::PushFilterDownTraverse(PushFilterDownTraverseRule::new());
    let _ = RewriteRuleEnum::ProjectionPushDown(ProjectionPushDownRule::new());
    let _ = RewriteRuleEnum::PushLimitDownGetVertices(PushLimitDownGetVerticesRule::new());
    let _ = RewriteRuleEnum::PushFilterDownAggregate(PushFilterDownAggregateRule::new());
}

#[test]
fn test_macro_generated_methods() {
    let rule = RewriteRuleEnum::EliminateFilter(EliminateFilterRule::new());

    // 验证宏生成的所有方法都可以正常调用
    let name = rule.name();
    assert_eq!(name, "EliminateFilterRule");

    let pattern = rule.pattern();
    let _ = pattern;
}
