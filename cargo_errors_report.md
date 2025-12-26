# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 37
- **Total Warnings**: 0
- **Total Issues**: 37
- **Unique Error Patterns**: 26
- **Unique Warning Patterns**: 0
- **Files with Issues**: 13

## Error Statistics

**Total Errors**: 37

### Error Type Breakdown

- **error[E0308]**: 12 errors
- **error[E0599]**: 8 errors
- **error[E0616]**: 5 errors
- **error[E0433]**: 4 errors
- **error[E0277]**: 2 errors
- **error[E0119]**: 1 errors
- **error[E0046]**: 1 errors
- **error[E0061]**: 1 errors
- **error[E0004]**: 1 errors
- **error[E0499]**: 1 errors
- **error[E0382]**: 1 errors

### Files with Errors (Top 10)

- `src\query\executor\factory.rs`: 13 errors
- `src\query\executor\data_processing\join\mod.rs`: 5 errors
- `src\query\executor\data_processing\join\full_outer_join.rs`: 4 errors
- `src\query\executor\data_processing\join\right_join.rs`: 4 errors
- `src\query\executor\data_processing\join\left_join.rs`: 2 errors
- `src\core\expression_visitor.rs`: 2 errors
- `src\query\executor\result_processing\aggregation.rs`: 1 errors
- `src\query\executor\cypher\clauses\match_executor.rs`: 1 errors
- `src\query\planner\plan\core\explain.rs`: 1 errors
- `src\core\context\traits.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `Expression`, found `String`

**Total Occurrences**: 12  
**Unique Files**: 7

#### `src\query\executor\data_processing\join\mod.rs`: 5 occurrences

- Line 153: arguments to this function are incorrect
- Line 163: arguments to this function are incorrect
- Line 176: arguments to this function are incorrect
- ... 2 more occurrences in this file

#### `src\query\executor\data_processing\join\left_join.rs`: 2 occurrences

- Line 543: mismatched types: expected `Expression`, found `String`
- Line 544: mismatched types: expected `Expression`, found `String`

#### `src\query\executor\cypher\clauses\match_executor.rs`: 1 occurrences

- Line 197: mismatched types: expected `usize`, found `i64`

#### `src\query\executor\data_processing\join\full_outer_join.rs`: 1 occurrences

- Line 32: arguments to this function are incorrect

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 252: arguments to this function are incorrect

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 579: mismatched types: expected `i64`, found `usize`

#### `src\query\executor\data_processing\join\right_join.rs`: 1 occurrences

- Line 32: arguments to this function are incorrect

### error[E0599]: the method `as_ref` exists for reference `&InnerJoinNode`, but its trait bounds were not satisfied: method cannot be called on `&InnerJoinNode` due to unsatisfied trait bounds

**Total Occurrences**: 8  
**Unique Files**: 3

#### `src\query\executor\factory.rs`: 6 occurrences

- Line 223: the method `as_ref` exists for reference `&InnerJoinNode`, but its trait bounds were not satisfied: method cannot be called on `&InnerJoinNode` due to unsatisfied trait bounds
- Line 226: the method `as_ref` exists for reference `&HashInnerJoinNode`, but its trait bounds were not satisfied: method cannot be called on `&HashInnerJoinNode` due to unsatisfied trait bounds
- Line 229: the method `as_ref` exists for reference `&LeftJoinNode`, but its trait bounds were not satisfied: method cannot be called on `&LeftJoinNode` due to unsatisfied trait bounds
- ... 3 more occurrences in this file

#### `src\query\executor\data_processing\join\right_join.rs`: 1 occurrences

- Line 160: no method named `parse` found for enum `core::types::expression::Expression` in the current scope: method not found in `Expression`

#### `src\query\context\execution\query_execution.rs`: 1 occurrences

- Line 570: no method named `error_count` found for reference `&core::context::validation::ValidationContext` in the current scope: method not found in `&ValidationContext`

### error[E0616]: field `list_expr` of struct `data_processing_node::UnwindNode` is private: private field

**Total Occurrences**: 5  
**Unique Files**: 1

#### `src\query\executor\factory.rs`: 5 occurrences

- Line 264: field `list_expr` of struct `data_processing_node::UnwindNode` is private: private field
- Line 269: field `alias` of struct `data_processing_node::UnwindNode` is private: private field
- Line 271: field `col_names` of struct `data_processing_node::UnwindNode` is private: private field
- ... 2 more occurrences in this file

### error[E0433]: failed to resolve: use of undeclared type `Expression`: use of undeclared type `Expression`

**Total Occurrences**: 4  
**Unique Files**: 2

#### `src\query\executor\data_processing\join\full_outer_join.rs`: 3 occurrences

- Line 106: failed to resolve: use of undeclared type `Expression`: use of undeclared type `Expression`
- Line 147: failed to resolve: use of undeclared type `Expression`: use of undeclared type `Expression`
- Line 192: failed to resolve: use of undeclared type `Expression`: use of undeclared type `Expression`

#### `src\query\executor\data_processing\join\right_join.rs`: 1 occurrences

- Line 106: failed to resolve: use of undeclared type `Expression`: use of undeclared type `Expression`

### error[E0277]: the size for values of type `Self` cannot be known at compilation time: doesn't have a size known at compile-time

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\core\context\traits.rs`: 1 occurrences

- Line 190: the size for values of type `Self` cannot be known at compilation time: doesn't have a size known at compile-time

#### `src\query\executor\data_processing\join\right_join.rs`: 1 occurrences

- Line 153: can't compare `std::string::String` with `core::types::expression::Expression`: no implementation for `std::string::String == core::types::expression::Expression`

### error[E0061]: this method takes 3 arguments but 2 arguments were supplied

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 473: this method takes 3 arguments but 2 arguments were supplied

### error[E0046]: not all trait items implemented, missing: `visit_assign`: missing `visit_assign` in implementation

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\planner\plan\core\explain.rs`: 1 occurrences

- Line 218: not all trait items implemented, missing: `visit_assign`: missing `visit_assign` in implementation

### error[E0004]: non-exhaustive patterns: `&nodes::plan_node_enum::PlanNodeEnum::Assign(_)` not covered: pattern `&nodes::plan_node_enum::PlanNodeEnum::Assign(_)` not covered

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\planner\plan\core\nodes\plan_node_enum.rs`: 1 occurrences

- Line 432: non-exhaustive patterns: `&nodes::plan_node_enum::PlanNodeEnum::Assign(_)` not covered: pattern `&nodes::plan_node_enum::PlanNodeEnum::Assign(_)` not covered

### error[E0382]: use of moved value: `expr_type`: value moved here, in previous iteration of loop

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\core\expression_visitor.rs`: 1 occurrences

- Line 647: use of moved value: `expr_type`: value moved here, in previous iteration of loop

### error[E0119]: conflicting implementations of trait `From<CypherExecutorError>` for type `core::error::DBError`: conflicting implementation for `core::error::DBError`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\cypher\mod.rs`: 1 occurrences

- Line 67: conflicting implementations of trait `From<CypherExecutorError>` for type `core::error::DBError`: conflicting implementation for `core::error::DBError`

### error[E0499]: cannot borrow `*self` as mutable more than once at a time: second mutable borrow occurs here

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\core\expression_visitor.rs`: 1 occurrences

- Line 494: cannot borrow `*self` as mutable more than once at a time: second mutable borrow occurs here

