# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 18
- **Total Issues**: 18
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 11
- **Files with Issues**: 6

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 18

### Warning Type Breakdown

- **warning**: 18 warnings

### Files with Warnings (Top 10)

- `src\query\planning\plan\core\nodes\base\plan_node_enum.rs`: 6 warnings
- `src\query\executor\object_pool.rs`: 5 warnings
- `src\query\cache\global_manager.rs`: 2 warnings
- `src\query\cache\plan_cache.rs`: 2 warnings
- `src\query\cache\warmup.rs`: 2 warnings
- `src\query\planning\plan\core\nodes\base\memory_estimation.rs`: 1 warnings

## Detailed Warning Categorization

### warning: method `update_ttl` is never used

**Total Occurrences**: 18  
**Unique Files**: 6

#### `src\query\planning\plan\core\nodes\base\plan_node_enum.rs`: 6 occurrences

- Line 1113: redundant closure: help: replace the closure with the associated function itself: `Self::estimate_input_memory`
- Line 1122: redundant closure: help: replace the closure with the associated function itself: `Self::estimate_input_memory`
- Line 1131: redundant closure: help: replace the closure with the associated function itself: `Self::estimate_input_memory`
- ... 3 more occurrences in this file

#### `src\query\executor\object_pool.rs`: 5 occurrences

- Line 113: field assignment outside of initializer for an instance created with Default::default()
- Line 118: you seem to want to iterate on a map's values
- Line 119: manual implementation of an assign operation: help: replace it with: `type_config.max_size /= 2`
- ... 2 more occurrences in this file

#### `src\query\cache\plan_cache.rs`: 2 occurrences

- Line 487: method `update_ttl` is never used
- Line 36: this `impl` can be derived

#### `src\query\cache\warmup.rs`: 2 occurrences

- Line 89: this let-binding has unit value
- Line 172: this `impl` can be derived

#### `src\query\cache\global_manager.rs`: 2 occurrences

- Line 14: unused import: `AtomicU64`
- Line 387: struct update has no effect, all the fields in the struct have already been specified

#### `src\query\planning\plan\core\nodes\base\memory_estimation.rs`: 1 occurrences

- Line 37: manual slice size calculation: help: try: `std::mem::size_of_val(vec)`

