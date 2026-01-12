//! 索引操作相关的计划节点
//! 包括创建/删除索引等操作

use crate::query::planner::plan::core::nodes::management_node_enum::ManagementNodeEnum;
use crate::query::planner::plan::core::nodes::management_node_traits::ManagementNode;

/// 索引类型枚举
#[derive(Debug, Clone)]
pub enum IndexType {
    Secondary,
    Unique,
    Fulltext,
}

/// 索引状态枚举
#[derive(Debug, Clone)]
pub enum IndexStatus {
    Creating,
    Active,
    Failed,
    Dropped,
}

/// 创建标签索引计划节点
#[derive(Debug, Clone)]
pub struct CreateTagIndex {
    pub id: i64,
    pub cost: f64,
    pub if_not_exists: bool,
    pub index_name: String,
    pub tag_name: String,
    pub fields: Vec<String>,
    pub index_type: IndexType,
}

impl CreateTagIndex {
    pub fn new(
        id: i64,
        cost: f64,
        if_not_exists: bool,
        index_name: &str,
        tag_name: &str,
        fields: Vec<String>,
        index_type: IndexType,
    ) -> Self {
        Self {
            id,
            cost,
            if_not_exists,
            index_name: index_name.to_string(),
            tag_name: tag_name.to_string(),
            fields,
            index_type,
        }
    }

    pub fn if_not_exists(&self) -> bool {
        self.if_not_exists
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }

    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }

    pub fn fields(&self) -> &[String] {
        &self.fields
    }

    pub fn index_type(&self) -> &IndexType {
        &self.index_type
    }
}

impl ManagementNode for CreateTagIndex {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "CreateTagIndex"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::CreateTagIndex(self)
    }
}

/// 创建边索引计划节点
#[derive(Debug, Clone)]
pub struct CreateEdgeIndex {
    pub id: i64,
    pub cost: f64,
    pub if_not_exists: bool,
    pub index_name: String,
    pub edge_name: String,
    pub fields: Vec<String>,
    pub index_type: IndexType,
}

impl CreateEdgeIndex {
    pub fn new(
        id: i64,
        cost: f64,
        if_not_exists: bool,
        index_name: &str,
        edge_name: &str,
        fields: Vec<String>,
        index_type: IndexType,
    ) -> Self {
        Self {
            id,
            cost,
            if_not_exists,
            index_name: index_name.to_string(),
            edge_name: edge_name.to_string(),
            fields,
            index_type,
        }
    }

    pub fn if_not_exists(&self) -> bool {
        self.if_not_exists
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }

    pub fn edge_name(&self) -> &str {
        &self.edge_name
    }

    pub fn fields(&self) -> &[String] {
        &self.fields
    }

    pub fn index_type(&self) -> &IndexType {
        &self.index_type
    }
}

impl ManagementNode for CreateEdgeIndex {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "CreateEdgeIndex"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::CreateEdgeIndex(self)
    }
}

/// 创建索引计划节点（已弃用，使用CreateTagIndex或CreateEdgeIndex）
#[derive(Debug, Clone)]
pub struct CreateIndex {
    pub id: i64,
    pub cost: f64,
    pub if_not_exists: bool,
    pub index_name: String,
    pub schema_name: String,
    pub fields: Vec<String>,
}

impl CreateIndex {
    pub fn new(
        id: i64,
        cost: f64,
        if_not_exists: bool,
        index_name: &str,
        schema_name: &str,
        fields: Vec<String>,
    ) -> Self {
        Self {
            id,
            cost,
            if_not_exists,
            index_name: index_name.to_string(),
            schema_name: schema_name.to_string(),
            fields,
        }
    }

    pub fn if_not_exists(&self) -> bool {
        self.if_not_exists
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }

    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }

    pub fn fields(&self) -> &[String] {
        &self.fields
    }
}

impl ManagementNode for CreateIndex {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "CreateIndex"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::CreateIndex(self)
    }
}

/// 删除标签索引计划节点
#[derive(Debug, Clone)]
pub struct DropTagIndex {
    pub id: i64,
    pub cost: f64,
    pub if_exists: bool,
    pub index_name: String,
}

impl DropTagIndex {
    pub fn new(id: i64, cost: f64, if_exists: bool, index_name: &str) -> Self {
        Self {
            id,
            cost,
            if_exists,
            index_name: index_name.to_string(),
        }
    }

    pub fn if_exists(&self) -> bool {
        self.if_exists
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }
}

