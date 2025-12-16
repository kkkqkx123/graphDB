//! 数据修改相关的计划节点
//! 如INSERT、UPDATE、DELETE等DML操作
//! 包括INSERT VERTICES、UPDATE VERTEX、DELETE EDGES等操作

use std::collections::HashMap;
use super::plan_node::{PlanNode as BasePlanNode, SingleDependencyNode};
use super::core::PlanNodeKind;
use crate::query::context::validate::types::Variable;
use super::plan_node_visitor::{PlanNodeVisitor, PlanNodeVisitError};

// 存储层的新顶点结构
#[derive(Debug, Clone)]
pub struct NewVertex {
    pub vid: String,
    pub tags: Vec<NewTag>,
}

#[derive(Debug, Clone)]
pub struct NewTag {
    pub tag_id: i32,
    pub props: Vec<NewProp>,
}

#[derive(Debug, Clone)]
pub struct NewProp {
    pub name: String,
    pub value: String,  // 这里简化为字符串，实际可能是复杂类型
}

// 存储层的新边结构
#[derive(Debug, Clone)]
pub struct NewEdge {
    pub src: String,
    pub dst: String,
    pub rank: i64,
    pub edge_type: i32,
    pub props: Vec<NewProp>,
}

/// 插入顶点计划节点
#[derive(Debug)]
pub struct InsertVertices {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub vertices: Vec<NewVertex>,
    pub tag_prop_names: HashMap<i32, Vec<String>>,  // TagID到属性名的映射
    pub if_not_exists: bool,
    pub ignore_existed_index: bool,
}

impl InsertVertices {
    pub fn new(
        id: i64,
        space_id: i32,
        vertices: Vec<NewVertex>,
        tag_prop_names: HashMap<i32, Vec<String>>,
        if_not_exists: bool,
        ignore_existed_index: bool,
    ) -> Self {
        Self {
            id,
            kind: PlanNodeKind::InsertVertices,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            vertices,
            tag_prop_names,
            if_not_exists,
            ignore_existed_index,
        }
    }

    pub fn get_vertices(&self) -> &Vec<NewVertex> {
        &self.vertices
    }

    pub fn get_prop_names(&self) -> &HashMap<i32, Vec<String>> {
        &self.tag_prop_names
    }

    pub fn get_space(&self) -> i32 {
        self.space_id
    }

    pub fn get_if_not_exists(&self) -> bool {
        self.if_not_exists
    }

    pub fn get_ignore_existed_index(&self) -> bool {
        self.ignore_existed_index
    }
}

impl Clone for InsertVertices {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            vertices: self.vertices.clone(),
            tag_prop_names: self.tag_prop_names.clone(),
            if_not_exists: self.if_not_exists,
            ignore_existed_index: self.ignore_existed_index,
        }
    }
}

impl BasePlanNode for InsertVertices {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }

    fn dependencies(&self) -> &Vec<Box<dyn BasePlanNode>> {
        &self.deps
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

    fn clone_plan_node(&self) -> Box<dyn BasePlanNode> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn BasePlanNode>>) {
        self.deps = deps;
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

/// 插入边计划节点
#[derive(Debug)]
pub struct InsertEdges {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub edges: Vec<NewEdge>,
    pub prop_names: Vec<String>,
    pub if_not_exists: bool,
    pub ignore_existed_index: bool,
    pub use_chain_insert: bool,  // 是否使用链式插入
}

impl InsertEdges {
    pub fn new(
        id: i64,
        space_id: i32,
        edges: Vec<NewEdge>,
        prop_names: Vec<String>,
        if_not_exists: bool,
        ignore_existed_index: bool,
        use_chain_insert: bool,
    ) -> Self {
        Self {
            id,
            kind: PlanNodeKind::InsertEdges,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            edges,
            prop_names,
            if_not_exists,
            ignore_existed_index,
            use_chain_insert,
        }
    }

    pub fn get_edges(&self) -> &Vec<NewEdge> {
        &self.edges
    }

    pub fn get_prop_names(&self) -> &Vec<String> {
        &self.prop_names
    }

    pub fn get_space(&self) -> i32 {
        self.space_id
    }

    pub fn get_if_not_exists(&self) -> bool {
        self.if_not_exists
    }

    pub fn get_ignore_existed_index(&self) -> bool {
        self.ignore_existed_index
    }

    pub fn get_use_chain_insert(&self) -> bool {
        self.use_chain_insert
    }
}

impl Clone for InsertEdges {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            edges: self.edges.clone(),
            prop_names: self.prop_names.clone(),
            if_not_exists: self.if_not_exists,
            ignore_existed_index: self.ignore_existed_index,
            use_chain_insert: self.use_chain_insert,
        }
    }
}

impl BasePlanNode for InsertEdges {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }

    fn dependencies(&self) -> &Vec<Box<dyn BasePlanNode>> {
        &self.deps
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

    fn clone_plan_node(&self) -> Box<dyn BasePlanNode> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn BasePlanNode>>) {
        self.deps = deps;
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

/// 更新顶点计划节点
#[derive(Debug)]
pub struct UpdateVertex {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub vid: String,  // 顶点ID
    pub tag_id: i32,  // 标签ID
    pub updated_props: HashMap<String, String>,  // 更新的属性
    pub condition: Option<String>,  // 更新条件
    pub insertable: bool,  // 是否可插入
}

impl UpdateVertex {
    pub fn new(
        id: i64,
        space_id: i32,
        vid: &str,
        tag_id: i32,
        updated_props: HashMap<String, String>,
        condition: Option<String>,
        insertable: bool,
    ) -> Self {
        Self {
            id,
            kind: PlanNodeKind::UpdateVertex,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            vid: vid.to_string(),
            tag_id,
            updated_props,
            condition,
            insertable,
        }
    }
}

impl Clone for UpdateVertex {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            vid: self.vid.clone(),
            tag_id: self.tag_id,
            updated_props: self.updated_props.clone(),
            condition: self.condition.clone(),
            insertable: self.insertable,
        }
    }
}

