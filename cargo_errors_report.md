# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 8
- **Total Warnings**: 114
- **Total Issues**: 122
- **Unique Error Patterns**: 6
- **Unique Warning Patterns**: 46
- **Files with Issues**: 48

## Error Statistics

**Total Errors**: 8

### Error Type Breakdown

- **error[E0412]**: 3 errors
- **error[E0433]**: 3 errors
- **error[E0061]**: 1 errors
- **error[E0422]**: 1 errors

### Files with Errors (Top 10)

- `src\query\visitor\deduce_type_visitor.rs`: 4 errors
- `src\storage\transaction\log.rs`: 1 errors
- `src\query\planner\statements\seeks\index_seek.rs`: 1 errors
- `src\query\optimizer\rule_registry.rs`: 1 errors
- `src\storage\transaction\traits.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 114

### Warning Type Breakdown

- **warning**: 114 warnings

### Files with Warnings (Top 10)

- `src\query\optimizer\operation_merge.rs`: 16 warnings
- `src\query\optimizer\limit_pushdown.rs`: 12 warnings
- `src\query\context\symbol\symbol_table.rs`: 10 warnings
- `src\query\executor\result_processing\projection.rs`: 8 warnings
- `src\query\optimizer\predicate_pushdown.rs`: 7 warnings
- `src\query\optimizer\projection_pushdown.rs`: 5 warnings
- `src\query\optimizer\engine\optimizer.rs`: 4 warnings
- `src\storage\iterator\composite.rs`: 3 warnings
- `src\query\optimizer\elimination_rules.rs`: 3 warnings
- `src\query\executor\result_processing\dedup.rs`: 2 warnings

## Detailed Error Categorization

### error[E0412]: cannot find type `TransactionId` in this scope: not found in this scope

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\query\visitor\deduce_type_visitor.rs`: 3 occurrences

- Line 669: cannot find type `TransactionId` in this scope: not found in this scope
- Line 673: cannot find type `TransactionId` in this scope: not found in this scope
- Line 677: cannot find type `TransactionId` in this scope: not found in this scope

### error[E0433]: failed to resolve: use of undeclared type `TransactionId`: use of undeclared type `TransactionId`

**Total Occurrences**: 3  
**Unique Files**: 3

#### `src\query\visitor\deduce_type_visitor.rs`: 1 occurrences

- Line 670: failed to resolve: use of undeclared type `TransactionId`: use of undeclared type `TransactionId`

#### `src\storage\transaction\traits.rs`: 1 occurrences

- Line 366: failed to resolve: could not find `IsolationLevel` in `super`: could not find `IsolationLevel` in `super`

#### `src\storage\transaction\log.rs`: 1 occurrences

- Line 610: failed to resolve: use of undeclared type `TransactionState`: use of undeclared type `TransactionState`

### error[E0061]: this function takes 0 arguments but 1 argument was supplied

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\optimizer\rule_registry.rs`: 1 occurrences

- Line 128: this function takes 0 arguments but 1 argument was supplied

### error[E0422]: cannot find struct, variant or union type `IndexInfo` in this scope: not found in this scope

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\planner\statements\seeks\index_seek.rs`: 1 occurrences

- Line 110: cannot find struct, variant or union type `IndexInfo` in this scope: not found in this scope

## Detailed Warning Categorization

### warning: unused variable: `idx`: help: if this is intentional, prefix it with an underscore: `_idx`

**Total Occurrences**: 114  
**Unique Files**: 44

#### `src\query\optimizer\operation_merge.rs`: 16 occurrences

- Line 131: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 231: unused variable: `node_ref`: help: if this is intentional, prefix it with an underscore: `_node_ref`
- Line 227: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- ... 13 more occurrences in this file

#### `src\query\optimizer\limit_pushdown.rs`: 12 occurrences

- Line 46: unused variable: `input_id`: help: if this is intentional, prefix it with an underscore: `_input_id`
- Line 199: unused variable: `node_ref`: help: if this is intentional, prefix it with an underscore: `_node_ref`
- Line 195: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- ... 9 more occurrences in this file

#### `src\query\context\symbol\symbol_table.rs`: 10 occurrences

- Line 161: unused variable: `symbol`: help: if this is intentional, prefix it with an underscore: `_symbol`
- Line 163: variable does not need to be mutable
- Line 173: unused variable: `symbol`: help: if this is intentional, prefix it with an underscore: `_symbol`
- ... 7 more occurrences in this file

#### `src\query\executor\result_processing\projection.rs`: 8 occurrences

- Line 319: unused import: `DataSet`
- Line 321: unused import: `crate::query::executor::executor_enum::ExecutorEnum`
- Line 322: unused import: `crate::query::executor::base::BaseExecutor`
- ... 5 more occurrences in this file

#### `src\query\optimizer\predicate_pushdown.rs`: 7 occurrences

- Line 198: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 727: unused variable: `node_ref`: help: if this is intentional, prefix it with an underscore: `_node_ref`
- Line 723: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- ... 4 more occurrences in this file

#### `src\query\optimizer\projection_pushdown.rs`: 5 occurrences

- Line 129: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 131: unused variable: `child`: help: if this is intentional, prefix it with an underscore: `_child`
- Line 200: unused variable: `node_ref`: help: if this is intentional, prefix it with an underscore: `_node_ref`
- ... 2 more occurrences in this file

#### `src\query\optimizer\engine\optimizer.rs`: 4 occurrences

- Line 416: unreachable pattern: no value can reach this
- Line 470: value assigned to `last_changes` is never read
- Line 576: unused variable: `node_id`: help: if this is intentional, prefix it with an underscore: `_node_id`
- ... 1 more occurrences in this file

#### `src\storage\iterator\composite.rs`: 3 occurrences

