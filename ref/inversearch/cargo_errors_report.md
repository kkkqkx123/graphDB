# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 1
- **Total Warnings**: 147
- **Total Issues**: 148
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 63
- **Files with Issues**: 45

## Error Statistics

**Total Errors**: 1

### Error Type Breakdown

- **error**: 1 errors

### Files with Errors (Top 10)

- `src\compress\lcg.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 147

### Warning Type Breakdown

- **warning**: 147 warnings

### Files with Warnings (Top 10)

- `src\keystore\mod.rs`: 19 warnings
- `src\resolver\enrich.rs`: 12 warnings
- `src\storage\mod.rs`: 9 warnings
- `src\search\coordinator.rs`: 8 warnings
- `src\index\builder.rs`: 6 warnings
- `src\service.rs`: 6 warnings
- `src\storage\redis.rs`: 6 warnings
- `src\serialize\index.rs`: 5 warnings
- `src\resolver\resolver.rs`: 5 warnings
- `src\intersect\suggestion.rs`: 5 warnings

## Detailed Error Categorization

### error: this comparison involving the minimum or maximum element for this type contains a case that is always true or always false

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\compress\lcg.rs`: 1 occurrences

- Line 48: this comparison involving the minimum or maximum element for this type contains a case that is always true or always false

## Detailed Warning Categorization

### warning: this `impl` can be derived

**Total Occurrences**: 147  
**Unique Files**: 44

#### `src\keystore\mod.rs`: 19 occurrences

- Line 45: use of `or_insert_with` to construct default value: help: try: `or_default()`
- Line 56: use of `or_insert_with` to construct default value: help: try: `or_default()`
- Line 67: this `map_or` can be simplified
- ... 16 more occurrences in this file

#### `src\resolver\enrich.rs`: 12 occurrences

- Line 36: very complex type used. Consider factoring parts into `type` definitions
- Line 37: very complex type used. Consider factoring parts into `type` definitions
- Line 97: you should consider adding a `Default` implementation for `HighlightConfig`
- ... 9 more occurrences in this file

#### `src\storage\mod.rs`: 9 occurrences

- Line 198: you seem to want to iterate on a map's values
- Line 205: you seem to want to iterate on a map's values
- Line 208: use of `or_insert_with` to construct default value: help: try: `or_default()`
- ... 6 more occurrences in this file

#### `src\search\coordinator.rs`: 8 occurrences

- Line 58: this `impl` can be derived
- Line 69: very complex type used. Consider factoring parts into `type` definitions
- Line 213: this `impl` can be derived
- ... 5 more occurrences in this file

#### `src\index\builder.rs`: 6 occurrences

- Line 16: this `if` statement can be collapsed
- Line 133: this function has too many arguments (11/7)
- Line 175: this function has too many arguments (10/7)
- ... 3 more occurrences in this file

#### `src\storage\redis.rs`: 6 occurrences

- Line 47: use of deprecated method `redis::Client::get_async_connection`: aio::Connection is deprecated. Use client::get_multiplexed_async_connection instead.
- Line 97: you seem to want to iterate on a map's values
- Line 106: you seem to want to iterate on a map's values
- ... 3 more occurrences in this file

#### `src\service.rs`: 6 occurrences

- Line 18: unused imports: `DocumentConfig`, `Document`, and `SearchResult`
- Line 31: field `storage` is never read
- Line 36: you should consider adding a `Default` implementation for `InversearchService`
- ... 3 more occurrences in this file

#### `src\intersect\suggestion.rs`: 5 occurrences

- Line 35: this `impl` can be derived
- Line 65: field assignment outside of initializer for an instance created with Default::default()
- Line 136: you seem to use `.enumerate()` and immediately discard the index
- ... 2 more occurrences in this file

#### `src\resolver\resolver.rs`: 5 occurrences

- Line 40: this `impl` can be derived
- Line 141: this `impl` can be derived
- Line 175: this pattern creates a reference to a reference: help: try: `query`
- ... 2 more occurrences in this file

#### `src\serialize\index.rs`: 5 occurrences

- Line 55: you seem to want to iterate on a map's values
- Line 68: you seem to want to iterate on a map's values
- Line 85: you seem to want to iterate on a map's values
- ... 2 more occurrences in this file

#### `src\serialize\chunked.rs`: 5 occurrences

- Line 69: you seem to want to iterate on a map's keys
- Line 76: manually reimplementing `div_ceil`: help: consider using `.div_ceil()`: `items.len().div_ceil(chunk_size)`
- Line 105: manually reimplementing `div_ceil`: help: consider using `.div_ceil()`: `items.len().div_ceil(chunk_size)`
- ... 2 more occurrences in this file

#### `src\document\tree.rs`: 4 occurrences

- Line 66: this `impl` can be derived
- Line 158: stripping a prefix manually
- Line 192: stripping a prefix manually
- ... 1 more occurrences in this file

#### `src\index\mod.rs`: 4 occurrences

- Line 169: use of `or_insert_with` to construct default value: help: try: `or_default()`
- Line 167: use of `or_insert_with` to construct default value: help: try: `or_default()`
- Line 194: use of `or_insert_with` to construct default value: help: try: `or_default()`
- ... 1 more occurrences in this file

#### `src\config\mod.rs`: 3 occurrences

- Line 123: this `impl` can be derived
- Line 197: used `assert_eq!` with a literal bool
- Line 204: used `assert_eq!` with a literal bool

#### `src\charset\latin\mod.rs`: 3 occurrences

