use super::{ColumnSchema, ResultSetSchema};
use crate::core::{DataType, Edge, Vertex};

#[derive(Clone)]
pub enum ScanTarget {
    AllVertices,
    VerticesByTag(String),
    AllEdges,
    EdgesByType(String),
}

#[derive(Clone)]
pub struct ScanNode {
    target: ScanTarget,
}

impl ScanNode {
    pub fn new(target: ScanTarget) -> Self {
        Self { target }
    }

    pub fn target(&self) -> &ScanTarget {
        &self.target
    }
}

#[derive(Clone)]
pub struct FilterNode {
    condition: String,
    input_schema: ResultSetSchema,
}

impl FilterNode {
    pub fn new(condition: String, input_schema: ResultSetSchema) -> Self {
        Self {
            condition,
            input_schema,
        }
    }

    pub fn condition(&self) -> &str {
        &self.condition
    }

    pub fn input_schema(&self) -> &ResultSetSchema {
        &self.input_schema
    }
}

#[derive(Clone)]
pub struct ProjectNode {
    expressions: Vec<String>,
    input_schema: ResultSetSchema,
    output_schema: ResultSetSchema,
}

impl ProjectNode {
    pub fn new(expressions: Vec<String>, input_schema: ResultSetSchema, output_schema: ResultSetSchema) -> Self {
        Self {
            expressions,
            input_schema,
            output_schema,
        }
    }

    pub fn expressions(&self) -> &[String] {
        &self.expressions
    }
}

#[derive(Clone)]
pub struct LimitNode {
    offset: usize,
    count: usize,
    schema: ResultSetSchema,
}

impl LimitNode {
    pub fn new(offset: usize, count: usize, schema: ResultSetSchema) -> Self {
        Self {
            offset,
            count,
            schema,
        }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

#[derive(Clone)]
pub struct GetNeighborsNode {
    src_vertex: String,
    edge_type: String,
    direction: crate::core::EdgeDirection,
    props: Vec<String>,
    schema: ResultSetSchema,
}

impl GetNeighborsNode {
    pub fn new(
        src_vertex: String,
        edge_type: String,
        direction: crate::core::EdgeDirection,
        props: Vec<String>,
        schema: ResultSetSchema,
    ) -> Self {
        Self {
            src_vertex,
            edge_type,
            direction,
            props,
            schema,
        }
    }

    pub fn src_vertex(&self) -> &str {
        &self.src_vertex
    }

    pub fn edge_type(&self) -> &str {
        &self.edge_type
    }

    pub fn direction(&self) -> crate::core::EdgeDirection {
        self.direction
    }

    pub fn props(&self) -> &[String] {
        &self.props
    }
}

#[derive(Clone)]
pub struct AggregateNode {
    group_keys: Vec<String>,
    agg_functions: Vec<String>,
    input_schema: ResultSetSchema,
    output_schema: ResultSetSchema,
}

impl AggregateNode {
    pub fn new(
        group_keys: Vec<String>,
        agg_functions: Vec<String>,
        input_schema: ResultSetSchema,
        output_schema: ResultSetSchema,
    ) -> Self {
        Self {
            group_keys,
            agg_functions,
            input_schema,
            output_schema,
        }
    }

    pub fn group_keys(&self) -> &[String] {
        &self.group_keys
    }

    pub fn agg_functions(&self) -> &[String] {
        &self.agg_functions
    }
}

#[derive(Clone)]
pub struct DedupNode {
    keys: Vec<String>,
    input_schema: ResultSetSchema,
    output_schema: ResultSetSchema,
}

impl DedupNode {
    pub fn new(keys: Vec<String>, input_schema: ResultSetSchema, output_schema: ResultSetSchema) -> Self {
        Self {
            keys,
            input_schema,
            output_schema,
        }
    }

    pub fn keys(&self) -> &[String] {
        &self.keys
    }
}

pub fn vertex_schema() -> ResultSetSchema {
    ResultSetSchema {
        columns: vec![
            ColumnSchema {
                name: "vid".to_string(),
                data_type: DataType::String,
                nullable: false,
            },
            ColumnSchema {
                name: "properties".to_string(),
                data_type: DataType::Map,
                nullable: true,
            },
        ],
    }
}

pub fn edge_schema() -> ResultSetSchema {
    ResultSetSchema {
        columns: vec![
            ColumnSchema {
                name: "src".to_string(),
                data_type: DataType::String,
                nullable: false,
            },
            ColumnSchema {
                name: "dst".to_string(),
                data_type: DataType::String,
                nullable: false,
            },
            ColumnSchema {
                name: "rank".to_string(),
                data_type: DataType::Int64,
                nullable: false,
            },
            ColumnSchema {
                name: "properties".to_string(),
                data_type: DataType::Map,
                nullable: true,
            },
        ],
    }
}
