# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 119
- **Total Warnings**: 14
- **Total Issues**: 133
- **Unique Error Patterns**: 12
- **Unique Warning Patterns**: 14
- **Files with Issues**: 16

## Error Statistics

**Total Errors**: 119

### Error Type Breakdown

- **error[E0412]**: 72 errors
- **error[E0405]**: 24 errors
- **error[E0433]**: 18 errors
- **error[E0425]**: 5 errors

### Files with Errors (Top 10)

- `src\query\optimizer\limit_pushdown.rs`: 105 errors
- `src\query\optimizer\elimination_rules.rs`: 5 errors
- `src\cache\global_manager.rs`: 3 errors
- `src\query\optimizer\join_optimization.rs`: 3 errors
- `src\query\optimizer\scan_optimization.rs`: 2 errors
- `src\cache\parser_cache.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 14

### Warning Type Breakdown

- **warning**: 14 warnings

### Files with Warnings (Top 10)

- `src\query\parser\cypher\expression_converter.rs`: 1 warnings
- `src\query\optimizer\elimination_rules.rs`: 1 warnings
- `src\query\planner\match_planning\utils\connection_builder.rs`: 1 warnings
- `src\query\planner\match_planning\utils\connection_strategy.rs`: 1 warnings
- `src\query\optimizer\join_optimization.rs`: 1 warnings
- `src\query\validator\strategies\aggregate_strategy.rs`: 1 warnings
- `src\query\executor\data_processing\transformations\append_vertices.rs`: 1 warnings
- `src\query\visitor\fold_constant_expr_visitor.rs`: 1 warnings
- `src\core\context\mod.rs`: 1 warnings
- `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 warnings

## Detailed Error Categorization

### error[E0412]: cannot find type `OptContext` in this scope: not found in this scope

**Total Occurrences**: 72  
**Unique Files**: 1

#### `src\query\optimizer\limit_pushdown.rs`: 72 occurrences

- Line 17: cannot find type `OptContext` in this scope: not found in this scope
- Line 18: cannot find type `OptGroupNode` in this scope: not found in this scope
- Line 19: cannot find type `OptGroupNode` in this scope: not found in this scope
- ... 69 more occurrences in this file

### error[E0405]: cannot find trait `OptRule` in this scope: not found in this scope

**Total Occurrences**: 24  
**Unique Files**: 1

#### `src\query\optimizer\limit_pushdown.rs`: 24 occurrences

- Line 10: cannot find trait `OptRule` in this scope: not found in this scope
- Line 46: cannot find trait `BaseOptRule` in this scope: not found in this scope
- Line 48: cannot find trait `PushDownRule` in this scope: not found in this scope
- ... 21 more occurrences in this file

### error[E0433]: failed to resolve: use of undeclared type `CacheStrategy`: use of undeclared type `CacheStrategy`

**Total Occurrences**: 18  
**Unique Files**: 5

#### `src\query\optimizer\limit_pushdown.rs`: 9 occurrences

- Line 42: failed to resolve: use of undeclared type `PatternBuilder`: use of undeclared type `PatternBuilder`
- Line 224: failed to resolve: use of undeclared type `PatternBuilder`: use of undeclared type `PatternBuilder`
- Line 311: failed to resolve: use of undeclared type `PatternBuilder`: use of undeclared type `PatternBuilder`
- ... 6 more occurrences in this file

#### `src\cache\global_manager.rs`: 3 occurrences

- Line 174: failed to resolve: use of undeclared type `CacheStrategy`: use of undeclared type `CacheStrategy`
- Line 222: failed to resolve: use of undeclared type `CacheStrategy`: use of undeclared type `CacheStrategy`
- Line 225: failed to resolve: use of undeclared type `CacheStrategy`: use of undeclared type `CacheStrategy`

#### `src\query\optimizer\join_optimization.rs`: 3 occurrences

- Line 134: failed to resolve: use of undeclared type `PlanNodeEnum`: use of undeclared type `PlanNodeEnum`
- Line 136: failed to resolve: use of undeclared type `PlanNodeEnum`: use of undeclared type `PlanNodeEnum`
- Line 145: failed to resolve: use of undeclared type `PlanNodeEnum`: use of undeclared type `PlanNodeEnum`

#### `src\query\optimizer\scan_optimization.rs`: 2 occurrences

- Line 123: failed to resolve: use of undeclared type `PlanNodeEnum`: use of undeclared type `PlanNodeEnum`
- Line 141: failed to resolve: use of undeclared type `PlanNodeEnum`: use of undeclared type `PlanNodeEnum`

#### `src\cache\parser_cache.rs`: 1 occurrences

- Line 516: failed to resolve: use of undeclared type `Duration`: use of undeclared type `Duration`

### error[E0425]: cannot find function `is_tautology` in this scope: not found in this scope

**Total Occurrences**: 5  
**Unique Files**: 1

#### `src\query\optimizer\elimination_rules.rs`: 5 occurrences

- Line 870: cannot find function `is_tautology` in this scope: not found in this scope
- Line 871: cannot find function `is_tautology` in this scope: not found in this scope
- Line 872: cannot find function `is_tautology` in this scope: not found in this scope
- ... 2 more occurrences in this file

## Detailed Warning Categorization

### warning: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

**Total Occurrences**: 14  
**Unique Files**: 14

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 110: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 883: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\planner\match_planning\utils\connection_builder.rs`: 1 occurrences

- Line 220: unused import: `crate::query::context::ast::base::AstContext`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 564: unused import: `SortNode`

#### `src\query\optimizer\projection_pushdown.rs`: 1 occurrences

- Line 119: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 100: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

#### `src\query\parser\cypher\expression_converter.rs`: 1 occurrences

- Line 269: unused imports: `FunctionCall`, `ListExpression`, `MapExpression`, `PropertyExpression`, and `UnaryExpression`

#### `src\query\planner\match_planning\utils\connection_strategy.rs`: 1 occurrences

- Line 491: unused import: `std::sync::Arc`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 347: unused import: `UnaryOperator`

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 466: unused import: `DedupNode as Dedup`

#### `src\query\visitor\fold_constant_expr_visitor.rs`: 1 occurrences

- Line 7: unused import: `std::collections::HashMap`

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 319: unused variable: `expr_context`: help: if this is intentional, prefix it with an underscore: `_expr_context`

#### `src\core\context\mod.rs`: 1 occurrences

- Line 22: ambiguous glob re-exports: the name `SessionInfo` in the type namespace is first re-exported here

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

