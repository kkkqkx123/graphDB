//! 数据构造操作相关的计划节点
//! 包括创建新顶点、标签和属性的操作

use crate::query::planner::plan::core::nodes::management_node_enum::ManagementNodeEnum;
use crate::query::planner::plan::core::nodes::management_node_traits::ManagementNode;

/// 创建新顶点计划节点
#[derive(Debug, Clone)]
pub struct NewVertex {
    pub id: i64,
    pub cost: f64,
    pub tag_id: i32,
    pub props: Vec<(String, String)>, // 属性名和值
}

impl NewVertex {
    pub fn new(id: i64, cost: f64, tag_id: i32, props: Vec<(String, String)>) -> Self {
        Self {
            id,
            cost,
            tag_id,
            props,
        }
    }

    pub fn tag_id(&self) -> i32 {
        self.tag_id
    }

    pub fn props(&self) -> &[(String, String)] {
        &self.props
    }
}

impl ManagementNode for NewVertex {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "NewVertex"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::NewVertex(self)
    }
}

/// 创建新标签计划节点
#[derive(Debug, Clone)]
pub struct NewTag {
    pub id: i64,
    pub cost: f64,
    pub tag_id: i32,
    pub props: Vec<(String, String)>, // 属性名和值
}

impl NewTag {
    pub fn new(id: i64, cost: f64, tag_id: i32, props: Vec<(String, String)>) -> Self {
        Self {
            id,
            cost,
            tag_id,
            props,
        }
    }

    pub fn tag_id(&self) -> i32 {
        self.tag_id
    }

    pub fn props(&self) -> &[(String, String)] {
        &self.props
    }
}

impl ManagementNode for NewTag {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "NewTag"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::NewTag(self)
    }
}

/// 创建新属性计划节点
#[derive(Debug, Clone)]
pub struct NewProp {
    pub id: i64,
    pub cost: f64,
    pub prop_name: String,
    pub prop_value: String,
}

impl NewProp {
    pub fn new(id: i64, cost: f64, prop_name: &str, prop_value: &str) -> Self {
        Self {
            id,
            cost,
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

impl ManagementNode for NewProp {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "NewProp"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::NewProp(self)
    }
}

/// 创建新边计划节点
#[derive(Debug, Clone)]
pub struct NewEdge {
    pub id: i64,
    pub cost: f64,
    pub edge_type_id: i32,
    pub props: Vec<(String, String)>, // 属性名和值
}

impl NewEdge {
    pub fn new(id: i64, cost: f64, edge_type_id: i32, props: Vec<(String, String)>) -> Self {
        Self {
            id,
            cost,
            edge_type_id,
            props,
        }
    }

    pub fn edge_type_id(&self) -> i32 {
        self.edge_type_id
    }

    pub fn props(&self) -> &[(String, String)] {
        &self.props
    }
}

impl ManagementNode for NewEdge {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "NewEdge"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::NewEdge(self)
    }
}
