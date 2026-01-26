//! 图扫描节点实现
//!
//! 包含获取顶点、边和邻居节点的计划节点

use super::super::common::{EdgeProp, TagProp};
use crate::core::Expression;
use crate::query::context::validate::types::Variable;

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
    dependencies: Vec<Box<super::plan_node_enum::PlanNodeEnum>>,
}

// 为 GetVerticesNode 实现 Clone
impl Clone for GetVerticesNode {
    fn clone(&self) -> Self {
        GetVerticesNode {
            id: self.id,
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
            dependencies: Vec::new(),
        }
    }
}

impl GetVerticesNode {
    pub fn new(space_id: i32, src_vids: &str) -> Self {
        Self {
            id: -1,
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

    /// 获取空间ID
    pub fn space_id(&self) -> i32 {
        self.space_id
    }

    /// 获取源顶点ID
    pub fn src_vids(&self) -> &str {
        &self.src_vids
    }

    /// 设置标签属性
    pub fn set_tag_props(&mut self, tag_props: Vec<TagProp>) {
        self.tag_props = tag_props;
    }
}

impl GetVerticesNode {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "GetVertices"
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
        super::plan_node_enum::PlanNodeEnum::GetVertices(self.clone())
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::GetVertices(cloned)
    }

    pub fn expression(&self) -> Option<&String> {
        self.expression.as_ref()
    }

    pub fn limit(&self) -> Option<i64> {
        self.limit
    }
}

// 为 GetVerticesNode 实现 PlanNode trait
impl super::plan_node_traits::PlanNode for GetVerticesNode {
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
        super::plan_node_enum::PlanNodeEnum::GetVertices(self)
    }
}

impl super::plan_node_traits::ZeroInputNode for GetVerticesNode {}

// 为 GetVerticesNode 实现 PlanNodeClonable trait
impl super::plan_node_traits::PlanNodeClonable for GetVerticesNode {
    fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_plan_node()
    }

    fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_with_new_id(new_id)
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

// 为 GetEdgesNode 实现 Clone
impl Clone for GetEdgesNode {
    fn clone(&self) -> Self {
        GetEdgesNode {
            id: self.id,
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
            id: -1,
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

    /// 获取空间ID
    pub fn space_id(&self) -> i32 {
        self.space_id
    }

    /// 获取源顶点
    pub fn src(&self) -> &str {
        &self.src
    }

    /// 获取边类型
    pub fn edge_type(&self) -> &str {
        &self.edge_type
    }

    /// 获取排名
    pub fn rank(&self) -> &str {
        &self.rank
    }

    /// 获取目标顶点
    pub fn dst(&self) -> &str {
        &self.dst
    }
}

impl GetEdgesNode {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "GetEdges"
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

    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    pub fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::GetEdges(self.clone())
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::GetEdges(cloned)
    }
}

// 为 GetEdgesNode 实现 PlanNode trait
impl super::plan_node_traits::PlanNode for GetEdgesNode {
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
        super::plan_node_enum::PlanNodeEnum::GetEdges(self)
    }
}

impl super::plan_node_traits::ZeroInputNode for GetEdgesNode {}

// 为 GetEdgesNode 实现 PlanNodeClonable trait
impl super::plan_node_traits::PlanNodeClonable for GetEdgesNode {
    fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_plan_node()
    }

    fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_with_new_id(new_id)
    }
}

/// 获取邻居节点
#[derive(Debug)]
pub struct GetNeighborsNode {
    id: i64,

    space_id: i32,

    src_vids: String,

    edge_types: Vec<String>,

    tag_props: Vec<TagProp>,

    edge_props: Vec<EdgeProp>,

    expression: Option<String>,

