//! 图扫描节点实现
//!
//! 包含获取顶点、边和邻居节点的计划节点

use super::super::common::{EdgeProp, TagProp};
use super::plan_node_enum::PlanNodeEnum;
use super::plan_node_traits::{MultipleInputNode, PlanNode, PlanNodeClonable, ZeroInputNode};
use crate::core::Expression;
use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::node_id_generator::next_node_id;

/// 获取顶点节点
#[derive(Debug)]
pub struct GetVerticesNode {
    id: i64,
    space_id: i32,
    src_ref: Expression,
    src_vids: String,
    tag_props: Vec<TagProp>,
    expression: Option<String>,
    dedup: bool,
    limit: Option<i64>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
    dependencies: Vec<Box<PlanNodeEnum>>,
}

impl Clone for GetVerticesNode {
    fn clone(&self) -> Self {
        Self {
            id: next_node_id(),
            space_id: self.space_id,
            src_ref: self.src_ref.clone(),
            src_vids: self.src_vids.clone(),
            tag_props: self.tag_props.clone(),
            expression: self.expression.clone(),
            dedup: self.dedup,
            limit: self.limit,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            dependencies: self.dependencies.clone(),
        }
    }
}

impl GetVerticesNode {
    pub fn new(space_id: i32, src_vids: &str) -> Self {
        Self {
            id: next_node_id(),
            space_id,
            src_ref: Expression::Variable(src_vids.to_string()),
            src_vids: src_vids.to_string(),
            tag_props: Vec::new(),
            expression: None,
            dedup: false,
            limit: None,
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            dependencies: Vec::new(),
        }
    }

    pub fn set_limit(&mut self, limit: i64) {
        self.limit = Some(limit);
    }

    pub fn has_effective_filter(&self) -> bool {
        self.expression.is_some()
    }

    pub fn space_id(&self) -> i32 {
        self.space_id
    }

    pub fn src_vids(&self) -> &str {
        &self.src_vids
    }

    pub fn set_tag_props(&mut self, tag_props: Vec<TagProp>) {
        self.tag_props = tag_props;
    }

    pub fn expression(&self) -> Option<&String> {
        self.expression.as_ref()
    }

    pub fn limit(&self) -> Option<i64> {
        self.limit
    }

    pub fn add_dependency(&mut self, dep: PlanNodeEnum) {
        self.dependencies.push(Box::new(dep));
    }
}

impl PlanNode for GetVerticesNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "GetVertices"
    }

    fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    fn col_names(&self) -> &[String] {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::GetVertices(self)
    }
}

impl ZeroInputNode for GetVerticesNode {}

impl MultipleInputNode for GetVerticesNode {
    fn inputs(&self) -> &[Box<PlanNodeEnum>] {
        &self.dependencies
    }

    fn add_input(&mut self, input: PlanNodeEnum) {
        self.dependencies.push(Box::new(input));
    }

    fn remove_input(&mut self, index: usize) -> Result<(), String> {
        if index < self.dependencies.len() {
            self.dependencies.remove(index);
            Ok(())
        } else {
            Err(format!("索引 {} 超出范围", index))
        }
    }
}

impl PlanNodeClonable for GetVerticesNode {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        PlanNodeEnum::GetVertices(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        PlanNodeEnum::GetVertices(cloned)
    }
}

