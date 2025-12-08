# GraphDB Planner Module: Complete Design Specification

## 1. Overview

This document provides a complete design specification for the planner module in GraphDB, a lightweight, single-node Rust implementation inspired by NebulaGraph. The planner is responsible for converting validated query contexts into executable plans that the execution engine can process.

## 2. Architecture Goals

1. **Mirror NebulaGraph Design**: Preserve the proven architecture patterns from NebulaGraph
2. **Leverage Rust Strengths**: Utilize Rust's type system, ownership model, and safety features
3. **Modular Design**: Ensure clear separation of concerns with well-defined interfaces
4. **Extensibility**: Design for easy addition of new query types and optimization rules
5. **Performance**: Optimize for single-node performance with efficient plan generation

## 3. Complete Directory Structure

```
graphDB/
├── src/
│   └── query/
│       └── planner/
│           ├── mod.rs                 # Module declarations and public exports
│           ├── planner.rs            # Main Planner trait, registry, and SequentialPlanner
│           ├── plan/
│           │   ├── mod.rs            # Plan structure module declarations
│           │   ├── plan_node.rs      # PlanNode trait and base implementations
│           │   ├── execution_plan.rs # ExecutionPlan and SubPlan structures
│           │   ├── plan_node_visitor.rs # PlanNodeVisitor trait
│           │   ├── query_nodes.rs    # Query operation plan nodes
│           │   ├── logic_nodes.rs    # Logic control plan nodes
│           │   ├── admin_nodes.rs    # Admin operation plan nodes
│           │   ├── algo_nodes.rs     # Algorithm operation plan nodes
│           │   ├── mutate_nodes.rs   # Mutation operation plan nodes
│           │   ├── maintain_nodes.rs # Maintenance operation plan nodes
│           │   ├── scan_nodes.rs     # Scan operation plan nodes
│           │   └── variable_dependency_node.rs # Variable dependency plan node
│           ├── match/
│           │   ├── mod.rs            # Match planner module declarations
│           │   ├── match_planner.rs  # Main match planner
│           │   ├── match_clause_planner.rs  # Individual match clause planning
│           │   ├── where_clause_planner.rs  # WHERE clause planning
│           │   ├── return_clause_planner.rs # RETURN clause planning
│           │   ├── with_clause_planner.rs   # WITH clause planning
│           │   ├── unwind_clause_planner.rs # UNWIND clause planning
│           │   ├── yield_clause_planner.rs  # YIELD clause planning
│           │   ├── order_by_clause_planner.rs # ORDER BY clause planning
│           │   ├── pagination_planner.rs      # LIMIT/OFFSET planning
│           │   ├── match_path_planner.rs      # Path matching planning
│           │   ├── shortest_path_planner.rs   # Shortest path planning
│           │   ├── argument_finder.rs         # Argument finding for query chaining
│           │   ├── start_vid_finder.rs        # Start vertex ID finding
│           │   ├── vertex_id_seek.rs          # Vertex ID seeking
│           │   ├── label_index_seek.rs        # Label index seeking
│           │   ├── prop_index_seek.rs         # Property index seeking
│           │   ├── variable_vertex_id_seek.rs # Variable vertex ID seeking
│           │   ├── variable_prop_index_seek.rs # Variable property seeking
│           │   ├── scan_seek.rs              # Scan operation planning
│           │   └── segments_connector.rs     # Segment connection logic
│           ├── ngql/
│           │   ├── mod.rs            # NGQL planner module declarations
│           │   ├── go_planner.rs     # GO statement planner
│           │   ├── lookup_planner.rs # LOOKUP statement planner
│           │   ├── path_planner.rs   # PATH query planner
│           │   ├── subgraph_planner.rs # SUBGRAPH planner
│           │   ├── fetch_vertices_planner.rs # FETCH VERTEX planner
│           │   ├── fetch_edges_planner.rs    # FETCH EDGE planner
│           │   └── maintain_planner.rs       # Maintenance operation planner
│           └── error.rs              # Planner-specific error types
├── docs/
│   └── graph/
│       ├── query_planner_design.md   # High-level planner design
│       ├── planner_mapping.md        # NebulaGraph to GraphDB mapping
│       ├── planner_modules_relationships.md # Module划分 and relationships
│       └── planner_implementation_guide.md  # Implementation guide
```

## 4. Core Module Specifications

### 4.1 Planner Trait (`planner.rs`)

