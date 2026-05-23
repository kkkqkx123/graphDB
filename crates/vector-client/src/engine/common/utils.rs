use serde_json::{json, Value};

pub fn parse_point_id(id: &str) -> Result<u64, &str> {
    id.parse::<u64>().map_err(|_| id)
}

pub fn point_id_to_json(id: &str) -> Value {
    match parse_point_id(id) {
        Ok(num) => json!(num),
        Err(s) => json!(s),
    }
}
