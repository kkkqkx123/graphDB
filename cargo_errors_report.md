# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 35
- **Total Issues**: 35
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 15
- **Files with Issues**: 18

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 35

### Warning Type Breakdown

- **warning**: 35 warnings

### Files with Warnings (Top 10)

- `src\query\executor\result_processing\transformations\rollup_apply.rs`: 5 warnings
- `src\query\validator\statements\lookup_validator.rs`: 5 warnings
- `src\query\executor\result_processing\transformations\pattern_apply.rs`: 4 warnings
- `src\query\executor\data_access\search.rs`: 3 warnings
- `src\query\executor\data_modification\delete.rs`: 2 warnings
- `src\query\planning\statements\dml\merge_planner.rs`: 2 warnings
- `src\query\validator\statements\match_validator.rs`: 2 warnings
- `tests\common\test_scenario.rs`: 2 warnings
- `src\query\planning\statements\dml\delete_planner.rs`: 1 warnings
- `src\query\executor\result_processing\transformations\append_vertices.rs`: 1 warnings

## Detailed Warning Categorization

### warning: associated items `get_schema_name`, `get_entity_property_for_filter`, `value_in_range`, and `values_equal` are never used

**Total Occurrences**: 35  
**Unique Files**: 18

#### `src\query\executor\result_processing\transformations\rollup_apply.rs`: 5 occurrences

- Line 558: variable does not need to be mutable
- Line 600: variable does not need to be mutable
- Line 660: variable does not need to be mutable
- ... 2 more occurrences in this file

#### `src\query\validator\statements\lookup_validator.rs`: 5 occurrences

- Line 224: unneeded `return` statement
- Line 230: unneeded `return` statement
- Line 233: unneeded `return` statement
- ... 2 more occurrences in this file

#### `src\query\executor\result_processing\transformations\pattern_apply.rs`: 4 occurrences

- Line 361: variable does not need to be mutable
- Line 393: variable does not need to be mutable
- Line 426: variable does not need to be mutable
- ... 1 more occurrences in this file

#### `src\query\executor\data_access\search.rs`: 3 occurrences

- Line 94: associated items `get_schema_name`, `get_entity_property_for_filter`, `value_in_range`, and `values_equal` are never used
- Line 211: `to_string` applied to a type that implements `Display` in `format!` args: help: remove this
- Line 212: `to_string` applied to a type that implements `Display` in `format!` args: help: remove this

#### `src\query\executor\data_modification\delete.rs`: 2 occurrences

- Line 237: redundant closure: help: replace the closure with the tuple variant itself: `crate::core::error::DBError::Storage`
- Line 242: redundant closure: help: replace the closure with the tuple variant itself: `crate::core::error::DBError::Storage`

#### `tests\common\test_scenario.rs`: 2 occurrences

- Line 522: variable does not need to be mutable
- Line 546: variable does not need to be mutable

#### `src\query\validator\statements\match_validator.rs`: 2 occurrences

- Line 253: unused import: `crate::query::parser::ast::PathElement`
- Line 281: this call to `clone` can be replaced with `std::slice::from_ref`: help: try: `std::slice::from_ref(pattern)`

#### `src\query\planning\statements\dml\merge_planner.rs`: 2 occurrences

- Line 106: this `if let` can be collapsed into the outer `if let`
- Line 121: this `if let` can be collapsed into the outer `if let`

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 367: variable does not need to be mutable

#### `src\query\executor\data_processing\graph_traversal\expand_all.rs`: 1 occurrences

- Line 5: unused import: `crate::core::value::list::List`

#### `src\query\planning\statements\dml\delete_planner.rs`: 1 occurrences

- Line 5: unused import: `crate::core::types::ContextualExpression`

#### `src\query\executor\data_processing\graph_traversal\all_paths.rs`: 1 occurrences

- Line 484: unused variable: `vertex`: help: if this is intentional, prefix it with an underscore: `_vertex`

#### `src\query\executor\factory\builders\data_modification_builder.rs`: 1 occurrences

- Line 12: unused imports: `EdgeUpdateInfo` and `VertexUpdateInfo`

#### `src\query\planning\statements\match_statement_planner.rs`: 1 occurrences

- Line 1021: this function has too many arguments (8/7)

#### `src\query\executor\result_processing\transformations\unwind.rs`: 1 occurrences

- Line 382: variable does not need to be mutable

#### `src\query\planning\plan\core\nodes\join\join_node.rs`: 1 occurrences

- Line 121: unused import: `crate::query::planning::plan::core::nodes::base::plan_node_traits::PlanNode`

#### `src\query\planning\statements\dql\lookup_planner.rs`: 1 occurrences

- Line 275: you seem to be trying to use `match` for destructuring a single pattern. Consider using `if let`

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 533: variable does not need to be mutable

