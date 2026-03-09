# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 6
- **Total Warnings**: 1
- **Total Issues**: 7
- **Unique Error Patterns**: 5
- **Unique Warning Patterns**: 1
- **Files with Issues**: 5

## Error Statistics

**Total Errors**: 6

### Error Type Breakdown

- **error[E0308]**: 3 errors
- **error[E0500]**: 2 errors
- **error[E0599]**: 1 errors

### Files with Errors (Top 10)

- `src\query\executor\factory\executor_factory.rs`: 3 errors
- `src\query\executor\factory\builders\admin_builder.rs`: 1 errors
- `src\query\executor\factory\executors\plan_executor.rs`: 1 errors
- `src\query\executor\factory\builders\data_processing_builder.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 1

### Warning Type Breakdown

- **warning**: 1 warnings

### Files with Warnings (Top 10)

- `src\query\executor\factory\validators\safety_validator.rs`: 1 warnings

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `&PlanNodeEnum`, found `Box<PlanNodeEnum>`

**Total Occurrences**: 3  
**Unique Files**: 3

#### `src\query\executor\factory\executor_factory.rs`: 1 occurrences

- Line 84: mismatched types: expected `&PlanNodeEnum`, found `Box<PlanNodeEnum>`

#### `src\query\executor\factory\builders\data_processing_builder.rs`: 1 occurrences

- Line 221: arguments to this function are incorrect

#### `src\query\executor\factory\builders\admin_builder.rs`: 1 occurrences

- Line 587: mismatched types: expected `Option<String>`, found `String`

### error[E0500]: closure requires unique access to `*self` but it is already borrowed: closure construction occurs here

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\executor\factory\executor_factory.rs`: 2 occurrences

- Line 250: closure requires unique access to `*self` but it is already borrowed: closure construction occurs here
- Line 258: closure requires unique access to `*self` but it is already borrowed: closure construction occurs here

### error[E0599]: no method named `execute` found for enum `executor_enum::ExecutorEnum` in the current scope: method not found in `ExecutorEnum<S>`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\factory\executors\plan_executor.rs`: 1 occurrences

- Line 60: no method named `execute` found for enum `executor_enum::ExecutorEnum` in the current scope: method not found in `ExecutorEnum<S>`

## Detailed Warning Categorization

### warning: unused import: `std::sync::Arc`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\factory\validators\safety_validator.rs`: 1 occurrences

- Line 7: unused import: `std::sync::Arc`

