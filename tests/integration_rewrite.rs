//! Plan to rewrite module integration tests
//!
//! Test Range.
//! - rewrite::rule_enum - static distribution rule enumeration
//! - rewrite::plan_rewriter - plan rewriter
//! - rewrite::RuleRegistry - rule registry
//! - Integration testing of various rewrite rules

mod common;

use graphdb::query::optimizer::heuristic::{
    create_default_rewriter, CombineFilterRule, EliminateFilterRule, PlanRewriter,
    PushFilterDownAggregateRule, PushFilterDownTraverseRule, PushLimitDownGetVerticesRule,
    RemoveNoopProjectRule, RewriteRuleEnum, RuleRegistry,
};

// ==================== RuleRegistry 集成测试 ====================

#[test]
fn test_rule_registry_default() {
    let registry = RuleRegistry::default();
    assert_eq!(
        registry.len(),
        39,
        "The default registry should contain 39 rules"
    );
    assert!(!registry.is_empty(), "The registry should not be empty");
}

#[test]
fn test_rule_registry_iter() {
    let registry = RuleRegistry::default();
    let mut count = 0;

    for rule in registry.iter() {
        count += 1;
        let name: &str = rule.name();
        assert!(!name.is_empty(), "The name of the rule should not be empty");
        assert!(name.ends_with("Rule"), "Rule names should end with 'Rule'");
    }

    assert_eq!(count, 39, "All 39 rules should be iterated");
}

#[test]
fn test_rule_registry_new() {
    let registry = RuleRegistry::new();
    assert_eq!(registry.len(), 0, "The new registry should be empty");
    assert!(registry.is_empty(), "The new registry should be empty");
}

#[test]
fn test_rule_registry_add() {
    let mut registry = RuleRegistry::new();

    registry.add(RewriteRuleEnum::EliminateFilter(EliminateFilterRule::new()));

    assert_eq!(
        registry.len(),
        1,
        "Adding a rule should result in a length of 1"
    );
    assert!(
        !registry.is_empty(),
        "Should not be empty after adding a rule"
    );
}

#[test]
fn test_rule_registry_clear() {
    let mut registry = RuleRegistry::default();
    assert_eq!(
        registry.len(),
        39,
        "The default registry should have 39 rules"
    );

    registry.clear();
    assert_eq!(registry.len(), 0, "The length should be 0 after clearing");
    assert!(registry.is_empty(), "Empty should be empty");
}

#[test]
fn test_rule_registry_into_vec() {
    let registry = RuleRegistry::default();
    let rules = registry.into_vec();

    assert_eq!(rules.len(), 39, "The converted Vec should contain 39 rules");
}

// ==================== RewriteRule Integration Testing ====================

#[test]
fn test_rewrite_rule_names() {
    let registry = RuleRegistry::default();

    let expected_names = vec![
        // Elimination rules
        "EliminateFilterRule",
        "RemoveNoopProjectRule",
        "EliminateAppendVerticesRule",
        "RemoveAppendVerticesBelowJoinRule",
        "EliminateRowCollectRule",
        "EliminateEmptySetOperationRule",
        "DedupEliminationRule",
        "EliminateSortRule",
        // Consolidation rules
        "CombineFilterRule",
        "CollapseProjectRule",
        "CollapseConsecutiveProjectRule",
        "MergeGetVerticesAndProjectRule",
        "MergeGetVerticesAndDedupRule",
        "MergeGetNbrsAndProjectRule",
        "MergeGetNbrsAndDedupRule",
        // predicate inference rule
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
        // Projected push-down rules
        "PushProjectDownScanVerticesRule",
        "PushProjectDownScanEdgesRule",
        "PushProjectDownGetVerticesRule",
        "PushProjectDownGetEdgesRule",
        "PushProjectDownGetNeighborsRule",
        "PushProjectDownEdgeIndexScanRule",
        // LIMIT push-down rule
        "PushLimitDownGetVerticesRule",
        "PushLimitDownGetEdgesRule",
        "PushLimitDownScanVerticesRule",
        "PushLimitDownScanEdgesRule",
        "PushLimitDownIndexScanRule",
        "PushTopNDownIndexScanRule",
        // Aggregation Optimization Rules
        "PushFilterDownAggregateRule",
    ];

    let mut actual_names: Vec<&str> = registry
        .iter()
        .map(|rule: &RewriteRuleEnum| rule.name())
        .collect();

    actual_names.sort();
    let mut expected_sorted = expected_names.clone();
    expected_sorted.sort();

    assert_eq!(
        actual_names, expected_sorted,
        "The list of rule names should match"
    );
}