/// 获取边节点
#[derive(Debug)]
pub struct GetEdgesNode {
    id: i64,
    space_id: i32,
    edge_ref: Expression,
    src: String,
    edge_type: String,
    rank: String,
    dst: String,
    edge_props: Vec<EdgeProp>,
    expression: Option<String>,
    dedup: bool,
    limit: Option<i64>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl Clone for GetEdgesNode {
    fn clone(&self) -> Self {
        Self {
            id: next_node_id(),
            space_id: self.space_id,
            edge_ref: self.edge_ref.clone(),
            src: self.src.clone(),
            edge_type: self.edge_type.clone(),
            rank: self.rank.clone(),
            dst: self.dst.clone(),
            edge_props: self.edge_props.clone(),
            expression: self.expression.clone(),
            dedup: self.dedup,
            limit: self.limit,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        }
    }
}

impl GetEdgesNode {
    pub fn new(space_id: i32, src: &str, edge_type: &str, rank: &str, dst: &str) -> Self {
        Self {
            id: next_node_id(),
            space_id,
            edge_ref: Expression::Variable(format!("{}->{}@{}", src, dst, edge_type)),
            src: src.to_string(),
            edge_type: edge_type.to_string(),
            rank: rank.to_string(),
            dst: dst.to_string(),
            edge_props: Vec::new(),
            expression: None,
            dedup: false,
            limit: None,
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }

    pub fn set_limit(&mut self, limit: i64) {
        self.limit = Some(limit);
    }

    pub fn has_effective_filter(&self) -> bool {
        self.expression.is_some()
    }

    pub fn space_id(&self) -> i32 {
        self.space_id
    }

    pub fn src(&self) -> &str {
        &self.src
    }

    pub fn edge_type(&self) -> &str {
        &self.edge_type
    }

    pub fn rank(&self) -> &str {
        &self.rank
    }

    pub fn dst(&self) -> &str {
        &self.dst
    }

    pub fn expression(&self) -> Option<&String> {
        self.expression.as_ref()
    }

    pub fn limit(&self) -> Option<i64> {
        self.limit
    }
}

impl PlanNode for GetEdgesNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "GetEdges"
    }

    fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    fn col_names(&self) -> &[String] {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::GetEdges(self)
    }
}

impl ZeroInputNode for GetEdgesNode {}

impl PlanNodeClonable for GetEdgesNode {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        PlanNodeEnum::GetEdges(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        PlanNodeEnum::GetEdges(cloned)
    }
}

/// 获取邻居节点
#[derive(Debug)]
pub struct GetNeighborsNode {
    id: i64,
    space_id: i32,
    src_vids: String,
    edge_types: Vec<String>,
    direction: String,
    edge_props: Vec<EdgeProp>,
    tag_props: Vec<TagProp>,
    expression: Option<String>,
    dedup: bool,
    limit: Option<i64>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
    dependencies: Vec<Box<PlanNodeEnum>>,
}

impl Clone for GetNeighborsNode {
    fn clone(&self) -> Self {
        Self {
            id: next_node_id(),
            space_id: self.space_id,
            src_vids: self.src_vids.clone(),
            edge_types: self.edge_types.clone(),
            direction: self.direction.clone(),
            edge_props: self.edge_props.clone(),
            tag_props: self.tag_props.clone(),
            expression: self.expression.clone(),
            dedup: self.dedup,
            limit: self.limit,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            dependencies: self.dependencies.clone(),
        }
    }
}

impl GetNeighborsNode {
    pub fn new(space_id: i32, src_vids: &str) -> Self {
        Self {
            id: next_node_id(),
            space_id,
            src_vids: src_vids.to_string(),
            edge_types: Vec::new(),
            direction: "BOTH".to_string(),
            edge_props: Vec::new(),
            tag_props: Vec::new(),
            expression: None,
            dedup: false,
            limit: None,
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            dependencies: Vec::new(),
        }
    }

    pub fn set_edge_types(&mut self, edge_types: Vec<String>) {
        self.edge_types = edge_types;
    }

    pub fn set_direction(&mut self, direction: &str) {
        self.direction = direction.to_string();
    }

    pub fn space_id(&self) -> i32 {
        self.space_id
    }

    pub fn src_vids(&self) -> &str {
        &self.src_vids
    }

    pub fn edge_types(&self) -> &[String] {
        &self.edge_types
    }

    pub fn direction(&self) -> &str {
        &self.direction
    }
}

impl PlanNode for GetNeighborsNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "GetNeighbors"
    }

    fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    fn col_names(&self) -> &[String] {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::GetNeighbors(self)
    }
}

impl ZeroInputNode for GetNeighborsNode {}

