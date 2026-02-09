//! 边类型管理节点实现
//!
//! 提供边类型管理相关的计划节点定义

use super::plan_node_enum::PlanNodeEnum;
use super::plan_node_traits::PlanNode;
use crate::core::types::PropertyDef;
use crate::query::context::validate::types::Variable;

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
