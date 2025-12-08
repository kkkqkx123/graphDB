# Planner Module: NebulaGraph to GraphDB Mapping

## Overview

This document details the mapping between NebulaGraph's planner module implementation and the GraphDB Rust implementation, focusing specifically on the plan generation components.

## NebulaGraph Planner Architecture

### Core Planner Files
- `Planner.h/cpp` - Base planner interface and registry
- `SequentialPlanner.h/cpp` - Handles sequences of statements
- `PlannersRegister.h/cpp` - Central registry for all planners

### Plan Node Structure
The `src/graph/planner/plan/` directory contains all plan node implementations:

- `PlanNode.h/cpp` - Base plan node class with kind, dependencies, and cost
  - `SingleDependencyNode` - Plan node with one dependency
  - `SingleInputNode` - Plan node that processes single input
  - `BinaryInputNode` - Plan node with two dependencies
  - `VariableDependencyNode` - Plan node with variable dependencies

- `ExecutionPlan.h/cpp` - Top-level execution plan container
- `PlanNodeVisitor.h` - Visitor pattern for plan traversal
- `Admin.h/cpp` - Admin command plan nodes
- `Algo.h/cpp` - Algorithm plan nodes
- `Logic.h/cpp` - Logic control plan nodes (Select, Loop, etc.)
- `Maintain.h/cpp` - Maintenance command plan nodes
- `Mutate.h/cpp` - Mutation command plan nodes
- `Query.h/cpp` - Query operation plan nodes
- `Scan.h` - Scan operation plan nodes

### MATCH-Specific Planners
The `src/graph/planner/match/` directory implements Cypher MATCH query planning:

- `MatchPlanner.h/cpp` - Top-level match planner
- `MatchClausePlanner.h/cpp` - Plans individual match clauses
- `WhereClausePlanner.h/cpp` - Plans WHERE conditions
- `ReturnClausePlanner.h/cpp` - Plans RETURN clauses
- `WithClausePlanner.h/cpp` - Plans WITH clauses
- `OrderByClausePlanner.h/cpp` - Plans ORDER BY clauses
- `UnwindClausePlanner.h/cpp` - Plans UNWIND clauses
- `YieldClausePlanner.h/cpp` - Plans YIELD clauses
- `MatchPathPlanner.h/cpp` - Plans path matching
- `ShortestPathPlanner.h/cpp` - Plans shortest path algorithms
- `PaginationPlanner.h/cpp` - Plans LIMIT/OFFSET clauses

### Index Seek Planners
- `StartVidFinder.h/cpp` - Finds starting vertex IDs
- `VertexIdSeek.h/cpp` - Seeks by vertex ID
- `LabelIndexSeek.h/cpp` - Seeks by label/index
- `PropIndexSeek.h/cpp` - Seeks by property/index
- `VariableVertexIdSeek.h/cpp` - Seeks by variable vertex ID
- `VariablePropIndexSeek.h/cpp` - Seeks by variable property
- `ScanSeek.h/cpp` - Full scan operations
- `ArgumentFinder.h/cpp` - Finds arguments for chaining

### NGQL-Specific Planners
The `src/graph/planner/ngql/` directory handles Nebula-specific queries:

- `GoPlanner.h/cpp` - Plans GO statements
- `LookupPlanner.h/cpp` - Plans LOOKUP statements
- `PathPlanner.h/cpp` - Plans PATH queries
- `SubgraphPlanner.h/cpp` - Plans SUBGRAPH queries
- `FetchVerticesPlanner.h/cpp` - Plans FETCH VERTEX queries
- `FetchEdgesPlanner.h/cpp` - Plans FETCH EDGE queries
- `MaintainPlanner.h/cpp` - Plans maintenance operations

## GraphDB Rust Planner Architecture

### Current Structure
- `planner.rs` - Planner trait and registry
- `plan.rs` - Plan node definitions
- `match_planner.rs` - MATCH query planner
- `go_planner.rs` - GO query planner
- `lookup_planner.rs` - LOOKUP query planner
- `path_planner.rs` - PATH query planner
- `subgraph_planner.rs` - SUBGRAPH planner
- `mod.rs` - Module declarations

## Detailed File Mapping

### 1. Base Plan Node System

| NebulaGraph | GraphDB Rust | Status | Notes |
|-------------|--------------|--------|-------|
| `PlanNode.h/cpp` | `plan.rs` | Partial | Need to implement visitor pattern |
| `ExecutionPlan.h/cpp` | `plan.rs` (ExecutionPlan struct) | Implemented | Basic functionality |
| `PlanNodeVisitor.h` | `plan.rs` (PlanNodeVisitor trait) | Needs Implementation | Visitor pattern for traversal |

