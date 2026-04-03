use crate::query::parser::ast::{Identifier, Expression};
use crate::search::engine::EngineType;

#[derive(Debug, Clone, PartialEq)]
pub struct CreateFulltextIndexStmt {
    pub index_name: Identifier,
    pub tag_name: Identifier,
    pub fields: Vec<Identifier>,
    pub engine_type: Option<EngineType>,
    pub engine_options: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DropFulltextIndexStmt {
    pub index_name: Identifier,
    pub if_exists: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShowFulltextIndexesStmt {
    pub show_status: bool,
    pub index_name: Option<Identifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RebuildFulltextIndexStmt {
    pub index_name: Identifier,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FulltextMatchExpr {
    pub field_ref: FieldReference,
    pub query: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldReference {
    pub variable: Identifier,
    pub field: Identifier,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FulltextFunction {
    Score(Expression),
    Highlight(FieldReference),
}