impl ManagementNode for DropTagIndex {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DropTagIndex"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::DropTagIndex(self)
    }
}

/// 删除边索引计划节点
#[derive(Debug, Clone)]
pub struct DropEdgeIndex {
    pub id: i64,
    pub cost: f64,
    pub if_exists: bool,
    pub index_name: String,
}

impl DropEdgeIndex {
    pub fn new(id: i64, cost: f64, if_exists: bool, index_name: &str) -> Self {
        Self {
            id,
            cost,
            if_exists,
            index_name: index_name.to_string(),
        }
    }

    pub fn if_exists(&self) -> bool {
        self.if_exists
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }
}

impl ManagementNode for DropEdgeIndex {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DropEdgeIndex"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::DropEdgeIndex(self)
    }
}

/// 删除索引计划节点（已弃用，使用DropTagIndex或DropEdgeIndex）
#[derive(Debug, Clone)]
pub struct DropIndex {
    pub id: i64,
    pub cost: f64,
    pub if_exists: bool,
    pub index_name: String,
}

impl DropIndex {
    pub fn new(id: i64, cost: f64, if_exists: bool, index_name: &str) -> Self {
        Self {
            id,
            cost,
            if_exists,
            index_name: index_name.to_string(),
        }
    }

    pub fn if_exists(&self) -> bool {
        self.if_exists
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }
}

impl ManagementNode for DropIndex {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DropIndex"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::DropIndex(self)
    }
}

/// 显示标签索引计划节点
#[derive(Debug, Clone)]
pub struct ShowTagIndexes {
    pub id: i64,
    pub cost: f64,
    pub tag_name: Option<String>,
}

impl ShowTagIndexes {
    pub fn new(id: i64, cost: f64, tag_name: Option<String>) -> Self {
        Self { id, cost, tag_name }
    }

    pub fn tag_name(&self) -> Option<&str> {
        self.tag_name.as_deref()
    }
}

impl ManagementNode for ShowTagIndexes {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowTagIndexes"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::ShowTagIndexes(self)
    }
}

/// 显示边索引计划节点
#[derive(Debug, Clone)]
pub struct ShowEdgeIndexes {
    pub id: i64,
    pub cost: f64,
    pub edge_name: Option<String>,
}

impl ShowEdgeIndexes {
    pub fn new(id: i64, cost: f64, edge_name: Option<String>) -> Self {
        Self {
            id,
            cost,
            edge_name,
        }
    }

    pub fn edge_name(&self) -> Option<&str> {
        self.edge_name.as_deref()
    }
}

impl ManagementNode for ShowEdgeIndexes {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowEdgeIndexes"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::ShowEdgeIndexes(self)
    }
}

/// 显示索引计划节点（已弃用，使用ShowTagIndexes或ShowEdgeIndexes）
#[derive(Debug, Clone)]
pub struct ShowIndexes {
    pub id: i64,
    pub cost: f64,
    pub schema_name: Option<String>,
}

impl ShowIndexes {
    pub fn new(id: i64, cost: f64, schema_name: Option<String>) -> Self {
        Self {
            id,
            cost,
            schema_name,
        }
    }

    pub fn schema_name(&self) -> Option<&str> {
        self.schema_name.as_deref()
    }
}

impl ManagementNode for ShowIndexes {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowIndexes"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::ShowIndexes(self)
    }
}

/// 描述索引计划节点
#[derive(Debug, Clone)]
pub struct DescIndex {
    pub id: i64,
    pub cost: f64,
    pub index_name: String,
}

impl DescIndex {
    pub fn new(id: i64, cost: f64, index_name: &str) -> Self {
        Self {
            id,
            cost,
            index_name: index_name.to_string(),
        }
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }
}

impl ManagementNode for DescIndex {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DescIndex"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::DescIndex(self)
    }
}

/// 显示索引状态计划节点
#[derive(Debug, Clone)]
pub struct ShowIndexStatus {
    pub id: i64,
    pub cost: f64,
    pub index_name: Option<String>,
}

impl ShowIndexStatus {
    pub fn new(id: i64, cost: f64, index_name: Option<String>) -> Self {
        Self {
            id,
            cost,
            index_name,
        }
    }

    pub fn index_name(&self) -> Option<&str> {
        self.index_name.as_deref()
    }
}

impl ManagementNode for ShowIndexStatus {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowIndexStatus"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::ShowIndexStatus(self)
    }
}