#[test]
fn test_rewrite_rule_pattern() {
    let rule = RewriteRuleEnum::EliminateFilter(EliminateFilterRule::new());

    let pattern = rule.pattern();
    // Verify that the pattern method can be called correctly
    let _ = pattern;
}

#[test]
fn test_rewrite_rule_trait_methods() {
    let rule = RewriteRuleEnum::EliminateFilter(EliminateFilterRule::new());

    // Testing the RewriteRule trait method
    let name = rule.name();
    assert_eq!(name, "EliminateFilterRule");

    let pattern = rule.pattern();
    let _ = pattern;
}

// ==================== PlanRewriter 集成测试 ====================

#[test]
fn test_plan_rewriter_new() {
    let rewriter = PlanRewriter::new();
    // Verify that the rewriter can create
    let _ = rewriter;
}

#[test]
fn test_plan_rewriter_from_registry() {
    let registry = RuleRegistry::default();
    let rewriter = PlanRewriter::from_registry(registry);
    // Verify that the rewriter can be created from the registry
    let _ = rewriter;
}

#[test]
fn test_plan_rewriter_default() {
    let rewriter = PlanRewriter::default();
    // Validating the rewriter default implementation
    let _ = rewriter;
}

#[test]
fn test_create_default_rewriter() {
    let rewriter = create_default_rewriter();
    // Validating the Default Rewriter Creation Function
    let _ = rewriter;
}

// ==================== Static Distribution Performance Testing ====================

#[test]
fn test_static_dispatch_overhead() {
    let registry = RuleRegistry::default();
    let rules: Vec<_> = registry.iter().collect();

    // Testing the performance of static distribution
    // Verify that there are no significant performance issues through multiple calls
    let iterations = 1000;

    for _ in 0..iterations {
        for rule in &rules {
            let name: &str = rule.name();
            assert!(!name.is_empty());
        }
    }
}

// ==================== Rule Classification Test ====================

#[test]
fn test_elimination_rules_count() {
    let registry = RuleRegistry::default();

    let elimination_rules: Vec<_> = registry
        .iter()
        .filter(|rule| {
            matches!(
                rule,
                RewriteRuleEnum::EliminateFilter(_)
                    | RewriteRuleEnum::RemoveNoopProject(_)
                    | RewriteRuleEnum::EliminateAppendVertices(_)
                    | RewriteRuleEnum::RemoveAppendVerticesBelowJoin(_)
                    | RewriteRuleEnum::EliminateRowCollect(_)
                    | RewriteRuleEnum::EliminateEmptySetOperation(_)
                    | RewriteRuleEnum::DedupElimination(_)
            )
        })
        .collect();

    assert_eq!(
        elimination_rules.len(),
        7,
        "There should be 7 elimination rules"
    );
}

#[test]
fn test_merge_rules_count() {
    let registry = RuleRegistry::default();

    let merge_rules: Vec<_> = registry
        .iter()
        .filter(|rule| {
            matches!(
                rule,
                RewriteRuleEnum::CombineFilter(_)
                    | RewriteRuleEnum::CollapseProject(_)
                    | RewriteRuleEnum::CollapseConsecutiveProject(_)
                    | RewriteRuleEnum::MergeGetVerticesAndProject(_)
                    | RewriteRuleEnum::MergeGetVerticesAndDedup(_)
                    | RewriteRuleEnum::MergeGetNbrsAndProject(_)
                    | RewriteRuleEnum::MergeGetNbrsAndDedup(_)
            )
        })
        .collect();

    assert_eq!(merge_rules.len(), 7, "There should be 7 merger rules");
}

