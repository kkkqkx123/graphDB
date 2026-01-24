# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 3
- **Total Warnings**: 48
- **Total Issues**: 51
- **Unique Error Patterns**: 3
- **Unique Warning Patterns**: 29
- **Files with Issues**: 48

## Error Statistics

**Total Errors**: 3

### Error Type Breakdown

- **error[E0061]**: 2 errors
- **error[E0560]**: 1 errors

### Files with Errors (Top 10)

- `src\query\executor\aggregation.rs`: 2 errors
- `src\query\executor\factory.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 48

### Warning Type Breakdown

- **warning**: 48 warnings

### Files with Warnings (Top 10)

- `src\query\validator\strategies\alias_strategy.rs`: 2 warnings
- `src\query\validator\go_validator.rs`: 1 warnings
- `src\query\context\validate\context.rs`: 1 warnings
- `src\query\parser\lexer\lexer.rs`: 1 warnings
- `src\query\validator\strategies\expression_operations.rs`: 1 warnings
- `src\query\validator\strategies\type_inference.rs`: 1 warnings
- `src\query\context\ast\query_types\fetch_vertices.rs`: 1 warnings
- `src\storage\memory_storage.rs`: 1 warnings
- `src\query\executor\result_processing\sample.rs`: 1 warnings
- `src\query\context\ast\query_types\go.rs`: 1 warnings

## Detailed Error Categorization

### error[E0061]: this function takes 4 arguments but 5 arguments were supplied

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\query\executor\aggregation.rs`: 1 occurrences

- Line 487: this function takes 4 arguments but 5 arguments were supplied

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 520: this function takes 9 arguments but 10 arguments were supplied

### error[E0560]: struct `query::executor::aggregation::AggregationExecutor<test_mock::MockStorage>` has no field named `filter_condition`: `query::executor::aggregation::AggregationExecutor<_>` does not have this field

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\aggregation.rs`: 1 occurrences

- Line 517: struct `query::executor::aggregation::AggregationExecutor<test_mock::MockStorage>` has no field named `filter_condition`: `query::executor::aggregation::AggregationExecutor<_>` does not have this field

## Detailed Warning Categorization

### warning: unused import: `crate::config::Config`

**Total Occurrences**: 48  
**Unique Files**: 47

#### `src\query\validator\strategies\alias_strategy.rs`: 2 occurrences

- Line 111: unreachable pattern: no value can reach this
- Line 112: unreachable pattern: no value can reach this

#### `src\api\service\query_engine.rs`: 1 occurrences

- Line 65: unused import: `crate::config::Config`

#### `src\query\visitor\evaluable_expr_visitor.rs`: 1 occurrences

- Line 7: unused import: `BinaryOperator`

#### `src\query\context\ast\query_types\fetch_vertices.rs`: 1 occurrences

- Line 4: unused imports: `FromType` and `Starts`

#### `src\index\binary.rs`: 1 occurrences

- Line 329: unused import: `TimeValue`

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 388: unused import: `crate::core::value::NullType`

#### `src\core\expression_visitor.rs`: 1 occurrences

- Line 11: unused import: `crate::query::parser::ast::expression::*`

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 394: unused import: `crate::config::test_config::test_config`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 267: unused import: `UnaryOperator`

#### `src\query\optimizer\rule_traits.rs`: 1 occurrences

- Line 726: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\visitor\deduce_props_visitor.rs`: 1 occurrences

- Line 8: unused import: `crate::query::parser::ast::expression::*`

#### `src\query\parser\parser\expr_parser.rs`: 1 occurrences

- Line 9: unused import: `crate::query::parser::core::position::Position`

#### `src\query\visitor\extract_prop_expr_visitor.rs`: 1 occurrences

- Line 15: unused import: `crate::query::parser::ast::expression::*`

#### `src\common\thread.rs`: 1 occurrences

- Line 89: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 493: unused import: `crate::core::value::NullType`

#### `src\query\optimizer\index_optimization.rs`: 1 occurrences

- Line 1017: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\parser\expressions\expression_converter.rs`: 1 occurrences

- Line 361: unused imports: `ListExpression`, `MapExpression`, `PathExpression`, `PropertyAccessExpression`, `RangeExpression`, and `SubscriptExpression`

#### `src\query\parser\expressions\mod.rs`: 1 occurrences

- Line 5: unused import: `crate::query::parser::ast::*`

#### `src\query\context\ast\base.rs`: 1 occurrences

- Line 107: variable does not need to be mutable

#### `src\query\optimizer\prune_properties_visitor.rs`: 1 occurrences

- Line 128: unreachable pattern: no value can reach this

#### `src\query\validator\go_validator.rs`: 1 occurrences

- Line 616: unreachable pattern: no value can reach this

#### `src\query\validator\strategies\type_inference.rs`: 1 occurrences

- Line 655: unreachable pattern: no value can reach this

#### `src\query\context\ast\query_types\go.rs`: 1 occurrences

- Line 4: unused imports: `FromType` and `Starts`

#### `src\query\executor\result_processing\sample.rs`: 1 occurrences

- Line 507: unused import: `crate::core::value::NullType`

#### `src\query\optimizer\optimizer.rs`: 1 occurrences

- Line 3: unused import: `crate::query::context::validate`

#### `src\query\visitor\find_visitor.rs`: 1 occurrences

- Line 9: unused import: `crate::query::parser::ast::expression::*`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 675: variable does not need to be mutable

#### `src\query\executor\object_pool.rs`: 1 occurrences

- Line 255: variable does not need to be mutable

#### `src\query\visitor\property_tracker_visitor.rs`: 1 occurrences

- Line 151: unused variable: `alias`: help: if this is intentional, prefix it with an underscore: `_alias`

#### `src\common\memory.rs`: 1 occurrences

- Line 188: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\parser\parser\stmt_parser.rs`: 1 occurrences

- Line 634: unused variable: `value`: help: if this is intentional, prefix it with an underscore: `_value`

#### `src\storage\memory_storage.rs`: 1 occurrences

- Line 175: variable does not need to be mutable

#### `src\query\validator\strategies\expression_operations.rs`: 1 occurrences

- Line 510: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\query\visitor\rewrite_visitor.rs`: 1 occurrences

- Line 15: unused import: `crate::query::parser::ast::expression::*`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 617: unused import: `super::super::schema::SchemaValidationError`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 574: unused import: `SortNode`

#### `src\query\executor\data_processing\join\mod.rs`: 1 occurrences

- Line 12: unused import: `crate::core::Expression`

#### `src\query\optimizer\transformation_rules.rs`: 1 occurrences

- Line 91: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\planner\statements\path_planner.rs`: 1 occurrences

- Line 75: unused variable: `min_hops`: help: if this is intentional, prefix it with an underscore: `_min_hops`

#### `src\query\executor\logic\loops.rs`: 1 occurrences

- Line 524: unused import: `crate::core::value::NullType`

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 322: unused import: `crate::storage::StorageEngine`

#### `src\query\executor\result_processing\transformations\assign.rs`: 1 occurrences

- Line 168: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\core\expression_utils.rs`: 1 occurrences

- Line 7: unused import: `crate::core::Value`

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 982: unused import: `crate::core::value::NullType`

#### `src\query\executor\data_access.rs`: 1 occurrences

- Line 657: unused variable: `end_vertex`: help: if this is intentional, prefix it with an underscore: `_end_vertex`

#### `src\query\parser\lexer\lexer.rs`: 1 occurrences

- Line 875: variable does not need to be mutable

#### `src\query\query_pipeline_manager.rs`: 1 occurrences

- Line 109: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`

