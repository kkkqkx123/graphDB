//! PlanNode特征和基础实现
//! 定义执行计划节点的通用接口和各种基础节点类型

use super::plan_node_visitor::{PlanNodeVisitError, PlanNodeVisitor};
use crate::query::validator::Variable;

/// 计划节点类型枚举，表示执行计划中的各种操作
#[derive(Debug, Clone, PartialEq)]
pub enum PlanNodeKind {
    // 查询节点
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

    // 数据处理节点
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
    HashJoin,
    HashLeftJoin,
    HashInnerJoin,
    CrossJoin,
    RollUpApply,
    PatternApply,
    Argument,

    // 控制流节点
    Select,
    Loop,
    PassThrough,
    Start,

    // 模式相关节点
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

    // 索引相关节点
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

    // 用户相关节点
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

    // 快照节点
    CreateSnapshot,
    DropSnapshot,
    ShowSnapshots,

    // 更新/删除节点
    DeleteVertices,
    DeleteEdges,
    UpdateVertex,
    DeleteTags,
    UpdateEdge,

    // 显示节点
    ShowParts,
    ShowCharset,
    ShowCollation,
    ShowStats,
    ShowConfigs,
    SetConfig,
    GetConfig,
    ShowMetaLeader,

    // 区域相关节点
    ShowZones,
    MergeZone,
    RenameZone,
    DropZone,
    DivideZone,
    AddHosts,
    DropHosts,
    DescribeZone,
    AddHostsIntoZone,

    // 监听器相关节点
    AddListener,
    RemoveListener,
    ShowListener,

    // 服务相关节点
    ShowServiceClients,
    ShowFTIndexes,
    SignInService,
    SignOutService,
    ShowSessions,
    UpdateSession,
    KillSession,

    ShowQueries,
    KillQuery,

    // 未知节点类型的占位符
    Unknown,
}

/// PlanNode特征，所有计划节点都应实现该特征
pub trait PlanNode: std::fmt::Debug {
    /// 获取节点的唯一ID
    fn id(&self) -> i64;

    /// 获取节点的类型
    fn kind(&self) -> PlanNodeKind;

    /// 获取节点的依赖节点列表
    fn dependencies(&self) -> &Vec<Box<dyn PlanNode>>;

    /// 获取节点的输出变量
    fn output_var(&self) -> &Option<Variable>;

    /// 获取列名列表
    fn col_names(&self) -> &Vec<String>;

    /// 获取节点的成本估计值
    fn cost(&self) -> f64;

    /// 克隆节点
    fn clone_plan_node(&self) -> Box<dyn PlanNode>;

    /// 使用访问者模式访问节点
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError>;

    /// 设置节点的依赖
    fn set_dependencies(&mut self, deps: Vec<Box<dyn PlanNode>>);

    /// 设置节点的输出变量
    fn set_output_var(&mut self, var: Variable);

    /// 设置列名
    fn set_col_names(&mut self, names: Vec<String>);

    /// 设置成本
    fn set_cost(&mut self, cost: f64);
}

/// 单一依赖节点 - 具有一个依赖的计划节点
#[derive(Debug)]
pub struct SingleDependencyNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub dependencies: Vec<Box<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

impl Clone for SingleDependencyNode {
    fn clone(&self) -> Self {
        // 创建一个基本结构体，不包含依赖项
        // 这是一个临时解决方案，因为PlanNode的克隆需要特别处理
        Self {
            id: self.id,
            kind: self.kind.clone(),
            dependencies: Vec::new(), // 克隆时清空依赖项
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        }
    }
}

impl SingleDependencyNode {
    pub fn new(kind: PlanNodeKind, dep: Box<dyn PlanNode>) -> Self {
        Self {
            id: -1, // 将在后续分配
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

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_plan_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn PlanNode>>) {
        self.dependencies = deps;
    }

    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    fn set_cost(&mut self, cost: f64) {
        self.cost = cost;
    }
}

/// 单一输入节点 - 处理单一输入的计划节点
#[derive(Debug)]
pub struct SingleInputNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub dependencies: Vec<Box<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

impl Clone for SingleInputNode {
    fn clone(&self) -> Self {
        // 创建一个基本结构体，不包含依赖项
        // 这是一个临时解决方案，因为PlanNode的克隆需要特别处理
        Self {
            id: self.id,
            kind: self.kind.clone(),
            dependencies: Vec::new(), // 克隆时清空依赖项
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        }
    }
}

impl SingleInputNode {
    pub fn new(kind: PlanNodeKind, dep: Box<dyn PlanNode>) -> Self {
        Self {
            id: -1, // 将在后续分配
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

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_plan_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn PlanNode>>) {
        self.dependencies = deps;
    }

    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    fn set_cost(&mut self, cost: f64) {
        self.cost = cost;
    }
}

/// 二元输入节点 - 具有两个依赖的计划节点
#[derive(Debug)]
pub struct BinaryInputNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub dependencies: Vec<Box<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

impl Clone for BinaryInputNode {
    fn clone(&self) -> Self {
        // 创建一个基本结构体，不包含依赖项
        // 这是一个临时解决方案，因为PlanNode的克隆需要特别处理
        Self {
            id: self.id,
            kind: self.kind.clone(),
            dependencies: Vec::new(), // 克隆时清空依赖项
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        }
    }
}

impl BinaryInputNode {
    pub fn new(kind: PlanNodeKind, left: Box<dyn PlanNode>, right: Box<dyn PlanNode>) -> Self {
        Self {
            id: -1, // 将在后续分配
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

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_plan_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn PlanNode>>) {
        self.dependencies = deps;
    }

    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    fn set_cost(&mut self, cost: f64) {
        self.cost = cost;
    }
}

/// 可变依赖节点 - 具有可变数量依赖的计划节点
#[derive(Debug)]
pub struct VariableDependencyNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub dependencies: Vec<Box<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

impl Clone for VariableDependencyNode {
    fn clone(&self) -> Self {
        // 创建一个基本结构体，不包含依赖项
        // 这是一个临时解决方案，因为PlanNode的克隆需要特别处理
        Self {
            id: self.id,
            kind: self.kind.clone(),
            dependencies: Vec::new(), // 克隆时清空依赖项
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        }
    }
}

impl VariableDependencyNode {
    pub fn new(kind: PlanNodeKind) -> Self {
        Self {
            id: -1, // 将在后续分配
            kind,
            dependencies: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }

    pub fn add_dependency(&mut self, dep: Box<dyn PlanNode>) {
        self.dependencies.push(dep);
    }
}

impl PlanNode for VariableDependencyNode {
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
        Box::new(VariableDependencyNode {
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

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_plan_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn PlanNode>>) {
        self.dependencies = deps;
    }

    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    fn set_cost(&mut self, cost: f64) {
        self.cost = cost;
    }
}