impl BasePlanNode for UpdateVertex {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }

    fn dependencies(&self) -> &Vec<Box<dyn BasePlanNode>> {
        &self.deps
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

    fn clone_plan_node(&self) -> Box<dyn BasePlanNode> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn BasePlanNode>>) {
        self.deps = deps;
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

/// 更新边计划节点
#[derive(Debug)]
pub struct UpdateEdge {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub src: String,  // 源顶点
    pub dst: String,  // 目标顶点
    pub rank: i64,    // 排名
    pub edge_type: i32,  // 边类型
    pub updated_props: HashMap<String, String>,  // 更新的属性
    pub condition: Option<String>,  // 更新条件
    pub insertable: bool,  // 是否可插入
}

impl UpdateEdge {
    pub fn new(
        id: i64,
        space_id: i32,
        src: &str,
        dst: &str,
        rank: i64,
        edge_type: i32,
        updated_props: HashMap<String, String>,
        condition: Option<String>,
        insertable: bool,
    ) -> Self {
        Self {
            id,
            kind: PlanNodeKind::UpdateEdge,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            src: src.to_string(),
            dst: dst.to_string(),
            rank,
            edge_type,
            updated_props,
            condition,
            insertable,
        }
    }
}

impl Clone for UpdateEdge {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            src: self.src.clone(),
            dst: self.dst.clone(),
            rank: self.rank,
            edge_type: self.edge_type,
            updated_props: self.updated_props.clone(),
            condition: self.condition.clone(),
            insertable: self.insertable,
        }
    }
}

impl BasePlanNode for UpdateEdge {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }

    fn dependencies(&self) -> &Vec<Box<dyn BasePlanNode>> {
        &self.deps
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

    fn clone_plan_node(&self) -> Box<dyn BasePlanNode> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn BasePlanNode>>) {
        self.deps = deps;
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

/// 删除顶点计划节点
#[derive(Debug)]
pub struct DeleteVertices {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub vids: Vec<String>,  // 顶点ID列表
    pub delete_e: bool,  // 是否同时删除边
}

impl DeleteVertices {
    pub fn new(id: i64, space_id: i32, vids: Vec<String>, delete_e: bool) -> Self {
        Self {
            id,
            kind: PlanNodeKind::DeleteVertices,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            vids,
            delete_e,
        }
    }
}

impl Clone for DeleteVertices {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            vids: self.vids.clone(),
            delete_e: self.delete_e,
        }
    }
}

impl BasePlanNode for DeleteVertices {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }

    fn dependencies(&self) -> &Vec<Box<dyn BasePlanNode>> {
        &self.deps
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

    fn clone_plan_node(&self) -> Box<dyn BasePlanNode> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn BasePlanNode>>) {
        self.deps = deps;
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

/// 删除边计划节点
#[derive(Debug)]
pub struct DeleteEdges {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub src: String,  // 源顶点
    pub dst: String,  // 目标顶点
    pub rank: i64,    // 排名
    pub edge_type: i32,  // 边类型
    pub condition: Option<String>,  // 删除条件
}

impl DeleteEdges {
    pub fn new(
        id: i64,
        space_id: i32,
        src: &str,
        dst: &str,
        rank: i64,
        edge_type: i32,
        condition: Option<String>,
    ) -> Self {
        Self {
            id,
            kind: PlanNodeKind::DeleteEdges,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            src: src.to_string(),
            dst: dst.to_string(),
            rank,
            edge_type,
            condition,
        }
    }
}

impl Clone for DeleteEdges {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            src: self.src.clone(),
            dst: self.dst.clone(),
            rank: self.rank,
            edge_type: self.edge_type,
            condition: self.condition.clone(),
        }
    }
}

impl BasePlanNode for DeleteEdges {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }

    fn dependencies(&self) -> &Vec<Box<dyn BasePlanNode>> {
        &self.deps
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

    fn clone_plan_node(&self) -> Box<dyn BasePlanNode> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn BasePlanNode>>) {
        self.deps = deps;
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

/// 删除标签计划节点
#[derive(Debug)]
pub struct DeleteTags {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub vid: String,  // 顶点ID
    pub tag_ids: Vec<i32>,  // 要删除的标签ID列表
}

impl DeleteTags {
    pub fn new(id: i64, space_id: i32, vid: &str, tag_ids: Vec<i32>) -> Self {
        Self {
            id,
            kind: PlanNodeKind::DeleteTags,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            vid: vid.to_string(),
            tag_ids,
        }
    }
}

impl Clone for DeleteTags {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            vid: self.vid.clone(),
            tag_ids: self.tag_ids.clone(),
        }
    }
}

impl BasePlanNode for DeleteTags {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }

    fn dependencies(&self) -> &Vec<Box<dyn BasePlanNode>> {
        &self.deps
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

    fn clone_plan_node(&self) -> Box<dyn BasePlanNode> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn BasePlanNode>>) {
        self.deps = deps;
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
