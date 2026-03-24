# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 38
- **Total Issues**: 38
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 13
- **Files with Issues**: 27

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 38

### Warning Type Breakdown

- **warning**: 38 warnings

### Files with Warnings (Top 10)

- `src\query\planning\plan\core\nodes\data_processing\data_processing_node.rs`: 3 warnings
- `src\query\validator\statements\create_validator.rs`: 2 warnings
- `src\query\validator\strategies\helpers\variable_checker.rs`: 2 warnings
- `src\query\validator\statements\fetch_edges_validator.rs`: 2 warnings
- `src\query\executor\data_processing\join\left_join.rs`: 2 warnings
- `src\query\executor\data_processing\join\base_join.rs`: 2 warnings
- `src\query\validator\helpers\variable_checker.rs`: 2 warnings
- `src\query\parser\parsing\ddl_parser.rs`: 2 warnings
- `src\query\executor\data_processing\join\inner_join.rs`: 2 warnings
- `src\query\executor\factory\builders\control_flow_builder.rs`: 2 warnings

## Detailed Warning Categorization

### warning: large size difference between variants: the entire enum is at least 752 bytes

**Total Occurrences**: 38  
**Unique Files**: 27

#### `src\query\planning\plan\core\nodes\data_processing\data_processing_node.rs`: 3 occurrences

- Line 168: `Vec<T>` is already on the heap, the boxing is unnecessary: help: try: `Vec<crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum>`
- Line 404: `Vec<T>` is already on the heap, the boxing is unnecessary: help: try: `Vec<crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum>`
- Line 721: `Vec<T>` is already on the heap, the boxing is unnecessary: help: try: `Vec<crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum>`

#### `src\query\validator\statements\fetch_edges_validator.rs`: 2 occurrences

- Line 226: this `if let` can be collapsed into the outer `if let`
- Line 288: this `if let` can be collapsed into the outer `if let`

#### `src\query\validator\strategies\helpers\variable_checker.rs`: 2 occurrences

- Line 279: this `match` can be collapsed into the outer `match`
- Line 290: this `match` can be collapsed into the outer `match`

#### `src\query\executor\data_processing\join\left_join.rs`: 2 occurrences

- Line 30: this function has too many arguments (8/7)
- Line 318: this function has too many arguments (8/7)

#### `src\query\validator\statements\create_validator.rs`: 2 occurrences

- Line 50: large size difference between variants: the entire enum is at least 288 bytes
- Line 487: this function has too many arguments (10/7)

#### `src\query\validator\helpers\variable_checker.rs`: 2 occurrences

- Line 279: this `match` can be collapsed into the outer `match`
- Line 290: this `match` can be collapsed into the outer `match`

#### `src\query\executor\data_processing\join\base_join.rs`: 2 occurrences

- Line 216: very complex type used. Consider factoring parts into `type` definitions
- Line 241: very complex type used. Consider factoring parts into `type` definitions

#### `src\query\executor\factory\builders\control_flow_builder.rs`: 2 occurrences

- Line 38: very complex type used. Consider factoring parts into `type` definitions
- Line 73: very complex type used. Consider factoring parts into `type` definitions

#### `src\query\executor\data_processing\join\inner_join.rs`: 2 occurrences

- Line 44: this function has too many arguments (8/7)
- Line 355: this function has too many arguments (8/7)

#### `src\query\parser\parsing\ddl_parser.rs`: 2 occurrences

- Line 595: very complex type used. Consider factoring parts into `type` definitions
- Line 801: very complex type used. Consider factoring parts into `type` definitions

#### `src\query\validator\structs\alias_structs.rs`: 1 occurrences

- Line 32: large size difference between variants: the entire enum is at least 752 bytes

#### `src\query\validator\statements\match_validator.rs`: 1 occurrences

- Line 582: writing `&mut Vec` instead of `&mut [_]` involves a new object where a slice will do: help: change this to: `&mut [Path]`

#### `src\transaction\manager.rs`: 1 occurrences

- Line 32: very complex type used. Consider factoring parts into `type` definitions

#### `src\query\planning\statements\paths\match_path_planner.rs`: 1 occurrences

- Line 75: this function has too many arguments (8/7)

#### `src\query\optimizer\cost\selectivity.rs`: 1 occurrences

- Line 351: this `if let` can be collapsed into the outer `if let`

#### `src\query\validator\statements\insert_edges_validator.rs`: 1 occurrences

- Line 283: this `if let` can be collapsed into the outer `if let`

#### `src\query\planning\statements\clauses\yield_planner.rs`: 1 occurrences

- Line 146: very complex type used. Consider factoring parts into `type` definitions

#### `src\query\executor\logic\loops.rs`: 1 occurrences

- Line 469: this function has too many arguments (8/7)

#### `src\query\executor\factory\executor_factory.rs`: 1 occurrences

- Line 68: parameter is only used in recursion: help: if this is intentional, prefix it with an underscore: `_loop_layers`

#### `src\core\stats\manager.rs`: 1 occurrences

- Line 109: very complex type used. Consider factoring parts into `type` definitions

#### `src\core\types\index.rs`: 1 occurrences

- Line 140: this function has too many arguments (8/7)

#### `src\query\executor\data_processing\join\full_outer_join.rs`: 1 occurrences

- Line 24: this function has too many arguments (8/7)

#### `src\query\optimizer\strategy\index.rs`: 1 occurrences

- Line 188: this `if let` can be collapsed into the outer `if let`

#### `src\query\parser\core\error.rs`: 1 occurrences

- Line 237: method `into_iter` can be confused for the standard trait method `std::iter::IntoIterator::into_iter`

#### `src\query\parser\ast\stmt.rs`: 1 occurrences

- Line 1097: this `if let` can be collapsed into the outer `match`

#### `src\query\planning\statements\seeks\prop_index_seek.rs`: 1 occurrences

- Line 40: method `from_str` can be confused for the standard trait method `std::str::FromStr::from_str`

#### `src\query\planning\statements\seeks\variable_prop_index_seek.rs`: 1 occurrences

- Line 37: method `from_str` can be confused for the standard trait method `std::str::FromStr::from_str`

