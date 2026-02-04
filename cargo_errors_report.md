# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 11
- **Total Warnings**: 73
- **Total Issues**: 84
- **Unique Error Patterns**: 11
- **Unique Warning Patterns**: 42
- **Files with Issues**: 30

## Error Statistics

**Total Errors**: 11

### Error Type Breakdown

- **error[E0425]**: 10 errors
- **error[E0596]**: 1 errors

### Files with Errors (Top 10)

- `src\query\optimizer\rule_enum.rs`: 5 errors
- `src\query\optimizer\rule_registrar.rs`: 5 errors
- `src\query\optimizer\engine\optimizer.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 73

### Warning Type Breakdown

- **warning**: 73 warnings

### Files with Warnings (Top 10)

- `src\storage\redb_storage.rs`: 18 warnings
- `src\query\context\symbol\symbol_table.rs`: 7 warnings
- `src\query\optimizer\predicate_pushdown.rs`: 6 warnings
- `src\query\executor\result_processing\projection.rs`: 5 warnings
- `src\api\service\graph_service.rs`: 4 warnings
- `src\query\optimizer\engine\optimizer.rs`: 3 warnings
- `src\core\vertex_edge_path.rs`: 3 warnings
- `src\query\optimizer\elimination_rules.rs`: 3 warnings
- `src\core\types\expression\visitor.rs`: 2 warnings
- `src\query\parser\lexer\lexer.rs`: 2 warnings

## Detailed Error Categorization

### error[E0425]: cannot find value `FilterPushDownRule` in module `crate::query::optimizer`: not found in `crate::query::optimizer`

**Total Occurrences**: 10  
**Unique Files**: 2

#### `src\query\optimizer\rule_registrar.rs`: 5 occurrences

- Line 14: cannot find value `FilterPushDownRule` in module `crate::query::optimizer`: not found in `crate::query::optimizer`
- Line 15: cannot find value `PredicatePushDownRule` in module `crate::query::optimizer`: help: a unit struct with a similar name exists: `ProjectionPushDownRule`
- Line 35: cannot find value `PushLimitDownRule` in module `crate::query::optimizer`: not found in `crate::query::optimizer`
- ... 2 more occurrences in this file

#### `src\query\optimizer\rule_enum.rs`: 5 occurrences

- Line 129: cannot find value `FilterPushDownRule` in module `super`: not found in `super`
- Line 130: cannot find value `PredicatePushDownRule` in module `super`: help: a unit struct with a similar name exists: `ProjectionPushDownRule`
- Line 153: cannot find value `PushLimitDownRule` in module `super`: not found in `super`
- ... 2 more occurrences in this file

### error[E0596]: cannot borrow `group.explored_rules` as mutable, as `group` is not declared as mutable: cannot borrow as mutable

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\optimizer\engine\optimizer.rs`: 1 occurrences

- Line 680: cannot borrow `group.explored_rules` as mutable, as `group` is not declared as mutable: cannot borrow as mutable

## Detailed Warning Categorization

### warning: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

**Total Occurrences**: 73  
**Unique Files**: 28

#### `src\storage\redb_storage.rs`: 18 occurrences

- Line 5: unused import: `UpdateTarget`
- Line 128: unused variable: `space`: help: if this is intentional, prefix it with an underscore: `_space`
- Line 128: unused variable: `vertex_id`: help: if this is intentional, prefix it with an underscore: `_vertex_id`
- ... 15 more occurrences in this file

#### `src\query\context\symbol\symbol_table.rs`: 7 occurrences

- Line 161: unused variable: `symbol`: help: if this is intentional, prefix it with an underscore: `_symbol`
- Line 173: unused variable: `symbol`: help: if this is intentional, prefix it with an underscore: `_symbol`
- Line 196: variable does not need to be mutable
- ... 4 more occurrences in this file

#### `src\query\optimizer\predicate_pushdown.rs`: 6 occurrences

