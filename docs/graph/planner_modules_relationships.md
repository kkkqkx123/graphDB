# Planner Module: Architecture and Relationships

## 1. Module划分 Overview

The planner module in GraphDB is structured to mirror NebulaGraph's design while leveraging Rust's module system. The planner is responsible for converting validated query contexts into executable plans that can be processed by the execution engine.

## 2. Module Hierarchy

```
query/
└── planner/
    ├── mod.rs                 # Module declarations and public exports
    ├── planner.rs            # Main Planner trait, registry, and SequentialPlanner
    ├── plan.rs               # Plan node definitions and structures
    ├── match/
    │   ├── mod.rs            # Match planner module declarations
    │   ├── match_planner.rs  # Main match planner
    │   ├── match_clause_planner.rs  # Individual match clause planning
    │   ├── where_clause_planner.rs  # WHERE clause planning
    │   ├── return_clause_planner.rs # RETURN clause planning
    │   ├── with_clause_planner.rs   # WITH clause planning
    │   ├── unwind_clause_planner.rs # UNWIND clause planning
    │   ├── yield_clause_planner.rs  # YIELD clause planning
    │   ├── order_by_clause_planner.rs # ORDER BY clause planning
    │   ├── pagination_planner.rs      # LIMIT/OFFSET planning
    │   ├── match_path_planner.rs      # Path matching planning
    │   ├── shortest_path_planner.rs   # Shortest path planning
    │   ├── argument_finder.rs         # Argument finding for query chaining
    │   ├── start_vid_finder.rs        # Start vertex ID finding
    │   ├── vertex_id_seek.rs          # Vertex ID seeking
    │   ├── label_index_seek.rs        # Label index seeking
    │   ├── prop_index_seek.rs         # Property index seeking
    │   ├── variable_vertex_id_seek.rs # Variable vertex ID seeking
    │   ├── variable_prop_index_seek.rs # Variable property seeking
    │   ├── scan_seek.rs              # Scan operation planning
    │   └── segments_connector.rs     # Segment connection logic
    ├── ngql/
    │   ├── mod.rs            # NGQL planner module declarations
    │   ├── go_planner.rs     # GO statement planner
    │   ├── lookup_planner.rs # LOOKUP statement planner
    │   ├── path_planner.rs   # PATH query planner
    │   ├── subgraph_planner.rs # SUBGRAPH planner
    │   ├── fetch_vertices_planner.rs # FETCH VERTEX planner
    │   ├── fetch_edges_planner.rs    # FETCH EDGE planner
    │   └── maintain_planner.rs       # Maintenance operation planner
    └── plan/
        ├── mod.rs            # Plan structure module declarations
        ├── plan_node.rs      # PlanNode trait and implementations
        ├── execution_plan.rs # ExecutionPlan structure
        ├── plan_node_visitor.rs # PlanNodeVisitor trait and implementations
        ├── query_nodes.rs    # Query operation plan nodes
        ├── logic_nodes.rs    # Logic control plan nodes
        ├── admin_nodes.rs    # Admin operation plan nodes
        ├── algo_nodes.rs     # Algorithm operation plan nodes
        ├── mutate_nodes.rs   # Mutation operation plan nodes
        ├── maintain_nodes.rs # Maintenance operation plan nodes
        └── scan_nodes.rs     # Scan operation plan nodes
```

## 3. Module Relationships

### 3.1 Core Dependencies

```
    planner.rs (main planner trait)
           │
           ▼
    plan.rs (PlanNode definitions)
           │
           ├─ query_nodes.rs
           ├─ logic_nodes.rs
           ├─ admin_nodes.rs
           ├─ algo_nodes.rs
           ├─ mutate_nodes.rs
           ├─ maintain_nodes.rs
           └─ scan_nodes.rs
           │
           ▼
    execution_plan.rs
           │
           ▼
    plan_node_visitor.rs
    
    match/ (MATCH query planning)
           │
           ├─ match_planner.rs
           ├─ clause planners (where, return, with, etc.)
           ├─ path planners (match_path, shortest_path)
           └─ index seekers (vertex_id_seek, label_index_seek, etc.)
           │
           ▼
    segments_connector.rs
    
    ngql/ (NGQL query planning)
           │
           ├─ go_planner.rs
           ├─ lookup_planner.rs
           ├─ path_planner.rs
           └─ ... other planners
```

### 3.2 Data Flow

