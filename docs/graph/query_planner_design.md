# Query Planner Design Document for GraphDB

## 1. Introduction

This document outlines the design and implementation plan for the query planner component in the Rust-based GraphDB, drawing from the NebulaGraph implementation as a reference model. The query planner is responsible for converting parsed queries (represented as AST contexts) into executable execution plans that can be processed by the query executor.

## 2. Architecture Overview

### 2.1 Core Components

The query planner architecture consists of several key components:

1. **PlanNode System** - Represents individual operations in the execution plan
2. **Planner Registry** - Manages and selects appropriate planners for different query types
3. **Specific Planners** - Handle different query patterns (MATCH, GO, LOOKUP, etc.)
4. **Execution Plan** - Coordinates the overall execution flow

### 2.2 Data Flow

1. Parser generates AST
2. Validator creates AstContext
3. Planner chooses appropriate planner based on query type
4. Planner generates execution plan from AstContext
5. Optimizer optimizes the execution plan
6. Executor executes the plan

## 3. PlanNode System Design

### 3.1 PlanNode Trait

```rust
// Plan node kinds that represent different operations in the execution plan
#[derive(Debug, Clone, PartialEq)]
pub enum PlanNodeKind {
    // Query nodes
    GetNeighbors,
    GetVertices,
    GetEdges,
    Expand,
    ExpandAll,
    Traverse,
    AppendVertices,
    ShortestPath,
    IndexScan,
    FulltextIndexScan,
    ScanVertices,
    ScanEdges,

    // Data processing nodes
    Filter,
    Union,
    UnionAllVersionVar,
    Intersect,
    Minus,
    Project,
    Unwind,
    Sort,
    TopN,
    Limit,
    Sample,
    Aggregate,
    Dedup,
    Assign,
    BFSShortest,
    MultiShortestPath,
    AllPaths,
    CartesianProduct,
    Subgraph,
    DataCollect,
    InnerJoin,
    HashLeftJoin,
    HashInnerJoin,
    CrossJoin,
    RollUpApply,
    PatternApply,
    Argument,

    // Control flow nodes
    Select,
    Loop,
    PassThrough,
    Start,

    // Schema-related nodes
    CreateSpace,
    CreateTag,
    CreateEdge,
    DescSpace,
    ShowCreateSpace,
    DescTag,
    DescEdge,
    AlterTag,
    AlterEdge,
    ShowSpaces,
    SwitchSpace,
    ShowTags,
    ShowEdges,
    ShowCreateTag,
    ShowCreateEdge,
    DropSpace,
    ClearSpace,
    DropTag,
    DropEdge,
    AlterSpace,

    // Index-related nodes
    CreateTagIndex,
    CreateEdgeIndex,
    CreateFTIndex,
    DropFTIndex,
    DropTagIndex,
    DropEdgeIndex,
    DescTagIndex,
    DescEdgeIndex,
    ShowCreateTagIndex,
    ShowCreateEdgeIndex,
    ShowTagIndexes,
    ShowEdgeIndexes,
    ShowTagIndexStatus,
    ShowEdgeIndexStatus,
    InsertVertices,
    InsertEdges,
    SubmitJob,
    ShowHosts,

    // User-related nodes
    CreateUser,
    DropUser,
    UpdateUser,
    GrantRole,
    RevokeRole,
    ChangePassword,
    ListUserRoles,
    ListUsers,
    ListRoles,
    DescribeUser,

    // Snapshot nodes
    CreateSnapshot,
    DropSnapshot,
    ShowSnapshots,

    // Update/Delete nodes
    DeleteVertices,
    DeleteEdges,
    UpdateVertex,
    DeleteTags,
    UpdateEdge,

    // Show nodes
    ShowParts,
    ShowCharset,
    ShowCollation,
    ShowStats,
    ShowConfigs,
    SetConfig,
    GetConfig,
    ShowMetaLeader,

    // Zone-related nodes
    ShowZones,
    MergeZone,
    RenameZone,
    DropZone,
    DivideZone,
    AddHosts,
    DropHosts,
    DescribeZone,
    AddHostsIntoZone,

    // Listener-related nodes
    AddListener,
    RemoveListener,
    ShowListener,

    // Service-related nodes
    ShowServiceClients,
    ShowFTIndexes,
    SignInService,
    SignOutService,
    ShowSessions,
    UpdateSession,
    KillSession,

    ShowQueries,
    KillQuery,

    // Placeholder for unknown node type
    Unknown,
}

// Base plan node trait that all plan nodes implement
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
    
    // For plan traversal and optimization
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError>;
}
```

