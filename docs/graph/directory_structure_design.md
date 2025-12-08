# GraphDB Directory Structure and Module Design

## Overview

This document outlines the proposed directory structure and module划分 for the GraphDB project, designed to be a lightweight, single-node Rust implementation of NebulaGraph concepts.

## Current NebulaGraph Architecture

### NebulaGraph Directory Structure
```
nebula-3.8.0/src/
├── clients/           # Client libraries
├── codec/             # Serialization/deserialization
├── common/            # Shared utilities
├── console/           # Console client
├── daemons/           # Service daemons
├── graph/             # Core graph query processing
│   ├── context/       # Query context management
│   ├── executor/      # Query execution
│   ├── gc/            # Garbage collection
│   ├── optimizer/     # Query optimization
│   ├── planner/       # Query planning
│   ├── scheduler/     # Query scheduling
│   ├── service/       # Service interface
│   ├── session/       # Session management
│   ├── stats/         # Statistics
│   ├── util/          # Utilities
│   ├── validator/     # Query validation
│   └── visitor/       # AST/Plan visitors
├── interface/         # Interface definitions
├── kvstore/           # Key-value store
├── meta/              # Metadata management
├── mock/              # Mock implementations
├── parser/            # Query parsing
├── storage/           # Storage engine
├── tools/             # Utility tools
├── version/           # Version management
└── webservice/        # Web service interfaces
```

## Proposed GraphDB Directory Structure

