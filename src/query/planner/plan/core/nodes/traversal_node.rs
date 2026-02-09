//! 图遍历节点实现
//!
//! 包含Expand、ExpandAll、Traverse等图遍历相关的计划节点

use super::super::common::{EdgeProp, TagProp};
use super::plan_node_enum::PlanNodeEnum;
use super::plan_node_traits::{MultipleInputNode, PlanNode, PlanNodeClonable, SingleInputNode};
use crate::core::types::EdgeDirection;
use crate::core::Expression;
use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::node_id_generator::next_node_id;

/// 扩展节点
#[derive(Debug)]
pub struct ExpandNode {
    id: i64,
    space_id: i32,
    edge_types: Vec<String>,
    direction: EdgeDirection,
    step_limit: Option<u32>,
    filter: Option<String>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
    dependencies: Vec<Box<PlanNodeEnum>>,
}

impl Clone for ExpandNode {
    fn clone(&self) -> Self {
        Self {
            id: next_node_id(),
            space_id: self.space_id,
            edge_types: self.edge_types.clone(),
            direction: self.direction,
            step_limit: self.step_limit,
            filter: self.filter.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            dependencies: self.dependencies.clone(),
        }
    }
}

impl ExpandNode {
    pub fn new(space_id: i32, edge_types: Vec<String>, direction: EdgeDirection) -> Self {
        Self {
            id: next_node_id(),
            space_id,
            edge_types,
            direction,
            step_limit: None,
            filter: None,
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            dependencies: Vec::new(),
        }
    }

    pub fn direction(&self) -> EdgeDirection {
        self.direction
    }

    pub fn edge_types(&self) -> &[String] {
        &self.edge_types
    }

    pub fn step_limit(&self) -> Option<u32> {
        self.step_limit
    }

    pub fn filter(&self) -> Option<&String> {
        self.filter.as_ref()
    }

    pub fn set_filter(&mut self, filter: String) {
        self.filter = Some(filter);
    }

    pub fn dependencies(&self) -> &[Box<PlanNodeEnum>] {
        &self.dependencies
    }
}

impl PlanNode for ExpandNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "Expand"
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
        PlanNodeEnum::Expand(self)
    }
}

impl MultipleInputNode for ExpandNode {
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

impl PlanNodeClonable for ExpandNode {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        PlanNodeEnum::Expand(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        PlanNodeEnum::Expand(cloned)
    }
}

/// 扩展全部节点
#[derive(Debug)]
pub struct ExpandAllNode {
    id: i64,
    space_id: i32,
    edge_types: Vec<String>,
    direction: String,
    step_limit: Option<u32>,
    step_limits: Option<Vec<u32>>,
    join_input: bool,
    sample: bool,
    edge_props: Vec<EdgeProp>,
    vertex_props: Vec<TagProp>,
    filter: Option<String>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
    dependencies: Vec<Box<PlanNodeEnum>>,
}

impl Clone for ExpandAllNode {
    fn clone(&self) -> Self {
        Self {
            id: next_node_id(),
            space_id: self.space_id,
            edge_types: self.edge_types.clone(),
            direction: self.direction.clone(),
            step_limit: self.step_limit,
            step_limits: self.step_limits.clone(),
            join_input: self.join_input,
            sample: self.sample,
            edge_props: self.edge_props.clone(),
            vertex_props: self.vertex_props.clone(),
            filter: self.filter.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            dependencies: self.dependencies.clone(),
        }
    }
}

impl ExpandAllNode {
    pub fn new(space_id: i32, edge_types: Vec<String>, direction: &str) -> Self {
        Self {
            id: next_node_id(),
            space_id,
            edge_types,
            direction: direction.to_string(),
            step_limit: None,
            step_limits: None,
            join_input: false,
            sample: false,
            edge_props: Vec::new(),
            vertex_props: Vec::new(),
            filter: None,
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            dependencies: Vec::new(),
        }
    }

    pub fn step_limits(&self) -> Option<&Vec<u32>> {
        self.step_limits.as_ref()
    }

    pub fn set_step_limits(&mut self, limits: Vec<u32>) {
        self.step_limits = Some(limits);
    }

    pub fn join_input(&self) -> bool {
        self.join_input
    }

