# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 42
- **Total Warnings**: 87
- **Total Issues**: 129
- **Unique Error Patterns**: 12
- **Unique Warning Patterns**: 30
- **Files with Issues**: 54

## Error Statistics

**Total Errors**: 42

### Error Type Breakdown

- **error[E0050]**: 13 errors
- **error[E0027]**: 11 errors
- **error[E0596]**: 6 errors
- **error[E0308]**: 5 errors
- **error[E0063]**: 4 errors
- **error[E0408]**: 2 errors
- **error[E0004]**: 1 errors

### Files with Errors (Top 10)

- `src\core\types\expression\mod.rs`: 5 errors
- `src\query\context\symbol\symbol_table.rs`: 3 errors
- `src\query\validator\strategies\type_inference.rs`: 2 errors
- `src\query\validator\strategies\expression_operations.rs`: 2 errors
- `src\api\service\graph_service.rs`: 2 errors
- `src\query\planner\plan\core\nodes\join_node.rs`: 2 errors
- `src\query\visitor\ast_transformer.rs`: 2 errors
- `src\query\validator\strategies\variable_validator.rs`: 2 errors
- `src\query\visitor\find_visitor.rs`: 2 errors
- `src\query\executor\search_executors.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 87

### Warning Type Breakdown

- **warning**: 87 warnings

### Files with Warnings (Top 10)

- `src\query\optimizer\operation_merge.rs`: 16 warnings
- `src\query\optimizer\limit_pushdown.rs`: 12 warnings
- `src\query\optimizer\predicate_pushdown.rs`: 7 warnings
- `src\query\context\symbol\symbol_table.rs`: 7 warnings
- `src\query\optimizer\projection_pushdown.rs`: 5 warnings
- `src\query\executor\result_processing\projection.rs`: 5 warnings
- `src\api\service\graph_service.rs`: 4 warnings
- `src\query\optimizer\engine\optimizer.rs`: 3 warnings
- `src\core\vertex_edge_path.rs`: 3 warnings
- `src\query\optimizer\elimination_rules.rs`: 3 warnings

## Detailed Error Categorization

### error[E0050]: method `visit_case` has 3 parameters but the declaration in trait `core::types::expression::visitor::ExpressionVisitor::visit_case` has 4: expected 4 parameters, found 3

**Total Occurrences**: 13  
**Unique Files**: 13

#### `src\query\visitor\extract_group_suite_visitor.rs`: 1 occurrences

- Line 242: method `visit_case` has 3 parameters but the declaration in trait `core::types::expression::visitor::ExpressionVisitor::visit_case` has 4: expected 4 parameters, found 3

#### `src\query\visitor\validate_pattern_expression_visitor.rs`: 1 occurrences

- Line 154: method `visit_case` has 3 parameters but the declaration in trait `core::types::expression::visitor::ExpressionVisitor::visit_case` has 4: expected 4 parameters, found 3

#### `src\query\visitor\property_tracker_visitor.rs`: 1 occurrences

- Line 275: method `visit_case` has 3 parameters but the declaration in trait `core::types::expression::visitor::ExpressionVisitor::visit_case` has 4: expected 4 parameters, found 3

#### `src\query\visitor\deduce_props_visitor.rs`: 1 occurrences

- Line 373: method `visit_case` has 3 parameters but the declaration in trait `core::types::expression::visitor::ExpressionVisitor::visit_case` has 4: expected 4 parameters, found 3

#### `src\query\optimizer\prune_properties_visitor.rs`: 1 occurrences

- Line 220: method `visit_case` has 3 parameters but the declaration in trait `core::types::expression::visitor::ExpressionVisitor::visit_case` has 4: expected 4 parameters, found 3

#### `src\query\visitor\vid_extract_visitor.rs`: 1 occurrences

- Line 327: method `visit_case` has 3 parameters but the declaration in trait `core::types::expression::visitor::ExpressionVisitor::visit_case` has 4: expected 4 parameters, found 3

#### `src\query\visitor\find_visitor.rs`: 1 occurrences

- Line 161: method `visit_case` has 3 parameters but the declaration in trait `core::types::expression::visitor::ExpressionVisitor::visit_case` has 4: expected 4 parameters, found 3

#### `src\query\visitor\extract_prop_expr_visitor.rs`: 1 occurrences

- Line 238: method `visit_case` has 3 parameters but the declaration in trait `core::types::expression::visitor::ExpressionVisitor::visit_case` has 4: expected 4 parameters, found 3

#### `src\query\visitor\variable_visitor.rs`: 1 occurrences

- Line 90: method `visit_case` has 3 parameters but the declaration in trait `core::types::expression::visitor::ExpressionVisitor::visit_case` has 4: expected 4 parameters, found 3

#### `src\query\visitor\deduce_alias_type_visitor.rs`: 1 occurrences

- Line 192: method `visit_case` has 3 parameters but the declaration in trait `core::types::expression::visitor::ExpressionVisitor::visit_case` has 4: expected 4 parameters, found 3

#### `src\query\visitor\fold_constant_expr_visitor.rs`: 1 occurrences

- Line 650: method `visit_case` has 3 parameters but the declaration in trait `core::types::expression::visitor::ExpressionVisitor::visit_case` has 4: expected 4 parameters, found 3

#### `src\query\visitor\rewrite_visitor.rs`: 1 occurrences

- Line 158: method `visit_case` has 3 parameters but the declaration in trait `core::types::expression::visitor::ExpressionVisitor::visit_case` has 4: expected 4 parameters, found 3

#### `src\query\visitor\extract_filter_expr_visitor.rs`: 1 occurrences

- Line 154: method `visit_case` has 3 parameters but the declaration in trait `core::types::expression::visitor::ExpressionVisitor::visit_case` has 4: expected 4 parameters, found 3

### error[E0027]: pattern does not mention field `test_expr`: missing field `test_expr`

**Total Occurrences**: 11  
**Unique Files**: 8

#### `src\query\validator\strategies\type_inference.rs`: 2 occurrences

- Line 508: pattern does not mention field `test_expr`: missing field `test_expr`
- Line 635: pattern does not mention field `test_expr`: missing field `test_expr`

#### `src\query\validator\strategies\expression_operations.rs`: 2 occurrences

- Line 64: pattern does not mention field `test_expr`: missing field `test_expr`
- Line 458: pattern does not mention field `test_expr`: missing field `test_expr`

#### `src\query\validator\strategies\variable_validator.rs`: 2 occurrences

- Line 157: pattern does not mention field `test_expr`: missing field `test_expr`
- Line 193: pattern does not mention field `test_expr`: missing field `test_expr`

#### `src\query\validator\strategies\alias_strategy.rs`: 1 occurrences

- Line 94: pattern does not mention field `test_expr`: missing field `test_expr`

#### `src\query\validator\order_by_validator.rs`: 1 occurrences

- Line 342: pattern does not mention field `test_expr`: missing field `test_expr`

#### `src\query\validator\go_validator.rs`: 1 occurrences

- Line 340: pattern does not mention field `test_expr`: missing field `test_expr`

#### `src\query\visitor\ast_traverser.rs`: 1 occurrences

- Line 185: pattern does not mention field `test_expr`: missing field `test_expr`

#### `src\query\visitor\ast_transformer.rs`: 1 occurrences

- Line 69: pattern does not mention field `test_expr`: missing field `test_expr`

### error[E0596]: cannot borrow `new_symbols` as mutable, as it is not declared as mutable: cannot borrow as mutable

**Total Occurrences**: 6  
**Unique Files**: 3

#### `src\query\context\symbol\symbol_table.rs`: 3 occurrences

- Line 163: cannot borrow `new_symbols` as mutable, as it is not declared as mutable: cannot borrow as mutable
- Line 175: cannot borrow `new_symbols` as mutable, as it is not declared as mutable: cannot borrow as mutable
- Line 186: cannot borrow `new_symbols` as mutable, as it is not declared as mutable: cannot borrow as mutable

#### `src\api\service\graph_service.rs`: 2 occurrences

- Line 344: cannot borrow `storage` as mutable, as it is not declared as mutable: cannot borrow as mutable
- Line 368: cannot borrow `storage` as mutable, as it is not declared as mutable: cannot borrow as mutable

#### `src\query\executor\search_executors.rs`: 1 occurrences

- Line 386: cannot borrow `vertices` as mutable, as it is not declared as mutable: cannot borrow as mutable

### error[E0308]: mismatched types: expected `Box<Expression>`, found `Expression`

**Total Occurrences**: 5  
**Unique Files**: 1

#### `src\core\types\expression\mod.rs`: 5 occurrences

- Line 395: mismatched types: expected `Box<Expression>`, found `Expression`
- Line 396: mismatched types: expected `Box<Expression>`, found `Expression`
- Line 401: mismatched types: expected `Vec<&Expression>`, found `Vec<Box<Expression>>`
- ... 2 more occurrences in this file

### error[E0063]: missing field `test_expr` in initializer of `core::types::expression::Expression`: missing `test_expr`

**Total Occurrences**: 4  
**Unique Files**: 4

#### `src\query\visitor\ast_transformer.rs`: 1 occurrences

- Line 82: missing field `test_expr` in initializer of `core::types::expression::Expression`: missing `test_expr`

#### `src\query\parser\ast\utils.rs`: 1 occurrences

- Line 62: missing field `test_expr` in initializer of `core::types::expression::Expression`: missing `test_expr`

#### `src\query\visitor\find_visitor.rs`: 1 occurrences

- Line 165: missing field `test_expr` in initializer of `core::types::expression::Expression`: missing `test_expr`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 521: missing field `test_expr` in initializer of `core::types::expression::Expression`: missing `test_expr`

### error[E0408]: variable `_r` is not bound in all patterns: pattern doesn't bind `_r`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\planner\plan\core\nodes\join_node.rs`: 2 occurrences