- Line 70: manual case-insensitive ASCII comparison
- Line 70: manual case-insensitive ASCII comparison
- Line 225: items after a test module

#### `src\resolver\mod.rs`: 3 occurrences

- Line 50: variable does not need to be mutable
- Line 17: module has the same name as its containing module
- Line 79: field assignment outside of initializer for an instance created with Default::default()

#### `src\encoder\mod.rs`: 3 occurrences

- Line 638: found call to `str::trim` before `str::split_whitespace`: help: remove `trim()`
- Line 716: field assignment outside of initializer for an instance created with Default::default()
- Line 755: field assignment outside of initializer for an instance created with Default::default()

#### `src\storage\wal.rs`: 3 occurrences

- Line 505: variable does not need to be mutable
- Line 196: manual implementation of `.is_multiple_of()`: help: replace with: `self.change_count.load(Ordering::Relaxed).is_multiple_of(self.config.snapshot_interval)`
- Line 369: unnecessary `if let` since only the `Ok` variant of the iterator element is used

#### `src\document\tag.rs`: 3 occurrences

- Line 27: very complex type used. Consider factoring parts into `type` definitions
- Line 58: you should consider adding a `Default` implementation for `TagSystem`
- Line 74: very complex type used. Consider factoring parts into `type` definitions

#### `src\highlight\core.rs`: 2 occurrences

- Line 147: casting to the same type is unnecessary (`i32` -> `i32`): help: try: `boundary_before`
- Line 153: casting to the same type is unnecessary (`i32` -> `i32`): help: try: `boundary_after`

#### `src\resolver\async_resolver.rs`: 2 occurrences

- Line 84: method `borrow` can be confused for the standard trait method `std::borrow::Borrow::borrow`
- Line 88: method `borrow_mut` can be confused for the standard trait method `std::borrow::BorrowMut::borrow_mut`

#### `src\resolver\handler.rs`: 2 occurrences

- Line 14: unused variable: `suggest`: help: if this is intentional, prefix it with an underscore: `_suggest`
- Line 399: field assignment outside of initializer for an instance created with Default::default()

#### `src\resolver\or.rs`: 2 occurrences

- Line 3: unused variable: `boost`: help: if this is intentional, prefix it with an underscore: `_boost`
- Line 20: usage of `contains_key` followed by `insert` on a `HashMap`

#### `src\index\remover.rs`: 2 occurrences

- Line 106: using `clone` on type `usize` which implements the `Copy` trait: help: try dereferencing it: `*term_hash`
- Line 124: using `clone` on type `usize` which implements the `Copy` trait: help: try dereferencing it: `*term_hash`

#### `src\search\multi_field.rs`: 2 occurrences

- Line 99: this expression creates a reference which is immediately dereferenced by the compiler: help: change this to: `field.index()`
- Line 109: unnecessary map of the identity function: help: remove the call to `map`

#### `src\intersect\scoring.rs`: 2 occurrences

- Line 159: you should consider adding a `Default` implementation for `ScoreManager`
- Line 209: you seem to use `.enumerate()` and immediately discard the index

#### `src\serialize\document.rs`: 2 occurrences

- Line 65: you seem to want to iterate on a map's values
- Line 133: casting to the same type is unnecessary (`u64` -> `u64`): help: try: `data.registry.next_doc_id`

#### `src\document\batch.rs`: 2 occurrences

- Line 277: the following explicit lifetimes could be elided: 'a
- Line 308: the following explicit lifetimes could be elided: 'a

#### `src\intersect\core.rs`: 2 occurrences

- Line 73: usage of `contains_key` followed by `insert` on a `HashMap`
- Line 180: usage of `contains_key` followed by `insert` on a `HashMap`

#### `src\search\cache.rs`: 1 occurrences

- Line 236: field assignment outside of initializer for an instance created with Default::default()

#### `src\compress\cache.rs`: 1 occurrences

- Line 32: struct `CompressCache` has a public `len` method, but no `is_empty` method

#### `src\main.rs`: 1 occurrences

- Line 49: this assertion is always `true`

#### `src\tokenizer\mod.rs`: 1 occurrences

- Line 107: the variable `position` is used as a loop counter: help: consider using: `for (position, token) in tokens.iter().enumerate()`

#### `src\resolver\not.rs`: 1 occurrences

- Line 11: this `if` statement can be collapsed

#### `src\highlight\tests.rs`: 1 occurrences

- Line 22: used `assert_eq!` with a literal bool

#### `src\serialize\format.rs`: 1 occurrences

- Line 28: redundant slicing of the whole range: help: use the original value instead: `bytes`

#### `src\resolver\xor.rs`: 1 occurrences

- Line 3: unused variable: `boost`: help: if this is intentional, prefix it with an underscore: `_boost`

#### `src\document\mod.rs`: 1 occurrences

- Line 342: very complex type used. Consider factoring parts into `type` definitions

#### `src\search\single_term.rs`: 1 occurrences

- Line 19: this function has too many arguments (8/7)

#### `src\highlight\types.rs`: 1 occurrences

- Line 113: you should consider adding a `Default` implementation for `EncoderCache`

#### `src\encoder\validator.rs`: 1 occurrences

- Line 132: field assignment outside of initializer for an instance created with Default::default()

#### `src\document\field.rs`: 1 occurrences

- Line 28: very complex type used. Consider factoring parts into `type` definitions

#### `src\serialize\async.rs`: 1 occurrences

- Line 63: this `if` has identical blocks

#### `src\async_.rs`: 1 occurrences

- Line 358: field assignment outside of initializer for an instance created with Default::default()

