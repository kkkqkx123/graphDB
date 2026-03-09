# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 94
- **Total Warnings**: 2
- **Total Issues**: 96
- **Unique Error Patterns**: 44
- **Unique Warning Patterns**: 2
- **Files with Issues**: 8

## Error Statistics

**Total Errors**: 94

### Error Type Breakdown

- **error[E0599]**: 74 errors
- **error[E0061]**: 9 errors
- **error[E0308]**: 8 errors
- **error[E0277]**: 3 errors

### Files with Errors (Top 10)

- `src\query\executor\factory\executor_factory.rs`: 61 errors
- `src\query\executor\factory\builders\data_processing_builder.rs`: 11 errors
- `src\query\executor\factory\builders\admin_builder.rs`: 11 errors
- `src\query\executor\factory\builders\traversal_builder.rs`: 6 errors
- `src\query\executor\factory\builders\set_operation_builder.rs`: 3 errors
- `src\query\executor\factory\executors\plan_executor.rs`: 1 errors
- `src\query\query_pipeline_manager.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 2

### Warning Type Breakdown

- **warning**: 2 warnings

### Files with Warnings (Top 10)

- `src\query\executor\factory\validators\safety_validator.rs`: 1 warnings
- `src\query\executor\factory\executor_factory.rs`: 1 warnings

## Detailed Error Categorization

### error[E0599]: no method named `input` found for reference `&filter_node::FilterNode` in the current scope: private field, not a method

**Total Occurrences**: 74  
**Unique Files**: 5

#### `src\query\executor\factory\executor_factory.rs`: 61 occurrences

- Line 87: no method named `input` found for reference `&filter_node::FilterNode` in the current scope: private field, not a method
- Line 90: no method named `input` found for reference `&project_node::ProjectNode` in the current scope: private field, not a method
- Line 93: no method named `input` found for reference `&sort_node::LimitNode` in the current scope: private field, not a method
- ... 58 more occurrences in this file

#### `src\query\executor\factory\builders\admin_builder.rs`: 9 occurrences

- Line 389: no method named `space_name` found for reference `&ShowTagIndexesNode` in the current scope: method not found in `&ShowTagIndexesNode`
- Line 485: no method named `space_name` found for reference `&ShowEdgeIndexesNode` in the current scope: method not found in `&ShowEdgeIndexesNode`
- Line 520: no method named `user_name` found for reference `&CreateUserNode` in the current scope
- ... 6 more occurrences in this file

#### `src\query\executor\factory\builders\data_processing_builder.rs`: 2 occurrences

- Line 197: no variant or associated item named `Null` found for enum `def::Expression` in the current scope: variant or associated item not found in `Expression`
- Line 197: no variant or associated item named `Unknown` found for enum `core::value::types::NullType` in the current scope: variant or associated item not found in `NullType`

#### `src\query\executor\factory\executors\plan_executor.rs`: 1 occurrences

- Line 60: no method named `execute` found for enum `executor_enum::ExecutorEnum` in the current scope: method not found in `ExecutorEnum<S>`

#### `src\query\query_pipeline_manager.rs`: 1 occurrences

- Line 486: no method named `execute_plan` found for struct `ExecutorFactory` in the current scope: method not found in `ExecutorFactory<S>`

### error[E0061]: this function takes 6 arguments but 8 arguments were supplied

**Total Occurrences**: 9  
**Unique Files**: 3

#### `src\query\executor\factory\builders\data_processing_builder.rs`: 4 occurrences

- Line 47: this function takes 3 arguments but 4 arguments were supplied
- Line 90: this function takes 4 arguments but 5 arguments were supplied
- Line 118: this function takes 5 arguments but 4 arguments were supplied
- ... 1 more occurrences in this file

#### `src\query\executor\factory\builders\traversal_builder.rs`: 3 occurrences

- Line 44: this function takes 6 arguments but 8 arguments were supplied
- Line 70: this function takes 7 arguments but 8 arguments were supplied
- Line 96: this function takes 7 arguments but 8 arguments were supplied

#### `src\query\executor\factory\builders\admin_builder.rs`: 2 occurrences

- Line 517: this function takes 4 arguments but 5 arguments were supplied
- Line 534: this function takes 4 arguments but 5 arguments were supplied

### error[E0308]: mismatched types: expected `Vec<ProjectionColumn>`, found `Vec<(String, Expression)>`

**Total Occurrences**: 8  
**Unique Files**: 2

#### `src\query\executor\factory\builders\data_processing_builder.rs`: 5 occurrences

- Line 77: mismatched types: expected `Vec<ProjectionColumn>`, found `Vec<(String, Expression)>`
- Line 124: mismatched types: expected `SortExecutor<S>`, found `Result<SortExecutor<S>, DBError>`
- Line 145: arguments to this function are incorrect
- ... 2 more occurrences in this file

#### `src\query\executor\factory\builders\set_operation_builder.rs`: 3 occurrences

- Line 46: mismatched types: expected `String`, found `Vec<String>`
- Line 69: mismatched types: expected `String`, found `Vec<String>`
- Line 92: mismatched types: expected `String`, found `Vec<String>`

### error[E0277]: the trait bound `std::option::Option<usize>: From<graph_schema::EdgeDirection>` is not satisfied: the trait `From<graph_schema::EdgeDirection>` is not implemented for `std::option::Option<usize>`

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\query\executor\factory\builders\traversal_builder.rs`: 3 occurrences

- Line 49: the trait bound `std::option::Option<usize>: From<graph_schema::EdgeDirection>` is not satisfied: the trait `From<graph_schema::EdgeDirection>` is not implemented for `std::option::Option<usize>`
- Line 75: the trait bound `bool: From<&str>` is not satisfied: the trait `From<&str>` is not implemented for `bool`
- Line 101: the trait bound `std::option::Option<usize>: From<graph_schema::EdgeDirection>` is not satisfied: the trait `From<graph_schema::EdgeDirection>` is not implemented for `std::option::Option<usize>`

## Detailed Warning Categorization

### warning: unused import: `Executor`

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\query\executor\factory\executor_factory.rs`: 1 occurrences

- Line 7: unused import: `Executor`

#### `src\query\executor\factory\validators\safety_validator.rs`: 1 occurrences

- Line 7: unused import: `std::sync::Arc`