```rust
use crate::core::AstContext;
use super::plan::{SubPlan, PlanNodeKind, ExecutionPlan};
use std::collections::HashMap;

// Match function type - takes AstContext and returns whether the planner matches
pub type MatchFunc = fn(&AstContext) -> bool;

// Planner instantiation function type - returns a new planner instance
pub type PlannerInstantiateFunc = fn() -> Box<dyn Planner>;

// Structure that combines match function and planner instantiate function
#[derive(Debug, Clone)]
pub struct MatchAndInstantiate {
    pub match_func: MatchFunc,
    pub instantiate_func: PlannerInstantiateFunc,
}

// Main planner trait that all planners implement
pub trait Planner: std::fmt::Debug {
    // The main transformation function that converts an AST context to an execution plan
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError>;

    // Match function for this planner - whether it can handle the given AST context
    fn match_planner(&self, ast_ctx: &AstContext) -> bool;
}

// Sequential planner that handles sequential execution of statements
#[derive(Debug)]
pub struct SequentialPlanner {
    registry: PlannerRegistry,
}

impl SequentialPlanner {
    pub fn new() -> Self {
        let mut registry = PlannerRegistry::new();
        Self::register_planners(&mut registry);
        
        Self { registry }
    }

    pub fn register_planners(registry: &mut PlannerRegistry) {
        // Register match planner
        registry.register_planner("MATCH", MatchPlanner::get_match_and_instantiate());
        
        // Register other planners
        registry.register_planner("GO", GoPlanner::get_match_and_instantiate());
        registry.register_planner("LOOKUP", LookupPlanner::get_match_and_instantiate());
        registry.register_planner("PATH", PathPlanner::get_match_and_instantiate());
        registry.register_planner("SUBGRAPH", SubgraphPlanner::get_match_and_instantiate());
    }

    pub fn to_plan(ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let seq_planner = Self::new();
        seq_planner.registry.create_plan(ast_ctx)
    }
}

// Planner registry that keeps track of all available planners
#[derive(Debug)]
pub struct PlannerRegistry {
    planners: HashMap<String, Vec<MatchAndInstantiate>>,
}

impl PlannerRegistry {
    pub fn new() -> Self {
        Self {
            planners: HashMap::new(),
        }
    }

    pub fn register_planner(&mut self, stmt_type: &str, match_and_instantiate: MatchAndInstantiate) {
        self.planners.entry(stmt_type.to_string()).or_default().push(match_and_instantiate);
    }

    pub fn create_plan(&self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let stmt_type = &ast_ctx.statement_type().to_uppercase();
        
        if let Some(candidates) = self.planners.get(stmt_type) {
            for planner_info in candidates {
                if (planner_info.match_func)(ast_ctx) {
                    let mut planner = (planner_info.instantiate_func)();
                    if planner.match_planner(ast_ctx) {
                        return planner.transform(ast_ctx);
                    }
                }
            }
        }

        Err(PlannerError::NoSuitablePlanner(
            format!("No suitable planner found for statement type: {}", stmt_type)
        ))
    }
}
```

### 4.2 Plan Structure (`plan/` module)

The `plan/` module contains the plan node hierarchy:

```rust
// Plan node kinds that represent different operations in the execution plan
#[derive(Debug, Clone, PartialEq)]
pub enum PlanNodeKind {
    // All the plan node types as defined previously
    GetNeighbors,
    GetVertices,
    GetEdges,
    Expand,
    // ... all other node kinds
}

// Base plan node trait
pub trait PlanNode: std::fmt::Debug {
    fn id(&self) -> i64;
    fn kind(&self) -> PlanNodeKind;
    fn dependencies(&self) -> &Vec<Box<dyn PlanNode>>;
    fn set_dependencies(&mut self, deps: Vec<Box<dyn PlanNode>>);
    fn output_var(&self) -> &Option<Variable>;
    fn set_output_var(&mut self, var: Variable);
    fn col_names(&self) -> &Vec<String>;
    fn set_col_names(&mut self, names: Vec<String>);
    fn cost(&self) -> f64;
    fn set_cost(&mut self, cost: f64);
    fn clone_plan_node(&self) -> Box<dyn PlanNode>;
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError>;
}

// Base implementations for different plan node types
pub struct SingleDependencyNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub dependencies: Vec<Box<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

pub struct SingleInputNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub dependencies: Vec<Box<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

pub struct BinaryInputNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub dependencies: Vec<Box<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

pub struct VariableDependencyNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub dependencies: Vec<Box<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

// Execution plan structure
#[derive(Debug)]
pub struct ExecutionPlan {
    pub root: Option<Box<dyn PlanNode>>,
    pub id: i64,
    pub optimize_time_in_us: u64,
    pub format: String,
}

#[derive(Debug)]
pub struct SubPlan {
    pub root: Option<Box<dyn PlanNode>>,
    pub tail: Option<Box<dyn PlanNode>>,
}
```

### 4.3 Match Planner Module (`match/`)

The `match/` module handles Cypher MATCH queries:

```rust
use crate::core::AstContext;
use super::{Planner, PlannerError};
use super::plan::{SubPlan, PlanNodeKind, ExecutionPlan, SingleInputNode, PlanNodeVisitor};
use std::collections::HashMap;

#[derive(Debug)]
pub struct MatchPlanner {
    tail_connected: bool,
}

impl MatchPlanner {
    pub fn new() -> Self {
        Self {
            tail_connected: false,
        }
    }

    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "MATCH"
    }
    
    pub fn get_match_and_instantiate() -> super::MatchAndInstantiate {
        super::MatchAndInstantiate {
            match_func: Self::match_ast_ctx,
            instantiate_func: Self::make,
        }
    }

    // Complex method to generate plan for MATCH queries
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // Implementation that uses:
        // - MatchClausePlanner for match clauses
        // - WhereClausePlanner for WHERE conditions
        // - ReturnClausePlanner for RETURN clauses
        // - SegmentsConnector to connect different plan segments
        // - Various index seekers based on query patterns
        unimplemented!("Complex MATCH query planning implementation")
    }
}
```

