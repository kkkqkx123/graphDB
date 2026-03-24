# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 142
- **Total Issues**: 142
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 79
- **Files with Issues**: 68

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 142

### Warning Type Breakdown

- **warning**: 142 warnings

### Files with Warnings (Top 10)

- `tests\common\c_api_helpers.rs`: 19 warnings
- `tests\common\data_fixtures.rs`: 10 warnings
- `tests\common\storage_helpers.rs`: 8 warnings
- `tests\common\assertions.rs`: 6 warnings
- `tests\integration_functions.rs`: 5 warnings
- `src\query\planning\statements\paths\match_path_planner.rs`: 4 warnings
- `tests\common\mod.rs`: 4 warnings
- `src\query\executor\expression\functions\builtin\container.rs`: 4 warnings
- `src\query\executor\expression\functions\builtin\graph.rs`: 3 warnings
- `src\query\validator\strategies\helpers\variable_checker.rs`: 3 warnings

## Detailed Warning Categorization

### warning: this assertion is always `true`

**Total Occurrences**: 142  
**Unique Files**: 68

#### `tests\common\c_api_helpers.rs`: 19 occurrences

- Line 12: static `TEST_COUNTER` is never used
- Line 17: struct `CApiTestDatabase` is never constructed
- Line 26: associated items `new`, `handle`, and `path` are never used
- ... 16 more occurrences in this file

#### `tests\common\data_fixtures.rs`: 10 occurrences

- Line 10: function `person_tag` is never used
- Line 18: function `company_tag` is never used
- Line 26: function `create_simple_vertex` is never used
- ... 7 more occurrences in this file

#### `tests\common\storage_helpers.rs`: 8 occurrences

- Line 12: function `create_test_space` is never used
- Line 19: function `create_tag_info` is never used
- Line 29: function `create_edge_type_info` is never used
- ... 5 more occurrences in this file

#### `tests\common\assertions.rs`: 6 occurrences

- Line 6: function `assert_ok` is never used
- Line 11: function `assert_err_with` is never used
- Line 26: function `assert_count` is never used
- ... 3 more occurrences in this file

#### `tests\integration_functions.rs`: 5 occurrences

- Line 609: this call to `clone` can be replaced with `std::slice::from_ref`: help: try: `std::slice::from_ref(&empty_list)`
- Line 617: this call to `clone` can be replaced with `std::slice::from_ref`: help: try: `std::slice::from_ref(&empty_list)`
- Line 625: this call to `clone` can be replaced with `std::slice::from_ref`: help: try: `std::slice::from_ref(&empty_list)`
- ... 2 more occurrences in this file

#### `src\query\executor\expression\functions\builtin\container.rs`: 4 occurrences

- Line 383: this call to `clone` can be replaced with `std::slice::from_ref`: help: try: `std::slice::from_ref(&null_value)`
- Line 389: this call to `clone` can be replaced with `std::slice::from_ref`: help: try: `std::slice::from_ref(&null_value)`
- Line 395: this call to `clone` can be replaced with `std::slice::from_ref`: help: try: `std::slice::from_ref(&null_value)`
- ... 1 more occurrences in this file

#### `src\query\planning\statements\paths\match_path_planner.rs`: 4 occurrences

- Line 75: this function has too many arguments (8/7)
- Line 766: this assertion is always `true`
- Line 788: this assertion is always `true`
- ... 1 more occurrences in this file

#### `tests\common\mod.rs`: 4 occurrences

- Line 20: struct `TestStorage` is never constructed
- Line 27: associated items `new` and `storage` are never used
- Line 52: struct `TestContext` is never constructed
- ... 1 more occurrences in this file

#### `src\query\planning\plan\core\nodes\data_processing\data_processing_node.rs`: 3 occurrences

- Line 168: `Vec<T>` is already on the heap, the boxing is unnecessary: help: try: `Vec<crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum>`
- Line 404: `Vec<T>` is already on the heap, the boxing is unnecessary: help: try: `Vec<crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum>`
- Line 721: `Vec<T>` is already on the heap, the boxing is unnecessary: help: try: `Vec<crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum>`

#### `tests\integration_c_api.rs`: 3 occurrences

- Line 145: unnecessary `unsafe` block: unnecessary `unsafe` block
- Line 304: unnecessary `unsafe` block: unnecessary `unsafe` block
- Line 253: casting raw pointers to the same type and constness is unnecessary (`*mut i8` -> `*mut i8`): help: try: `col_name`

#### `src\query\validator\helpers\variable_checker.rs`: 3 occurrences

- Line 279: this `match` can be collapsed into the outer `match`
- Line 290: this `match` can be collapsed into the outer `match`
- Line 315: this assertion is always `true`

#### `src\query\executor\expression\functions\builtin\graph.rs`: 3 occurrences