- Line 120: unused variable: `idx`: help: if this is intentional, prefix it with an underscore: `_idx`
- Line 141: unused variable: `row`: help: if this is intentional, prefix it with an underscore: `_row`
- Line 705: variable does not need to be mutable

#### `src\query\optimizer\elimination_rules.rs`: 3 occurrences

- Line 93: variable does not need to be mutable
- Line 436: variable does not need to be mutable
- Line 647: variable does not need to be mutable

#### `src\storage\transaction\log.rs`: 2 occurrences

- Line 460: unused variable: `flushed`: help: if this is intentional, prefix it with an underscore: `_flushed`
- Line 458: unused variable: `min_lsn`: help: if this is intentional, prefix it with an underscore: `_min_lsn`

#### `src\core\types\expression\visitor.rs`: 2 occurrences

- Line 149: unused variable: `property`: help: if this is intentional, prefix it with an underscore: `_property`
- Line 177: unused variable: `variable`: help: if this is intentional, prefix it with an underscore: `_variable`

#### `src\expression\context\row_context.rs`: 2 occurrences

- Line 249: function cannot return without recursing: cannot return without recursing
- Line 268: function cannot return without recursing: cannot return without recursing

#### `src\query\parser\lexer\lexer.rs`: 2 occurrences

- Line 961: variable does not need to be mutable
- Line 1009: variable does not need to be mutable

#### `src\expression\context\query_expression_context.rs`: 2 occurrences

- Line 444: function cannot return without recursing: cannot return without recursing
- Line 463: function cannot return without recursing: cannot return without recursing

#### `src\expression\context\default_context.rs`: 2 occurrences

- Line 524: function cannot return without recursing: cannot return without recursing
- Line 543: function cannot return without recursing: cannot return without recursing

#### `src\query\executor\result_processing\dedup.rs`: 2 occurrences

- Line 494: unused import: `crate::query::executor::base::BaseExecutor`
- Line 495: unused import: `crate::query::executor::executor_enum::ExecutorEnum`

#### `src\expression\context\basic_context.rs`: 2 occurrences

- Line 592: function cannot return without recursing: cannot return without recursing
- Line 611: function cannot return without recursing: cannot return without recursing

#### `src\query\optimizer\optimizer_config.rs`: 2 occurrences

- Line 134: unused import: `std::io::Write`
- Line 135: unused import: `tempfile::NamedTempFile`

#### `src\query\planner\statements\match_planner.rs`: 2 occurrences

- Line 303: unreachable pattern: no value can reach this
- Line 568: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`

#### `src\query\planner\plan\core\nodes\join_node.rs`: 2 occurrences

- Line 1056: unused variable: `l`: help: if this is intentional, prefix it with an underscore: `_l`
- Line 1057: unused variable: `r`: help: if this is intentional, prefix it with an underscore: `_r`

#### `src\query\planner\statements\seeks\scan_seek.rs`: 1 occurrences

- Line 82: unused variable: `seek`: help: if this is intentional, prefix it with an underscore: `_seek`

#### `src\query\context\ast\query_types\fetch_vertices.rs`: 1 occurrences

- Line 47: unused variable: `ids`: help: try ignoring the field: `ids: _`

#### `src\storage\iterator\predicate.rs`: 1 occurrences

- Line 486: unused variable: `pred2`: help: if this is intentional, prefix it with an underscore: `_pred2`

#### `src\query\context\ast\query_types\go.rs`: 1 occurrences

- Line 92: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\planner\planner.rs`: 1 occurrences

- Line 210: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 1 occurrences

- Line 36: unused variable: `ast_ctx`: help: if this is intentional, prefix it with an underscore: `_ast_ctx`

#### `src\query\planner\statements\match_statement_planner.rs`: 1 occurrences

- Line 353: unreachable pattern: no value can reach this

#### `src\query\validator\insert_vertices_validator.rs`: 1 occurrences

- Line 204: unused import: `crate::core::Value`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 365: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\query\executor\operation_kind_support.rs`: 1 occurrences

- Line 101: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`

#### `src\core\types\expression\expression.rs`: 1 occurrences

- Line 279: unused variable: `meta2`: help: if this is intentional, prefix it with an underscore: `_meta2`

#### `src\expression\evaluator\traits.rs`: 1 occurrences

- Line 55: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\parser\ast\utils.rs`: 1 occurrences

- Line 14: unused variable: `span`: help: if this is intentional, prefix it with an underscore: `_span`

#### `src\storage\transaction\snapshot.rs`: 1 occurrences

- Line 290: unused variable: `key_lock`: help: if this is intentional, prefix it with an underscore: `_key_lock`

#### `src\common\memory.rs`: 1 occurrences

- Line 188: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\parser\parser\expr_parser.rs`: 1 occurrences

- Line 450: unused variable: `test_expr`: help: if this is intentional, prefix it with an underscore: `_test_expr`

#### `src\query\executor\graph_query_executor.rs`: 1 occurrences

- Line 138: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 348: unnecessary parentheses around function argument

#### `src\query\optimizer\index_optimization.rs`: 1 occurrences

- Line 25: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\expression\evaluator\expression_evaluator.rs`: 1 occurrences

- Line 437: unreachable pattern: no value can reach this

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 45: unused variable: `ast_ctx`: help: if this is intentional, prefix it with an underscore: `_ast_ctx`

#### `src\query\planner\plan\execution_plan.rs`: 1 occurrences

- Line 68: unused variable: `n`: help: if this is intentional, prefix it with an underscore: `_n`

#### `src\query\executor\search_executors.rs`: 1 occurrences

- Line 357: value assigned to `vertices` is never read

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 465: unused variable: `target_type`: help: if this is intentional, prefix it with an underscore: `_target_type`

