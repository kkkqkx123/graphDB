# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 2
- **Total Issues**: 2
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 2
- **Files with Issues**: 2

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 2

### Warning Type Breakdown

- **warning**: 2 warnings

### Files with Warnings (Top 10)

- `src\query\parser\lexer\lexer.rs`: 1 warnings
- `src\query\parser\parser\stmt_parser.rs`: 1 warnings

## Detailed Warning Categorization

### warning: unused import: `crate::core::types::expression::Expression`

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\query\parser\parser\stmt_parser.rs`: 1 occurrences

- Line 8: unused import: `crate::core::types::expression::Expression`

#### `src\query\parser\lexer\lexer.rs`: 1 occurrences

- Line 829: method `is_multitoken_keyword` is never used

