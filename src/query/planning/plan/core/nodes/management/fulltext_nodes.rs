//! Full-text search plan nodes
//!
//! This module defines plan nodes for full-text search operations.

use crate::core::types::FulltextEngineType;
use crate::query::parser::ast::fulltext::{
    AlterIndexAction, FulltextMatchCondition, FulltextQueryExpr, IndexFieldDef, IndexOptions,
    OrderClause, WhereClause, YieldClause,
};
use crate::query::planning::plan::core::nodes::base::memory_estimation::MemoryEstimatable;
use crate::query::planning::plan::core::nodes::base::plan_node_traits::{PlanNode, ZeroInputNode};
use crate::query::planning::plan::core::nodes::base::plan_node_category::PlanNodeCategory;
use serde::{Deserialize, Serialize};

/// CREATE FULLTEXT INDEX plan node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFulltextIndexNode {
    pub index_name: String,
    pub schema_name: String,
    pub fields: Vec<IndexFieldDef>,
    pub engine_type: FulltextEngineType,
    pub options: IndexOptions,
    pub if_not_exists: bool,
}

impl CreateFulltextIndexNode {
    pub fn new(
        index_name: String,
        schema_name: String,
        fields: Vec<IndexFieldDef>,
        engine_type: FulltextEngineType,
        options: IndexOptions,
        if_not_exists: bool,
    ) -> Self {
        Self {
            index_name,
            schema_name,
            fields,
            engine_type,
            options,
            if_not_exists,
        }
    }
}

impl PlanNode for CreateFulltextIndexNode {
    fn id(&self) -> i64 {
        0
    }

    fn name(&self) -> &'static str {
        "CreateFulltextIndex"
    }

    fn category(&self) -> PlanNodeCategory {
        PlanNodeCategory::Management
    }

    fn output_var(&self) -> Option<&str> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn set_output_var(&mut self, _var: String) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum {
        crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum::CreateFulltextIndex(self)
    }
}

impl ZeroInputNode for CreateFulltextIndexNode {}

/// DROP FULLTEXT INDEX plan node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropFulltextIndexNode {
    pub index_name: String,
    pub if_exists: bool,
}

impl DropFulltextIndexNode {
    pub fn new(index_name: String, if_exists: bool) -> Self {
        Self {
            index_name,
            if_exists,
        }
    }
}

impl PlanNode for DropFulltextIndexNode {
    fn id(&self) -> i64 {
        0
    }

    fn name(&self) -> &'static str {
        "DropFulltextIndex"
    }

    fn category(&self) -> PlanNodeCategory {
        PlanNodeCategory::Management
    }

    fn output_var(&self) -> Option<&str> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn set_output_var(&mut self, _var: String) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum {
        crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum::DropFulltextIndex(self)
    }
}

impl ZeroInputNode for DropFulltextIndexNode {}

/// ALTER FULLTEXT INDEX plan node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlterFulltextIndexNode {
    pub index_name: String,
    pub actions: Vec<AlterIndexAction>,
}

impl AlterFulltextIndexNode {
    pub fn new(index_name: String, actions: Vec<AlterIndexAction>) -> Self {
        Self {
            index_name,
            actions,
        }
    }
}

impl PlanNode for AlterFulltextIndexNode {
    fn id(&self) -> i64 {
        0
    }

    fn name(&self) -> &'static str {
        "AlterFulltextIndex"
    }

    fn category(&self) -> PlanNodeCategory {
        PlanNodeCategory::Management
    }

    fn output_var(&self) -> Option<&str> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn set_output_var(&mut self, _var: String) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum {
        crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum::AlterFulltextIndex(self)
    }
}

impl ZeroInputNode for AlterFulltextIndexNode {}

/// SHOW FULLTEXT INDEX plan node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowFulltextIndexNode {
    pub pattern: Option<String>,
    pub from_schema: Option<String>,
}

impl ShowFulltextIndexNode {
    pub fn new(pattern: Option<String>, from_schema: Option<String>) -> Self {
        Self {
            pattern,
            from_schema,
        }
    }
}

impl PlanNode for ShowFulltextIndexNode {
    fn id(&self) -> i64 {
        0
    }

    fn name(&self) -> &'static str {
        "ShowFulltextIndex"
    }

    fn category(&self) -> PlanNodeCategory {
        PlanNodeCategory::Management
    }

    fn output_var(&self) -> Option<&str> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn set_output_var(&mut self, _var: String) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum {
        crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum::ShowFulltextIndex(self)
    }
}

