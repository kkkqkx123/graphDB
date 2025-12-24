# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 41
- **Total Warnings**: 0
- **Total Issues**: 41
- **Unique Error Patterns**: 39
- **Unique Warning Patterns**: 0
- **Files with Issues**: 8

## Error Statistics

**Total Errors**: 41

### Error Type Breakdown

- **error[E0412]**: 29 errors
- **error[E0599]**: 7 errors
- **error[E0046]**: 2 errors
- **error[E0255]**: 1 errors
- **error[E0277]**: 1 errors
- **error[E0609]**: 1 errors

### Files with Errors (Top 10)

- `src\query\planner\plan\core\explain.rs`: 29 errors
- `src\core\evaluator\expression_evaluator.rs`: 4 errors
- `src\query\parser\cypher\expression_evaluator.rs`: 3 errors
- `src\expression\visitor.rs`: 1 errors
- `src\query\visitor\find_visitor.rs`: 1 errors
- `src\query\visitor\mod.rs`: 1 errors
- `src\query\visitor\extract_filter_expr_visitor.rs`: 1 errors
- `src\query\visitor\evaluable_expr_visitor.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0412]: cannot find type `StartNode` in this scope: not found in this scope

**Total Occurrences**: 29  
**Unique Files**: 1

#### `src\query\planner\plan\core\explain.rs`: 29 occurrences

- Line 221: cannot find type `StartNode` in this scope: not found in this scope
- Line 225: cannot find type `ProjectNode` in this scope: not found in this scope
- Line 229: cannot find type `SortNode` in this scope: not found in this scope
- ... 26 more occurrences in this file

### error[E0599]: no function or associated item named `not_implemented` found for struct `core::error::ExpressionError` in the current scope: function or associated item not found in `ExpressionError`

**Total Occurrences**: 7  
**Unique Files**: 3

#### `src\core\evaluator\expression_evaluator.rs`: 4 occurrences

- Line 1126: no function or associated item named `is_cypher_constant` found for struct `CypherExpressionOptimizer` in the current scope: function or associated item not found in `CypherExpressionOptimizer`
- Line 1135: no function or associated item named `collect_cypher_variables` found for struct `CypherEvaluator` in the current scope: function or associated item not found in `CypherEvaluator`
- Line 1149: no function or associated item named `contains_cypher_aggregate` found for struct `CypherEvaluator` in the current scope: function or associated item not found in `CypherEvaluator`
- ... 1 more occurrences in this file

#### `src\query\parser\cypher\expression_evaluator.rs`: 2 occurrences

- Line 47: no function or associated item named `not_implemented` found for struct `core::error::ExpressionError` in the current scope: function or associated item not found in `ExpressionError`
- Line 94: no function or associated item named `not_implemented` found for struct `core::error::ExpressionError` in the current scope: function or associated item not found in `ExpressionError`

#### `src\query\visitor\mod.rs`: 1 occurrences

- Line 65: no function or associated item named `with_config` found for struct `DeducePropsVisitor` in the current scope: function or associated item not found in `DeducePropsVisitor`

### error[E0046]: not all trait items implemented, missing: `visit_unary_plus`, `visit_unary_negate`, `visit_unary_not`, `visit_unary_incr`, `visit_unary_decr`, `visit_is_null`, `visit_is_not_null`, `visit_is_empty`, `visit_is_not_empty`, `visit_type_casting`, `visit_list_comprehension`, `visit_predicate`, `visit_reduce`, `visit_path_build`, `visit_es_query`, `visit_uuid`, `visit_subscript_range`, `visit_match_path_pattern`: missing `visit_unary_plus`, `visit_unary_negate`, `visit_unary_not`, `visit_unary_incr`, `visit_unary_decr`, `visit_is_null`, `visit_is_not_null`, `visit_is_empty`, `visit_is_not_empty`, `visit_type_casting`, `visit_list_comprehension`, `visit_predicate`, `visit_reduce`, `visit_path_build`, `visit_es_query`, `visit_uuid`, `visit_subscript_range`, `visit_match_path_pattern` in implementation

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\query\visitor\extract_filter_expr_visitor.rs`: 1 occurrences

- Line 122: not all trait items implemented, missing: `visit_unary_plus`, `visit_unary_negate`, `visit_unary_not`, `visit_unary_incr`, `visit_unary_decr`, `visit_is_null`, `visit_is_not_null`, `visit_is_empty`, `visit_is_not_empty`, `visit_type_casting`, `visit_list_comprehension`, `visit_predicate`, `visit_reduce`, `visit_path_build`, `visit_es_query`, `visit_uuid`, `visit_subscript_range`, `visit_match_path_pattern`: missing `visit_unary_plus`, `visit_unary_negate`, `visit_unary_not`, `visit_unary_incr`, `visit_unary_decr`, `visit_is_null`, `visit_is_not_null`, `visit_is_empty`, `visit_is_not_empty`, `visit_type_casting`, `visit_list_comprehension`, `visit_predicate`, `visit_reduce`, `visit_path_build`, `visit_es_query`, `visit_uuid`, `visit_subscript_range`, `visit_match_path_pattern` in implementation

#### `src\query\visitor\evaluable_expr_visitor.rs`: 1 occurrences

- Line 43: not all trait items implemented, missing: `visit_unary_plus`, `visit_unary_negate`, `visit_unary_not`, `visit_unary_incr`, `visit_unary_decr`, `visit_is_null`, `visit_is_not_null`, `visit_is_empty`, `visit_is_not_empty`, `visit_type_casting`, `visit_list_comprehension`, `visit_predicate`, `visit_reduce`, `visit_path_build`, `visit_es_query`, `visit_uuid`, `visit_subscript_range`, `visit_match_path_pattern`: missing `visit_unary_plus`, `visit_unary_negate`, `visit_unary_not`, `visit_unary_incr`, `visit_unary_decr`, `visit_is_null`, `visit_is_not_null`, `visit_is_empty`, `visit_is_not_empty`, `visit_type_casting`, `visit_list_comprehension`, `visit_predicate`, `visit_reduce`, `visit_path_build`, `visit_es_query`, `visit_uuid`, `visit_subscript_range`, `visit_match_path_pattern` in implementation

### error[E0277]: the trait bound `HashSet<core::types::expression::ExpressionType>: std::hash::Hash` is not satisfied: the trait `std::hash::Hash` is not implemented for `HashSet<core::types::expression::ExpressionType>`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\expression\visitor.rs`: 1 occurrences

- Line 220: the trait bound `HashSet<core::types::expression::ExpressionType>: std::hash::Hash` is not satisfied: the trait `std::hash::Hash` is not implemented for `HashSet<core::types::expression::ExpressionType>`

### error[E0609]: no field `props` on type `Box<vertex_edge_path::Vertex>`: unknown field

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\parser\cypher\expression_evaluator.rs`: 1 occurrences

- Line 34: no field `props` on type `Box<vertex_edge_path::Vertex>`: unknown field

### error[E0255]: the name `ExpressionType` is defined multiple times: `ExpressionType` redefined here

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\visitor\find_visitor.rs`: 1 occurrences

- Line 18: the name `ExpressionType` is defined multiple times: `ExpressionType` redefined here

