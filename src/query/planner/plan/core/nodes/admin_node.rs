//! 管理计划节点定义
//!
//! 提供数据库管理操作（空间、标签、边类型、索引等）的计划节点定义。

use super::plan_node_enum::PlanNodeEnum;
use super::plan_node_traits::PlanNode;
use crate::core::types::PropertyDef;
use crate::query::context::validate::types::Variable;

/// 空间管理信息
#[derive(Debug, Clone)]
pub struct SpaceManageInfo {
    pub space_name: String,
    pub partition_num: usize,
    pub replica_factor: usize,
    pub vid_type: String,
}

impl SpaceManageInfo {
    pub fn new(space_name: String) -> Self {
        Self {
            space_name,
            partition_num: 1,
            replica_factor: 1,
            vid_type: "FIXED_STRING(32)".to_string(),
        }
    }

    pub fn with_partition_num(mut self, partition_num: usize) -> Self {
        self.partition_num = partition_num;
        self
    }

    pub fn with_replica_factor(mut self, replica_factor: usize) -> Self {
        self.replica_factor = replica_factor;
        self
    }

    pub fn with_vid_type(mut self, vid_type: String) -> Self {
        self.vid_type = vid_type;
        self
    }
}

/// 标签管理信息
#[derive(Debug, Clone)]
pub struct TagManageInfo {
    pub space_name: String,
    pub tag_name: String,
    pub properties: Vec<PropertyDef>,
}

impl TagManageInfo {
    pub fn new(space_name: String, tag_name: String) -> Self {
        Self {
            space_name,
            tag_name,
            properties: Vec::new(),
        }
    }

    pub fn with_properties(mut self, properties: Vec<PropertyDef>) -> Self {
        self.properties = properties;
        self
    }
}

/// 标签修改信息
#[derive(Debug, Clone)]
pub struct TagAlterInfo {
    pub space_name: String,
    pub tag_name: String,
    pub additions: Vec<PropertyDef>,
    pub deletions: Vec<String>,
}

impl TagAlterInfo {
    pub fn new(space_name: String, tag_name: String) -> Self {
        Self {
            space_name,
            tag_name,
            additions: Vec::new(),
            deletions: Vec::new(),
        }
    }

    pub fn with_additions(mut self, additions: Vec<PropertyDef>) -> Self {
        self.additions = additions;
        self
    }

    pub fn with_deletions(mut self, deletions: Vec<String>) -> Self {
        self.deletions = deletions;
        self
    }
}

/// 边类型管理信息
#[derive(Debug, Clone)]
pub struct EdgeManageInfo {
    pub space_name: String,
    pub edge_name: String,
    pub properties: Vec<PropertyDef>,
}

impl EdgeManageInfo {
    pub fn new(space_name: String, edge_name: String) -> Self {
        Self {
            space_name,
            edge_name,
            properties: Vec::new(),
        }
    }

    pub fn with_properties(mut self, properties: Vec<PropertyDef>) -> Self {
        self.properties = properties;
        self
    }
}

/// 边类型修改信息
#[derive(Debug, Clone)]
pub struct EdgeAlterInfo {
    pub space_name: String,
    pub edge_name: String,
    pub additions: Vec<PropertyDef>,
    pub deletions: Vec<String>,
}

impl EdgeAlterInfo {
    pub fn new(space_name: String, edge_name: String) -> Self {
        Self {
            space_name,
            edge_name,
            additions: Vec::new(),
            deletions: Vec::new(),
        }
    }

    pub fn with_additions(mut self, additions: Vec<PropertyDef>) -> Self {
        self.additions = additions;
        self
    }

    pub fn with_deletions(mut self, deletions: Vec<String>) -> Self {
        self.deletions = deletions;
        self
    }
}

/// 索引管理信息
#[derive(Debug, Clone)]
pub struct IndexManageInfo {
    pub space_name: String,
    pub index_name: String,
    pub target_type: String,
    pub target_name: String,
    pub properties: Vec<String>,
}

impl IndexManageInfo {
    pub fn new(space_name: String, index_name: String, target_type: String) -> Self {
        Self {
            space_name,
            index_name,
            target_type,
            target_name: String::new(),
            properties: Vec::new(),
        }
    }

    pub fn with_target_name(mut self, target_name: String) -> Self {
        self.target_name = target_name;
        self
    }

    pub fn with_properties(mut self, properties: Vec<String>) -> Self {
        self.properties = properties;
        self
    }
}

/// 创建图空间计划节点
#[derive(Debug, Clone)]
pub struct CreateSpaceNode {
    id: i64,
    info: SpaceManageInfo,
}

