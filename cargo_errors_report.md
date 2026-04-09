# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 6
- **Total Issues**: 6
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 6
- **Files with Issues**: 5

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 6

### Warning Type Breakdown

- **warning**: 6 warnings

### Files with Warnings (Top 10)

- `src\query\planning\plan\core\nodes\data_access\vector_search.rs`: 2 warnings
- `src\vector\coordinator.rs`: 1 warnings
- `src\query\planning\statements\dml\insert_planner.rs`: 1 warnings
- `crates\inversearch\src\config\validator.rs`: 1 warnings
- `src\api\core\query_api.rs`: 1 warnings

## Detailed Warning Categorization

### warning: unused import: `std::fmt`

**Total Occurrences**: 6  
**Unique Files**: 5

#### `src\query\planning\plan\core\nodes\data_access\vector_search.rs`: 2 occurrences

- Line 67: this function has too many arguments (11/7)
- Line 149: this function has too many arguments (9/7)

#### `crates\inversearch\src\config\validator.rs`: 1 occurrences

- Line 16: unused import: `std::fmt`

#### `src\query\planning\statements\dml\insert_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::query::metadata::MetadataContext`

#### `src\api\core\query_api.rs`: 1 occurrences

- Line 18: field `vector_coordinator` is never read

#### `src\vector\coordinator.rs`: 1 occurrences

- Line 413: this function has too many arguments (8/7)

