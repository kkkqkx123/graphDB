//! Vector Search Plan Nodes

use crate::core::types::expr::contextual::ContextualExpression;
use crate::core::types::span::Span;
use crate::query::parser::ast::vector::{VectorDistance, VectorQueryExpr};
use crate::query::planning::plan::core::node_id_generator::next_node_id;
use crate::query::planning::plan::core::nodes::base::plan_node_category::PlanNodeCategory;
use crate::query::planning::plan::core::nodes::base::plan_node_traits::{PlanNode, ZeroInputNode};
use serde::{Deserialize, Serialize};

/// Output field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputField {
    pub name: String,
    pub alias: Option<String>,
}

/// Vector search plan node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchNode {
    id: i64,
    pub index_name: String,
    pub space_id: u64,
    pub tag_name: String,
    pub field_name: String,
    pub query: VectorQueryExpr,
    pub threshold: Option<f32>,
    pub filter: Option<ContextualExpression>,
    pub limit: usize,
    pub offset: usize,
    pub output_fields: Vec<OutputField>,
}

impl VectorSearchNode {
    pub fn new(
        index_name: String,
        space_id: u64,
        tag_name: String,
        field_name: String,
        query: VectorQueryExpr,
        threshold: Option<f32>,
        filter: Option<ContextualExpression>,
        limit: usize,
        offset: usize,
        output_fields: Vec<OutputField>,
    ) -> Self {
        Self {
            id: next_node_id(),
            index_name,
            space_id,
            tag_name,
            field_name,
            query,
            threshold,
            filter,
            limit,
            offset,
            output_fields,
        }
    }
}

impl PlanNode for VectorSearchNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "VectorSearch"
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

    fn into_enum(
        self,
    ) -> crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum {
        crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum::VectorSearch(self)
    }
}

impl ZeroInputNode for VectorSearchNode {}

/// Create vector index plan node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVectorIndexNode {
    id: i64,
    pub index_name: String,
    pub space_name: String,
    pub tag_name: String,
    pub field_name: String,
    pub vector_size: usize,
    pub distance: VectorDistance,
    pub hnsw_m: Option<usize>,
    pub hnsw_ef_construct: Option<usize>,
    pub if_not_exists: bool,
}

impl CreateVectorIndexNode {
    pub fn new(
        index_name: String,
        space_name: String,
        tag_name: String,
        field_name: String,
        vector_size: usize,
        distance: VectorDistance,
        hnsw_m: Option<usize>,
        hnsw_ef_construct: Option<usize>,
        if_not_exists: bool,
    ) -> Self {
        Self {
            id: next_node_id(),
            index_name,
            space_name,
            tag_name,
            field_name,
            vector_size,
            distance,
            hnsw_m,
            hnsw_ef_construct,
            if_not_exists,
        }
    }
}

impl PlanNode for CreateVectorIndexNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "CreateVectorIndex"
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

    fn into_enum(
        self,
    ) -> crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum {
        crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum::CreateVectorIndex(self)
    }
}

impl ZeroInputNode for CreateVectorIndexNode {}

/// Drop vector index plan node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropVectorIndexNode {
    id: i64,
    pub index_name: String,
    pub space_name: String,
    pub if_exists: bool,
}

impl DropVectorIndexNode {
    pub fn new(index_name: String, space_name: String, if_exists: bool) -> Self {
        Self {
            id: next_node_id(),
            index_name,
            space_name,
            if_exists,
        }
    }
}

impl PlanNode for DropVectorIndexNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DropVectorIndex"
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

    fn into_enum(
        self,
    ) -> crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum {
        crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum::DropVectorIndex(self)
    }
}

impl ZeroInputNode for DropVectorIndexNode {}

/// Lookup vector plan node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorLookupNode {
    id: i64,
    pub schema_name: String,
    pub index_name: String,
    pub query: VectorQueryExpr,
    pub yield_fields: Vec<OutputField>,
    pub limit: usize,
}

impl VectorLookupNode {
    pub fn new(
        schema_name: String,
        index_name: String,
        query: VectorQueryExpr,
        yield_fields: Vec<OutputField>,
        limit: usize,
    ) -> Self {
        Self {
            id: next_node_id(),
            schema_name,
            index_name,
            query,
            yield_fields,
            limit,
        }
    }
}

impl PlanNode for VectorLookupNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "VectorLookup"
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

    fn into_enum(
        self,
    ) -> crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum {
        crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum::VectorLookup(self)
    }
}

impl ZeroInputNode for VectorLookupNode {}

/// Match vector plan node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMatchNode {
    id: i64,
    pub pattern: String,
    pub field: String,
    pub query: VectorQueryExpr,
    pub threshold: Option<f32>,
    pub yield_fields: Vec<OutputField>,
}

impl VectorMatchNode {
    pub fn new(
        pattern: String,
        field: String,
        query: VectorQueryExpr,
        threshold: Option<f32>,
        yield_fields: Vec<OutputField>,
    ) -> Self {
        Self {
            id: next_node_id(),
            pattern,
            field,
            query,
            threshold,
            yield_fields,
        }
    }
}

impl PlanNode for VectorMatchNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "VectorMatch"
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

    fn into_enum(
        self,
    ) -> crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum {
        crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum::VectorMatch(self)
    }
}

impl ZeroInputNode for VectorMatchNode {}

impl crate::query::planning::plan::core::nodes::base::memory_estimation::MemoryEstimatable
    for VectorLookupNode
{
    fn estimate_memory(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.schema_name.capacity()
            + self.index_name.capacity()
            + self.yield_fields.iter().map(|f| std::mem::size_of::<OutputField>() + f.name.capacity() + f.alias.as_ref().map_or(0, |a| a.capacity())).sum::<usize>()
    }
}

impl crate::query::planning::plan::core::nodes::base::memory_estimation::MemoryEstimatable
    for VectorMatchNode
{
    fn estimate_memory(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.pattern.capacity()
            + self.field.capacity()
            + self.yield_fields.iter().map(|f| std::mem::size_of::<OutputField>() + f.name.capacity() + f.alias.as_ref().map_or(0, |a| a.capacity())).sum::<usize>()
    }
}