impl CreateSpaceNode {
    pub fn new(id: i64, info: SpaceManageInfo) -> Self {
        Self { id, info }
    }

    pub fn info(&self) -> &SpaceManageInfo {
        &self.info
    }
}

impl PlanNode for CreateSpaceNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "CreateSpace"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::CreateSpace(self)
    }
}

/// 删除图空间计划节点
#[derive(Debug, Clone)]
pub struct DropSpaceNode {
    id: i64,
    space_name: String,
}

impl DropSpaceNode {
    pub fn new(id: i64, space_name: String) -> Self {
        Self { id, space_name }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }
}

impl PlanNode for DropSpaceNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DropSpace"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::DropSpace(self)
    }
}

/// 描述图空间计划节点
#[derive(Debug, Clone)]
pub struct DescSpaceNode {
    id: i64,
    space_name: String,
}

impl DescSpaceNode {
    pub fn new(id: i64, space_name: String) -> Self {
        Self { id, space_name }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }
}

impl PlanNode for DescSpaceNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DescSpace"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::DescSpace(self)
    }
}

/// 显示所有图空间计划节点
#[derive(Debug, Clone)]
pub struct ShowSpacesNode {
    id: i64,
}

impl ShowSpacesNode {
    pub fn new(id: i64) -> Self {
        Self { id }
    }
}

impl PlanNode for ShowSpacesNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowSpaces"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::ShowSpaces(self)
    }
}

/// 创建标签计划节点
#[derive(Debug, Clone)]
pub struct CreateTagNode {
    id: i64,
    info: TagManageInfo,
}

impl CreateTagNode {
    pub fn new(id: i64, info: TagManageInfo) -> Self {
        Self { id, info }
    }

    pub fn info(&self) -> &TagManageInfo {
        &self.info
    }

    pub fn space_name(&self) -> &str {
        &self.info.space_name
    }

    pub fn tag_name(&self) -> &str {
        &self.info.tag_name
    }
}

impl PlanNode for CreateTagNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "CreateTag"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::CreateTag(self)
    }
}

/// 修改标签计划节点
#[derive(Debug, Clone)]
pub struct AlterTagNode {
    id: i64,
    info: TagAlterInfo,
}

impl AlterTagNode {
    pub fn new(id: i64, info: TagAlterInfo) -> Self {
        Self { id, info }
    }

    pub fn info(&self) -> &TagAlterInfo {
        &self.info
    }

    pub fn space_name(&self) -> &str {
        &self.info.space_name
    }

    pub fn tag_name(&self) -> &str {
        &self.info.tag_name
    }
}

impl PlanNode for AlterTagNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "AlterTag"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::AlterTag(self)
    }
}

/// 描述标签计划节点
#[derive(Debug, Clone)]
pub struct DescTagNode {
    id: i64,
    space_name: String,
    tag_name: String,
}

impl DescTagNode {
    pub fn new(id: i64, space_name: String, tag_name: String) -> Self {
        Self { id, space_name, tag_name }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }
}

impl PlanNode for DescTagNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DescTag"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::DescTag(self)
    }
}

/// 删除标签计划节点
#[derive(Debug, Clone)]
pub struct DropTagNode {
    id: i64,
    space_name: String,
    tag_name: String,
}

impl DropTagNode {
    pub fn new(id: i64, space_name: String, tag_name: String) -> Self {
        Self { id, space_name, tag_name }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }
}

impl PlanNode for DropTagNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DropTag"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::DropTag(self)
    }
}

/// 显示所有标签计划节点
#[derive(Debug, Clone)]
pub struct ShowTagsNode {
    id: i64,
}

impl ShowTagsNode {
    pub fn new(id: i64) -> Self {
        Self { id }
    }
}

impl PlanNode for ShowTagsNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowTags"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::ShowTags(self)
    }
}

/// 创建边类型计划节点
#[derive(Debug, Clone)]
pub struct CreateEdgeNode {
    id: i64,
    info: EdgeManageInfo,
}

impl CreateEdgeNode {
    pub fn new(id: i64, info: EdgeManageInfo) -> Self {
        Self { id, info }
    }

    pub fn info(&self) -> &EdgeManageInfo {
        &self.info
    }

    pub fn space_name(&self) -> &str {
        &self.info.space_name
    }

    pub fn edge_name(&self) -> &str {
        &self.info.edge_name
    }
}

impl PlanNode for CreateEdgeNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "CreateEdge"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::CreateEdge(self)
    }
}

/// 修改边类型计划节点
#[derive(Debug, Clone)]
pub struct AlterEdgeNode {
    id: i64,
    info: EdgeAlterInfo,
}

