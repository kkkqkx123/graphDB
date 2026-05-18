use std::collections::HashMap;

use crate::core::Value;

pub fn props_to_map(props: &[(String, Value)]) -> HashMap<String, Value> {
    props.iter().cloned().collect()
}
