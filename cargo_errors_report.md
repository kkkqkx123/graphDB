# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 43
- **Total Warnings**: 145
- **Total Issues**: 188
- **Unique Error Patterns**: 19
- **Unique Warning Patterns**: 66
- **Files with Issues**: 65

## Error Statistics

**Total Errors**: 43

### Error Type Breakdown

- **error[E0506]**: 12 errors
- **error[E0061]**: 10 errors
- **error[E0502]**: 7 errors
- **error[E0599]**: 6 errors
- **error[E0308]**: 6 errors
- **error[E0658]**: 1 errors
- **error[E0505]**: 1 errors

### Files with Errors (Top 10)

- `src\query\optimizer\limit_pushdown.rs`: 12 errors
- `src\query\optimizer\engine\optimizer.rs`: 10 errors
- `src\query\optimizer\index_optimization.rs`: 4 errors
- `src\query\optimizer\predicate_pushdown.rs`: 3 errors
- `src\query\optimizer\elimination_rules.rs`: 3 errors
- `src\query\optimizer\plan\node.rs`: 2 errors
- `src\query\optimizer\operation_merge.rs`: 2 errors
- `src\query\optimizer\rule_registry.rs`: 1 errors
- `src\query\optimizer\transformation_rules.rs`: 1 errors
- `src\query\optimizer\subquery_optimization.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 145

### Warning Type Breakdown

- **warning**: 145 warnings

### Files with Warnings (Top 10)

- `src\query\optimizer\predicate_pushdown.rs`: 22 warnings
- `src\query\optimizer\operation_merge.rs`: 16 warnings
- `src\query\optimizer\limit_pushdown.rs`: 12 warnings
- `src\query\context\symbol\symbol_table.rs`: 10 warnings
- `src\query\executor\result_processing\projection.rs`: 8 warnings
- `src\query\optimizer\projection_pushdown.rs`: 6 warnings
- `src\query\optimizer\engine\optimizer.rs`: 4 warnings
- `src\query\planner\statements\match_planner.rs`: 3 warnings
- `src\storage\transaction\log.rs`: 3 warnings
- `src\storage\iterator\composite.rs`: 3 warnings

## Detailed Error Categorization

### error[E0506]: cannot assign to `self.merged` because it is borrowed: `self.merged` is assigned to here but it was already borrowed

**Total Occurrences**: 12  
**Unique Files**: 5

#### `src\query\optimizer\limit_pushdown.rs`: 5 occurrences

- Line 67: cannot assign to `self.pushed_down` because it is borrowed: `self.pushed_down` is assigned to here but it was already borrowed
- Line 81: cannot assign to `self.pushed_down` because it is borrowed: `self.pushed_down` is assigned to here but it was already borrowed
- Line 95: cannot assign to `self.pushed_down` because it is borrowed: `self.pushed_down` is assigned to here but it was already borrowed
- ... 2 more occurrences in this file

#### `src\query\optimizer\elimination_rules.rs`: 3 occurrences

- Line 190: cannot assign to `self.eliminated` because it is borrowed: `self.eliminated` is assigned to here but it was already borrowed
- Line 342: cannot assign to `self.eliminated` because it is borrowed: `self.eliminated` is assigned to here but it was already borrowed
- Line 537: cannot assign to `self.eliminated` because it is borrowed: `self.eliminated` is assigned to here but it was already borrowed

#### `src\query\optimizer\predicate_pushdown.rs`: 2 occurrences

- Line 100: cannot assign to `self.pushed_down` because it is borrowed: `self.pushed_down` is assigned to here but it was already borrowed
- Line 104: cannot assign to `self.pushed_down` because it is borrowed: `self.pushed_down` is assigned to here but it was already borrowed

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 65: cannot assign to `self.merged` because it is borrowed: `self.merged` is assigned to here but it was already borrowed

#### `src\query\optimizer\transformation_rules.rs`: 1 occurrences

- Line 106: cannot assign to `self.converted` because it is borrowed: `self.converted` is assigned to here but it was already borrowed

### error[E0061]: this function takes 2 arguments but 1 argument was supplied

**Total Occurrences**: 10  
**Unique Files**: 6

#### `src\query\optimizer\index_optimization.rs`: 4 occurrences

- Line 91: this function takes 2 arguments but 1 argument was supplied
- Line 95: this function takes 2 arguments but 1 argument was supplied
- Line 192: this function takes 2 arguments but 1 argument was supplied
- ... 1 more occurrences in this file

#### `src\query\optimizer\engine\optimizer.rs`: 2 occurrences

- Line 499: this method takes 0 arguments but 1 argument was supplied
- Line 642: this method takes 2 arguments but 1 argument was supplied

#### `src\query\optimizer\predicate_pushdown.rs`: 1 occurrences

- Line 193: this function takes 2 arguments but 1 argument was supplied

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 107: this function takes 2 arguments but 1 argument was supplied

#### `src\query\optimizer\plan\node.rs`: 1 occurrences

- Line 592: this function takes 4 arguments but 2 arguments were supplied

#### `src\query\optimizer\rule_registry.rs`: 1 occurrences

- Line 129: this function takes 0 arguments but 1 argument was supplied

### error[E0502]: cannot borrow `*ctx` as mutable because it is also borrowed as immutable: mutable borrow occurs here

**Total Occurrences**: 7  
**Unique Files**: 1

#### `src\query\optimizer\limit_pushdown.rs`: 7 occurrences

- Line 230: cannot borrow `*ctx` as mutable because it is also borrowed as immutable: mutable borrow occurs here
- Line 321: cannot borrow `*ctx` as mutable because it is also borrowed as immutable: mutable borrow occurs here
- Line 407: cannot borrow `*ctx` as mutable because it is also borrowed as immutable: mutable borrow occurs here
- ... 4 more occurrences in this file

### error[E0308]: mismatched types: expected `&Rc<RefCell<OptGroupNode>>`, found `&OptGroupNode`

**Total Occurrences**: 6  
**Unique Files**: 5

#### `src\query\optimizer\engine\optimizer.rs`: 2 occurrences

- Line 609: mismatched types: expected `Rc<RefCell<OptGroupNode>>`, found `OptGroupNode`
- Line 695: mismatched types: expected `Option<PlanNodeEnum>`, found `PlanNodeEnum`

#### `src\query\optimizer\predicate_reorder.rs`: 1 occurrences

- Line 181: mismatched types: expected `&Rc<RefCell<OptGroupNode>>`, found `&OptGroupNode`

#### `src\query\optimizer\constant_folding.rs`: 1 occurrences

- Line 488: mismatched types: expected `&Rc<RefCell<OptGroupNode>>`, found `&OptGroupNode`

#### `src\query\optimizer\subquery_optimization.rs`: 1 occurrences

- Line 149: mismatched types: expected `&Rc<RefCell<OptGroupNode>>`, found `&OptGroupNode`

#### `src\query\optimizer\loop_unrolling.rs`: 1 occurrences

- Line 383: mismatched types: expected `&Rc<RefCell<OptGroupNode>>`, found `&OptGroupNode`

### error[E0599]: no method named `add_matcher` found for struct `node::Pattern` in the current scope

**Total Occurrences**: 6  
**Unique Files**: 2

#### `src\query\optimizer\engine\optimizer.rs`: 5 occurrences

- Line 161: no method named `borrow` found for reference `&OptGroup` in the current scope
- Line 188: no method named `borrow` found for reference `&OptGroup` in the current scope
- Line 560: no method named `borrow` found for reference `&OptGroup` in the current scope
- ... 2 more occurrences in this file

#### `src\query\optimizer\plan\node.rs`: 1 occurrences

- Line 613: no method named `add_matcher` found for struct `node::Pattern` in the current scope

### error[E0505]: cannot move out of `node` because it is borrowed: move out of `node` occurs here

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\optimizer\push_filter_down_aggregate.rs`: 1 occurrences

