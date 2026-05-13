# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 52
- **Total Issues**: 52
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 23
- **Files with Issues**: 19

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 52

### Warning Type Breakdown

- **warning**: 52 warnings

### Files with Warnings (Top 10)

- `src\transaction\rollback.rs`: 14 warnings
- `src\query\planning\statements\dml\create_planner.rs`: 6 warnings
- `src\query\query_pipeline_manager.rs`: 4 warnings
- `src\transaction\index_buffer.rs`: 4 warnings
- `src\query\planning\statements\clauses\order_by_planner.rs`: 3 warnings
- `src\query\planning\statements\clauses\return_clause_planner.rs`: 3 warnings
- `src\query\planning\statements\clauses\pagination_planner.rs`: 3 warnings
- `src\transaction\mod.rs`: 2 warnings
- `src\query\planning\statements\clauses\yield_planner.rs`: 2 warnings
- `src\query\planning\statements\clauses\where_clause_planner.rs`: 2 warnings

## Detailed Warning Categorization

### warning: usage of an `Arc` that is not `Send` and `Sync`

**Total Occurrences**: 52  
**Unique Files**: 19

#### `src\transaction\rollback.rs`: 14 occurrences

- Line 23: trait `OperationLogContext` is never used
- Line 57: trait `UndoLogContext` is never used
- Line 87: struct `UndoLogRollback` is never constructed
- ... 11 more occurrences in this file

#### `src\query\planning\statements\dml\create_planner.rs`: 6 occurrences

- Line 488: usage of an `Arc` that is not `Send` and `Sync`
- Line 509: usage of an `Arc` that is not `Send` and `Sync`
- Line 529: usage of an `Arc` that is not `Send` and `Sync`
- ... 3 more occurrences in this file

#### `src\query\query_pipeline_manager.rs`: 4 occurrences

- Line 211: usage of an `Arc` that is not `Send` and `Sync`
- Line 290: usage of an `Arc` that is not `Send` and `Sync`
- Line 350: usage of an `Arc` that is not `Send` and `Sync`
- ... 1 more occurrences in this file

#### `src\transaction\index_buffer.rs`: 4 occurrences

- Line 7: struct `IndexUpdateBuffer` is never constructed
- Line 15: multiple associated items are never used
- Line 92: struct `BufferStats` is never constructed
- ... 1 more occurrences in this file

#### `src\query\planning\statements\clauses\order_by_planner.rs`: 3 occurrences

- Line 242: usage of an `Arc` that is not `Send` and `Sync`
- Line 286: usage of an `Arc` that is not `Send` and `Sync`
- Line 335: usage of an `Arc` that is not `Send` and `Sync`

#### `src\query\planning\statements\clauses\return_clause_planner.rs`: 3 occurrences

- Line 416: usage of an `Arc` that is not `Send` and `Sync`
- Line 478: usage of an `Arc` that is not `Send` and `Sync`
- Line 537: usage of an `Arc` that is not `Send` and `Sync`

#### `src\query\planning\statements\clauses\pagination_planner.rs`: 3 occurrences

- Line 179: usage of an `Arc` that is not `Send` and `Sync`
- Line 223: usage of an `Arc` that is not `Send` and `Sync`
- Line 265: usage of an `Arc` that is not `Send` and `Sync`

#### `src\transaction\mod.rs`: 2 occurrences

- Line 61: unused import: `index_buffer::IndexUpdateBuffer`
- Line 81: unused imports: `CombinedRollback`, `CreateRemoveEdgeUndoParams`, `CreateRemoveVertexUndoParams`, `CreateUpdateEdgePropUndoParams`, `OperationLogContext`, `RollbackHelper`, `UndoLogContext`, and `UndoLogRollback`

#### `src\query\planning\statements\clauses\yield_planner.rs`: 2 occurrences

- Line 409: usage of an `Arc` that is not `Send` and `Sync`
- Line 460: usage of an `Arc` that is not `Send` and `Sync`

#### `src\query\planning\statements\clauses\where_clause_planner.rs`: 2 occurrences

- Line 156: usage of an `Arc` that is not `Send` and `Sync`
- Line 200: usage of an `Arc` that is not `Send` and `Sync`

#### `src\query\context\query_context.rs`: 1 occurrences

- Line 145: associated function `from_components_with_arena` is never used

#### `src\query\validator\statements\go_validator.rs`: 1 occurrences

- Line 577: usage of an `Arc` that is not `Send` and `Sync`

#### `src\query\validator\statements\insert_edges_validator.rs`: 1 occurrences

- Line 478: usage of an `Arc` that is not `Send` and `Sync`

#### `src\query\validator\clauses\limit_validator.rs`: 1 occurrences

- Line 338: usage of an `Arc` that is not `Send` and `Sync`

#### `src\query\validator\statements\insert_vertices_validator.rs`: 1 occurrences

- Line 496: usage of an `Arc` that is not `Send` and `Sync`

#### `src\query\planning\statements\statement_planner.rs`: 1 occurrences

- Line 136: usage of an `Arc` that is not `Send` and `Sync`

#### `tests\integration_query.rs`: 1 occurrences

- Line 30: usage of an `Arc` that is not `Send` and `Sync`

#### `src\query\planning\statements\dml\insert_planner.rs`: 1 occurrences

- Line 241: usage of an `Arc` that is not `Send` and `Sync`

#### `src\transaction\wal\mod.rs`: 1 occurrences

- Line 70: unused import: `writer::GroupCommitManager`

