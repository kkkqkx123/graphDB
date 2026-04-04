# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 94
- **Total Issues**: 94
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 53
- **Files with Issues**: 28

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 94

### Warning Type Breakdown

- **warning**: 94 warnings

### Files with Warnings (Top 10)

- `crates\inversearch\src\config\mod.rs`: 18 warnings
- `src\query\planning\planner.rs`: 11 warnings
- `src\search\adapters\bm25_adapter.rs`: 6 warnings
- `crates\inversearch\src\storage\mod.rs`: 6 warnings
- `src\query\parser\ast\mod.rs`: 5 warnings
- `crates\inversearch\src\lib.rs`: 5 warnings
- `src\query\executor\data_access\fulltext_search.rs`: 4 warnings
- `src\query\executor\admin\index\fulltext_index\create_fulltext_index.rs`: 3 warnings
- `src\query\executor\expression\functions\fulltext.rs`: 3 warnings
- `tests\integration_management.rs`: 3 warnings

## Detailed Warning Categorization

### warning: unexpected `cfg` condition value: `store-memory`

**Total Occurrences**: 94  
**Unique Files**: 28

#### `crates\inversearch\src\config\mod.rs`: 18 occurrences

- Line 108: unexpected `cfg` condition value: `store-redis`
- Line 115: unexpected `cfg` condition value: `store-redis`
- Line 125: unexpected `cfg` condition value: `store-file`
- ... 15 more occurrences in this file

#### `src\query\planning\planner.rs`: 11 occurrences

- Line 115: unused imports: `AlterFulltextIndex`, `CreateFulltextIndex`, `DescribeFulltextIndex`, `DropFulltextIndex`, `FulltextMatchCondition`, `LookupFulltext`, `MatchFulltext`, `SearchStatement`, and `ShowFulltextIndex`
- Line 143: unused import: `crate::query::planning::plan::PlanNodeEnum`
- Line 157: variable does not need to be mutable
- ... 8 more occurrences in this file

#### `crates\inversearch\src\storage\mod.rs`: 6 occurrences

- Line 46: unexpected `cfg` condition value: `store-memory`
- Line 49: unexpected `cfg` condition value: `store-file`
- Line 52: unexpected `cfg` condition value: `store-redis`
- ... 3 more occurrences in this file

#### `src\search\adapters\bm25_adapter.rs`: 6 occurrences

- Line 2: unused import: `delete_document`
- Line 3: unused import: `add_document`
- Line 318: unused import: `tokio_test`
- ... 3 more occurrences in this file

#### `crates\inversearch\src\lib.rs`: 5 occurrences

- Line 76: unexpected `cfg` condition value: `store-memory`
- Line 79: unexpected `cfg` condition value: `store-file`
- Line 82: unexpected `cfg` condition value: `store-wal`
- ... 2 more occurrences in this file

#### `src\query\parser\ast\mod.rs`: 5 occurrences

- Line 12: ambiguous glob re-exports: the name `WhereClause` in the type namespace is first re-exported here
- Line 12: ambiguous glob re-exports: the name `YieldClause` in the type namespace is first re-exported here
- Line 12: ambiguous glob re-exports: the name `YieldItem` in the type namespace is first re-exported here
- ... 2 more occurrences in this file

#### `src\query\executor\data_access\fulltext_search.rs`: 4 occurrences

- Line 10: unused import: `FulltextQueryExpr`
- Line 22: fields `statement`, `engine`, and `context` are never read
- Line 60: fields `index_name`, `query`, `engine`, `context`, and `limit` are never read
- ... 1 more occurrences in this file

#### `src\query\executor\admin\index\fulltext_index\create_fulltext_index.rs`: 3 occurrences

- Line 7: unused imports: `DataSet` and `Value`
- Line 16: fields `index_name`, `schema_name`, `fields`, `engine_type`, `options`, and `if_not_exists` are never read
- Line 25: this function has too many arguments (9/7)

#### `src\query\executor\expression\functions\fulltext.rs`: 3 occurrences

