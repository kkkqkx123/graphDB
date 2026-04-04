# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 74
- **Total Issues**: 74
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 36
- **Files with Issues**: 21

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 74

### Warning Type Breakdown

- **warning**: 74 warnings

### Files with Warnings (Top 10)

- `crates\inversearch\src\config\mod.rs`: 18 warnings
- `src\query\planning\planner.rs`: 11 warnings
- `crates\inversearch\src\storage\mod.rs`: 6 warnings
- `crates\inversearch\src\lib.rs`: 5 warnings
- `src\query\parser\ast\mod.rs`: 5 warnings
- `src\search\adapters\bm25_adapter.rs`: 4 warnings
- `src\sync\scheduler.rs`: 3 warnings
- `src\query\executor\admin\index\fulltext_index\create_fulltext_index.rs`: 2 warnings
- `src\query\executor\admin\index\fulltext_index\alter_fulltext_index.rs`: 2 warnings
- `src\query\executor\admin\index\fulltext_index\describe_fulltext_index.rs`: 2 warnings

## Detailed Warning Categorization

### warning: unexpected `cfg` condition value: `store-memory`

**Total Occurrences**: 74  
**Unique Files**: 21

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

#### `crates\inversearch\src\lib.rs`: 5 occurrences

- Line 76: unexpected `cfg` condition value: `store-memory`
- Line 79: unexpected `cfg` condition value: `store-file`
- Line 82: unexpected `cfg` condition value: `store-wal`
- ... 2 more occurrences in this file

#### `src\query\parser\ast\mod.rs`: 5 occurrences

- Line 12: ambiguous glob re-exports
- Line 12: ambiguous glob re-exports
- Line 12: ambiguous glob re-exports
- ... 2 more occurrences in this file

#### `src\search\adapters\bm25_adapter.rs`: 4 occurrences

- Line 3: unused import: `add_document`
- Line 4: unused import: `delete_document`
- Line 149: unused variable: `batch_count`
- ... 1 more occurrences in this file

#### `src\sync\scheduler.rs`: 3 occurrences

- Line 4: unused import: `BatchConfig`
- Line 6: unused import: `crate::sync::task::SyncTask`
- Line 10: field `queue` is never read

#### `src\query\executor\admin\index\fulltext_index\alter_fulltext_index.rs`: 2 occurrences

- Line 6: unused imports: `DataSet` and `Value`
- Line 15: fields `index_name` and `actions` are never read

#### `src\query\executor\data_access\fulltext_search.rs`: 2 occurrences

- Line 20: fields `statement`, `engine`, and `context` are never read
- Line 54: fields `index_name`, `query`, `engine`, `context`, and `limit` are never read

#### `src\query\parser\ast\utils.rs`: 2 occurrences

- Line 3: unused import: `super::pattern::*`
- Line 5: unused import: `super::types::*`

#### `src\query\executor\admin\index\fulltext_index\drop_fulltext_index.rs`: 2 occurrences

- Line 6: unused imports: `DataSet` and `Value`
- Line 14: fields `index_name` and `if_exists` are never read

#### `src\query\executor\admin\index\fulltext_index\create_fulltext_index.rs`: 2 occurrences

- Line 7: unused imports: `DataSet` and `Value`
- Line 16: fields `index_name`, `schema_name`, `fields`, `engine_type`, `options`, and `if_not_exists` are never read

#### `src\query\executor\admin\index\fulltext_index\describe_fulltext_index.rs`: 2 occurrences

- Line 6: unused imports: `DataSet` and `Value`
- Line 14: field `index_name` is never read

#### `src\query\executor\data_access\match_fulltext.rs`: 2 occurrences

- Line 6: unused imports: `DataSet` and `Value`
- Line 15: fields `pattern`, `fulltext_condition`, and `yield_clause` are never read

#### `src\sync\queue.rs`: 2 occurrences

- Line 2: unused import: `std::collections::VecDeque`
- Line 56: unused implementer of `futures::Future` that must be used

#### `build.rs`: 1 occurrences

- Line 6: unused import: `std::path::PathBuf`

#### `src\query\executor\admin\index\fulltext_index\show_fulltext_index.rs`: 1 occurrences

- Line 6: unused imports: `DataSet` and `Value`

#### `src\query\validator\statements\lookup_validator.rs`: 1 occurrences

- Line 13: unused import: `crate::query::parser::ast::fulltext::YieldItem as FulltextYieldItem`

#### `src\query\validator\fulltext_validator.rs`: 1 occurrences

- Line 12: unused imports: `FulltextMatchCondition` and `ShowFulltextIndex`

#### `src\sync\recovery.rs`: 1 occurrences

- Line 2: unused import: `SyncState`

#### `src\query\executor\expression\functions\mod.rs`: 1 occurrences

- Line 244: unused variable: `f`