    pub fn set_join_input(&mut self, join: bool) {
        self.join_input = join;
    }

    pub fn sample(&self) -> bool {
        self.sample
    }

    pub fn set_sample(&mut self, sample: bool) {
        self.sample = sample;
    }

    pub fn edge_props(&self) -> &[EdgeProp] {
        &self.edge_props
    }

    pub fn set_edge_props(&mut self, props: Vec<EdgeProp>) {
        self.edge_props = props;
    }

    pub fn vertex_props(&self) -> &[TagProp] {
        &self.vertex_props
    }

    pub fn set_vertex_props(&mut self, props: Vec<TagProp>) {
        self.vertex_props = props;
    }

    pub fn step_limit(&self) -> Option<u32> {
        self.step_limit
    }

    pub fn set_step_limit(&mut self, limit: u32) {
        self.step_limit = Some(limit);
    }

    pub fn direction(&self) -> &str {
        &self.direction
    }

    pub fn edge_types(&self) -> &[String] {
        &self.edge_types
    }

    pub fn dependencies(&self) -> &[Box<PlanNodeEnum>] {
        &self.dependencies
    }
}

impl PlanNode for ExpandAllNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ExpandAll"
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
        PlanNodeEnum::ExpandAll(self)
    }
}

impl MultipleInputNode for ExpandAllNode {
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

impl PlanNodeClonable for ExpandAllNode {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        PlanNodeEnum::ExpandAll(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        PlanNodeEnum::ExpandAll(cloned)
    }
}

/// 遍历节点
#[derive(Debug)]
pub struct TraverseNode {
    id: i64,
    space_id: i32,
    start_vids: String,
    end_vids: Option<String>,
    edge_types: Vec<String>,
    direction: EdgeDirection,
    min_steps: u32,
    max_steps: u32,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
    input: Option<Box<PlanNodeEnum>>,
    edge_alias: Option<String>,
    e_filter: Option<Expression>,
}

impl Clone for TraverseNode {
    fn clone(&self) -> Self {
        Self {
            id: next_node_id(),
            space_id: self.space_id,
            start_vids: self.start_vids.clone(),
            end_vids: self.end_vids.clone(),
            edge_types: self.edge_types.clone(),
            direction: self.direction,
            min_steps: self.min_steps,
            max_steps: self.max_steps,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            input: self.input.clone(),
            edge_alias: self.edge_alias.clone(),
            e_filter: self.e_filter.clone(),
        }
    }
}

impl TraverseNode {
    pub fn new(space_id: i32, start_vids: &str, min_steps: u32, max_steps: u32) -> Self {
        Self {
            id: next_node_id(),
            space_id,
            start_vids: start_vids.to_string(),
            end_vids: None,
            edge_types: Vec::new(),
            direction: EdgeDirection::Both,
            min_steps,
            max_steps,
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            input: None,
            edge_alias: None,
            e_filter: None,
        }
    }

    pub fn set_end_vids(&mut self, end_vids: &str) {
        self.end_vids = Some(end_vids.to_string());
    }

    pub fn set_edge_types(&mut self, edge_types: Vec<String>) {
        self.edge_types = edge_types;
    }

    pub fn set_direction(&mut self, direction: EdgeDirection) {
        self.direction = direction;
    }

    pub fn start_vids(&self) -> &str {
        &self.start_vids
    }

    pub fn end_vids(&self) -> Option<&String> {
        self.end_vids.as_ref()
    }

    pub fn edge_types(&self) -> &[String] {
        &self.edge_types
    }

    pub fn direction(&self) -> EdgeDirection {
        self.direction
    }

    pub fn min_steps(&self) -> u32 {
        self.min_steps
    }

    pub fn max_steps(&self) -> u32 {
        self.max_steps
    }

    pub fn step_limit(&self) -> Option<u32> {
        Some(self.max_steps)
    }

    pub fn filter(&self) -> Option<&String> {
        None
    }

    pub fn dependencies(&self) -> Vec<&PlanNodeEnum> {
        self.input.as_ref().map(|i| vec![i.as_ref()]).unwrap_or_default()
    }

    pub fn is_one_step(&self) -> bool {
        self.min_steps == 1 && self.max_steps == 1
    }

    pub fn edge_alias(&self) -> Option<&String> {
        self.edge_alias.as_ref()
    }