impl ZeroInputNode for ShowFulltextIndexNode {}

/// DESCRIBE FULLTEXT INDEX plan node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescribeFulltextIndexNode {
    pub index_name: String,
}

impl DescribeFulltextIndexNode {
    pub fn new(index_name: String) -> Self {
        Self { index_name }
    }
}

impl PlanNode for DescribeFulltextIndexNode {
    fn id(&self) -> i64 {
        0
    }

    fn name(&self) -> &'static str {
        "DescribeFulltextIndex"
    }

    fn category(&self) -> PlanNodeCategory {
        PlanNodeCategory::Management
    }

    fn output_var(&self) -> Option<&str> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn set_output_var(&mut self, _var: String) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum {
        crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum::DescribeFulltextIndex(self)
    }
}

impl ZeroInputNode for DescribeFulltextIndexNode {}

/// Full-text search plan node (SEARCH statement)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulltextSearchNode {
    id: i64,
    pub index_name: String,
    pub query: FulltextQueryExpr,
    pub yield_clause: Option<YieldClause>,
    pub where_clause: Option<WhereClause>,
    pub order_clause: Option<OrderClause>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl FulltextSearchNode {
    pub fn new(
        index_name: String,
        query: FulltextQueryExpr,
        yield_clause: Option<YieldClause>,
        where_clause: Option<WhereClause>,
        order_clause: Option<OrderClause>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Self {
        use crate::query::planning::plan::core::node_id_generator::next_node_id;
        Self {
            id: next_node_id(),
            index_name,
            query,
            yield_clause,
            where_clause,
            order_clause,
            limit,
            offset,
        }
    }

    pub fn id(&self) -> i64 {
        self.id
    }
}

impl PlanNode for FulltextSearchNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "FulltextSearch"
    }

    fn category(&self) -> PlanNodeCategory {
        PlanNodeCategory::DataAccess
    }

    fn output_var(&self) -> Option<&str> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn set_output_var(&mut self, _var: String) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum {
        crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum::FulltextSearch(self)
    }
}

impl ZeroInputNode for FulltextSearchNode {}

/// Full-text lookup plan node (LOOKUP FULLTEXT statement)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulltextLookupNode {
    id: i64,
    pub schema_name: String,
    pub index_name: String,
    pub query: String,
    pub yield_clause: Option<YieldClause>,
    pub limit: Option<usize>,
}

impl FulltextLookupNode {
    pub fn new(
        schema_name: String,
        index_name: String,
        query: String,
        yield_clause: Option<YieldClause>,
        limit: Option<usize>,
    ) -> Self {
        use crate::query::planning::plan::core::node_id_generator::next_node_id;
        Self {
            id: next_node_id(),
            schema_name,
            index_name,
            query,
            yield_clause,
            limit,
        }
    }

    pub fn id(&self) -> i64 {
        self.id
    }
}

impl PlanNode for FulltextLookupNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "FulltextLookup"
    }

    fn category(&self) -> PlanNodeCategory {
        PlanNodeCategory::DataAccess
    }

    fn output_var(&self) -> Option<&str> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn set_output_var(&mut self, _var: String) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum {
        crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum::FulltextLookup(self)
    }
}

impl ZeroInputNode for FulltextLookupNode {}

/// Match with full-text plan node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchFulltextNode {
    pub pattern: String,
    pub fulltext_condition: FulltextMatchCondition,
    pub yield_clause: Option<YieldClause>,
}

impl MatchFulltextNode {
    pub fn new(
        pattern: String,
        fulltext_condition: FulltextMatchCondition,
        yield_clause: Option<YieldClause>,
    ) -> Self {
        Self {
            pattern,
            fulltext_condition,
            yield_clause,
        }
    }
}

impl PlanNode for MatchFulltextNode {
    fn id(&self) -> i64 {
        0
    }

    fn name(&self) -> &'static str {
        "MatchFulltext"
    }

    fn category(&self) -> PlanNodeCategory {
        PlanNodeCategory::DataAccess
    }

    fn output_var(&self) -> Option<&str> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn set_output_var(&mut self, _var: String) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(self) -> crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum {
        crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum::MatchFulltext(self)
    }
}

impl ZeroInputNode for MatchFulltextNode {}