- Line 73: cannot move out of `node` because it is borrowed: move out of `node` occurs here

### error[E0658]: use of unstable library feature `str_as_str`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\optimizer\engine\optimizer.rs`: 1 occurrences

- Line 537: use of unstable library feature `str_as_str`

## Detailed Warning Categorization

### warning: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`

**Total Occurrences**: 145  
**Unique Files**: 56

#### `src\query\optimizer\predicate_pushdown.rs`: 22 occurrences

- Line 11: unused import: `combine_conditions`
- Line 218: unused variable: `node_ref`: help: if this is intentional, prefix it with an underscore: `_node_ref`
- Line 214: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- ... 19 more occurrences in this file

#### `src\query\optimizer\operation_merge.rs`: 16 occurrences

- Line 128: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 225: unused variable: `node_ref`: help: if this is intentional, prefix it with an underscore: `_node_ref`
- Line 221: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- ... 13 more occurrences in this file

#### `src\query\optimizer\limit_pushdown.rs`: 12 occurrences

- Line 45: unused variable: `input_id`: help: if this is intentional, prefix it with an underscore: `_input_id`
- Line 194: unused variable: `node_ref`: help: if this is intentional, prefix it with an underscore: `_node_ref`
- Line 190: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
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

#### `src\query\optimizer\projection_pushdown.rs`: 6 occurrences

