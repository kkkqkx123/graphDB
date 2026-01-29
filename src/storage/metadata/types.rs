use crate::core::{DataType, Value};

#[derive(Clone, Debug, PartialEq)]
pub struct SpaceInfo {
    pub name: String,
    pub partition_num: u32,
    pub replica_factor: u32,
    pub charset: Option<String>,
    pub collate: Option<String>,
}

impl SpaceInfo {
    pub fn new(name: String) -> Self {
        Self {
            name,
            partition_num: 1,
            replica_factor: 1,
            charset: None,
            collate: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TagInfo {
    pub space_name: String,
    pub name: String,
    pub properties: Vec<PropertyDef>,
    pub comment: Option<String>,
}

impl TagInfo {
    pub fn new(space_name: String, name: String) -> Self {
        Self {
            space_name,
            name,
            properties: Vec::new(),
            comment: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EdgeTypeSchema {
    pub space_name: String,
    pub name: String,
    pub properties: Vec<PropertyDef>,
    pub comment: Option<String>,
}

impl EdgeTypeSchema {
    pub fn new(space_name: String, name: String) -> Self {
        Self {
            space_name,
            name,
            properties: Vec::new(),
            comment: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PropertyDef {
    pub name: String,
    pub type_def: DataType,
    pub is_nullable: bool,
    pub default_value: Option<Value>,
    pub comment: Option<String>,
}

impl PropertyDef {
    pub fn new(name: String, type_def: DataType) -> Self {
        Self {
            name,
            type_def,
            is_nullable: false,
            default_value: None,
            comment: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct IndexInfo {
    pub space_name: String,
    pub name: String,
    pub index_type: IndexType,
    pub fields: Vec<String>,
    pub is_unique: bool,
    pub comment: Option<String>,
}

impl IndexInfo {
    pub fn new(space_name: String, name: String, index_type: IndexType) -> Self {
        Self {
            space_name,
            name,
            index_type,
            fields: Vec::new(),
            is_unique: false,
            comment: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum IndexType {
    TagIndex,
    EdgeIndex,
    CompositeIndex,
    FulltextIndex,
}
