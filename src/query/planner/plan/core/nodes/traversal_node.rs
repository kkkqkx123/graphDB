//! 图遍历节点实现
//!
//! 包含Expand、ExpandAll、Traverse等图遍历相关的计划节点

use super::super::common::{EdgeProp, TagProp};
use crate::core::types::EdgeDirection;
use crate::core::{Expression, Value};
use crate::query::context::validate::types::Variable;

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
    dependencies: Vec<Box<super::plan_node_enum::PlanNodeEnum>>,
}

// 为 ExpandNode 实现 Clone
impl Clone for ExpandNode {
    fn clone(&self) -> Self {
        ExpandNode {
            id: self.id,
            space_id: self.space_id,
            edge_types: self.edge_types.clone(),
            direction: self.direction,
            step_limit: self.step_limit,
            filter: self.filter.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            dependencies: Vec::new(),
        }
    }
}

impl ExpandNode {
    pub fn new(space_id: i32, edge_types: Vec<String>, direction: EdgeDirection) -> Self {
        Self {
            id: -1,
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
}

impl ExpandNode {
    /// 获取方向
    pub fn direction(&self) -> EdgeDirection {
        self.direction
    }

    /// 获取边类型
    pub fn edge_types(&self) -> &[String] {
        &self.edge_types
    }

    /// 获取步数限制
    pub fn step_limit(&self) -> Option<u32> {
        self.step_limit
    }

    /// 获取过滤条件
    pub fn filter(&self) -> Option<&String> {
        self.filter.as_ref()
    }

    /// 设置过滤条件
    pub fn set_filter(&mut self, filter: String) {
        self.filter = Some(filter);
    }
}

impl ExpandNode {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "Expand"
    }

    pub fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    pub fn cost(&self) -> f64 {
        self.cost
    }

    pub fn dependencies(&self) -> &[Box<super::plan_node_enum::PlanNodeEnum>] {
        &self.dependencies
    }

    pub fn add_dependency(&mut self, dep: super::plan_node_enum::PlanNodeEnum) {
        self.dependencies.push(Box::new(dep));
    }

    pub fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.dependencies.iter().position(|dep| dep.id() == id) {
            self.dependencies.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    pub fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::Expand(self.clone())
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::Expand(cloned)
    }
}

// 为 ExpandNode 实现 PlanNode trait
impl super::plan_node_traits::PlanNode for ExpandNode {
    fn id(&self) -> i64 {
        self.id()
    }

    fn name(&self) -> &'static str {
        self.type_name()
    }

    fn output_var(&self) -> Option<&Variable> {
        self.output_var()
    }

    fn col_names(&self) -> &[String] {
        self.col_names()
    }

    fn cost(&self) -> f64 {
        self.cost()
    }

    fn set_output_var(&mut self, var: Variable) {
        self.set_output_var(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.set_col_names(names);
    }

    fn into_enum(self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::Expand(self)
    }
}

// 为 ExpandNode 实现 MultipleInputNode trait
impl super::plan_node_traits::MultipleInputNode for ExpandNode {
    fn inputs(&self) -> &[Box<super::plan_node_enum::PlanNodeEnum>] {
        &self.dependencies
    }

    fn add_input(&mut self, input: super::plan_node_enum::PlanNodeEnum) {
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

    fn input_count(&self) -> usize {
        self.dependencies.len()
    }
}

// 为 ExpandNode 实现 PlanNodeClonable trait
impl super::plan_node_traits::PlanNodeClonable for ExpandNode {
    fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_plan_node()
    }

    fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_with_new_id(new_id)
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
    dependencies: Vec<Box<super::plan_node_enum::PlanNodeEnum>>,
}

// 为 ExpandAllNode 实现 Clone
impl Clone for ExpandAllNode {
    fn clone(&self) -> Self {
        ExpandAllNode {
            id: self.id,
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
            dependencies: Vec::new(),
        }
    }
}

impl ExpandAllNode {
    pub fn new(space_id: i32, edge_types: Vec<String>, direction: &str) -> Self {
        Self {
            id: -1,
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

    /// 获取步数限制列表
    pub fn step_limits(&self) -> Option<&Vec<u32>> {
        self.step_limits.as_ref()
    }

    /// 设置步数限制列表
    pub fn set_step_limits(&mut self, limits: Vec<u32>) {
        self.step_limits = Some(limits);
    }

    /// 获取是否连接输入
    pub fn join_input(&self) -> bool {
        self.join_input
    }

    /// 设置是否连接输入
    pub fn set_join_input(&mut self, join: bool) {
        self.join_input = join;
    }

    /// 获取是否采样
    pub fn sample(&self) -> bool {
        self.sample
    }

    /// 设置是否采样
    pub fn set_sample(&mut self, sample: bool) {
        self.sample = sample;
    }

    /// 获取边类型
    pub fn edge_types(&self) -> &[String] {
        &self.edge_types
    }

    /// 获取方向
    pub fn direction(&self) -> &str {
        &self.direction
    }

    /// 获取步数限制
    pub fn step_limit(&self) -> Option<u32> {
        self.step_limit
    }
}

impl ExpandAllNode {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "ExpandAll"
    }

    pub fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    pub fn cost(&self) -> f64 {
        self.cost
    }

    pub fn dependencies(&self) -> &[Box<super::plan_node_enum::PlanNodeEnum>] {
        &self.dependencies
    }

    pub fn add_dependency(&mut self, dep: super::plan_node_enum::PlanNodeEnum) {
        self.dependencies.push(Box::new(dep));
    }

    pub fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.dependencies.iter().position(|dep| dep.id() == id) {
            self.dependencies.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    pub fn set_step_limit(&mut self, limit: u32) {
        self.step_limit = Some(limit);
    }

    pub fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::ExpandAll(self.clone())
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::ExpandAll(cloned)
    }
}

// 为 ExpandAllNode 实现 PlanNode trait
impl super::plan_node_traits::PlanNode for ExpandAllNode {
    fn id(&self) -> i64 {
        self.id()
    }

    fn name(&self) -> &'static str {
        self.type_name()
    }

    fn output_var(&self) -> Option<&Variable> {
        self.output_var()
    }

    fn col_names(&self) -> &[String] {
        self.col_names()
    }

    fn cost(&self) -> f64 {
        self.cost()
    }

    fn set_output_var(&mut self, var: Variable) {
        self.set_output_var(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.set_col_names(names);
    }

    fn into_enum(self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::ExpandAll(self)
    }
}

// 为 ExpandAllNode 实现 MultipleInputNode trait
impl super::plan_node_traits::MultipleInputNode for ExpandAllNode {
    fn inputs(&self) -> &[Box<super::plan_node_enum::PlanNodeEnum>] {
        &self.dependencies
    }

    fn add_input(&mut self, input: super::plan_node_enum::PlanNodeEnum) {
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

    fn input_count(&self) -> usize {
        self.dependencies.len()
    }
}

// 为 ExpandAllNode 实现 PlanNodeClonable trait
impl super::plan_node_traits::PlanNodeClonable for ExpandAllNode {
    fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_plan_node()
    }

    fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_with_new_id(new_id)
    }
}

/// 遍历节点
#[derive(Debug)]
pub struct TraverseNode {
    id: i64,

    space_id: i32,
    edge_types: Vec<String>,
    direction: String,
    step_limit: Option<u32>,
    filter: Option<String>,
    v_filter: Option<Expression>,
    e_filter: Option<Expression>,
    track_prev_path: bool,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
    dependencies: Vec<Box<super::plan_node_enum::PlanNodeEnum>>,
}

// 为 TraverseNode 实现 Clone
impl Clone for TraverseNode {
    fn clone(&self) -> Self {
        TraverseNode {
            id: self.id,
            space_id: self.space_id,
            edge_types: self.edge_types.clone(),
            direction: self.direction.clone(),
            step_limit: self.step_limit,
            filter: self.filter.clone(),
            v_filter: self.v_filter.clone(),
            e_filter: self.e_filter.clone(),
            track_prev_path: self.track_prev_path,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            dependencies: Vec::new(),
        }
    }
}

impl TraverseNode {
    pub fn new(space_id: i32, edge_types: Vec<String>, direction: &str) -> Self {
        Self {
            id: -1,
            space_id,
            edge_types,
            direction: direction.to_string(),
            step_limit: None,
            filter: None,
            v_filter: None,
            e_filter: None,
            track_prev_path: false,
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            dependencies: Vec::new(),
        }
    }