### 2. Plan Node Types

| NebulaGraph | GraphDB Rust | Status | Notes |
|-------------|--------------|--------|-------|
| SingleDependencyNode | `SingleDependencyNode` struct in `plan.rs` | Implemented | Basic functionality |
| SingleInputNode | `SingleInputNode` struct in `plan.rs` | Implemented | Basic functionality |
| BinaryInputNode | `BinaryInputNode` struct in `plan.rs` | Implemented | Basic functionality |
| VariableDependencyNode | Not implemented | Planned | Needed for complex dependencies |

### 3. Query Operation Plan Nodes

| NebulaGraph | GraphDB Rust | Status | Notes |
|-------------|--------------|--------|-------|
| `Query.h/cpp` Plan Nodes | `PlanNodeKind` enum in `plan.rs` | Implemented | All basic query nodes defined |
| GetNeighbors | `PlanNodeKind::GetNeighbors` | Implemented | |
| GetVertices | `PlanNodeKind::GetVertices` | Implemented | |
| GetEdges | `PlanNodeKind::GetEdges` | Implemented | |
| Expand | `PlanNodeKind::Expand` | Implemented | |
| ExpandAll | `PlanNodeKind::ExpandAll` | Implemented | |
| Traverse | `PlanNodeKind::Traverse` | Implemented | |
| AppendVertices | `PlanNodeKind::AppendVertices` | Implemented | |
| ShortestPath | `PlanNodeKind::ShortestPath` | Implemented | |
| Filter | `PlanNodeKind::Filter` | Implemented | |
| Project | `PlanNodeKind::Project` | Implemented | |
| Aggregate | `PlanNodeKind::Aggregate` | Implemented | |

### 4. Logic Control Plan Nodes

| NebulaGraph | GraphDB Rust | Status | Notes |
|-------------|--------------|--------|-------|
| `Logic.h/cpp` Plan Nodes | `PlanNodeKind` enum in `plan.rs` | Implemented | |
| Select | `PlanNodeKind::Select` | Implemented | |
| Loop | `PlanNodeKind::Loop` | Implemented | |
| PassThrough | `PlanNodeKind::PassThrough` | Implemented | |
| Start | `PlanNodeKind::Start` | Implemented | |

### 5. Data Processing Plan Nodes

| NebulaGraph | GraphDB Rust | Status | Notes |
|-------------|--------------|--------|-------|
| Union | `PlanNodeKind::Union` | Implemented | |
| Intersect | `PlanNodeKind::Intersect` | Implemented | |
| Minus | `PlanNodeKind::Minus` | Implemented | |
| Sort | `PlanNodeKind::Sort` | Implemented | |
| TopN | `PlanNodeKind::TopN` | Implemented | |
| Limit | `PlanNodeKind::Limit` | Implemented | |
| Dedup | `PlanNodeKind::Dedup` | Implemented | |
| Unwind | `PlanNodeKind::Unwind` | Implemented | |

### 6. Index and Scan Plan Nodes

| NebulaGraph | GraphDB Rust | Status | Notes |
|-------------|--------------|--------|-------|
| `Scan.h` Plan Nodes | `PlanNodeKind` enum in `plan.rs` | Implemented | |
| IndexScan | `PlanNodeKind::IndexScan` | Implemented | |
| ScanVertices | `PlanNodeKind::ScanVertices` | Implemented | |
| ScanEdges | `PlanNodeKind::ScanEdges` | Implemented | |
| FulltextIndexScan | `PlanNodeKind::FulltextIndexScan` | Implemented | |

### 7. MATCH-Specific Components

| NebulaGraph | GraphDB Rust | Status | Notes |
|-------------|--------------|--------|-------|
| `MatchPlanner.h/cpp` | `match_planner.rs` | Implemented | Basic functionality only |
| `MatchClausePlanner.h/cpp` | Not implemented | Planned | Needed for complex matching |
| `WhereClausePlanner.h/cpp` | Not implemented | Planned | Needed for WHERE clause |
| `ReturnClausePlanner.h/cpp` | Not implemented | Planned | Needed for RETURN clause |
| `WithClausePlanner.h/cpp` | Not implemented | Planned | Needed for WITH clause |
| `OrderByClausePlanner.h/cpp` | Not implemented | Planned | Needed for ORDER BY |
| `UnwindClausePlanner.h/cpp` | Not implemented | Planned | Needed for UNWIND |
| `YieldClausePlanner.h/cpp` | Not implemented | Planned | Needed for YIELD |
| `PaginationPlanner.h/cpp` | Not implemented | Planned | Needed for LIMIT/OFFSET |
| `MatchPathPlanner.h/cpp` | Not implemented | Planned | Needed for path matching |
| `ShortestPathPlanner.h/cpp` | Not implemented | Planned | Needed for shortest path |