- Line 125: unused variable: `node_ref`: help: if this is intentional, prefix it with an underscore: `_node_ref`
- Line 121: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 123: unused variable: `child`: help: if this is intentional, prefix it with an underscore: `_child`
- ... 3 more occurrences in this file

#### `src\query\optimizer\engine\optimizer.rs`: 4 occurrences

- Line 14: unused import: `Cost`
- Line 16: unused import: `TransformResult`
- Line 18: unused import: `crate::query::optimizer::property_tracker::PropertyTracker`
- ... 1 more occurrences in this file

#### `src\storage\iterator\composite.rs`: 3 occurrences

- Line 120: unused variable: `idx`: help: if this is intentional, prefix it with an underscore: `_idx`
- Line 141: unused variable: `row`: help: if this is intentional, prefix it with an underscore: `_row`
- Line 705: variable does not need to be mutable

#### `src\storage\transaction\log.rs`: 3 occurrences

- Line 15: unused import: `self`
- Line 460: unused variable: `flushed`: help: if this is intentional, prefix it with an underscore: `_flushed`
- Line 458: unused variable: `min_lsn`: help: if this is intentional, prefix it with an underscore: `_min_lsn`

#### `src\query\planner\statements\match_planner.rs`: 3 occurrences

- Line 22: unused import: `PlanNodeEnum`
- Line 303: unreachable pattern: no value can reach this
- Line 568: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`

#### `src\query\executor\search_executors.rs`: 2 occurrences

- Line 13: unused import: `crate::expression::evaluator::traits::ExpressionContext`
- Line 358: value assigned to `vertices` is never read

#### `src\query\executor\result_processing\dedup.rs`: 2 occurrences

- Line 494: unused import: `crate::query::executor::base::BaseExecutor`
- Line 495: unused import: `crate::query::executor::executor_enum::ExecutorEnum`

#### `src\storage\transaction\traits.rs`: 2 occurrences

- Line 10: unused import: `Value`
- Line 11: unused imports: `LockManager`, `LogRecord`, `TransactionLog`, and `VersionVec`

#### `src\core\types\expression\visitor.rs`: 2 occurrences

- Line 149: unused variable: `property`: help: if this is intentional, prefix it with an underscore: `_property`
- Line 177: unused variable: `variable`: help: if this is intentional, prefix it with an underscore: `_variable`

#### `src\expression\context\basic_context.rs`: 2 occurrences

- Line 592: function cannot return without recursing: cannot return without recursing
- Line 611: function cannot return without recursing: cannot return without recursing

#### `src\query\parser\lexer\lexer.rs`: 2 occurrences

- Line 961: variable does not need to be mutable
- Line 1009: variable does not need to be mutable

#### `src\query\optimizer\optimizer_config.rs`: 2 occurrences

- Line 134: unused import: `std::io::Write`
- Line 135: unused import: `tempfile::NamedTempFile`

#### `src\storage\transaction\snapshot.rs`: 2 occurrences

- Line 9: unused import: `LockType`
- Line 290: unused variable: `key_lock`: help: if this is intentional, prefix it with an underscore: `_key_lock`

#### `src\expression\context\query_expression_context.rs`: 2 occurrences

- Line 444: function cannot return without recursing: cannot return without recursing
- Line 463: function cannot return without recursing: cannot return without recursing

#### `src\expression\context\default_context.rs`: 2 occurrences

- Line 524: function cannot return without recursing: cannot return without recursing
- Line 543: function cannot return without recursing: cannot return without recursing

#### `src\query\planner\plan\core\nodes\join_node.rs`: 2 occurrences

- Line 1056: unused variable: `l`: help: if this is intentional, prefix it with an underscore: `_l`
- Line 1057: unused variable: `r`: help: if this is intentional, prefix it with an underscore: `_r`

#### `src\expression\context\row_context.rs`: 2 occurrences

- Line 249: function cannot return without recursing: cannot return without recursing
- Line 268: function cannot return without recursing: cannot return without recursing

#### `src\query\executor\operation_kind_support.rs`: 1 occurrences

- Line 101: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`

#### `src\query\visitor\ast_transformer.rs`: 1 occurrences