- Line 1056: variable `_r` is not bound in all patterns: pattern doesn't bind `_r`
- Line 1056: variable `_l` is not bound in all patterns: pattern doesn't bind `_l`

### error[E0004]: non-exhaustive patterns: `&plan_node_enum::PlanNodeEnum::Start(_)`, `&plan_node_enum::PlanNodeEnum::TopN(_)`, `&plan_node_enum::PlanNodeEnum::Sample(_)` and 40 more not covered: patterns `&plan_node_enum::PlanNodeEnum::Start(_)`, `&plan_node_enum::PlanNodeEnum::TopN(_)`, `&plan_node_enum::PlanNodeEnum::Sample(_)` and 40 more not covered

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\optimizer\engine\optimizer.rs`: 1 occurrences

- Line 308: non-exhaustive patterns: `&plan_node_enum::PlanNodeEnum::Start(_)`, `&plan_node_enum::PlanNodeEnum::TopN(_)`, `&plan_node_enum::PlanNodeEnum::Sample(_)` and 40 more not covered: patterns `&plan_node_enum::PlanNodeEnum::Start(_)`, `&plan_node_enum::PlanNodeEnum::TopN(_)`, `&plan_node_enum::PlanNodeEnum::Sample(_)` and 40 more not covered

## Detailed Warning Categorization

### warning: unused variable: `ids`: help: if this is intentional, prefix it with an underscore: `_ids`

**Total Occurrences**: 87  
**Unique Files**: 29

#### `src\query\optimizer\operation_merge.rs`: 16 occurrences

- Line 129: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 229: unused variable: `node_ref`: help: if this is intentional, prefix it with an underscore: `_node_ref`
- Line 225: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- ... 13 more occurrences in this file

#### `src\query\optimizer\limit_pushdown.rs`: 12 occurrences

- Line 46: unused variable: `input_id`: help: if this is intentional, prefix it with an underscore: `_input_id`
- Line 197: unused variable: `node_ref`: help: if this is intentional, prefix it with an underscore: `_node_ref`
- Line 193: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- ... 9 more occurrences in this file

#### `src\query\optimizer\predicate_pushdown.rs`: 7 occurrences

- Line 198: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 707: unused variable: `node_ref`: help: if this is intentional, prefix it with an underscore: `_node_ref`
- Line 703: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- ... 4 more occurrences in this file

#### `src\query\context\symbol\symbol_table.rs`: 7 occurrences

- Line 161: unused variable: `symbol`: help: if this is intentional, prefix it with an underscore: `_symbol`
- Line 173: unused variable: `symbol`: help: if this is intentional, prefix it with an underscore: `_symbol`
- Line 196: variable does not need to be mutable
- ... 4 more occurrences in this file

#### `src\query\executor\result_processing\projection.rs`: 5 occurrences

- Line 321: unused imports: `ExecutionResult` and `Executor`
- Line 334: variable does not need to be mutable
- Line 370: variable does not need to be mutable
- ... 2 more occurrences in this file

#### `src\query\optimizer\projection_pushdown.rs`: 5 occurrences

- Line 136: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 138: unused variable: `child`: help: if this is intentional, prefix it with an underscore: `_child`
- Line 207: unused variable: `node_ref`: help: if this is intentional, prefix it with an underscore: `_node_ref`
- ... 2 more occurrences in this file

#### `src\api\service\graph_service.rs`: 4 occurrences

- Line 8: unused import: `crate::utils::safe_lock`
- Line 336: variable does not need to be mutable
- Line 375: variable does not need to be mutable
- ... 1 more occurrences in this file

#### `src\query\optimizer\engine\optimizer.rs`: 3 occurrences

- Line 466: value assigned to `last_changes` is never read
- Line 572: unused variable: `node_id`: help: if this is intentional, prefix it with an underscore: `_node_id`
- Line 550: unused variable: `root_group`: help: if this is intentional, prefix it with an underscore: `_root_group`

#### `src\query\optimizer\elimination_rules.rs`: 3 occurrences

- Line 90: variable does not need to be mutable
- Line 429: variable does not need to be mutable
- Line 624: variable does not need to be mutable

#### `src\core\vertex_edge_path.rs`: 3 occurrences

- Line 268: unused variable: `v`: help: if this is intentional, prefix it with an underscore: `_v`
- Line 272: unused variable: `v`: help: if this is intentional, prefix it with an underscore: `_v`
- Line 378: unused variable: `v`: help: if this is intentional, prefix it with an underscore: `_v`

#### `src\query\executor\data_access.rs`: 2 occurrences

- Line 152: unused variable: `ids`: help: if this is intentional, prefix it with an underscore: `_ids`
- Line 531: unused variable: `idx`: help: if this is intentional, prefix it with an underscore: `_idx`

#### `src\core\types\expression\visitor.rs`: 2 occurrences

- Line 150: unused variable: `property`: help: if this is intentional, prefix it with an underscore: `_property`
- Line 178: unused variable: `variable`: help: if this is intentional, prefix it with an underscore: `_variable`

#### `src\query\parser\lexer\lexer.rs`: 2 occurrences

- Line 961: variable does not need to be mutable
- Line 1009: variable does not need to be mutable

#### `src\storage\iterator\predicate.rs`: 1 occurrences

- Line 492: unused variable: `pred2`: help: if this is intentional, prefix it with an underscore: `_pred2`

#### `src\storage\processor\base.rs`: 1 occurrences

- Line 531: unused variable: `counters`: help: if this is intentional, prefix it with an underscore: `_counters`

#### `src\core\types\expression\expression.rs`: 1 occurrences

- Line 279: unused variable: `meta2`: help: if this is intentional, prefix it with an underscore: `_meta2`

#### `src\query\executor\graph_query_executor.rs`: 1 occurrences

- Line 138: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`

#### `src\query\context\ast\query_types\go.rs`: 1 occurrences

- Line 92: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 21: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 365: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\common\memory.rs`: 1 occurrences

- Line 222: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\parser\ast\utils.rs`: 1 occurrences

- Line 14: unused variable: `span`: help: if this is intentional, prefix it with an underscore: `_span`

#### `src\query\planner\statements\seeks\scan_seek.rs`: 1 occurrences

- Line 82: unused variable: `seek`: help: if this is intentional, prefix it with an underscore: `_seek`

#### `src\expression\evaluator\traits.rs`: 1 occurrences

- Line 19: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\executor\operation_kind_support.rs`: 1 occurrences

- Line 101: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`

#### `src\query\planner\statements\match_planner.rs`: 1 occurrences

- Line 567: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`

#### `src\query\validator\insert_vertices_validator.rs`: 1 occurrences

- Line 204: unused import: `crate::core::Value`

#### `src\query\context\ast\query_types\fetch_vertices.rs`: 1 occurrences

- Line 47: unused variable: `ids`: help: try ignoring the field: `ids: _`

#### `src\query\optimizer\index_optimization.rs`: 1 occurrences

- Line 25: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