impl MultipleInputNode for GetNeighborsNode {
    fn inputs(&self) -> &[Box<PlanNodeEnum>] {
        &self.dependencies
    }

    fn add_input(&mut self, input: PlanNodeEnum) {
        self.dependencies.push(Box::new(input));
    }

    fn remove_input(&mut self, index: usize) -> Result<(), String> {
        if index < self.dependencies.len() {
            self.dependencies.remove(index);
            Ok(())
        } else {
            Err(format!("索引 {} 超出范围", index))
        }
    }
}

impl PlanNodeClonable for GetNeighborsNode {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        PlanNodeEnum::GetNeighbors(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        PlanNodeEnum::GetNeighbors(cloned)
    }
}

/// 扫描顶点节点
#[derive(Debug)]
pub struct ScanVerticesNode {
    id: i64,
    space_id: i32,
    tag: Option<String>,
    expression: Option<String>,
    limit: Option<i64>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl Clone for ScanVerticesNode {
    fn clone(&self) -> Self {
        Self {
            id: next_node_id(),
            space_id: self.space_id,
            tag: self.tag.clone(),
            expression: self.expression.clone(),
            limit: self.limit,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        }
    }
}

impl ScanVerticesNode {
    pub fn new(space_id: i32) -> Self {
        Self {
            id: next_node_id(),
            space_id,
            tag: None,
            expression: None,
            limit: None,
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }

    pub fn set_tag(&mut self, tag: &str) {
        self.tag = Some(tag.to_string());
    }

    pub fn set_limit(&mut self, limit: i64) {
        self.limit = Some(limit);
    }

    pub fn space_id(&self) -> i32 {
        self.space_id
    }

    pub fn tag(&self) -> Option<&String> {
        self.tag.as_ref()
    }

    pub fn tag_filter(&self) -> Option<&String> {
        self.tag.as_ref()
    }

    pub fn vertex_filter(&self) -> Option<&String> {
        self.expression.as_ref()
    }

    pub fn limit(&self) -> Option<i64> {
        self.limit
    }
}

impl PlanNode for ScanVerticesNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ScanVertices"
    }

    fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    fn col_names(&self) -> &[String] {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::ScanVertices(self)
    }
}

impl ZeroInputNode for ScanVerticesNode {}

impl PlanNodeClonable for ScanVerticesNode {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        PlanNodeEnum::ScanVertices(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        PlanNodeEnum::ScanVertices(cloned)
    }
}

/// 扫描边节点
#[derive(Debug)]
pub struct ScanEdgesNode {
    id: i64,
    space_id: i32,
    edge_type: Option<String>,
    expression: Option<String>,
    limit: Option<i64>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl Clone for ScanEdgesNode {
    fn clone(&self) -> Self {
        Self {
            id: next_node_id(),
            space_id: self.space_id,
            edge_type: self.edge_type.clone(),
            expression: self.expression.clone(),
            limit: self.limit,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        }
    }
}

impl ScanEdgesNode {
    pub fn new(space_id: i32, edge_type: &str) -> Self {
        Self {
            id: next_node_id(),
            space_id,
            edge_type: Some(edge_type.to_string()),
            expression: None,
            limit: None,
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }

    pub fn set_limit(&mut self, limit: i64) {
        self.limit = Some(limit);
    }

    pub fn space_id(&self) -> i32 {
        self.space_id
    }

    pub fn edge_type(&self) -> Option<String> {
        self.edge_type.clone()
    }

    pub fn filter(&self) -> Option<&String> {
        self.expression.as_ref()
    }

    pub fn limit(&self) -> Option<i64> {
        self.limit
    }
}

impl PlanNode for ScanEdgesNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ScanEdges"
    }

    fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    fn col_names(&self) -> &[String] {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::ScanEdges(self)
    }
}

impl ZeroInputNode for ScanEdgesNode {}

impl PlanNodeClonable for ScanEdgesNode {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        PlanNodeEnum::ScanEdges(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        PlanNodeEnum::ScanEdges(cloned)
    }
}