1. **Query Entry**: `SequentialPlanner` in `planner.rs` receives validated AST context
2. **Planner Selection**: Registry selects appropriate planner based on query type
3. **Plan Generation**: Specific planner (e.g., `MatchPlanner`) creates execution plan
4. **Node Creation**: Planner creates series of `PlanNode` instances
5. **Plan Assembly**: Plan nodes are assembled into execution plan
6. **Optimization**: Plan may be optimized before execution
7. **Execution**: Execution engine processes the plan

## 4. Public API Design

### 4.1 Planner Module Interface

```rust
// In query/planner/mod.rs
pub use planner::{Planner, SequentialPlanner, PlannerRegistry, PlannerError};
pub use plan::{PlanNode, PlanNodeKind, ExecutionPlan, SubPlan, PlanNodeVisitor};
```

### 4.2 Core Planner Trait

The main `Planner` trait in `planner.rs` defines the interface:

```rust
pub trait Planner: std::fmt::Debug {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError>;
    fn match_planner(&self, ast_ctx: &AstContext) -> bool;
}
```

### 4.3 Plan Node Trait

The `PlanNode` trait in `plan.rs` defines the execution plan interface:

```rust
pub trait PlanNode: std::fmt::Debug {
    fn id(&self) -> i64;
    fn kind(&self) -> PlanNodeKind;
    fn dependencies(&self) -> &Vec<Box<dyn PlanNode>>;
    fn output_var(&self) -> &Option<Variable>;
    fn col_names(&self) -> &Vec<String>;
    fn cost(&self) -> f64;
    fn clone_plan_node(&self) -> Box<dyn PlanNode>;
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError>;
}
```

## 5. Internal Module Relationships

### 5.1 Match Module Dependencies

The `match/` module has complex internal relationships:

```rust
match_planner.rs 
    ├── uses ──┐
    │         │
    ▼         │
match_clause_planner.rs 
    │         │
    ├── uses ─┤
    ▼         │
where_clause_planner.rs, return_clause_planner.rs, etc.
    │         │
    ├── uses ─┤
    ▼         │
segments_connector.rs ◄──┤
    │                   │
    └── uses ─── index seekers (vertex_id_seek.rs, label_index_seek.rs, etc.)
```

### 5.2 Plan Node Sub-modules

The `plan/` module has a structured relationship:

```rust
plan_node.rs (base trait)
    │
    ├── query_nodes.rs (query operations)
    ├── logic_nodes.rs (control flow)
    ├── admin_nodes.rs (admin operations)
    ├── algo_nodes.rs (algorithm operations)
    ├── mutate_nodes.rs (mutation operations)
    ├── maintain_nodes.rs (maintenance operations)
    ├── scan_nodes.rs (scan operations)
    │
    ▼
execution_plan.rs (assembles plan nodes)
    │
    ▼
plan_node_visitor.rs (traverses plan nodes)
```

## 6. Cross-Module Dependencies

### 6.1 Data Sharing Between Modules

1. **Core Context**: All planner modules depend on `AstContext` from `core` module
2. **Variable Management**: Planner modules use `Variable` type from `query` module
3. **Error Handling**: All modules use common error types from `planner.rs`

### 6.2 Integration Points

1. **Validator Integration**: Planner receives validated `AstContext` from validator module
2. **Optimizer Integration**: Generated plans may be passed to optimizer module
3. **Executor Integration**: Final execution plans are passed to executor module

## 7. Module Access Patterns

### 7.1 External Access

- `query/mod.rs` exports the planner module functionality
- `services/` and `api/` modules call the planner to generate execution plans
- `optimizer/` module receives plans from the planner for optimization

### 7.2 Internal Access

- `planner.rs` coordinates access to all other planner modules
- Each query-specific module (match/, ngql/) handles its own internal coordination
- `plan/` module is accessed by all planner implementations to create plan nodes

## 8. Module Lifecycle

1. **Initialization**: `SequentialPlanner::register_planners()` registers all planner types
2. **Selection**: `PlannerRegistry::create_plan()` selects appropriate planner
3. **Execution**: Selected planner generates execution plan from `AstContext`
4. **Cleanup**: Plan is passed to execution engine; planner resources can be freed

This architecture ensures clear separation of concerns while maintaining the flexibility to handle complex query planning scenarios, mirroring NebulaGraph's proven design while leveraging Rust's module system for better organization and maintainability.