### 3.2 Plan Node Types

1. **SingleDependencyNode** - nodes with one dependency
2. **SingleInputNode** - nodes with one input that processes data
3. **BinaryInputNode** - nodes with two input dependencies
4. **VariableDependencyNode** - nodes with variable number of dependencies

### 3.3 Plan Node Visitors for Optimization and Analysis

```rust
pub trait PlanNodeVisitor {
    fn visit(&mut self, node: &dyn PlanNode) -> Result<(), PlanNodeVisitError>;
    fn visit_get_neighbors(&mut self, node: &dyn PlanNode) -> Result<(), PlanNodeVisitError>;
    fn visit_filter(&mut self, node: &dyn PlanNode) -> Result<(), PlanNodeVisitError>;
    // ... other visit methods for different node types
}

#[derive(Debug, thiserror::Error)]
pub enum PlanNodeVisitError {
    #[error("Node visitation failed: {0}")]
    VisitFailed(String),
    
    #[error("Invalid plan node: {0}")]
    InvalidNode(String),
}
```

## 4. Planner System Design

### 4.1 Base Planner Trait

```rust
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
```

### 4.2 Planner Registry

```rust
use std::collections::HashMap;

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

        // If no specific planner matches, try to find a general planner
        for planners_list in self.planners.values() {
            for planner_info in planners_list {
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

### 4.3 Sequential Planner

```rust
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
        registry.register_planner(
            "MATCH",
            MatchAndInstantiate {
                match_func: MatchPlanner::match_ast_ctx,
                instantiate_func: MatchPlanner::make,
            }
        );

        // Register go planner
        registry.register_planner(
            "GO",
            MatchAndInstantiate {
                match_func: GoPlanner::match_ast_ctx,
                instantiate_func: GoPlanner::make,
            }
        );

        // Register lookup planner
        registry.register_planner(
            "LOOKUP",
            MatchAndInstantiate {
                match_func: LookupPlanner::match_ast_ctx,
                instantiate_func: LookupPlanner::make,
            }
        );

        // Register path planner
        registry.register_planner(
            "PATH",
            MatchAndInstantiate {
                match_func: PathPlanner::match_ast_ctx,
                instantiate_func: PathPlanner::make,
            }
        );

        // Register subgraph planner
        registry.register_planner(
            "SUBGRAPH",
            MatchAndInstantiate {
                match_func: SubgraphPlanner::match_ast_ctx,
                instantiate_func: SubgraphPlanner::make,
            }
        );
    }

    pub fn to_plan(ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let mut seq_planner = Self::new();
        seq_planner.registry.create_plan(ast_ctx)
    }
}
```

## 5. Specific Planners Design

### 5.1 Match Planner

The MatchPlanner handles Cypher MATCH queries and is one of the most complex planners:

```rust
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

    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // Verify this is a match statement
        if !Self::match_ast_ctx(ast_ctx) {
            return Err(PlannerError::InvalidAstContext(
                "AST context is not a match statement".to_string()
            ));
        }

        // Parse the match clauses and build the plan
        let mut query_plan = SubPlan::new(None, None);
        
        // This would involve parsing different clauses like:
        // - Match clauses
        // - Where clauses
        // - Return clauses
        // - With clauses
        // - Order By clauses
        // - Limit clauses
        // And building the execution plan accordingly

        // For now, create a basic plan that gets neighbors
        let start_node = Box::new(SingleDependencyNode::new(PlanNodeKind::Start, vec![]));
        let get_neighbors_node = Box::new(SingleInputNode::new(
            PlanNodeKind::GetNeighbors,
            vec![start_node],
        ));

        query_plan.root = Some(get_neighbors_node.clone());
        query_plan.tail = Some(get_neighbors_node);

        Ok(query_plan)
    }
}
```

### 5.2 Go Planner

Handles the GO statement for traversing the graph:

```rust
#[derive(Debug)]
pub struct GoPlanner {
    // Go-specific configurations
}

