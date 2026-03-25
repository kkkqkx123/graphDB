//! Query Type Base Definition

/// Enumeration of query types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryType {
    #[default]
    ReadQuery,
    WriteQuery,
    AdminQuery,
    SchemaQuery,
}