    pub fn e_filter(&self) -> Option<&Expression> {
        self.e_filter.as_ref()
    }

    pub fn set_e_filter(&mut self, filter: Expression) {
        self.e_filter = Some(filter);
    }

    pub fn v_filter(&self) -> Option<&Expression> {
        None
    }
}

impl PlanNode for TraverseNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "Traverse"
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
        PlanNodeEnum::Traverse(self)
    }
}

impl SingleInputNode for TraverseNode {
    fn input(&self) -> &PlanNodeEnum {
        self.input.as_ref().expect("输入节点不存在")
    }

    fn set_input(&mut self, input: PlanNodeEnum) {
        self.input = Some(Box::new(input));
    }
}

impl PlanNodeClonable for TraverseNode {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        PlanNodeEnum::Traverse(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        PlanNodeEnum::Traverse(cloned)
    }
}

/// 追加顶点节点
#[derive(Debug)]
pub struct AppendVerticesNode {
    id: i64,
    space_id: i32,
    vertex_tag: String,
    vertex_props: Vec<TagProp>,
    filter: Option<String>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
    dependencies: Vec<Box<PlanNodeEnum>>,
    input_var: Option<Variable>,
    src_expression: Option<Expression>,
    dedup: bool,
    track_prev_path: bool,
    need_fetch_prop: bool,
    vids: Vec<String>,
    tag_ids: Vec<i32>,
}

impl Clone for AppendVerticesNode {
    fn clone(&self) -> Self {
        Self {
            id: next_node_id(),
            space_id: self.space_id,
            vertex_tag: self.vertex_tag.clone(),
            vertex_props: self.vertex_props.clone(),
            filter: self.filter.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            dependencies: self.dependencies.clone(),
            input_var: self.input_var.clone(),
            src_expression: self.src_expression.clone(),
            dedup: self.dedup,
            track_prev_path: self.track_prev_path,
            need_fetch_prop: self.need_fetch_prop,
            vids: self.vids.clone(),
            tag_ids: self.tag_ids.clone(),
        }
    }
}

impl AppendVerticesNode {
    pub fn new(space_id: i32, vertex_tag: &str) -> Self {
        Self {
            id: next_node_id(),
            space_id,
            vertex_tag: vertex_tag.to_string(),
            vertex_props: Vec::new(),
            filter: None,
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            dependencies: Vec::new(),
            input_var: None,
            src_expression: None,
            dedup: false,
            track_prev_path: false,
            need_fetch_prop: false,
            vids: Vec::new(),
            tag_ids: Vec::new(),
        }
    }

    pub fn vertex_tag(&self) -> &str {
        &self.vertex_tag
    }

    pub fn vertex_props(&self) -> &[TagProp] {
        &self.vertex_props
    }

    pub fn set_vertex_props(&mut self, props: Vec<TagProp>) {
        self.vertex_props = props;
    }

    pub fn filter(&self) -> Option<&String> {
        self.filter.as_ref()
    }

    pub fn set_filter(&mut self, filter: String) {
        self.filter = Some(filter);
    }

    pub fn input_var(&self) -> Option<&Variable> {
        self.input_var.as_ref()
    }

    pub fn src_expression(&self) -> Option<&Expression> {
        self.src_expression.as_ref()
    }

    pub fn v_filter(&self) -> Option<&Expression> {
        None
    }

    pub fn dedup(&self) -> bool {
        self.dedup
    }

    pub fn track_prev_path(&self) -> bool {
        self.track_prev_path
    }

    pub fn need_fetch_prop(&self) -> bool {
        self.need_fetch_prop
    }

    pub fn dependencies(&self) -> &[Box<PlanNodeEnum>] {
        &self.dependencies
    }

    pub fn vids(&self) -> &[String] {
        &self.vids
    }

    pub fn tag_ids(&self) -> &[i32] {
        &self.tag_ids
    }
}

impl PlanNode for AppendVerticesNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "AppendVertices"
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
        PlanNodeEnum::AppendVertices(self)
    }
}

impl MultipleInputNode for AppendVerticesNode {
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

impl PlanNodeClonable for AppendVerticesNode {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        PlanNodeEnum::AppendVertices(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        PlanNodeEnum::AppendVertices(cloned)
    }
}