### GraphDB Directory Structure
```
graphDB/
├── Cargo.toml         # Project configuration
├── Cargo.lock         # Dependency lock file
├── config.toml        # Configuration file
├── README.md          # Documentation
├── docs/              # Documentation
│   ├── architecture/  # Architecture documentation
│   │   ├── overview.md
│   │   ├── storage.md
│   │   ├── query.md
│   │   └── network.md
│   ├── graph/         # Graph-specific documentation
│   │   ├── query_planner_design.md
│   │   ├── execution_model.md
│   │   └── data_model.md
│   └── development/   # Development guidelines
│       └── contributing.md
├── src/               # Source code
│   ├── lib.rs         # Library entry point
│   ├── main.rs        # Executable entry point
│   ├── api/           # API interfaces (REST/gRPC)
│   ├── common/        # Common utilities and types
│   ├── config/        # Configuration management
│   ├── core/          # Core data structures and types
│   │   ├── ast_context.rs     # AST context management
│   │   ├── execution_context.rs # Execution context
│   │   ├── query_context.rs   # Query context
│   │   ├── schema.rs          # Schema management
│   │   ├── symbols.rs         # Symbol management
│   │   ├── validate_context.rs # Validation context
│   │   ├── value.rs           # Value types
│   │   ├── vertex_edge_path.rs # Graph primitives
│   │   └── mod.rs             # Module declarations
│   ├── graph/         # Graph-specific operations
│   │   ├── algorithms/ # Graph algorithms
│   │   │   ├── shortest_path.rs
│   │   │   ├── connected_components.rs
│   │   │   └── mod.rs
│   │   ├── operations.rs      # Basic graph operations
│   │   └── mod.rs             # Module declarations
│   ├── query/         # Query processing layer
│   │   ├── executor/   # Query execution engine
│   │   │   ├── admin/     # Admin operation executors
│   │   │   ├── algo/      # Algorithm executors
│   │   │   ├── logic/     # Logical operation executors
│   │   │   ├── maintain/  # Maintenance executors
│   │   │   ├── mutate/    # Mutation executors
│   │   │   ├── query/     # Query operation executors
│   │   │   └── mod.rs     # Executor module declarations
│   │   ├── optimizer/  # Query optimization
│   │   │   ├── rule/      # Optimization rules
│   │   │   │   ├── predicate_pushdown.rs
│   │   │   │   ├── join_order.rs
│   │   │   │   └── mod.rs
│   │   │   ├── patterns.rs  # Pattern matching for optimization
│   │   │   └── mod.rs       # Optimizer module declarations
│   │   ├── planner/    # Query planning engine
│   │   │   ├── match/     # MATCH-specific planners
│   │   │   │   ├── match_planner.rs
│   │   │   │   ├── match_clause_planner.rs
│   │   │   │   ├── where_clause_planner.rs
│   │   │   │   ├── return_clause_planner.rs
│   │   │   │   └── mod.rs
│   │   │   ├── ngql/      # NGQL-specific planners
│   │   │   │   ├── go_planner.rs
│   │   │   │   ├── lookup_planner.rs
│   │   │   │   ├── path_planner.rs
│   │   │   │   ├── subgraph_planner.rs
│   │   │   │   └── mod.rs
│   │   │   ├── plan/      # Plan node definitions
│   │   │   │   ├── execution_plan.rs
│   │   │   │   ├── plan_node.rs
│   │   │   │   ├── visitor.rs
│   │   │   │   └── mod.rs
│   │   │   ├── planner.rs # Planner trait and registry
│   │   │   └── mod.rs     # Planner module declarations
│   │   ├── scheduler/  # Query scheduling
│   │   ├── validator/  # Query validation
│   │   │   ├── admin_validator.rs
│   │   │   ├── assignment_validator.rs
│   │   │   ├── explain_validator.rs
│   │   │   ├── fetch_vertices_validator.rs
│   │   │   ├── fetch_edges_validator.rs
│   │   │   ├── find_path_validator.rs
│   │   │   ├── get_subgraph_validator.rs
│   │   │   ├── go_validator.rs
│   │   │   ├── group_by_validator.rs
│   │   │   ├── limit_validator.rs
│   │   │   ├── lookup_validator.rs
│   │   │   ├── match_validator.rs
│   │   │   ├── mutate_validator.rs
│   │   │   ├── order_by_validator.rs
│   │   │   ├── pipe_validator.rs
│   │   │   ├── set_validator.rs
│   │   │   ├── unwind_validator.rs
│   │   │   ├── use_validator.rs
│   │   │   ├── yield_validator.rs
│   │   │   └── mod.rs
│   │   ├── visitor/    # AST/Plan visitors
│   │   │   ├── expression_visitor.rs
│   │   │   ├── statement_visitor.rs
│   │   │   └── mod.rs
│   │   └── mod.rs      # Query module declarations
│   ├── services/       # Service layer
│   ├── stats/          # Statistics and metrics
│   ├── storage/        # Storage engine
│   │   ├── native_storage.rs    # Native storage implementation
│   │   ├── storage_engine.rs    # Storage engine interface
│   │   ├── storage_error.rs     # Storage errors
│   │   ├── index/      # Index implementations
│   │   │   ├── vertex_index.rs
│   │   │   ├── edge_index.rs
│   │   │   └── mod.rs
│   │   ├── transaction/  # Transaction management
│   │   │   ├── transaction.rs
│   │   │   ├── transaction_manager.rs
│   │   │   └── mod.rs
│   │   └── mod.rs      # Storage module declarations
│   ├── utils/          # Utility functions
│   │   ├── error.rs    # Error handling utilities
│   │   ├── result.rs   # Result type utilities
│   │   └── mod.rs
│   └── mod.rs          # Root module declarations
├── tests/              # Integration tests
├── benches/            # Benchmark tests
└── target/             # Build artifacts (gitignored)
```

## Module划分 and Relationships

### Core Module Dependencies
```
    ┌─────────────┐
    │   API       │
    └──────┬──────┘
           │
    ┌──────▼──────┐
    │  SERVICES   │
    └──────┬──────┘
           │
    ┌──────▼──────┐
    │   QUERY     │
    └─────┬─┬─┬───┘
          │ │ │
    ┌─────▼─▼─▼───┐
    │ VALIDATOR   │
    └──────┬──────┘
           │
    ┌──────▼──────┐
    │  PLANNER    │
    └──────┬──────┘
           │
    ┌──────▼──────┐
    │  OPTIMIZER  │
    └──────┬──────┘
           │
    ┌──────▼──────┐
    │  EXECUTOR   │
    └──────┬──────┘
           │
    ┌──────▼──────┐
    │  STORAGE    │
    └─────────────┘

    ┌─────────────┐
    │    CORE     │ ← Independent base types
    └─────────────┘

    ┌─────────────┐
    │   GRAPH     │ ← Graph algorithms
    └─────────────┘
```

