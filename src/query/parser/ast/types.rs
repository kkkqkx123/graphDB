//! Type definitions for the AST

use std::collections::HashMap;
use super::expression::*;

#[derive(Debug, Clone, PartialEq)]
pub struct TagIdentifier {
    pub name: Identifier,
    pub properties: Option<HashMap<String, Expression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub name: Identifier,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub prop: PropertyRef,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyRef {
    Prop(Identifier, Identifier), // tagName.propName
    InlineProp(Identifier),       // propName without tagName
}

// Type aliases for common structures
pub type Identifier = String;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_definitions() {
        let prop_ref = PropertyRef::Prop("tag".to_string(), "prop".to_string());
        assert!(matches!(prop_ref, PropertyRef::Prop(_, _)));
    }
}