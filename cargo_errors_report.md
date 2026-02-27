# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 3
- **Total Issues**: 3
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 2
- **Files with Issues**: 3

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 3

### Warning Type Breakdown

- **warning**: 3 warnings

### Files with Warnings (Top 10)

- `src\query\validator\schema_validator.rs`: 1 warnings
- `src\api\server\session\network_session.rs`: 1 warnings
- `src\query\validator\update_validator.rs`: 1 warnings

## Detailed Warning Categorization

### warning: struct `MockSchemaManager` is never constructed

**Total Occurrences**: 3  
**Unique Files**: 3

#### `src\query\validator\update_validator.rs`: 1 occurrences

- Line 708: struct `MockSchemaManager` is never constructed

#### `src\query\validator\schema_validator.rs`: 1 occurrences

- Line 588: struct `MockSchemaManager` is never constructed

#### `src\api\server\session\network_session.rs`: 1 occurrences

- Line 378: comparison is useless due to type limits