### 8. Index Seek Components

| NebulaGraph | GraphDB Rust | Status | Notes |
|-------------|--------------|--------|-------|
| `StartVidFinder.h/cpp` | Not implemented | Planned | Needed for start vertex finding |
| `VertexIdSeek.h/cpp` | Not implemented | Planned | Needed for vertex ID seeking |
| `LabelIndexSeek.h/cpp` | Not implemented | Planned | Needed for label index seeking |
| `PropIndexSeek.h/cpp` | Not implemented | Planned | Needed for property index seeking |
| `VariableVertexIdSeek.h/cpp` | Not implemented | Planned | Needed for variable vertex seeking |
| `VariablePropIndexSeek.h/cpp` | Not implemented | Planned | Needed for variable property seeking |
| `ScanSeek.h/cpp` | Not implemented | Planned | Needed for scan operations |
| `ArgumentFinder.h/cpp` | Not implemented | Planned | Needed for argument finding |

### 9. NGQL-Specific Planners

| NebulaGraph | GraphDB Rust | Status | Notes |
|-------------|--------------|--------|-------|
| `GoPlanner.h/cpp` | `go_planner.rs` | Implemented | Basic functionality only |
| `LookupPlanner.h/cpp` | `lookup_planner.rs` | Implemented | Basic functionality only |
| `PathPlanner.h/cpp` | `path_planner.rs` | Implemented | Basic functionality only |
| `SubgraphPlanner.h/cpp` | `subgraph_planner.rs` | Implemented | Basic functionality only |
| `FetchVerticesPlanner.h/cpp` | Not implemented | Planned | Needed for fetch vertices |
| `FetchEdgesPlanner.h/cpp` | Not implemented | Planned | Needed for fetch edges |

### 10. Other Planner Components

| NebulaGraph | GraphDB Rust | Status | Notes |
|-------------|--------------|--------|-------|
| `Planner.h/cpp` | `planner.rs` | Implemented | Planner trait and registry |
| `SequentialPlanner.h/cpp` | `planner.rs` | Implemented | Sequential planning logic |
| `PlannersRegister.h/cpp` | `planner.rs` | Implemented | Planner registration system |

## Implementation Gaps and Recommendations

### 1. Core Architecture Gaps

1. **PlanNodeVisitor Pattern**: Not implemented in Rust version
   - NebulaGraph implements visitor pattern for plan traversal
   - Needed for optimization, serialization, and analysis

2. **Advanced Plan Generation**: Current Rust planners only create basic plans
   - NebulaGraph planners have complex logic for building sophisticated execution plans
   - Rust planners need enhanced functionality to match NebulaGraph

### 2. MATCH Query Support

1. **Clause-Based Planning**: NebulaGraph has separate planners for each MATCH clause
   - Rust version needs implementations for:
     - `MatchClausePlanner` - Planning individual match clauses
     - `WhereClausePlanner` - Planning WHERE conditions
     - `ReturnClausePlanner` - Planning RETURN clauses
     - `WithClausePlanner` - Planning WITH clauses
     - `OrderByClausePlanner` - Planning ORDER BY
     - `UnwindClausePlanner` - Planning UNWIND operations
     - `YieldClausePlanner` - Planning YIELD operations

2. **Path and Algorithm Planning**: 
   - Rust needs `MatchPathPlanner` and `ShortestPathPlanner`

### 3. Index Seek Operations

NebulaGraph has sophisticated index seek strategies:
- Rust needs implementations of all index seek planners to optimize query execution

### 4. Enhanced NGQL Support

- Current Rust planners are basic implementations
- Should be enhanced to match NebulaGraph's sophisticated planning logic

## Implementation Plan for GraphDB Planner

### Phase 1: Foundation (Week 1-2)
1. Implement PlanNodeVisitor trait and pattern
2. Enhance basic planner functionality to match NebulaGraph complexity
3. Add proper error handling throughout planner

### Phase 2: MATCH Query Planning (Week 3-4)
1. Implement clause-specific planners for MATCH queries
2. Add sophisticated plan generation logic
3. Implement path planning and algorithm planning

### Phase 3: Optimization (Week 5-6)
1. Add index seek strategies
2. Implement plan optimization passes
3. Add cost estimation to plan nodes

### Phase 4: Advanced Queries (Week 7-8)
1. Enhance GO, LOOKUP, PATH, SUBGRAPH planners
2. Add complex query support
3. Performance testing and refinement

This mapping ensures GraphDB's planner module follows NebulaGraph's proven design patterns while leveraging Rust's strengths.