impl GoPlanner {
    pub fn new() -> Self {
        Self {}
    }

    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "GO"
    }

    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        if !Self::match_ast_ctx(ast_ctx) {
            return Err(PlannerError::InvalidAstContext(
                "AST context is not a GO statement".to_string()
            ));
        }

        // This would parse the GO query structure:
        // GO FROM $var OVER edge_type WHERE condition YIELD columns
        // And build the corresponding execution plan

        let start_node = Box::new(SingleDependencyNode::new(PlanNodeKind::Start, vec![]));
        let get_neighbors_node = Box::new(SingleInputNode::new(
            PlanNodeKind::GetNeighbors,
            vec![start_node],
        ));

        let sub_plan = SubPlan::new(Some(get_neighbors_node.clone()), Some(get_neighbors_node));

        Ok(sub_plan)
    }
}
```

### 5.3 Other Planners

Similar patterns would apply to LookupPlanner, PathPlanner, and SubgraphPlanner with their specific logic.

## 6. Plan Generation Process

### 6.1 Clause-Based Planning (for MATCH)

For complex MATCH queries, the planner would:

1. Parse different clauses (MATCH, WHERE, RETURN, WITH, etc.)
2. Generate plan for each clause
3. Connect the plans appropriately
4. Handle variable dependencies between clauses

### 6.2 Plan Optimization

An optimization layer would be added to:
1. Apply transformation rules
2. Estimate and minimize costs
3. Optimize join orders
4. Optimize predicate pushdowns

## 7. Implementation Plan

### Phase 1: Foundation (Week 1-2)
1. Implement the PlanNode trait and basic node types
2. Implement the Planner trait and Registry
3. Create basic SequentialPlanner

### Phase 2: Core Planners (Week 3-4)
1. Implement MatchPlanner with basic functionality 
2. Implement GoPlanner with basic functionality
3. Implement LookupPlanner with basic functionality

### Phase 3: Advanced Features (Week 5-6)
1. Add support for complex MATCH clauses
2. Implement PathPlanner
3. Implement SubgraphPlanner
4. Add plan optimization capabilities

### Phase 4: Integration & Testing (Week 7-8)
1. Integrate with query validator and executor
2. Write comprehensive unit tests
3. Performance testing and optimization

## 8. Error Handling

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
}
```

## 9. Execution Plan Structure

```rust
#[derive(Debug)]
pub struct ExecutionPlan {
    pub root: Option<Box<dyn PlanNode>>,
    pub id: i64,
    pub optimize_time_in_us: u64,
    pub format: String, // explain format
}

impl ExecutionPlan {
    pub fn new(root: Option<Box<dyn PlanNode>>) -> Self {
        Self {
            root,
            id: -1, // Will be assigned by ID generator
            optimize_time_in_us: 0,
            format: "row".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct SubPlan {
    pub root: Option<Box<dyn PlanNode>>,
    pub tail: Option<Box<dyn PlanNode>>,
}

impl SubPlan {
    pub fn new(root: Option<Box<dyn PlanNode>>, tail: Option<Box<dyn PlanNode>>) -> Self {
        Self { root, tail }
    }
    
    // Connect this subplan to another subplan
    pub fn connect_to(&mut self, other: &mut SubPlan) -> Result<(), PlannerError> {
        if let Some(ref mut tail_node) = self.tail {
            if let Some(ref root_node) = other.root {
                // Add the root node of 'other' as a dependency to the tail of 'self'
                tail_node.set_dependencies(vec![root_node.clone_plan_node()]);
                Ok(())
            } else {
                Err(PlannerError::PlanGenerationFailed(
                    "Cannot connect: other subplan has no root".to_string()
                ))
            }
        } else {
            Err(PlannerError::PlanGenerationFailed(
                "Cannot connect: self subplan has no tail".to_string()
            ))
        }
    }
}
```

## 10. Conclusion

This design provides a flexible and extensible architecture for the query planner that can handle various types of graph queries while maintaining compatibility with the NebulaGraph concepts. The design emphasizes modularity, allowing for easy addition of new query types and optimization rules.