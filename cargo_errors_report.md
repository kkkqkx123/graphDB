# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 16
- **Total Warnings**: 72
- **Total Issues**: 88
- **Unique Error Patterns**: 5
- **Unique Warning Patterns**: 42
- **Files with Issues**: 44

## Error Statistics

**Total Errors**: 16

### Error Type Breakdown

- **error[E0599]**: 10 errors
- **error[E0034]**: 6 errors

### Files with Errors (Top 10)

- `src\core\result\iterator.rs`: 10 errors
- `src\expression\context\row_context.rs`: 6 errors

## Warning Statistics

**Total Warnings**: 72

### Warning Type Breakdown

- **warning**: 72 warnings

### Files with Warnings (Top 10)

- `src\query\optimizer\elimination_rules.rs`: 6 warnings
- `src\query\executor\result_processing\projection.rs`: 4 warnings
- `src\query\planner\statements\match_planner.rs`: 3 warnings
- `src\query\validator\insert_vertices_validator.rs`: 3 warnings
- `src\query\planner\statements\paths\shortest_path_planner.rs`: 3 warnings
- `src\query\planner\statements\paths\match_path_planner.rs`: 3 warnings
- `src\query\executor\search_executors.rs`: 2 warnings
- `src\query\planner\statements\match_statement_planner.rs`: 2 warnings
- `src\expression\context\basic_context.rs`: 2 warnings
- `src\query\parser\lexer\lexer.rs`: 2 warnings

## Detailed Error Categorization

### error[E0599]: no method named `next` found for struct `core::result::iterator::DefaultIterator` in the current scope: method not found in `DefaultIterator`

**Total Occurrences**: 10  
**Unique Files**: 1

#### `src\core\result\iterator.rs`: 10 occurrences

- Line 296: no method named `next` found for struct `core::result::iterator::DefaultIterator` in the current scope: method not found in `DefaultIterator`
- Line 300: no method named `next` found for struct `core::result::iterator::DefaultIterator` in the current scope: method not found in `DefaultIterator`
- Line 304: no method named `next` found for struct `core::result::iterator::DefaultIterator` in the current scope: method not found in `DefaultIterator`
- ... 7 more occurrences in this file

### error[E0034]: multiple applicable items in scope: multiple `get_variable` found

**Total Occurrences**: 6  
**Unique Files**: 1

#### `src\expression\context\row_context.rs`: 6 occurrences

- Line 369: multiple applicable items in scope: multiple `get_variable` found
- Line 370: multiple applicable items in scope: multiple `get_variable` found
- Line 371: multiple applicable items in scope: multiple `has_variable` found
- ... 3 more occurrences in this file

## Detailed Warning Categorization

### warning: unused variable: `meta2`: help: if this is intentional, prefix it with an underscore: `_meta2`

**Total Occurrences**: 72  
**Unique Files**: 44

#### `src\query\optimizer\elimination_rules.rs`: 6 occurrences

- Line 86: unused variable: `output_var`: help: if this is intentional, prefix it with an underscore: `_output_var`
- Line 169: unused variable: `output_var`: help: if this is intentional, prefix it with an underscore: `_output_var`
- Line 312: unused variable: `output_var`: help: if this is intentional, prefix it with an underscore: `_output_var`
- ... 3 more occurrences in this file

#### `src\query\executor\result_processing\projection.rs`: 4 occurrences

- Line 437: unused variable: `vertex1`: help: if this is intentional, prefix it with an underscore: `_vertex1`
- Line 450: unused variable: `vertex2`: help: if this is intentional, prefix it with an underscore: `_vertex2`
- Line 514: unused variable: `edge1`: help: if this is intentional, prefix it with an underscore: `_edge1`
- ... 1 more occurrences in this file

#### `src\query\planner\statements\match_planner.rs`: 3 occurrences

- Line 75: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 296: unreachable pattern: no value can reach this
- Line 470: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`

#### `src\query\planner\statements\paths\shortest_path_planner.rs`: 3 occurrences

- Line 23: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`
- Line 461: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`
- Line 467: unused variable: `pattern`: help: if this is intentional, prefix it with an underscore: `_pattern`

#### `src\query\validator\insert_vertices_validator.rs`: 3 occurrences

- Line 204: unused import: `crate::core::Value`
- Line 48: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 79: unused variable: `tag_name`: help: if this is intentional, prefix it with an underscore: `_tag_name`

#### `src\query\planner\statements\paths\match_path_planner.rs`: 3 occurrences

- Line 415: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`
- Line 421: unused variable: `pattern`: help: if this is intentional, prefix it with an underscore: `_pattern`
- Line 443: unused variable: `pattern`: help: if this is intentional, prefix it with an underscore: `_pattern`

#### `src\expression\context\default_context.rs`: 2 occurrences

- Line 524: function cannot return without recursing: cannot return without recursing
- Line 543: function cannot return without recursing: cannot return without recursing

#### `src\expression\context\query_expression_context.rs`: 2 occurrences

- Line 444: function cannot return without recursing: cannot return without recursing
- Line 463: function cannot return without recursing: cannot return without recursing

#### `src\query\executor\search_executors.rs`: 2 occurrences

- Line 13: unused import: `crate::expression::evaluator::traits::ExpressionContext`
- Line 358: value assigned to `vertices` is never read