- Line 8: unused imports: `AlterStmt`, `Assignment`, `ChangePasswordStmt`, `CreateStmt`, `DeleteStmt`, `DescStmt`, `DropStmt`, `ExplainStmt`, `FetchStmt`, `FindPathStmt`, `GoStmt`, `InsertStmt`, `LookupStmt`, `MatchStmt`, `MergeStmt`, `PipeStmt`, `QueryStmt`, `RemoveStmt`, `ReturnStmt`, `SetStmt`, `ShowStmt`, `Stmt`, `SubgraphStmt`, `UnwindStmt`, `UpdateStmt`, `UseStmt`, and `WithStmt`

#### `src\query\parser\ast\utils.rs`: 1 occurrences

- Line 14: unused variable: `span`: help: if this is intentional, prefix it with an underscore: `_span`

#### `src\query\executor\admin\tag\create_tag.rs`: 1 occurrences

- Line 9: unused import: `crate::core::types::graph_schema::PropertyType`

#### `src\query\validator\insert_vertices_validator.rs`: 1 occurrences

- Line 204: unused import: `crate::core::Value`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 365: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\storage\transaction\mvcc.rs`: 1 occurrences

- Line 11: unused import: `TransactionState`

#### `src\core\types\expression\expression.rs`: 1 occurrences

- Line 279: unused variable: `meta2`: help: if this is intentional, prefix it with an underscore: `_meta2`

#### `src\query\planner\plan\execution_plan.rs`: 1 occurrences

- Line 68: unused variable: `n`: help: if this is intentional, prefix it with an underscore: `_n`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 45: unused variable: `ast_ctx`: help: if this is intentional, prefix it with an underscore: `_ast_ctx`

#### `src\expression\evaluator\expression_evaluator.rs`: 1 occurrences

- Line 437: unreachable pattern: no value can reach this

#### `src\query\context\ast\query_types\go.rs`: 1 occurrences

- Line 92: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 465: unused variable: `target_type`: help: if this is intentional, prefix it with an underscore: `_target_type`

#### `src\query\planner\statements\seeks\scan_seek.rs`: 1 occurrences

- Line 82: unused variable: `seek`: help: if this is intentional, prefix it with an underscore: `_seek`

#### `src\storage\iterator\predicate.rs`: 1 occurrences

- Line 486: unused variable: `pred2`: help: if this is intentional, prefix it with an underscore: `_pred2`

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 348: unnecessary parentheses around function argument

#### `src\query\scheduler\execution_plan_analyzer.rs`: 1 occurrences

- Line 110: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode`

#### `src\query\executor\graph_query_executor.rs`: 1 occurrences

- Line 138: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`

#### `src\expression\evaluator\traits.rs`: 1 occurrences

- Line 55: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\optimizer\push_filter_down_aggregate.rs`: 1 occurrences

- Line 13: unused import: `crate::query::planner::plan::core::nodes::aggregate_node::AggregateNode`

#### `src\query\context\ast\query_types\fetch_vertices.rs`: 1 occurrences

- Line 47: unused variable: `ids`: help: try ignoring the field: `ids: _`

#### `src\query\parser\parser\expr_parser.rs`: 1 occurrences

- Line 450: unused variable: `test_expr`: help: if this is intentional, prefix it with an underscore: `_test_expr`

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 12: unused import: `crate::core::StorageError`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 49: unused import: `crate::query::planner::plan::core::nodes::admin_node`

#### `src\query\executor\executor_enum.rs`: 1 occurrences

- Line 27: unused import: `GetEdgesExecutor`

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 1 occurrences

- Line 36: unused variable: `ast_ctx`: help: if this is intentional, prefix it with an underscore: `_ast_ctx`

#### `src\query\planner\statements\match_statement_planner.rs`: 1 occurrences

- Line 353: unreachable pattern: no value can reach this

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 32: unused variable: `matched`: help: if this is intentional, prefix it with an underscore: `_matched`

#### `src\query\planner\planner.rs`: 1 occurrences

- Line 210: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`

#### `src\common\memory.rs`: 1 occurrences

- Line 188: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\executor\admin\edge\create_edge.rs`: 1 occurrences

- Line 8: unused import: `EdgeTypeInfo`

#### `src\query\optimizer\plan\group.rs`: 1 occurrences

- Line 14: unused import: `super::Pattern`

#### `src\storage\transaction\lock.rs`: 1 occurrences

- Line 11: unused import: `crate::core::StorageError`

#### `src\storage\index\index_manager.rs`: 1 occurrences

- Line 7: unused import: `IndexType`