- Line 125: casting to the same type is unnecessary (`f64` -> `f64`): help: try: `context.score`
- Line 197: this `if let` can be collapsed into the outer `if let`
- Line 276: this `if let` can be collapsed into the outer `if let`

#### `src\core\types\index.rs`: 3 occurrences

- Line 251: this `impl` can be derived
- Line 270: this `impl` can be derived
- Line 390: field assignment outside of initializer for an instance created with Default::default()

#### `src\sync\queue.rs`: 3 occurrences

- Line 2: unused import: `std::collections::VecDeque`
- Line 43: manual implementation of `ok`: help: replace with: `receiver.try_recv().ok()`
- Line 54: unused implementer of `futures::Future` that must be used

#### `tests\integration_management.rs`: 3 occurrences

- Line 1208: you seem to use `.enumerate()` and immediately discard the index
- Line 1232: you seem to use `.enumerate()` and immediately discard the index
- Line 1256: you seem to use `.enumerate()` and immediately discard the index

#### `src\sync\scheduler.rs`: 3 occurrences

- Line 1: unused import: `BatchConfig`
- Line 3: unused import: `crate::sync::task::SyncTask`
- Line 10: field `queue` is never read

#### `src\query\validator\fulltext_validator.rs`: 2 occurrences

- Line 12: unused imports: `FulltextMatchCondition` and `ShowFulltextIndex`
- Line 109: manual `!RangeInclusive::contains` implementation: help: use: `!(0.0..=1.0).contains(&b)`

#### `src\query\executor\admin\index\fulltext_index\describe_fulltext_index.rs`: 2 occurrences

- Line 6: unused imports: `DataSet` and `Value`
- Line 14: field `index_name` is never read

#### `src\query\executor\data_access\match_fulltext.rs`: 2 occurrences

- Line 6: unused imports: `DataSet` and `Value`
- Line 15: fields `pattern`, `fulltext_condition`, and `yield_clause` are never read

#### `src\query\executor\admin\index\fulltext_index\alter_fulltext_index.rs`: 2 occurrences

- Line 6: unused imports: `DataSet` and `Value`
- Line 15: fields `index_name` and `actions` are never read

#### `src\query\executor\admin\index\fulltext_index\drop_fulltext_index.rs`: 2 occurrences

- Line 6: unused imports: `DataSet` and `Value`
- Line 14: fields `index_name` and `if_exists` are never read

#### `src\query\parser\ast\utils.rs`: 2 occurrences

- Line 3: unused import: `super::pattern::*`
- Line 5: unused import: `super::types::*`

#### `src\sync\recovery.rs`: 1 occurrences

- Line 1: unused import: `SyncState`

#### `src\query\parser\parsing\fulltext_parser.rs`: 1 occurrences

- Line 640: unused import: `super::*`

#### `src\query\executor\expression\functions\mod.rs`: 1 occurrences

- Line 244: unused variable: `f`: help: if this is intentional, prefix it with an underscore: `_f`

#### `src\coordinator\types.rs`: 1 occurrences

- Line 21: this `impl` can be derived

#### `src\query\executor\admin\index\fulltext_index\tests.rs`: 1 occurrences

- Line 4: module has the same name as its containing module

#### `src\query\executor\admin\index\fulltext_index\show_fulltext_index.rs`: 1 occurrences

- Line 6: unused imports: `DataSet` and `Value`

#### `src\query\validator\statements\lookup_validator.rs`: 1 occurrences

- Line 11: unused import: `crate::query::parser::ast::fulltext::YieldItem as FulltextYieldItem`

#### `src\query\validator\statements\remove_validator.rs`: 1 occurrences

- Line 263: function `create_contextual_expr` is never used

#### `src\search\adapters\inversearch_adapter.rs`: 1 occurrences

- Line 177: called `map(..).flatten()` on `Iterator`: help: try replacing `map` with `flat_map` and remove the `.flatten()`: `flat_map(|v| v.values())`