### Detailed Module Breakdown

#### 1. Core Module
- **Purpose**: Contains fundamental data structures and types used throughout the system
- **Files**: 
  - `ast_context.rs`: AST context management
  - `execution_context.rs`: Execution context information
  - `query_context.rs`: Query-specific context
  - `schema.rs`: Schema definitions
  - `symbols.rs`: Symbol table management
  - `validate_context.rs`: Validation context
  - `value.rs`: Value types (Node, Edge, Path, etc.)
  - `vertex_edge_path.rs`: Graph primitives

#### 2. Graph Module
- **Purpose**: Contains graph-specific operations and algorithms
- **Submodules**:
  - `algorithms/`: Graph algorithms (shortest path, connected components)
  - `operations.rs`: Basic graph operations (add vertex, add edge, etc.)

#### 3. Storage Module
- **Purpose**: Handles data persistence and retrieval
- **Submodules**:
  - `index/`: Index implementations (vertex, edge, property indices)
  - `transaction/`: Transaction management system
  - Core files: native storage, storage engine interface, error handling

#### 4. Query Module (Main Processing Layer)
- **Purpose**: Main query processing pipeline
- **Submodules**:
  - `validator/`: Query validation logic
  - `planner/`: Query plan generation
  - `optimizer/`: Query optimization
  - `executor/`: Query execution
  - `scheduler/`: Query scheduling
  - `visitor/`: AST/Plan traversal utilities

#### 5. API Module
- **Purpose**: External interfaces (REST, gRPC)
- **Files**: 
  - `server.rs`: Main API server
  - `handlers/`: Request handlers
  - `responses.rs`: API response structures

## Mapping NebulaGraph Components to GraphDB

The following table maps NebulaGraph C++ components to GraphDB Rust components:

| NebulaGraph Component | GraphDB Equivalent | Notes |
|----------------------|-------------------|-------|
| `src/graph/validator/*` | `src/query/validator/` | Query validation in Rust |
| `src/graph/planner/*` | `src/query/planner/` | Query planning engine |
| `src/graph/planner/plan/*` | `src/query/planner/plan/` | Plan node definitions |
| `src/graph/planner/match/*` | `src/query/planner/match/` | MATCH-specific planning |
| `src/graph/planner/ngql/*` | `src/query/planner/ngql/` | NGQL-specific planning |
| `src/graph/optimizer/*` | `src/query/optimizer/` | Query optimization |
| `src/graph/executor/*` | `src/query/executor/` | Query execution |
| `src/graph/context/*` | `src/core/` | Context management |
| `src/storage/*` | `src/storage/` | Storage engine |
| `src/common/*` | `src/common/` and `src/utils/` | Common utilities |
| `src/graph/service/*` | `src/api/` and `src/services/` | API and service layer |

## Implementation Strategy

### Phase 1: Core Infrastructure (Current State)
- Core data structures and types
- Basic storage implementation
- Simple planner structure

### Phase 2: Query Processing Pipeline
- Complete validator implementation
- Planner with all query types
- Basic executor
- Optimizer

### Phase 3: Advanced Features
- Advanced graph algorithms
- Optimized storage engine
- Performance enhancements
- Full API

## Design Principles

1. **Rust Idiomatic Code**: Use Rust traits, ownership model, and concurrency patterns
2. **Modular Architecture**: Clear separation of concerns with well-defined interfaces
3. **Type Safety**: Leverage Rust's type system for safety and correctness
4. **Performance**: Optimize for single-node performance
5. **Memory Safety**: Use Rust's ownership to prevent memory issues
6. **Extensibility**: Design for easy addition of new query types and features