impl AlterEdgeNode {
    pub fn new(id: i64, info: EdgeAlterInfo) -> Self {
        Self { id, info }
    }

    pub fn info(&self) -> &EdgeAlterInfo {
        &self.info
    }

    pub fn space_name(&self) -> &str {
        &self.info.space_name
    }

    pub fn edge_name(&self) -> &str {
        &self.info.edge_name
    }
}

impl PlanNode for AlterEdgeNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "AlterEdge"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::AlterEdge(self)
    }
}

/// 描述边类型计划节点
#[derive(Debug, Clone)]
pub struct DescEdgeNode {
    id: i64,
    space_name: String,
    edge_name: String,
}

impl DescEdgeNode {
    pub fn new(id: i64, space_name: String, edge_name: String) -> Self {
        Self { id, space_name, edge_name }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn edge_name(&self) -> &str {
        &self.edge_name
    }
}

impl PlanNode for DescEdgeNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DescEdge"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::DescEdge(self)
    }
}

/// 删除边类型计划节点
#[derive(Debug, Clone)]
pub struct DropEdgeNode {
    id: i64,
    space_name: String,
    edge_name: String,
}

impl DropEdgeNode {
    pub fn new(id: i64, space_name: String, edge_name: String) -> Self {
        Self { id, space_name, edge_name }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn edge_name(&self) -> &str {
        &self.edge_name
    }
}

impl PlanNode for DropEdgeNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DropEdge"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::DropEdge(self)
    }
}

/// 显示所有边类型计划节点
#[derive(Debug, Clone)]
pub struct ShowEdgesNode {
    id: i64,
}

impl ShowEdgesNode {
    pub fn new(id: i64) -> Self {
        Self { id }
    }
}

impl PlanNode for ShowEdgesNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowEdges"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::ShowEdges(self)
    }
}

/// 创建标签索引计划节点
#[derive(Debug, Clone)]
pub struct CreateTagIndexNode {
    id: i64,
    info: IndexManageInfo,
}

impl CreateTagIndexNode {
    pub fn new(id: i64, info: IndexManageInfo) -> Self {
        Self { id, info }
    }

    pub fn info(&self) -> &IndexManageInfo {
        &self.info
    }

    pub fn space_name(&self) -> &str {
        &self.info.space_name
    }

    pub fn index_name(&self) -> &str {
        &self.info.index_name
    }
}

impl PlanNode for CreateTagIndexNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "CreateTagIndex"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::CreateTagIndex(self)
    }
}

/// 删除标签索引计划节点
#[derive(Debug, Clone)]
pub struct DropTagIndexNode {
    id: i64,
    space_name: String,
    index_name: String,
}

impl DropTagIndexNode {
    pub fn new(id: i64, space_name: String, index_name: String) -> Self {
        Self { id, space_name, index_name }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }
}

impl PlanNode for DropTagIndexNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DropTagIndex"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::DropTagIndex(self)
    }
}

/// 描述标签索引计划节点
#[derive(Debug, Clone)]
pub struct DescTagIndexNode {
    id: i64,
    space_name: String,
    index_name: String,
}

impl DescTagIndexNode {
    pub fn new(id: i64, space_name: String, index_name: String) -> Self {
        Self { id, space_name, index_name }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }
}

impl PlanNode for DescTagIndexNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DescTagIndex"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::DescTagIndex(self)
    }
}

/// 显示所有标签索引计划节点
#[derive(Debug, Clone)]
pub struct ShowTagIndexesNode {
    id: i64,
}

impl ShowTagIndexesNode {
    pub fn new(id: i64) -> Self {
        Self { id }
    }
}

impl PlanNode for ShowTagIndexesNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowTagIndexes"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::ShowTagIndexes(self)
    }
}

/// 创建边索引计划节点
#[derive(Debug, Clone)]
pub struct CreateEdgeIndexNode {
    id: i64,
    info: IndexManageInfo,
}

impl CreateEdgeIndexNode {
    pub fn new(id: i64, info: IndexManageInfo) -> Self {
        Self { id, info }
    }

    pub fn info(&self) -> &IndexManageInfo {
        &self.info
    }

    pub fn space_name(&self) -> &str {
        &self.info.space_name
    }

    pub fn index_name(&self) -> &str {
        &self.info.index_name
    }
}

impl PlanNode for CreateEdgeIndexNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "CreateEdgeIndex"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::CreateEdgeIndex(self)
    }
}

/// 删除边索引计划节点
#[derive(Debug, Clone)]
pub struct DropEdgeIndexNode {
    id: i64,
    space_name: String,
    index_name: String,
}