    /// 获取是否追踪前序路径
    pub fn track_prev_path(&self) -> bool {
        self.track_prev_path
    }

    /// 设置是否追踪前序路径
    pub fn set_track_prev_path(&mut self, track: bool) {
        self.track_prev_path = track;
    }

    /// 获取边类型
    pub fn edge_types(&self) -> &[String] {
        &self.edge_types
    }

    /// 获取方向
    pub fn direction(&self) -> &str {
        &self.direction
    }

    /// 获取步数限制
    pub fn step_limit(&self) -> Option<u32> {
        self.step_limit
    }

    /// 检查是否为单步遍历
    pub fn is_one_step(&self) -> bool {
        self.step_limit == Some(1)
    }

    /// 检查是否为零步遍历
    pub fn is_zero_step(&self) -> bool {
        self.step_limit == Some(0)
    }

    /// 获取过滤条件
    pub fn filter(&self) -> Option<&String> {
        self.filter.as_ref()
    }

    /// 设置过滤条件
    pub fn set_filter(&mut self, filter: String) {
        self.filter = Some(filter);
    }

    /// 获取顶点过滤表达式
    pub fn v_filter(&self) -> Option<&Expression> {
        self.v_filter.as_ref()
    }

    /// 设置顶点过滤表达式
    pub fn set_v_filter(&mut self, v_filter: Expression) {
        self.v_filter = Some(v_filter);
    }

    /// 获取边过滤表达式
    pub fn e_filter(&self) -> Option<&Expression> {
        self.e_filter.as_ref()
    }

    /// 设置边过滤表达式
    pub fn set_e_filter(&mut self, e_filter: Expression) {
        self.e_filter = Some(e_filter);
    }

    /// 获取边别名
    /// 参考 nebula-graph 的 edgeAlias() 实现
    pub fn edge_alias(&self) -> Option<&str> {
        let col_names = &self.col_names;
        if col_names.is_empty() {
            return None;
        }
        let n = col_names.len();
        Some(&col_names[n - 1])
    }

    /// 获取节点别名
    /// 参考 nebula-graph 的 nodeAlias() 实现
    pub fn node_alias(&self) -> Option<&str> {
        let col_names = &self.col_names;
        if col_names.len() < 2 {
            return None;
        }
        Some(&col_names[col_names.len() - 2])
    }
}

impl TraverseNode {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "Traverse"
    }

    pub fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    pub fn cost(&self) -> f64 {
        self.cost
    }

    pub fn dependencies(&self) -> &[Box<super::plan_node_enum::PlanNodeEnum>] {
        &self.dependencies
    }

    pub fn add_dependency(&mut self, dep: super::plan_node_enum::PlanNodeEnum) {
        self.dependencies.push(Box::new(dep));
    }

    pub fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.dependencies.iter().position(|dep| dep.id() == id) {
            self.dependencies.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    pub fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::Traverse(self.clone())
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::Traverse(cloned)
    }
}

// 为 TraverseNode 实现 PlanNode trait
impl super::plan_node_traits::PlanNode for TraverseNode {
    fn id(&self) -> i64 {
        self.id()
    }

    fn name(&self) -> &'static str {
        self.type_name()
    }

    fn output_var(&self) -> Option<&Variable> {
        self.output_var()
    }

    fn col_names(&self) -> &[String] {
        self.col_names()
    }

    fn cost(&self) -> f64 {
        self.cost()
    }

    fn set_output_var(&mut self, var: Variable) {
        self.set_output_var(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.set_col_names(names);
    }

    fn into_enum(self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::Traverse(self)
    }
}