- Line 333: this call to `clone` can be replaced with `std::slice::from_ref`: help: try: `std::slice::from_ref(&null_value)`
- Line 339: this call to `clone` can be replaced with `std::slice::from_ref`: help: try: `std::slice::from_ref(&null_value)`
- Line 345: this call to `clone` can be replaced with `std::slice::from_ref`: help: try: `std::slice::from_ref(&null_value)`

#### `src\query\validator\strategies\helpers\variable_checker.rs`: 3 occurrences

- Line 279: this `match` can be collapsed into the outer `match`
- Line 290: this `match` can be collapsed into the outer `match`
- Line 321: this assertion is always `true`

#### `src\storage\operations\rollback.rs`: 2 occurrences

- Line 116: stripping a prefix manually
- Line 126: stripping a prefix manually

#### `src\query\planning\statements\paths\shortest_path_planner.rs`: 2 occurrences

- Line 820: this assertion is always `true`
- Line 830: this assertion is always `true`

#### `src\query\validator\statements\create_validator.rs`: 2 occurrences

- Line 50: large size difference between variants: the entire enum is at least 288 bytes
- Line 487: this function has too many arguments (10/7)

#### `src\query\executor\data_processing\join\base_join.rs`: 2 occurrences

- Line 216: very complex type used. Consider factoring parts into `type` definitions
- Line 241: very complex type used. Consider factoring parts into `type` definitions

#### `src\query\validator\statements\fetch_edges_validator.rs`: 2 occurrences

- Line 226: this `if let` can be collapsed into the outer `if let`
- Line 288: this `if let` can be collapsed into the outer `if let`

#### `src\query\executor\factory\builders\control_flow_builder.rs`: 2 occurrences

- Line 38: very complex type used. Consider factoring parts into `type` definitions
- Line 73: very complex type used. Consider factoring parts into `type` definitions

#### `src\query\executor\data_processing\join\inner_join.rs`: 2 occurrences

- Line 44: this function has too many arguments (8/7)
- Line 355: this function has too many arguments (8/7)

#### `src\query\validator\strategies\clause_strategy.rs`: 2 occurrences

- Line 50: this call to `clone` can be replaced with `std::slice::from_ref`: help: try: `std::slice::from_ref(&col.expression)`
- Line 234: this call to `clone` can be replaced with `std::slice::from_ref`: help: try: `std::slice::from_ref(&col.expression)`

#### `src\api\core\schema_api.rs`: 2 occurrences

- Line 540: this assertion is always `true`
- Line 548: this assertion is always `true`

#### `src\query\executor\data_processing\join\left_join.rs`: 2 occurrences

- Line 30: this function has too many arguments (8/7)
- Line 318: this function has too many arguments (8/7)

#### `src\query\executor\expression\functions\builtin\path.rs`: 2 occurrences

- Line 221: this call to `clone` can be replaced with `std::slice::from_ref`: help: try: `std::slice::from_ref(&null_value)`
- Line 227: this call to `clone` can be replaced with `std::slice::from_ref`: help: try: `std::slice::from_ref(&null_value)`

#### `src\query\parser\parsing\ddl_parser.rs`: 2 occurrences

- Line 595: very complex type used. Consider factoring parts into `type` definitions
- Line 801: very complex type used. Consider factoring parts into `type` definitions

#### `src\query\planning\statements\seeks\index_seek.rs`: 1 occurrences

- Line 96: this assertion is always `true`

#### `src\query\parser\core\error.rs`: 1 occurrences

- Line 237: method `into_iter` can be confused for the standard trait method `std::iter::IntoIterator::into_iter`

#### `tests\integration_graph_traversal.rs`: 1 occurrences

- Line 25: function `get_storage` is never used

#### `src\query\optimizer\strategy\index.rs`: 1 occurrences

- Line 188: this `if let` can be collapsed into the outer `if let`

#### `src\query\planning\rewrite\projection_pushdown\push_project_down_get_edges.rs`: 1 occurrences

- Line 95: this expression creates a reference which is immediately dereferenced by the compiler: help: change this to: `get_edges_node`

#### `tests\integration_rewrite.rs`: 1 occurrences

- Line 11: unused import: `graphdb::query::planning::rewrite::rule::RewriteRule`

#### `src\query\planning\statements\clauses\yield_planner.rs`: 1 occurrences

- Line 146: very complex type used. Consider factoring parts into `type` definitions

#### `src\query\planning\rewrite\projection_pushdown\push_project_down_scan_vertices.rs`: 1 occurrences

- Line 95: this expression creates a reference which is immediately dereferenced by the compiler: help: change this to: `scan_node`

#### `tests\integration_dcl.rs`: 1 occurrences

- Line 521: useless use of `vec!`

#### `src\query\validator\strategies\helpers\expression_checker.rs`: 1 occurrences

- Line 568: this assertion is always `true`

#### `src\query\executor\admin\index\tests.rs`: 1 occurrences