impl DropEdgeIndexNode {
    pub fn new(id: i64, space_name: String, index_name: String) -> Self {
        Self { id, space_name, index_name }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }
}

impl PlanNode for DropEdgeIndexNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DropEdgeIndex"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::DropEdgeIndex(self)
    }
}

/// 描述边索引计划节点
#[derive(Debug, Clone)]
pub struct DescEdgeIndexNode {
    id: i64,
    space_name: String,
    index_name: String,
}

impl DescEdgeIndexNode {
    pub fn new(id: i64, space_name: String, index_name: String) -> Self {
        Self { id, space_name, index_name }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }
}

impl PlanNode for DescEdgeIndexNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DescEdgeIndex"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::DescEdgeIndex(self)
    }
}

/// 显示所有边索引计划节点
#[derive(Debug, Clone)]
pub struct ShowEdgeIndexesNode {
    id: i64,
}

impl ShowEdgeIndexesNode {
    pub fn new(id: i64) -> Self {
        Self { id }
    }
}

impl PlanNode for ShowEdgeIndexesNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowEdgeIndexes"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::ShowEdgeIndexes(self)
    }
}

/// 重建标签索引计划节点
#[derive(Debug, Clone)]
pub struct RebuildTagIndexNode {
    id: i64,
    space_name: String,
    index_name: String,
}

impl RebuildTagIndexNode {
    pub fn new(id: i64, space_name: String, index_name: String) -> Self {
        Self { id, space_name, index_name }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }
}

impl PlanNode for RebuildTagIndexNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "RebuildTagIndex"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::RebuildTagIndex(self)
    }
}

/// 重建边索引计划节点
#[derive(Debug, Clone)]
pub struct RebuildEdgeIndexNode {
    id: i64,
    space_name: String,
    index_name: String,
}

impl RebuildEdgeIndexNode {
    pub fn new(id: i64, space_name: String, index_name: String) -> Self {
        Self { id, space_name, index_name }
    }

    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }
}

impl PlanNode for RebuildEdgeIndexNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "RebuildEdgeIndex"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::RebuildEdgeIndex(self)
    }
}

/// 创建用户计划节点
#[derive(Debug, Clone)]
pub struct CreateUserNode {
    id: i64,
    username: String,
    password: String,
    role: String,
}

impl CreateUserNode {
    pub fn new(id: i64, username: String, password: String) -> Self {
        Self {
            id,
            username,
            password,
            role: "user".to_string(),
        }
    }

    pub fn with_role(mut self, role: String) -> Self {
        self.role = role;
        self
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn role(&self) -> &str {
        &self.role
    }
}

impl PlanNode for CreateUserNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "CreateUser"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::CreateUser(self)
    }
}

/// 修改用户计划节点
#[derive(Debug, Clone)]
pub struct AlterUserNode {
    id: i64,
    username: String,
    new_role: Option<String>,
    is_locked: Option<bool>,
}

impl AlterUserNode {
    pub fn new(id: i64, username: String) -> Self {
        Self {
            id,
            username,
            new_role: None,
            is_locked: None,
        }
    }

    pub fn with_role(mut self, role: String) -> Self {
        self.new_role = Some(role);
        self
    }

    pub fn with_locked(mut self, is_locked: bool) -> Self {
        self.is_locked = Some(is_locked);
        self
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn new_role(&self) -> Option<&String> {
        self.new_role.as_ref()
    }

    pub fn is_locked(&self) -> Option<bool> {
        self.is_locked
    }
}

impl PlanNode for AlterUserNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "AlterUser"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::AlterUser(self)
    }
}

/// 删除用户计划节点
#[derive(Debug, Clone)]
pub struct DropUserNode {
    id: i64,
    username: String,
}

impl DropUserNode {
    pub fn new(id: i64, username: String) -> Self {
        Self { id, username }
    }

    pub fn username(&self) -> &str {
        &self.username
    }
}

impl PlanNode for DropUserNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DropUser"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::DropUser(self)
    }
}

#[derive(Debug, Clone)]
pub struct ChangePasswordNode {
    id: i64,
    password_info: crate::core::types::metadata::PasswordInfo,
}

impl ChangePasswordNode {
    pub fn new(id: i64, password_info: crate::core::types::metadata::PasswordInfo) -> Self {
        Self {
            id,
            password_info,
        }
    }

    pub fn password_info(&self) -> &crate::core::types::metadata::PasswordInfo {
        &self.password_info
    }
}

impl PlanNode for ChangePasswordNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ChangePassword"
    }

    fn output_var(&self) -> Option<&Variable> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn cost(&self) -> f64 {
        1.0
    }

    fn set_output_var(&mut self, _var: Variable) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::ChangePassword(self)
    }
}