#### `src\query\executor\graph_query_executor.rs`: 2 occurrences

- Line 138: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`
- Line 152: variable does not need to be mutable

#### `src\expression\context\basic_context.rs`: 2 occurrences

- Line 592: function cannot return without recursing: cannot return without recursing
- Line 611: function cannot return without recursing: cannot return without recursing

#### `src\query\parser\ast\utils.rs`: 2 occurrences

- Line 14: unused variable: `span`: help: if this is intentional, prefix it with an underscore: `_span`
- Line 55: unused variable: `match_expression`: help: if this is intentional, prefix it with an underscore: `_match_expression`

#### `src\query\planner\statements\match_statement_planner.rs`: 2 occurrences

- Line 86: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 353: unreachable pattern: no value can reach this

#### `src\query\validator\insert_edges_validator.rs`: 2 occurrences

- Line 53: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 84: unused variable: `edge_name`: help: if this is intentional, prefix it with an underscore: `_edge_name`

#### `src\query\validator\update_validator.rs`: 2 occurrences

- Line 34: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 168: unused variable: `op`: help: try ignoring the field: `op: _`

#### `src\expression\context\row_context.rs`: 2 occurrences

- Line 249: function cannot return without recursing: cannot return without recursing
- Line 268: function cannot return without recursing: cannot return without recursing

#### `src\query\parser\lexer\lexer.rs`: 2 occurrences

- Line 961: variable does not need to be mutable
- Line 1009: variable does not need to be mutable

#### `src\core\types\expression\visitor.rs`: 2 occurrences

- Line 149: unused variable: `property`: help: if this is intentional, prefix it with an underscore: `_property`
- Line 177: unused variable: `variable`: help: if this is intentional, prefix it with an underscore: `_variable`

#### `src\core\types\expression\expression.rs`: 1 occurrences

- Line 279: unused variable: `meta2`: help: if this is intentional, prefix it with an underscore: `_meta2`

#### `src\common\memory.rs`: 1 occurrences

- Line 188: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\expression\evaluator\traits.rs`: 1 occurrences

- Line 55: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\executor\operation_kind_support.rs`: 1 occurrences

- Line 100: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`

#### `src\query\optimizer\loop_unrolling.rs`: 1 occurrences

- Line 71: variable does not need to be mutable

#### `src\query\validator\delete_validator.rs`: 1 occurrences

- Line 32: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`

#### `src\expression\context\traits.rs`: 1 occurrences

- Line 5: unused import: `crate::core::error::ExpressionError`

#### `src\query\context\managers\schema_traits.rs`: 1 occurrences

- Line 247: unexpected `cfg` condition value: `schema-manager-default`

#### `src\query\planner\plan\execution_plan.rs`: 1 occurrences

- Line 68: unused variable: `n`: help: if this is intentional, prefix it with an underscore: `_n`

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 1 occurrences

- Line 36: unused variable: `ast_ctx`: help: if this is intentional, prefix it with an underscore: `_ast_ctx`

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 514: unused variable: `input_result`: help: if this is intentional, prefix it with an underscore: `_input_result`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 45: unused variable: `ast_ctx`: help: if this is intentional, prefix it with an underscore: `_ast_ctx`

#### `src\query\planner\statements\seeks\scan_seek.rs`: 1 occurrences

- Line 82: unused variable: `seek`: help: if this is intentional, prefix it with an underscore: `_seek`

#### `src\core\result\builder.rs`: 1 occurrences

- Line 2: unused import: `ResultMeta`

#### `src\query\visitor\ast_transformer.rs`: 1 occurrences

- Line 8: unused imports: `AlterStmt`, `Assignment`, `ChangePasswordStmt`, `CreateStmt`, `DeleteStmt`, `DescStmt`, `DropStmt`, `ExplainStmt`, `FetchStmt`, `FindPathStmt`, `GoStmt`, `InsertStmt`, `LookupStmt`, `MatchStmt`, `MergeStmt`, `PipeStmt`, `QueryStmt`, `RemoveStmt`, `ReturnStmt`, `SetStmt`, `ShowStmt`, `Stmt`, `SubgraphStmt`, `UnwindStmt`, `UpdateStmt`, `UseStmt`, and `WithStmt`

#### `src\query\planner\planner.rs`: 1 occurrences

- Line 191: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`

#### `src\core\result\iterator.rs`: 1 occurrences

- Line 2: unused import: `crate::core::DBResult`

#### `src\query\scheduler\execution_plan_analyzer.rs`: 1 occurrences

- Line 110: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode`

#### `src\query\optimizer\predicate_pushdown.rs`: 1 occurrences

- Line 180: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 50: unused import: `SpaceManageInfo`

#### `src\expression\evaluator\expression_evaluator.rs`: 1 occurrences

- Line 437: unreachable pattern: no value can reach this

#### `src\query\parser\parser\stmt_parser.rs`: 1 occurrences

- Line 305: unused variable: `tag_name`: help: if this is intentional, prefix it with an underscore: `_tag_name`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 365: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\query\parser\parser\expr_parser.rs`: 1 occurrences

- Line 450: unused variable: `test_expr`: help: if this is intentional, prefix it with an underscore: `_test_expr`

#### `src\query\optimizer\engine\optimizer.rs`: 1 occurrences

- Line 184: value assigned to `last_changes` is never read

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 348: unnecessary parentheses around function argument