- Line 5: unused import: `CommonPatterns`
- Line 7: unused import: `crate::query::planner::plan::PlanNodeEnum`
- Line 8: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode`
- ... 3 more occurrences in this file

#### `src\query\executor\result_processing\projection.rs`: 5 occurrences

- Line 321: unused imports: `ExecutionResult` and `Executor`
- Line 334: variable does not need to be mutable
- Line 370: variable does not need to be mutable
- ... 2 more occurrences in this file

#### `src\api\service\graph_service.rs`: 4 occurrences

- Line 8: unused import: `crate::utils::safe_lock`
- Line 336: variable does not need to be mutable
- Line 375: variable does not need to be mutable
- ... 1 more occurrences in this file

#### `src\core\vertex_edge_path.rs`: 3 occurrences

- Line 268: unused variable: `v`: help: if this is intentional, prefix it with an underscore: `_v`
- Line 272: unused variable: `v`: help: if this is intentional, prefix it with an underscore: `_v`
- Line 378: unused variable: `v`: help: if this is intentional, prefix it with an underscore: `_v`

#### `src\query\optimizer\elimination_rules.rs`: 3 occurrences

- Line 90: variable does not need to be mutable
- Line 429: variable does not need to be mutable
- Line 624: variable does not need to be mutable

#### `src\query\optimizer\engine\optimizer.rs`: 3 occurrences

- Line 566: value assigned to `last_changes` is never read
- Line 669: unused variable: `node_id`: help: if this is intentional, prefix it with an underscore: `_node_id`
- Line 647: unused variable: `root_group`: help: if this is intentional, prefix it with an underscore: `_root_group`

#### `src\query\optimizer\plan_validator.rs`: 2 occurrences

- Line 87: unused variable: `node_ref`: help: if this is intentional, prefix it with an underscore: `_node_ref`
- Line 88: unused variable: `boundary`: help: if this is intentional, prefix it with an underscore: `_boundary`

#### `src\core\types\expression\visitor.rs`: 2 occurrences

- Line 150: unused variable: `property`: help: if this is intentional, prefix it with an underscore: `_property`
- Line 178: unused variable: `variable`: help: if this is intentional, prefix it with an underscore: `_variable`

#### `src\query\executor\data_access.rs`: 2 occurrences

- Line 152: unused variable: `ids`: help: if this is intentional, prefix it with an underscore: `_ids`
- Line 531: unused variable: `idx`: help: if this is intentional, prefix it with an underscore: `_idx`

#### `src\query\parser\lexer\lexer.rs`: 2 occurrences

- Line 961: variable does not need to be mutable
- Line 1009: variable does not need to be mutable

#### `src\expression\evaluator\traits.rs`: 1 occurrences

- Line 19: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\core\types\expression\expression.rs`: 1 occurrences

- Line 279: unused variable: `meta2`: help: if this is intentional, prefix it with an underscore: `_meta2`

#### `src\query\validator\insert_vertices_validator.rs`: 1 occurrences

- Line 204: unused import: `crate::core::Value`

#### `src\query\executor\operation_kind_support.rs`: 1 occurrences

- Line 101: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 21: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\query\optimizer\index_optimization.rs`: 1 occurrences

- Line 25: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\query\context\ast\query_types\go.rs`: 1 occurrences

- Line 92: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\common\memory.rs`: 1 occurrences

- Line 222: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\planner\statements\seeks\scan_seek.rs`: 1 occurrences

- Line 82: unused variable: `seek`: help: if this is intentional, prefix it with an underscore: `_seek`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 8: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\query\executor\graph_query_executor.rs`: 1 occurrences

- Line 138: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 365: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\query\planner\statements\match_planner.rs`: 1 occurrences

- Line 567: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`

#### `src\query\context\ast\query_types\fetch_vertices.rs`: 1 occurrences

- Line 47: unused variable: `ids`: help: try ignoring the field: `ids: _`

#### `src\query\parser\ast\utils.rs`: 1 occurrences

- Line 14: unused variable: `span`: help: if this is intentional, prefix it with an underscore: `_span`

#### `src\storage\processor\base.rs`: 1 occurrences

- Line 531: unused variable: `counters`: help: if this is intentional, prefix it with an underscore: `_counters`