impl MemoryEstimatable for CreateFulltextIndexNode {
    fn estimate_memory(&self) -> usize {
        let base = std::mem::size_of::<CreateFulltextIndexNode>();
        let index_name_size = std::mem::size_of::<String>() + self.index_name.capacity();
        let schema_name_size = std::mem::size_of::<String>() + self.schema_name.capacity();
        let fields_size = std::mem::size_of::<Vec<IndexFieldDef>>()
            + self.fields.iter().map(|f| std::mem::size_of::<IndexFieldDef>() + f.field_name.capacity()).sum::<usize>();
        let options_size = std::mem::size_of::<IndexOptions>();
        base + index_name_size + schema_name_size + fields_size + options_size
    }
}

impl MemoryEstimatable for DropFulltextIndexNode {
    fn estimate_memory(&self) -> usize {
        let base = std::mem::size_of::<DropFulltextIndexNode>();
        let index_name_size = std::mem::size_of::<String>() + self.index_name.capacity();
        base + index_name_size
    }
}

impl MemoryEstimatable for AlterFulltextIndexNode {
    fn estimate_memory(&self) -> usize {
        let base = std::mem::size_of::<AlterFulltextIndexNode>();
        let index_name_size = std::mem::size_of::<String>() + self.index_name.capacity();
        let actions_size = std::mem::size_of::<Vec<AlterIndexAction>>();
        base + index_name_size + actions_size
    }
}

impl MemoryEstimatable for ShowFulltextIndexNode {
    fn estimate_memory(&self) -> usize {
        let base = std::mem::size_of::<ShowFulltextIndexNode>();
        let pattern_size = self.pattern.as_ref().map(|s| std::mem::size_of::<String>() + s.capacity()).unwrap_or(0);
        let from_schema_size = self.from_schema.as_ref().map(|s| std::mem::size_of::<String>() + s.capacity()).unwrap_or(0);
        base + pattern_size + from_schema_size
    }
}

impl MemoryEstimatable for DescribeFulltextIndexNode {
    fn estimate_memory(&self) -> usize {
        let base = std::mem::size_of::<DescribeFulltextIndexNode>();
        let index_name_size = std::mem::size_of::<String>() + self.index_name.capacity();
        base + index_name_size
    }
}

impl MemoryEstimatable for FulltextSearchNode {
    fn estimate_memory(&self) -> usize {
        let base = std::mem::size_of::<FulltextSearchNode>();
        let index_name_size = std::mem::size_of::<String>() + self.index_name.capacity();
        let query_size = std::mem::size_of::<FulltextQueryExpr>();
        let yield_size = self.yield_clause.as_ref().map(|_| std::mem::size_of::<YieldClause>()).unwrap_or(0);
        let where_size = self.where_clause.as_ref().map(|_| std::mem::size_of::<WhereClause>()).unwrap_or(0);
        let order_size = self.order_clause.as_ref().map(|_| std::mem::size_of::<OrderClause>()).unwrap_or(0);
        let limit_size = self.limit.map(|_| std::mem::size_of::<usize>()).unwrap_or(0);
        let offset_size = self.offset.map(|_| std::mem::size_of::<usize>()).unwrap_or(0);
        base + index_name_size + query_size + yield_size + where_size + order_size + limit_size + offset_size
    }
}

impl MemoryEstimatable for FulltextLookupNode {
    fn estimate_memory(&self) -> usize {
        let base = std::mem::size_of::<FulltextLookupNode>();
        let schema_name_size = std::mem::size_of::<String>() + self.schema_name.capacity();
        let index_name_size = std::mem::size_of::<String>() + self.index_name.capacity();
        let query_size = std::mem::size_of::<String>() + self.query.capacity();
        let yield_size = self.yield_clause.as_ref().map(|_| std::mem::size_of::<YieldClause>()).unwrap_or(0);
        let limit_size = self.limit.map(|_| std::mem::size_of::<usize>()).unwrap_or(0);
        base + schema_name_size + index_name_size + query_size + yield_size + limit_size
    }
}

impl MemoryEstimatable for MatchFulltextNode {
    fn estimate_memory(&self) -> usize {
        let base = std::mem::size_of::<MatchFulltextNode>();
        let pattern_size = std::mem::size_of::<String>() + self.pattern.capacity();
        let condition_size = std::mem::size_of::<FulltextMatchCondition>();
        let yield_size = self.yield_clause.as_ref().map(|_| std::mem::size_of::<YieldClause>()).unwrap_or(0);
        base + pattern_size + condition_size + yield_size
    }
}