### 4.4 NGQL Planner Module (`ngql/`)

The `ngql/` module handles Nebula-specific queries:

```rust
use crate::core::AstContext;
use super::{Planner, PlannerError};
use super::plan::{SubPlan, PlanNodeKind, ExecutionPlan, SingleInputNode};

pub struct GoPlanner;

impl GoPlanner {
    pub fn new() -> Self {
        Self
    }

    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "GO"
    }
    
    pub fn get_match_and_instantiate() -> super::MatchAndInstantiate {
        super::MatchAndInstantiate {
            match_func: Self::match_ast_ctx,
            instantiate_func: Self::make,
        }
    }

    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // Implementation for GO queries
        unimplemented!("GO query planning implementation")
    }
}

pub struct LookupPlanner;

impl LookupPlanner {
    pub fn new() -> Self {
        Self
    }

    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "LOOKUP"
    }
    
    pub fn get_match_and_instantiate() -> super::MatchAndInstantiate {
        super::MatchAndInstantiate {
            match_func: Self::match_ast_ctx,
            instantiate_func: Self::make,
        }
    }

    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // Implementation for LOOKUP queries
        unimplemented!("LOOKUP query planning implementation")
    }
}
```

## 5. Data Flow and Control Flow

### 5.1 Query Processing Pipeline

```
1. Parser
   │
   ▼
2. Validator (creates AstContext)
   │
   ▼
3. PlannerRegistry::create_plan(AstContext)
   │
   ├─ SequentialPlanner::to_plan()
   │  │
   │  ├─ Determine query type from AstContext
   │  │
   │  └─ Select appropriate planner
   │     │
   │     ▼
4. Specific Planner::transform()
   │  │
   │  ├─ Generate plan using domain-specific logic
   │  │
   │  └─ Create plan nodes
   │     │
   │     ▼
5. ExecutionPlan/SubPlan (ready for execution)
   │
   ▼
6. Optimizer (optional)
   │
   ▼
7. Executor
```

### 5.2 Match Query Flow

```
AstContext with MATCH query
   │
   ▼
MatchPlanner::transform()
   │
   ├─ Parse MATCH clauses
   │  │
   │  └─ MatchClausePlanner::plan()
   │
   ├─ Parse WHERE clause
   │  │
   │  └─ WhereClausePlanner::plan()
   │
   ├─ Parse RETURN clause
   │  │
   │  └─ ReturnClausePlanner::plan()
   │
   └─ Connect segments using SegmentsConnector
      │
      ▼
ExecutionPlan with connected plan nodes
```

## 6. Error Handling Strategy

```rust
#[derive(Debug, thiserror::Error)]
pub enum PlannerError {
    #[error("No suitable planner found: {0}")]
    NoSuitablePlanner(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Plan generation failed: {0}")]
    PlanGenerationFailed(String),

    #[error("Invalid AST context: {0}")]
    InvalidAstContext(String),

    #[error("Plan validation failed: {0}")]
    PlanValidationError(String),

    #[error("Index seek failed: {0}")]
    IndexSeekFailed(String),

    #[error("Path planning failed: {0}")]
    PathPlanningFailed(String),
}
```

## 7. Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)
1. Complete basic plan node implementations
2. Implement Planner trait and registry
3. Create basic execution plan structures

### Phase 2: Core Planners (Weeks 3-4)
1. Implement GoPlanner and LookupPlanner with full functionality
2. Implement PathPlanner and SubgraphPlanner
3. Basic MatchPlanner implementation

### Phase 3: MATCH Query Support (Weeks 5-6)
1. Implement clause-specific planners (Where, Return, With, etc.)
2. Add sophisticated plan generation for MATCH queries
3. Implement index seek strategies

### Phase 4: Optimizations (Weeks 7-8)
1. Add PlanNodeVisitor pattern
2. Implement plan optimization capabilities
3. Performance testing and refinement

## 8. Design Principles Maintained

1. **Preserve NebulaGraph's Architecture**: All major design concepts from NebulaGraph are preserved
2. **Rust Idiomatic Design**: Use of traits, enums, and ownership model
3. **Modular Organization**: Clear separation between different query types and plan operations
4. **Extensibility**: Easy to add new query types or plan node types
5. **Type Safety**: Leverage Rust's type system for correctness
6. **Performance**: Optimized for single-node execution with minimal overhead

This design provides a complete architecture for the GraphDB planner module that follows NebulaGraph's proven patterns while leveraging Rust's strengths for safety, performance, and maintainability.