- Line 2: module has the same name as its containing module

#### `src\query\executor\admin\edge\tests.rs`: 1 occurrences

- Line 2: module has the same name as its containing module

#### `tests\integration_query.rs`: 1 occurrences

- Line 227: this assertion is always `true`

#### `src\query\executor\factory\executor_factory.rs`: 1 occurrences

- Line 68: parameter is only used in recursion: help: if this is intentional, prefix it with an underscore: `_loop_layers`

#### `src\query\planning\rewrite\projection_pushdown\push_project_down_scan_edges.rs`: 1 occurrences

- Line 95: this expression creates a reference which is immediately dereferenced by the compiler: help: change this to: `scan_node`

#### `src\query\validator\strategies\expression_strategy_test.rs`: 1 occurrences

- Line 17: this assertion is always `true`

#### `src\query\planning\rewrite\projection_pushdown\push_project_down_get_vertices.rs`: 1 occurrences

- Line 95: this expression creates a reference which is immediately dereferenced by the compiler: help: change this to: `get_vertices_node`

#### `src\query\executor\admin\space\tests.rs`: 1 occurrences

- Line 2: module has the same name as its containing module

#### `src\query\planning\rewrite\projection_pushdown\push_project_down_edge_index_scan.rs`: 1 occurrences

- Line 102: this expression creates a reference which is immediately dereferenced by the compiler: help: change this to: `edge_index_scan_node`

#### `src\core\types\graph_schema.rs`: 1 occurrences

- Line 295: this assertion is always `true`

#### `src\core\stats\manager.rs`: 1 occurrences

- Line 109: very complex type used. Consider factoring parts into `type` definitions

#### `src\query\executor\logic\loops.rs`: 1 occurrences

- Line 469: this function has too many arguments (8/7)

#### `src\query\optimizer\cost\selectivity.rs`: 1 occurrences

- Line 351: this `if let` can be collapsed into the outer `if let`

#### `src\core\types\index.rs`: 1 occurrences

- Line 140: this function has too many arguments (8/7)

#### `src\query\planning\rewrite\projection_pushdown\push_project_down_get_neighbors.rs`: 1 occurrences

- Line 95: this expression creates a reference which is immediately dereferenced by the compiler: help: change this to: `get_neighbors_node`

#### `src\query\planning\statements\seeks\scan_seek.rs`: 1 occurrences

- Line 141: this assertion is always `true`

#### `src\query\planning\statements\seeks\vertex_seek.rs`: 1 occurrences

- Line 139: this assertion is always `true`

#### `src\query\planning\statements\seeks\prop_index_seek.rs`: 1 occurrences

- Line 40: method `from_str` can be confused for the standard trait method `std::str::FromStr::from_str`

#### `src\query\validator\structs\alias_structs.rs`: 1 occurrences

- Line 32: large size difference between variants: the entire enum is at least 752 bytes

#### `src\query\executor\data_processing\join\full_outer_join.rs`: 1 occurrences

- Line 24: this function has too many arguments (8/7)

#### `tests\integration_ddl.rs`: 1 occurrences

- Line 929: useless use of `vec!`

#### `src\transaction\manager.rs`: 1 occurrences

- Line 32: very complex type used. Consider factoring parts into `type` definitions

#### `src\query\executor\data_processing\graph_traversal\algorithms\multi_shortest_path.rs`: 1 occurrences

- Line 80: useless use of `vec!`: help: you can use a slice directly: `&[]`

#### `src\query\parser\ast\stmt.rs`: 1 occurrences

- Line 1097: this `if let` can be collapsed into the outer `match`

#### `src\query\parser\parsing\tests.rs`: 1 occurrences

- Line 5: module has the same name as its containing module

#### `src\query\validator\strategies\expression_operations.rs`: 1 occurrences

- Line 690: this assertion is always `true`

#### `src\query\validator\helpers\expression_checker.rs`: 1 occurrences

- Line 567: this assertion is always `true`

#### `src\query\planning\statements\seeks\seek_strategy_base.rs`: 1 occurrences

- Line 172: this call to `clone` can be replaced with `std::slice::from_ref`: help: try: `std::slice::from_ref(prop)`

#### `src\query\planning\statements\seeks\variable_prop_index_seek.rs`: 1 occurrences

- Line 37: method `from_str` can be confused for the standard trait method `std::str::FromStr::from_str`

#### `src\query\validator\statements\match_validator.rs`: 1 occurrences

- Line 582: writing `&mut Vec` instead of `&mut [_]` involves a new object where a slice will do: help: change this to: `&mut [Path]`

#### `src\query\validator\statements\insert_edges_validator.rs`: 1 occurrences

- Line 283: this `if let` can be collapsed into the outer `if let`

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 2: module has the same name as its containing module

#### `src\query\executor\admin\tag\tests.rs`: 1 occurrences

- Line 2: module has the same name as its containing module

