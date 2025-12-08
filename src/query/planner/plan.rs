//! Plan node definitions for execution plans
use crate::query::validator::Variable;

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
    fn output_var(&self) -> &Option<Variable>;
    fn col_names(&self) -> &Vec<String>;
    fn cost(&self) -> f64;
    fn clone_plan_node(&self) -> Box<dyn PlanNode>;
}

// Single dependency node - a plan node with one dependency
#[derive(Debug)]
pub struct SingleDependencyNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub dependencies: Vec<Box<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

impl SingleDependencyNode {
    pub fn new(kind: PlanNodeKind, dep: Box<dyn PlanNode>) -> Self {
        Self {
            id: -1, // Will be assigned later
            kind,
            dependencies: vec![dep],
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }
}

impl PlanNode for SingleDependencyNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }

    fn dependencies(&self) -> &Vec<Box<dyn PlanNode>> {
        &self.dependencies
    }

    fn output_var(&self) -> &Option<Variable> {
        &self.output_var
    }

    fn col_names(&self) -> &Vec<String> {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn clone_plan_node(&self) -> Box<dyn PlanNode> {
        Box::new(SingleDependencyNode {
            id: self.id,
            kind: self.kind.clone(),
            dependencies: self
                .dependencies
                .iter()
                .map(|dep| dep.clone_plan_node())
                .collect(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }
}

// Single input node - a plan node that takes single input
#[derive(Debug)]
pub struct SingleInputNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub dependencies: Vec<Box<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

impl SingleInputNode {
    pub fn new(kind: PlanNodeKind, dep: Box<dyn PlanNode>) -> Self {
        Self {
            id: -1, // Will be assigned later
            kind,
            dependencies: vec![dep],
            output_var: None,
            col_names: vec!["default".to_string()],
            cost: 0.0,
        }
    }
}

impl PlanNode for SingleInputNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }

    fn dependencies(&self) -> &Vec<Box<dyn PlanNode>> {
        &self.dependencies
    }

    fn output_var(&self) -> &Option<Variable> {
        &self.output_var
    }

    fn col_names(&self) -> &Vec<String> {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn clone_plan_node(&self) -> Box<dyn PlanNode> {
        Box::new(SingleInputNode {
            id: self.id,
            kind: self.kind.clone(),
            dependencies: self
                .dependencies
                .iter()
                .map(|dep| dep.clone_plan_node())
                .collect(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }
}

// Binary input node - a plan node with two dependencies
#[derive(Debug)]
pub struct BinaryInputNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub dependencies: Vec<Box<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

impl BinaryInputNode {
    pub fn new(kind: PlanNodeKind, left: Box<dyn PlanNode>, right: Box<dyn PlanNode>) -> Self {
        Self {
            id: -1, // Will be assigned later
            kind,
            dependencies: vec![left, right],
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }
}

impl PlanNode for BinaryInputNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }

    fn dependencies(&self) -> &Vec<Box<dyn PlanNode>> {
        &self.dependencies
    }

    fn output_var(&self) -> &Option<Variable> {
        &self.output_var
    }

    fn col_names(&self) -> &Vec<String> {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn clone_plan_node(&self) -> Box<dyn PlanNode> {
        Box::new(BinaryInputNode {
            id: self.id,
            kind: self.kind.clone(),
            dependencies: self
                .dependencies
                .iter()
                .map(|dep| dep.clone_plan_node())
                .collect(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }
}

// Execution plan structure
#[derive(Debug)]
pub struct ExecutionPlan {
    pub root: Option<Box<dyn PlanNode>>,
    pub id: i64,
}

impl ExecutionPlan {
    pub fn new(root: Option<Box<dyn PlanNode>>) -> Self {
        Self {
            root,
            id: -1, // Will be assigned later
        }
    }

    pub fn set_root(&mut self, root: Box<dyn PlanNode>) {
        self.root = Some(root);
    }

    pub fn root(&self) -> &Option<Box<dyn PlanNode>> {
        &self.root
    }
}

// SubPlan structure for representing a section of the overall execution plan
#[derive(Debug)]
pub struct SubPlan {
    pub root: Option<Box<dyn PlanNode>>,
    pub tail: Option<Box<dyn PlanNode>>,
}

impl SubPlan {
    pub fn new(root: Option<Box<dyn PlanNode>>, tail: Option<Box<dyn PlanNode>>) -> Self {
        Self { root, tail }
    }
}