// 为 TraverseNode 实现 MultipleInputNode trait
impl super::plan_node_traits::MultipleInputNode for TraverseNode {
    fn inputs(&self) -> &[Box<super::plan_node_enum::PlanNodeEnum>] {
        &self.dependencies
    }

    fn add_input(&mut self, input: super::plan_node_enum::PlanNodeEnum) {
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

    fn input_count(&self) -> usize {
        self.dependencies.len()
    }
}

// 为 TraverseNode 实现 PlanNodeClonable trait
impl super::plan_node_traits::PlanNodeClonable for TraverseNode {
    fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_plan_node()
    }

    fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_with_new_id(new_id)
    }
}

/// 追加顶点节点
#[derive(Debug)]
pub struct AppendVerticesNode {
    id: i64,
    space_id: i32,
    vids: Vec<Value>,
    tag_ids: Vec<i32>,
    filter: Option<String>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
    dependencies: Vec<Box<super::plan_node_enum::PlanNodeEnum>>,
    input_var: Option<String>,
    src_expression: Option<Expression>,
    props: Vec<String>,
    v_filter: Option<Expression>,
    dedup: bool,
    track_prev_path: bool,
    need_fetch_prop: bool,
}

// 为 AppendVerticesNode 实现 Clone
impl Clone for AppendVerticesNode {
    fn clone(&self) -> Self {
        AppendVerticesNode {
            id: self.id,
            space_id: self.space_id,
            vids: self.vids.clone(),
            tag_ids: self.tag_ids.clone(),
            filter: self.filter.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            dependencies: Vec::new(),
            input_var: self.input_var.clone(),
            src_expression: self.src_expression.clone(),
            props: self.props.clone(),
            v_filter: self.v_filter.clone(),
            dedup: self.dedup,
            track_prev_path: self.track_prev_path,
            need_fetch_prop: self.need_fetch_prop,
        }
    }
}

impl AppendVerticesNode {
    pub fn new(space_id: i32, vids: Vec<Value>, tag_ids: Vec<i32>) -> Self {
        Self {
            id: -1,
            space_id,
            vids,
            tag_ids,
            filter: None,
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            dependencies: Vec::new(),
            input_var: None,
            src_expression: None,
            props: Vec::new(),
            v_filter: None,
            dedup: false,
            track_prev_path: true,
            need_fetch_prop: true,
        }
    }

    /// 获取空间ID
    pub fn space_id(&self) -> i32 {
        self.space_id
    }

    /// 获取顶点ID列表
    pub fn vids(&self) -> &[Value] {
        &self.vids
    }

    /// 获取标签ID列表
    pub fn tag_ids(&self) -> &[i32] {
        &self.tag_ids
    }

    /// 获取过滤条件
    pub fn filter(&self) -> Option<&String> {
        self.filter.as_ref()
    }

    /// 获取输入变量名
    pub fn input_var(&self) -> Option<&String> {
        self.input_var.as_ref()
    }

    /// 获取源表达式
    pub fn src_expression(&self) -> Option<&Expression> {
        self.src_expression.as_ref()
    }

    /// 获取属性列表
    pub fn props(&self) -> &[String] {
        &self.props
    }

    /// 获取顶点过滤表达式
    pub fn v_filter(&self) -> Option<&Expression> {
        self.v_filter.as_ref()
    }

    /// 是否去重
    pub fn dedup(&self) -> bool {
        self.dedup
    }

    /// 是否跟踪前一个路径
    pub fn track_prev_path(&self) -> bool {
        self.track_prev_path
    }

    /// 是否需要获取属性
    pub fn need_fetch_prop(&self) -> bool {
        self.need_fetch_prop
    }

    /// 设置输入变量名
    pub fn set_input_var(&mut self, input_var: String) {
        self.input_var = Some(input_var);
    }

    /// 设置源表达式
    pub fn set_src_expression(&mut self, src_expression: Expression) {
        self.src_expression = Some(src_expression);
    }

