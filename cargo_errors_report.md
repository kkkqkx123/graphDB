# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 3
- **Total Issues**: 3
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 2
- **Files with Issues**: 1

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 3

### Warning Type Breakdown

- **warning**: 3 warnings

### Files with Warnings (Top 10)

- `src\query\executor\data_access\fulltext_search.rs`: 3 warnings

## Detailed Warning Categorization

### warning: redundant closure: help: replace the closure with the tuple variant itself: `DBError::Storage`

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\query\executor\data_access\fulltext_search.rs`: 3 occurrences

- Line 364: redundant closure: help: replace the closure with the tuple variant itself: `DBError::Storage`
- Line 445: this expression can be written more simply using `.retain()`: help: consider calling `.retain()` instead: `rows.retain(|row| self.evaluate_where_condition(row, &where_clause.condition))`
- Line 555: redundant closure: help: replace the closure with the tuple variant itself: `DBError::Storage`

