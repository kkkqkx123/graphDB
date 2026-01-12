use crate::core::Value;
use std::collections::HashMap;

/// Builds a key-value map from a list of key-value pairs
pub fn build_key_value_map(pairs: Vec<(&str, Value)>) -> HashMap<String, Value> {
    pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}

/// Merges two key-value maps, with values from the second map overwriting values in the first
pub fn merge_key_value_maps(
    mut base: HashMap<String, Value>,
    additional: HashMap<String, Value>,
) -> HashMap<String, Value> {
    for (key, value) in additional {
        base.insert(key, value);
    }
    base
}

/// Converts a key-value map to a vector of (key, value) pairs
pub fn to_pairs(map: &HashMap<String, Value>) -> Vec<(String, Value)> {
    map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
}

/// Creates a map from a slice of string keys and a slice of Value objects
pub fn from_keys_and_values(
    keys: &[&str],
    values: &[Value],
) -> Result<HashMap<String, Value>, String> {
    if keys.len() != values.len() {
        return Err("Keys and values must have the same length".to_string());
    }

    let mut map = HashMap::new();
    for (key, value) in keys.iter().zip(values.iter()) {
        map.insert(key.to_string(), value.clone());
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_kv_builder() {
        let pairs = vec![
            ("name", Value::String("Alice".to_string())),
            ("age", Value::Int(30)),
            ("active", Value::Bool(true)),
        ];

        let map = build_key_value_map(pairs);
        assert_eq!(map.get("name"), Some(&Value::String("Alice".to_string())));
        assert_eq!(map.get("age"), Some(&Value::Int(30)));
        assert_eq!(map.get("active"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_merge_key_value_maps() {
        let mut base = HashMap::new();
        base.insert("name".to_string(), Value::String("Alice".to_string()));
        base.insert("age".to_string(), Value::Int(30));

        let mut additional = HashMap::new();
        additional.insert("age".to_string(), Value::Int(31)); // This should overwrite
        additional.insert("active".to_string(), Value::Bool(true));

        let merged = merge_key_value_maps(base, additional);

        assert_eq!(
            merged.get("name"),
            Some(&Value::String("Alice".to_string()))
        );
        assert_eq!(merged.get("age"), Some(&Value::Int(31))); // Overwritten value
        assert_eq!(merged.get("active"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_to_pairs() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), Value::String("Alice".to_string()));
        map.insert("age".to_string(), Value::Int(30));

        let pairs = to_pairs(&map);

        assert_eq!(pairs.len(), 2);
        assert!(pairs.contains(&("name".to_string(), Value::String("Alice".to_string()))));
        assert!(pairs.contains(&("age".to_string(), Value::Int(30))));
    }

    #[test]
    fn test_from_keys_and_values() {
        let keys = vec!["name", "age", "active"];
        let values = vec![
            Value::String("Bob".to_string()),
            Value::Int(25),
            Value::Bool(false),
        ];

        let result = from_keys_and_values(&keys, &values);
        let map = result.expect("from_keys_and_values should succeed");
        assert_eq!(map.get("name"), Some(&Value::String("Bob".to_string())));
        assert_eq!(map.get("age"), Some(&Value::Int(25)));
        assert_eq!(map.get("active"), Some(&Value::Bool(false)));
    }

    #[test]
    fn test_from_keys_and_values_mismatch() {
        let keys = vec!["name", "age"];
        let values = vec![Value::String("Bob".to_string())]; // Missing one value

        let result = from_keys_and_values(&keys, &values);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Keys and values must have the same length"
        );
    }
}