    /// 设置属性列表
    pub fn set_props(&mut self, props: Vec<String>) {
        self.props = props;
    }

    /// 设置顶点过滤表达式
    pub fn set_v_filter(&mut self, v_filter: Expression) {
        self.v_filter = Some(v_filter);
    }

    /// 设置是否去重
    pub fn set_dedup(&mut self, dedup: bool) {
        self.dedup = dedup;
    }

    /// 设置是否跟踪前一个路径
    pub fn set_track_prev_path(&mut self, track_prev_path: bool) {
        self.track_prev_path = track_prev_path;
    }

    /// 设置是否需要获取属性
    pub fn set_need_fetch_prop(&mut self, need_fetch_prop: bool) {
        self.need_fetch_prop = need_fetch_prop;
    }

    /// 设置过滤条件
    pub fn set_filter(&mut self, filter: String) {
        self.filter = Some(filter);
    }
}

impl AppendVerticesNode {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "AppendVertices"
    }

    pub fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    pub fn cost(&self) -> f64 {
        self.cost
    }

    pub fn dependencies(&self) -> &[Box<super::plan_node_enum::PlanNodeEnum>] {
        &self.dependencies
    }

    pub fn add_dependency(&mut self, dep: super::plan_node_enum::PlanNodeEnum) {
        self.dependencies.push(Box::new(dep));
    }

    pub fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.dependencies.iter().position(|dep| dep.id() == id) {
            self.dependencies.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    pub fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::AppendVertices(self.clone())
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::AppendVertices(cloned)
    }
}

// 为 AppendVerticesNode 实现 PlanNode trait
impl super::plan_node_traits::PlanNode for AppendVerticesNode {
    fn id(&self) -> i64 {
        self.id()
    }

    fn name(&self) -> &'static str {
        self.type_name()
    }

    fn output_var(&self) -> Option<&Variable> {
        self.output_var()
    }

    fn col_names(&self) -> &[String] {
        self.col_names()
    }

    fn cost(&self) -> f64 {
        self.cost()
    }

    fn set_output_var(&mut self, var: Variable) {
        self.set_output_var(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.set_col_names(names);
    }

    fn into_enum(self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::AppendVertices(self)
    }
}

// 为 AppendVerticesNode 实现 MultipleInputNode trait
impl super::plan_node_traits::MultipleInputNode for AppendVerticesNode {
    fn inputs(&self) -> &[Box<super::plan_node_enum::PlanNodeEnum>] {
        &self.dependencies
    }

    fn add_input(&mut self, input: super::plan_node_enum::PlanNodeEnum) {
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

    fn input_count(&self) -> usize {
        self.dependencies.len()
    }
}

// 为 AppendVerticesNode 实现 PlanNodeClonable trait
impl super::plan_node_traits::PlanNodeClonable for AppendVerticesNode {
    fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_plan_node()
    }

    fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_with_new_id(new_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_node_creation() {
        let edge_types = vec!["edge1".to_string(), "edge2".to_string()];
        let node = ExpandNode::new(1, edge_types, EdgeDirection::Out);
        assert_eq!(node.type_name(), "Expand");
        assert_eq!(node.space_id, 1);
        assert_eq!(node.direction, EdgeDirection::Out);
        assert_eq!(node.edge_types.len(), 2);
    }

    #[test]
    fn test_traverse_node_creation() {
        let edge_types = vec!["edge1".to_string()];
        let node = TraverseNode::new(1, edge_types, "BOTH");
        assert_eq!(node.type_name(), "Traverse");
        assert_eq!(node.space_id, 1);
        assert_eq!(node.direction, "BOTH");
    }

    #[test]
    fn test_append_vertices_node_creation() {
        let vids = vec![Value::String("vid1".to_string())];
        let tag_ids = vec![1, 2];
        let node = AppendVerticesNode::new(1, vids, tag_ids);
        assert_eq!(node.type_name(), "AppendVertices");
        assert_eq!(node.space_id, 1);
        assert_eq!(node.vids.len(), 1);
        assert_eq!(node.tag_ids.len(), 2);
    }
}
