//! 数据构造操作相关的计划节点
//! 包括创建新顶点、标签和属性的操作

use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use std::sync::Arc;

/// 创建新顶点计划节点
#[derive(Debug, Clone)]
pub struct NewVertex {
    pub tag_id: i32,
    pub props: Vec<(String, String)>, // 属性名和值
}

impl NewVertex {
    pub fn new(tag_id: i32, props: Vec<(String, String)>) -> Self {
        Self { tag_id, props }
    }

    pub fn tag_id(&self) -> i32 {
        self.tag_id
    }

    pub fn props(&self) -> &[(String, String)] {
        &self.props
    }
}

impl From<NewVertex> for PlanNodeEnum {
    fn from(vertex: NewVertex) -> Self {
        PlanNodeEnum::NewVertex(Arc::new(vertex))
    }
}

/// 创建新标签计划节点
#[derive(Debug, Clone)]
pub struct NewTag {
    pub tag_id: i32,
    pub props: Vec<(String, String)>, // 属性名和值
}

impl NewTag {
    pub fn new(tag_id: i32, props: Vec<(String, String)>) -> Self {
        Self { tag_id, props }
    }

    pub fn tag_id(&self) -> i32 {
        self.tag_id
    }

    pub fn props(&self) -> &[(String, String)] {
        &self.props
    }
}

impl From<NewTag> for PlanNodeEnum {
    fn from(tag: NewTag) -> Self {
        PlanNodeEnum::NewTag(Arc::new(tag))
    }
}

/// 创建新属性计划节点
#[derive(Debug, Clone)]
pub struct NewProp {
    pub prop_name: String,
    pub prop_value: String,
}

impl NewProp {
    pub fn new(prop_name: &str, prop_value: &str) -> Self {
        Self {
            prop_name: prop_name.to_string(),
            prop_value: prop_value.to_string(),
        }
    }

    pub fn prop_name(&self) -> &str {
        &self.prop_name
    }

    pub fn prop_value(&self) -> &str {
        &self.prop_value
    }
}

impl From<NewProp> for PlanNodeEnum {
    fn from(prop: NewProp) -> Self {
        PlanNodeEnum::NewProp(Arc::new(prop))
    }
}

/// 创建新边计划节点
#[derive(Debug, Clone)]
pub struct NewEdge {
    pub edge_type_id: i32,
    pub props: Vec<(String, String)>, // 属性名和值
}

impl NewEdge {
    pub fn new(edge_type_id: i32, props: Vec<(String, String)>) -> Self {
        Self { edge_type_id, props }
    }

    pub fn edge_type_id(&self) -> i32 {
        self.edge_type_id
    }

    pub fn props(&self) -> &[(String, String)] {
        &self.props
    }
}

impl From<NewEdge> for PlanNodeEnum {
    fn from(edge: NewEdge) -> Self {
        PlanNodeEnum::NewEdge(Arc::new(edge))
    }
}