#[test]
fn test_predicate_pushdown_rules_count() {
    let registry = RuleRegistry::default();

    let predicate_pushdown_rules: Vec<_> = registry
        .iter()
        .filter(|rule| {
            matches!(
                rule,
                RewriteRuleEnum::PushFilterDownTraverse(_)
                    | RewriteRuleEnum::PushFilterDownExpandAll(_)
                    | RewriteRuleEnum::PushFilterDownNode(_)
                    | RewriteRuleEnum::PushEFilterDown(_)
                    | RewriteRuleEnum::PushVFilterDownScanVertices(_)
                    | RewriteRuleEnum::PushFilterDownInnerJoin(_)
                    | RewriteRuleEnum::PushFilterDownHashInnerJoin(_)
                    | RewriteRuleEnum::PushFilterDownHashLeftJoin(_)
                    | RewriteRuleEnum::PushFilterDownCrossJoin(_)
                    | RewriteRuleEnum::PushFilterDownGetNbrs(_)
                    | RewriteRuleEnum::PushFilterDownAllPaths(_)
            )
        })
        .collect();

    assert_eq!(
        predicate_pushdown_rules.len(),
        11,
        "There should be 11 predicate inference rules"
    );
}

#[test]
fn test_projection_pushdown_rules_count() {
    let registry = RuleRegistry::default();

    let projection_pushdown_rules: Vec<_> = registry
        .iter()
        .filter(|rule| {
            matches!(
                rule,
                RewriteRuleEnum::PushProjectDownScanVertices(_)
                    | RewriteRuleEnum::PushProjectDownScanEdges(_)
                    | RewriteRuleEnum::PushProjectDownGetVertices(_)
                    | RewriteRuleEnum::PushProjectDownGetEdges(_)
                    | RewriteRuleEnum::PushProjectDownGetNeighbors(_)
                    | RewriteRuleEnum::PushProjectDownEdgeIndexScan(_)
            )
        })
        .collect();

    assert_eq!(
        projection_pushdown_rules.len(),
        6,
        "There should be 6 projective extrapolation rules"
    );
}

#[test]
fn test_limit_pushdown_rules_count() {
    let registry = RuleRegistry::default();

    let limit_pushdown_rules: Vec<_> = registry
        .iter()
        .filter(|rule| {
            matches!(
                rule,
                RewriteRuleEnum::PushLimitDownGetVertices(_)
                    | RewriteRuleEnum::PushLimitDownGetEdges(_)
                    | RewriteRuleEnum::PushLimitDownScanVertices(_)
                    | RewriteRuleEnum::PushLimitDownScanEdges(_)
                    | RewriteRuleEnum::PushLimitDownIndexScan(_)
                    | RewriteRuleEnum::PushTopNDownIndexScan(_)
            )
        })
        .collect();

    assert_eq!(
        limit_pushdown_rules.len(),
        6,
        "There should be 6 LIMIT pushdown rules"
    );
}

#[test]
fn test_aggregate_rules_count() {
    let registry = RuleRegistry::default();

    let aggregate_rules: Vec<_> = registry
        .iter()
        .filter(|rule| matches!(rule, RewriteRuleEnum::PushFilterDownAggregate(_)))
        .collect();

    assert_eq!(
        aggregate_rules.len(),
        1,
        "There should be 1 aggregation optimization rule"
    );
}

// ==================== Rule Uniqueness Test ====================

#[test]
fn test_rule_names_unique() {
    let registry = RuleRegistry::default();

    let mut names: Vec<&str> = registry
        .iter()
        .map(|rule: &RewriteRuleEnum| rule.name())
        .collect();

    names.sort();
    names.dedup();

    assert_eq!(names.len(), 39, "All rule names should be unique");
}

// ==================== Macro Generation Code Verification Test ====================

#[test]
fn test_macro_generated_enum() {
    // Verify that the enumeration generated by the macro contains all the expected variants
    let _ = RewriteRuleEnum::EliminateFilter(EliminateFilterRule::new());
    let _ = RewriteRuleEnum::RemoveNoopProject(RemoveNoopProjectRule::new());
    let _ = RewriteRuleEnum::CombineFilter(CombineFilterRule::new());
    let _ = RewriteRuleEnum::PushFilterDownTraverse(PushFilterDownTraverseRule::new());
    let _ = RewriteRuleEnum::PushLimitDownGetVertices(PushLimitDownGetVerticesRule::new());
    let _ = RewriteRuleEnum::PushFilterDownAggregate(PushFilterDownAggregateRule::new());
}

#[test]
fn test_macro_generated_methods() {
    let rule = RewriteRuleEnum::EliminateFilter(EliminateFilterRule::new());

    // Verify that all methods generated by the macro can be called properly
    let name = rule.name();
    assert_eq!(name, "EliminateFilterRule");

    let pattern = rule.pattern();
    let _ = pattern;
}
