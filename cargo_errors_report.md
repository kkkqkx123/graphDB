# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 3
- **Total Issues**: 3
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 3
- **Files with Issues**: 1

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 3

### Warning Type Breakdown

- **warning**: 3 warnings

### Files with Warnings (Top 10)

- `src\query\planner\statements\match_statement_planner.rs`: 3 warnings

## Detailed Warning Categorization

### warning: methods `convert_pattern_to_path_pattern`, `convert_path_element_to_node`, `convert_path_element_to_edge`, `extract_properties_from_expr`, `expr_to_value`, and `select_scan_strategy` are never used

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\query\planner\statements\match_statement_planner.rs`: 3 occurrences

- Line 75: methods `convert_pattern_to_path_pattern`, `convert_path_element_to_node`, `convert_path_element_to_edge`, `extract_properties_from_expr`, `expr_to_value`, and `select_scan_strategy` are never used
- Line 220: enum `ScanStrategy` is never used
- Line 231: struct `DummyStorage` is never constructed

