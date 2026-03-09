# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 4
- **Total Issues**: 4
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 4
- **Files with Issues**: 2

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 4

### Warning Type Breakdown

- **warning**: 4 warnings

### Files with Warnings (Top 10)

- `src\query\executor\factory\executor_factory.rs`: 2 warnings
- `src\query\executor\data_processing\materialize.rs`: 2 warnings

## Detailed Warning Categorization

### warning: unreachable pattern: no value can reach this

**Total Occurrences**: 4  
**Unique Files**: 2

#### `src\query\executor\factory\executor_factory.rs`: 2 occurrences

- Line 389: unreachable pattern: no value can reach this
- Line 28: field `safety_validator` is never read

#### `src\query\executor\data_processing\materialize.rs`: 2 occurrences

- Line 235: unused import: `super::*`
- Line 236: unused import: `crate::storage::StorageClient`

