# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 129
- **Total Warnings**: 18
- **Total Issues**: 147
- **Unique Error Patterns**: 22
- **Unique Warning Patterns**: 16
- **Files with Issues**: 20

## Error Statistics

**Total Errors**: 129

### Error Type Breakdown

- **error[E0412]**: 72 errors
- **error[E0405]**: 25 errors
- **error[E0433]**: 18 errors
- **error[E0425]**: 5 errors
- **error[E0277]**: 3 errors
- **error[E0583]**: 2 errors
- **error[E0223]**: 1 errors
- **error[E0761]**: 1 errors
- **error**: 1 errors
- **error[E0432]**: 1 errors

### Files with Errors (Top 10)

- `src\query\optimizer\limit_pushdown.rs`: 105 errors
- `src\query\optimizer\elimination_rules.rs`: 5 errors
- `src\query\optimizer\join_optimization.rs`: 3 errors
- `src\core\mod.rs`: 3 errors
- `src\query\visitor\deduce_type_visitor.rs`: 3 errors
- `src\cache\global_manager.rs`: 3 errors
- `src\query\optimizer\scan_optimization.rs`: 2 errors
- `src\core\visitor.rs`: 2 errors
- `src\cache\parser_cache.rs`: 1 errors
- `src\query\visitor\extract_filter_expr_visitor.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 18

### Warning Type Breakdown

- **warning**: 18 warnings

### Files with Warnings (Top 10)

- `src\query\visitor\deduce_props_visitor.rs`: 2 warnings
- `src\core\mod.rs`: 2 warnings
- `src\query\visitor\extract_filter_expr_visitor.rs`: 2 warnings
- `src\query\planner\match_planning\utils\connection_strategy.rs`: 1 warnings
- `src\query\optimizer\join_optimization.rs`: 1 warnings
- `src\query\optimizer\elimination_rules.rs`: 1 warnings
- `src\query\optimizer\projection_pushdown.rs`: 1 warnings
- `src\query\parser\cypher\expression_converter.rs`: 1 warnings
- `src\query\visitor\find_visitor.rs`: 1 warnings
- `src\query\optimizer\scan_optimization.rs`: 1 warnings

## Detailed Error Categorization

### error[E0412]: cannot find type `OptContext` in this scope: not found in this scope

**Total Occurrences**: 72  
**Unique Files**: 1

#### `src\query\optimizer\limit_pushdown.rs`: 72 occurrences

- Line 17: cannot find type `OptContext` in this scope: not found in this scope
- Line 18: cannot find type `OptGroupNode` in this scope: not found in this scope
- Line 19: cannot find type `OptGroupNode` in this scope: not found in this scope
- ... 69 more occurrences in this file

### error[E0405]: cannot find trait `QueryVisitor` in this scope: not found in this scope

**Total Occurrences**: 25  
**Unique Files**: 2

#### `src\query\optimizer\limit_pushdown.rs`: 24 occurrences

- Line 10: cannot find trait `OptRule` in this scope: not found in this scope
- Line 46: cannot find trait `BaseOptRule` in this scope: not found in this scope
- Line 48: cannot find trait `PushDownRule` in this scope: not found in this scope
- ... 21 more occurrences in this file

#### `src\query\visitor\extract_filter_expr_visitor.rs`: 1 occurrences

- Line 124: cannot find trait `QueryVisitor` in this scope: not found in this scope

### error[E0433]: failed to resolve: use of undeclared type `PlanNodeEnum`: use of undeclared type `PlanNodeEnum`

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

### error[E0277]: `deduce_type_visitor::DeduceTypeVisitor<'a, S>` doesn't implement `std::fmt::Debug`: `deduce_type_visitor::DeduceTypeVisitor<'a, S>` cannot be formatted using `{:?}`

**Total Occurrences**: 3  
**Unique Files**: 2

#### `src\core\visitor.rs`: 2 occurrences

- Line 405: `T` cannot be shared between threads safely: `T` cannot be shared between threads safely
- Line 405: `T` cannot be sent between threads safely: `T` cannot be sent between threads safely

#### `src\query\visitor\deduce_type_visitor.rs`: 1 occurrences

- Line 758: `deduce_type_visitor::DeduceTypeVisitor<'a, S>` doesn't implement `std::fmt::Debug`: `deduce_type_visitor::DeduceTypeVisitor<'a, S>` cannot be formatted using `{:?}`

### error[E0583]: file not found for module `visitor_state_enum`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\core\mod.rs`: 2 occurrences

- Line 15: file not found for module `visitor_state_enum`
- Line 21: file not found for module `unified_visitor`

### error: expected one of `!`, `(`, `+`, `::`, `<`, `where`, or `{`, found `.`: expected one of 7 possible tokens

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\core\expression_visitor.rs`: 1 occurrences

- Line 311: expected one of `!`, `(`, `+`, `::`, `<`, `where`, or `{`, found `.`: expected one of 7 possible tokens

### error[E0432]: unresolved import `crate::query::visitor::QueryVisitor`: no `QueryVisitor` in `query::visitor`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\visitor\deduce_type_visitor.rs`: 1 occurrences

- Line 7: unresolved import `crate::query::visitor::QueryVisitor`: no `QueryVisitor` in `query::visitor`

### error[E0223]: ambiguous associated type

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\visitor\deduce_type_visitor.rs`: 1 occurrences

- Line 744: ambiguous associated type

### error[E0761]: file for module `context` found at both "src\core\context.rs" and "src\core\context\mod.rs"

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\core\mod.rs`: 1 occurrences

- Line 18: file for module `context` found at both "src\core\context.rs" and "src\core\context\mod.rs"

## Detailed Warning Categorization

### warning: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

**Total Occurrences**: 18  
**Unique Files**: 15

#### `src\core\mod.rs`: 2 occurrences

- Line 51: unused import: `visitor_state_enum::*`
- Line 60: unused import: `context::*`

#### `src\query\visitor\extract_filter_expr_visitor.rs`: 2 occurrences

- Line 5: unused imports: `AggregateFunction`, `BinaryOperator`, `DataType`, and `UnaryOperator`
- Line 7: unused import: `crate::core::Value`

#### `src\query\visitor\deduce_props_visitor.rs`: 2 occurrences

- Line 5: unused imports: `AggregateFunction`, `BinaryOperator`, `DataType`, and `UnaryOperator`
- Line 7: unused import: `crate::core::Value`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 100: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

#### `src\query\optimizer\projection_pushdown.rs`: 1 occurrences

- Line 119: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 564: unused import: `SortNode`

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\parser\cypher\expression_converter.rs`: 1 occurrences

- Line 269: unused imports: `FunctionCall`, `ListExpression`, `MapExpression`, `PropertyExpression`, and `UnaryExpression`

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 110: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 347: unused import: `UnaryOperator`

#### `src\query\planner\match_planning\utils\connection_builder.rs`: 1 occurrences

- Line 220: unused import: `crate::query::context::ast::base::AstContext`

#### `src\query\planner\match_planning\utils\connection_strategy.rs`: 1 occurrences

- Line 491: unused import: `std::sync::Arc`

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 466: unused import: `DedupNode as Dedup`

#### `src\query\visitor\find_visitor.rs`: 1 occurrences

- Line 4: unused import: `crate::core::types::expression::DataType`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 883: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