    dedup: bool,
    limit: Option<i64>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

// 为 GetNeighborsNode 实现 Clone
impl Clone for GetNeighborsNode {
    fn clone(&self) -> Self {
        GetNeighborsNode {
            id: self.id,
            space_id: self.space_id,
            src_vids: self.src_vids.clone(),
            edge_types: self.edge_types.clone(),
            tag_props: self.tag_props.clone(),
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

impl GetNeighborsNode {
    pub fn new(space_id: i32, src_vids: &str) -> Self {
        Self {
            id: -1,
            space_id,
            src_vids: src_vids.to_string(),
            edge_types: Vec::new(),
            tag_props: Vec::new(),
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

    pub fn src_vids(&self) -> &str {
        &self.src_vids
    }

    pub fn edge_types(&self) -> &[String] {
        &self.edge_types
    }

    pub fn tag_props(&self) -> &[TagProp] {
        &self.tag_props
    }

    pub fn edge_props(&self) -> &[EdgeProp] {
        &self.edge_props
    }

    pub fn expression(&self) -> Option<&String> {
        self.expression.as_ref()
    }

    pub fn dedup(&self) -> bool {
        self.dedup
    }

    pub fn limit(&self) -> Option<i64> {
        self.limit
    }
}

impl GetNeighborsNode {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "GetNeighbors"
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

    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    pub fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::GetNeighbors(self.clone())
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::GetNeighbors(cloned)
    }
}

// 为 GetNeighborsNode 实现 PlanNode trait
impl super::plan_node_traits::PlanNode for GetNeighborsNode {
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
        super::plan_node_enum::PlanNodeEnum::GetNeighbors(self)
    }
}

impl super::plan_node_traits::ZeroInputNode for GetNeighborsNode {}

// 为 GetNeighborsNode 实现 PlanNodeClonable trait
impl super::plan_node_traits::PlanNodeClonable for GetNeighborsNode {
    fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_plan_node()
    }

    fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_with_new_id(new_id)
    }
}

/// 扫描顶点节点
#[derive(Debug)]
pub struct ScanVerticesNode {
    id: i64,
    space_id: i32,
    tag_filter: Option<String>,
    vertex_filter: Option<String>,
    limit: Option<i64>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

// 为 ScanVerticesNode 实现 Clone
impl Clone for ScanVerticesNode {
    fn clone(&self) -> Self {
        ScanVerticesNode {
            id: self.id,
            space_id: self.space_id,
            tag_filter: self.tag_filter.clone(),
            vertex_filter: self.vertex_filter.clone(),
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
            id: -1,
            space_id,
            tag_filter: None,
            vertex_filter: None,
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
        self.tag_filter.is_some() || self.vertex_filter.is_some()
    }

    /// 获取空间ID
    pub fn space_id(&self) -> i32 {
        self.space_id
    }

    /// 获取标签过滤器
    pub fn tag_filter(&self) -> &Option<String> {
        &self.tag_filter
    }

    /// 获取顶点过滤器
    pub fn vertex_filter(&self) -> &Option<String> {
        &self.vertex_filter
    }

    /// 获取限制
    pub fn limit(&self) -> Option<i64> {
        self.limit
    }
}

impl ScanVerticesNode {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "ScanVertices"
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

    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    pub fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::ScanVertices(self.clone())
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::ScanVertices(cloned)
    }
}

// 为 ScanVerticesNode 实现 PlanNode trait
impl super::plan_node_traits::PlanNode for ScanVerticesNode {
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
        super::plan_node_enum::PlanNodeEnum::ScanVertices(self)
    }
}

impl super::plan_node_traits::ZeroInputNode for ScanVerticesNode {}

// 为 ScanVerticesNode 实现 PlanNodeClonable trait
impl super::plan_node_traits::PlanNodeClonable for ScanVerticesNode {
    fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_plan_node()
    }

    fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_with_new_id(new_id)
    }
}

/// 扫描边节点
#[derive(Debug)]
pub struct ScanEdgesNode {
    id: i64,
    space_id: i32,
    edge_type: String,
    limit: Option<i64>,
    filter: Option<String>,

    props: Vec<EdgeProp>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
    dependencies: Vec<Box<super::plan_node_enum::PlanNodeEnum>>,
}

// 为 ScanEdgesNode 实现 Clone
impl Clone for ScanEdgesNode {
    fn clone(&self) -> Self {
        ScanEdgesNode {
            id: self.id,
            space_id: self.space_id,
            edge_type: self.edge_type.clone(),
            limit: self.limit,
            filter: self.filter.clone(),
            props: self.props.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            dependencies: Vec::new(), // 依赖关系不复制，因为它们在新的上下文中无效
        }
    }
}

impl ScanEdgesNode {
    pub fn new(space_id: i32, edge_type: &str) -> Self {
        Self {
            id: -1,
            space_id,
            edge_type: edge_type.to_string(),
            limit: None,
            filter: None,
            props: Vec::new(),
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
        self.filter.is_some()
    }

    /// 获取空间ID
    pub fn space_id(&self) -> i32 {
        self.space_id
    }

    /// 获取边类型
    pub fn edge_type(&self) -> &str {
        &self.edge_type
    }

    /// 获取限制
    pub fn limit(&self) -> Option<i64> {
        self.limit
    }

    /// 获取过滤条件
    pub fn filter(&self) -> Option<&String> {
        self.filter.as_ref()
    }
}

impl ScanEdgesNode {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "ScanEdges"
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
        super::plan_node_enum::PlanNodeEnum::ScanEdges(self.clone())
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::ScanEdges(cloned)
    }
}

// 为 ScanEdgesNode 实现 PlanNode trait
impl super::plan_node_traits::PlanNode for ScanEdgesNode {
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
        super::plan_node_enum::PlanNodeEnum::ScanEdges(self)
    }
}

// 为 ScanEdgesNode 实现 PlanNodeClonable trait
impl super::plan_node_traits::PlanNodeClonable for ScanEdgesNode {
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
    fn test_get_vertices_node_creation() {
        let node = GetVerticesNode::new(1, "vids");
        assert_eq!(node.type_name(), "GetVertices");
        assert_eq!(node.space_id, 1);
        assert_eq!(node.src_vids, "vids");
    }

    #[test]
    fn test_get_edges_node_creation() {
        let node = GetEdgesNode::new(1, "src", "edge", "0", "dst");
        assert_eq!(node.type_name(), "GetEdges");
        assert_eq!(node.space_id, 1);
        assert_eq!(node.src, "src");
        assert_eq!(node.edge_type, "edge");
    }

    #[test]
    fn test_scan_vertices_node_creation() {
        let node = ScanVerticesNode::new(1);
        assert_eq!(node.type_name(), "ScanVertices");
        assert_eq!(node.space_id, 1);
    }
}
