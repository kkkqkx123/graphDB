# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 1
- **Total Warnings**: 23
- **Total Issues**: 24
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 17
- **Files with Issues**: 23

## Error Statistics

**Total Errors**: 1

### Error Type Breakdown

- **error[E0560]**: 1 errors

### Files with Errors (Top 10)

- `src\query\executor\aggregation.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 23

### Warning Type Breakdown

- **warning**: 23 warnings

### Files with Warnings (Top 10)

- `src\query\validator\strategies\alias_strategy.rs`: 2 warnings
- `src\query\validator\strategies\type_inference.rs`: 1 warnings
- `src\query\validator\go_validator.rs`: 1 warnings
- `src\query\executor\logic\loops.rs`: 1 warnings
- `src\common\memory.rs`: 1 warnings
- `src\query\optimizer\transformation_rules.rs`: 1 warnings
- `src\query\parser\lexer\lexer.rs`: 1 warnings
- `src\query\optimizer\index_optimization.rs`: 1 warnings
- `src\query\query_pipeline_manager.rs`: 1 warnings
- `src\index\binary.rs`: 1 warnings

## Detailed Error Categorization

### error[E0560]: struct `query::executor::aggregation::AggregationExecutor<test_mock::MockStorage>` has no field named `filter_condition`: `query::executor::aggregation::AggregationExecutor<_>` does not have this field

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\aggregation.rs`: 1 occurrences

- Line 516: struct `query::executor::aggregation::AggregationExecutor<test_mock::MockStorage>` has no field named `filter_condition`: `query::executor::aggregation::AggregationExecutor<_>` does not have this field

## Detailed Warning Categorization

### warning: unused import: `crate::storage::StorageEngine`

**Total Occurrences**: 23  
**Unique Files**: 22

#### `src\query\validator\strategies\alias_strategy.rs`: 2 occurrences

- Line 111: unreachable pattern: no value can reach this
- Line 112: unreachable pattern: no value can reach this

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 322: unused import: `crate::storage::StorageEngine`

#### `src\query\validator\strategies\type_inference.rs`: 1 occurrences

- Line 655: unreachable pattern: no value can reach this

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 267: unused import: `UnaryOperator`

#### `src\query\executor\data_access.rs`: 1 occurrences

- Line 657: unused variable: `end_vertex`: help: if this is intentional, prefix it with an underscore: `_end_vertex`

#### `src\query\executor\logic\loops.rs`: 1 occurrences

- Line 524: unused import: `crate::core::value::NullType`

#### `src\query\optimizer\index_optimization.rs`: 1 occurrences

- Line 1017: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\optimizer\transformation_rules.rs`: 1 occurrences

- Line 91: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\planner\statements\path_planner.rs`: 1 occurrences

- Line 75: unused variable: `min_hops`: help: if this is intentional, prefix it with an underscore: `_min_hops`

#### `src\query\parser\lexer\lexer.rs`: 1 occurrences

- Line 875: variable does not need to be mutable

#### `src\index\binary.rs`: 1 occurrences

- Line 329: unused import: `TimeValue`

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 982: unused import: `crate::core::value::NullType`

#### `src\query\executor\result_processing\transformations\assign.rs`: 1 occurrences

- Line 168: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\common\memory.rs`: 1 occurrences

- Line 188: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 574: unused import: `SortNode`

#### `src\query\optimizer\rule_traits.rs`: 1 occurrences

- Line 726: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\query_pipeline_manager.rs`: 1 occurrences

- Line 109: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`

#### `src\query\validator\go_validator.rs`: 1 occurrences

- Line 616: unreachable pattern: no value can reach this

#### `src\core\expression_utils.rs`: 1 occurrences

- Line 7: unused import: `crate::core::Value`

#### `src\query\validator\strategies\expression_operations.rs`: 1 occurrences

- Line 510: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\query\executor\data_processing\join\mod.rs`: 1 occurrences

- Line 12: unused import: `crate::core::Expression`

#### `src\query\visitor\deduce_props_visitor.rs`: 1 occurrences

- Line 8: unused import: `crate::query::parser::ast::expression::*`

