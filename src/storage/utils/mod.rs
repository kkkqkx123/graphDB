use crate::core::types::{EdgeTypeInfo, PropertyDef, TagInfo};
use crate::core::value::Value;
use crate::storage::{FieldDef, Schema};
use std::collections::{BTreeMap, HashMap};

pub fn tag_info_to_schema(tag_name: &str, tag_info: &TagInfo) -> Schema {
    let fields: Vec<FieldDef> = tag_info.properties.iter().map(|prop| {
        FieldDef {
            name: prop.name.clone(),
            field_type: prop.data_type.clone(),
            nullable: prop.nullable,
            default_value: prop.default.clone(),
            fixed_length: None,
            offset: 0,
            null_flag_pos: None,
            geo_shape: None,
        }
    }).collect();

    Schema {
        name: tag_name.to_string(),
        version: 1,
        fields: fields.into_iter().map(|f| (f.name.clone(), f)).collect(),
    }
}

pub fn edge_type_info_to_schema(edge_type_name: &str, edge_info: &EdgeTypeInfo) -> Schema {
    let fields: Vec<FieldDef> = edge_info.properties.iter().map(|prop| {
        FieldDef {
            name: prop.name.clone(),
            field_type: prop.data_type.clone(),
            nullable: prop.nullable,
            default_value: prop.default.clone(),
            fixed_length: None,
            offset: 0,
            null_flag_pos: None,
            geo_shape: None,
        }
    }).collect();

    Schema {
        name: edge_type_name.to_string(),
        version: 1,
        fields: fields.into_iter().map(|f| (f.name.clone(), f)).collect(),
    }
}

pub fn property_defs_to_fields(properties: &[PropertyDef]) -> BTreeMap<String, FieldDef> {
    let mut fields = BTreeMap::new();
    for prop in properties {
        let field = FieldDef {
            name: prop.name.clone(),
            field_type: prop.data_type.clone(),
            nullable: prop.nullable,
            default_value: prop.default.clone(),
            fixed_length: None,
            offset: 0,
            null_flag_pos: None,
            geo_shape: None,
        };
        fields.insert(prop.name.clone(), field);
    }
    fields
}

pub fn property_defs_to_hashmap(properties: &[PropertyDef]) -> HashMap<String, Value> {
    let mut map = HashMap::new();
    for prop in properties {
        if let Some(default_value) = &prop.default {
            map.insert(prop.name.clone(), default_value.clone());
        }
    }
    map
}
