# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 4
- **Total Warnings**: 29
- **Total Issues**: 33
- **Unique Error Patterns**: 4
- **Unique Warning Patterns**: 22
- **Files with Issues**: 17

## Error Statistics

**Total Errors**: 4

### Error Type Breakdown

- **error[E0407]**: 2 errors
- **error[E0277]**: 1 errors
- **error[E0404]**: 1 errors

### Files with Errors (Top 10)

- `src\query\optimizer\rule_registry.rs`: 4 errors

## Warning Statistics

**Total Warnings**: 29

### Warning Type Breakdown

- **warning**: 29 warnings

### Files with Warnings (Top 10)

- `src\query\optimizer\rule_registrar.rs`: 10 warnings
- `src\query\planner\mod.rs`: 3 warnings
- `src\query\executor\factory.rs`: 2 warnings
- `src\query\executor\graph_query_executor.rs`: 2 warnings
- `src\query\planner\statements\seeks\seek_strategy.rs`: 1 warnings
- `src\query\context\managers\schema_traits.rs`: 1 warnings
- `src\common\memory.rs`: 1 warnings
- `src\query\optimizer\constant_folding.rs`: 1 warnings
- `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 warnings
- `src\query\executor\result_processing\projection.rs`: 1 warnings

## Detailed Error Categorization

### error[E0407]: method `name` is not a member of trait `BaseOptRule`: not a member of trait `BaseOptRule`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\optimizer\rule_registry.rs`: 2 occurrences

- Line 85: method `name` is not a member of trait `BaseOptRule`: not a member of trait `BaseOptRule`
- Line 89: method `transform` is not a member of trait `BaseOptRule`: not a member of trait `BaseOptRule`

### error[E0277]: the trait bound `TestRule: node::OptRule` is not satisfied: the trait `node::OptRule` is not implemented for `TestRule`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\optimizer\rule_registry.rs`: 1 occurrences

- Line 84: the trait bound `TestRule: node::OptRule` is not satisfied: the trait `node::OptRule` is not implemented for `TestRule`

### error[E0404]: expected trait, found struct `crate::query::optimizer::OptContext`: not a trait

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\optimizer\rule_registry.rs`: 1 occurrences

- Line 91: expected trait, found struct `crate::query::optimizer::OptContext`: not a trait

## Detailed Warning Categorization

### warning: use of deprecated type alias `query::planner::planner::ConfigurablePlannerRegistry`: 请使用 StaticConfigurablePlannerRegistry 替代

**Total Occurrences**: 29  
**Unique Files**: 16

#### `src\query\optimizer\rule_registrar.rs`: 10 occurrences

- Line 7: unexpected `cfg` condition value: `optimizer_registration`
- Line 14: unexpected `cfg` condition value: `optimizer_registration`
- Line 55: unexpected `cfg` condition value: `optimizer_registration`
- ... 7 more occurrences in this file

#### `src\query\planner\mod.rs`: 3 occurrences

- Line 16: use of deprecated type alias `query::planner::planner::ConfigurablePlannerRegistry`: 请使用 StaticConfigurablePlannerRegistry 替代
- Line 17: use of deprecated type alias `query::planner::planner::PlannerRegistry`: 请使用 StaticPlannerRegistry 替代
- Line 17: use of deprecated type alias `query::planner::planner::SequentialPlanner`: 请使用 StaticSequentialPlanner 替代

#### `src\query\executor\graph_query_executor.rs`: 2 occurrences

- Line 138: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`
- Line 152: variable does not need to be mutable

#### `src\query\executor\factory.rs`: 2 occurrences

- Line 49: unused imports: `EdgeAlterInfo`, `EdgeManageInfo`, `IndexManageInfo`, `SpaceManageInfo`, `TagAlterInfo`, and `TagManageInfo`
- Line 1009: unused import: `AlterEdgeOp`

#### `src\common\memory.rs`: 1 occurrences

- Line 188: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 323: unused import: `ExecutorStats`

#### `src\query\executor\admin\data\update.rs`: 1 occurrences

- Line 8: unused imports: `UpdateOp` and `UpdateTarget`

#### `src\query\context\managers\schema_traits.rs`: 1 occurrences

- Line 247: unexpected `cfg` condition value: `schema-manager-default`

#### `src\query\executor\data_processing\graph_traversal\impls.rs`: 1 occurrences

- Line 10: unused macro definition: `impl_graph_traversal_executor`

#### `src\query\planner\statements\seeks\seek_strategy.rs`: 1 occurrences

- Line 11: unused imports: `IndexInfo` and `NodePattern`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 8: unused import: `EliminationRule`

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 7: unused import: `Vertex`

#### `src\query\optimizer\constant_folding.rs`: 1 occurrences

- Line 62: unused import: `crate::core::Expression`

#### `src\query\optimizer\optimizer_config.rs`: 1 occurrences

- Line 4: unused import: `std::collections::HashMap`

#### `src\query\planner\statements\seeks\scan_seek.rs`: 1 occurrences

- Line 5: unused import: `SeekStrategyTraitObject`

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 348: unnecessary parentheses around function argument

