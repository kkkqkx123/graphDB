//! 索引管理节点实现
//!
//! 提供索引管理相关的计划节点定义

use super::plan_node_enum::PlanNodeEnum;
use super::plan_node_traits::PlanNode;
use crate::query::context::validate::types::Variable;

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
