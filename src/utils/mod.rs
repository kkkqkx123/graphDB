use std::collections::HashMap;
use std::hash::Hash;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::core::Value;

/// A simple object pool for reusing objects to reduce allocation overhead
pub struct ObjectPool<T> {
    pool: Vec<T>,
    max_size: usize,
}

impl<T: Default + Clone> ObjectPool<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: Vec::new(),
            max_size,
        }
    }

    pub fn get(&mut self) -> T {
        self.pool.pop().unwrap_or_default()
    }

    pub fn put(&mut self, obj: T) {
        if self.pool.len() < self.max_size {
            self.pool.push(obj);
        }
    }
}

/// A simple LRU cache implementation
pub struct LruCache<K, V> {
    capacity: usize,
    cache: HashMap<K, V>,
    access_order: std::collections::VecDeque<K>,
}

impl<K, V> LruCache<K, V>
where
    K: Eq + Hash + Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: HashMap::new(),
            access_order: std::collections::VecDeque::new(),
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        if self.cache.contains_key(key) {
            // Move the key to the back of the access order (most recent)
            let pos = self.access_order.iter().position(|k| k == key);
            if let Some(pos) = pos {
                let k = self.access_order.remove(pos).unwrap();
                self.access_order.push_back(k);
            }
            self.cache.get(key)
        } else {
            None
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        if self.cache.contains_key(&key) {
            // Key exists, update the access order
            let pos = self.access_order.iter().position(|k| k == &key);
            if let Some(pos) = pos {
                self.access_order.remove(pos);
            }
        } else if self.cache.len() >= self.capacity {
            // Cache is full, remove the least recently used item
            if let Some(old_key) = self.access_order.pop_front() {
                self.cache.remove(&old_key);
            }
        }

        self.cache.insert(key.clone(), value);
        self.access_order.push_back(key);
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let result = self.cache.remove(key);
        if result.is_some() {
            // Remove from access order as well
            let pos = self.access_order.iter().position(|k| k == key);
            if let Some(pos) = pos {
                self.access_order.remove(pos);
            }
        }
        result
    }
}

/// Utility function to generate unique IDs
pub fn generate_id() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos() as u64
}

/// Utility function for validating node/edge IDs
pub fn is_valid_id(id: u64) -> bool {
    id != 0
}

/// A simple logger wrapper for consistent logging format
pub struct Logger;

impl Logger {
    pub fn info(message: &str) {
        println!("[INFO] {}", message);
    }

    pub fn warn(message: &str) {
        eprintln!("[WARN] {}", message);
    }

    pub fn error(message: &str) {
        eprintln!("[ERROR] {}", message);
    }
}

/// Key-Value building utilities
pub mod kv_builder {
    use std::collections::HashMap;
    use crate::core::Value;

    /// Builds a key-value map from a list of key-value pairs
    pub fn build_key_value_map(pairs: Vec<(&str, Value)>) -> HashMap<String, Value> {
        pairs.into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect()
    }

    /// Merges two key-value maps, with values from the second map overwriting values in the first
    pub fn merge_key_value_maps(
        mut base: HashMap<String, Value>,
        additional: HashMap<String, Value>
    ) -> HashMap<String, Value> {
        for (key, value) in additional {
            base.insert(key, value);
        }
        base
    }

    /// Converts a key-value map to a vector of (key, value) pairs
    pub fn to_pairs(map: &HashMap<String, Value>) -> Vec<(String, Value)> {
        map.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Creates a map from a slice of string keys and a slice of Value objects
    pub fn from_keys_and_values(keys: &[&str], values: &[Value]) -> Result<HashMap<String, Value>, String> {
        if keys.len() != values.len() {
            return Err("Keys and values must have the same length".to_string());
        }

        let mut map = HashMap::new();
        for (key, value) in keys.iter().zip(values.iter()) {
            map.insert(key.to_string(), value.clone());
        }

        Ok(map)
    }
}

/// String utilities for database operations
pub mod string_utils {
    use std::collections::HashMap;

    /// Escapes special characters in a string for use in queries
    pub fn escape_for_query(s: &str) -> String {
        s.replace('\\', "\\\\")
         .replace('\'', "\\'")
         .replace('\"', "\\\"")
         .replace('\n', "\\n")
         .replace('\r', "\\r")
         .replace('\t', "\\t")
    }

    /// Unescapes special characters in a string
    pub fn unescape_for_query(s: &str) -> String {
        s.replace("\\t", "\t")
         .replace("\\r", "\r")
         .replace("\\n", "\n")
         .replace("\\\"", "\"")
         .replace("\\'", "'")
         .replace("\\\\", "\\")
    }

    /// Normalizes identifier names (table names, column names, etc.)
    pub fn normalize_identifier(name: &str) -> String {
        // Replace spaces with underscores and convert to lowercase
        name.trim()
           .replace(' ', "_")
           .replace('-', "_")
           .to_lowercase()
    }

    /// Sanitizes input to prevent injection attacks
    pub fn sanitize_input(input: &str) -> String {
        // Remove potentially dangerous characters/sequences
        input.replace(";", "")
             .replace("--", "")
             .replace("/*", "")
             .replace("*/", "")
             .replace("xp_", "")  // Prevent calls to extended procedures
             .replace("sp_", "")  // Prevent calls to stored procedures
    }
}

/// Type conversion utilities
pub mod type_utils {
    use crate::core::Value;

    /// Attempts to convert a Value to a boolean
    pub fn value_to_bool(value: &Value) -> Option<bool> {
        match value {
            Value::Bool(b) => Some(*b),
            Value::Int(i) => Some(*i != 0),
            Value::Float(f) => Some(*f != 0.0 && !f.is_nan()),
            Value::String(s) => {
                if s.to_lowercase() == "true" {
                    Some(true)
                } else if s.to_lowercase() == "false" {
                    Some(false)
                } else {
                    None  // Cannot convert string to bool without more context
                }
            }
            Value::Empty => Some(false),
            Value::Null(_) => None,
            _ => None,
        }
    }

    /// Attempts to convert a Value to an integer
    pub fn value_to_i64(value: &Value) -> Option<i64> {
        match value {
            Value::Int(i) => Some(*i),
            Value::Float(f) => Some(*f as i64),
            Value::String(s) => s.parse::<i64>().ok(),
            Value::Bool(b) => Some(if *b { 1 } else { 0 }),
            Value::Empty => Some(0),
            Value::Null(_) => None,
            _ => None,
        }
    }

    /// Attempts to convert a Value to a float
    pub fn value_to_f64(value: &Value) -> Option<f64> {
        match value {
            Value::Float(f) => Some(*f),
            Value::Int(i) => Some(*i as f64),
            Value::String(s) => s.parse::<f64>().ok(),
            Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            Value::Empty => Some(0.0),
            Value::Null(_) => None,
            _ => None,
        }
    }

    /// Attempts to convert a Value to a string
    pub fn value_to_string(value: &Value) -> Option<String> {
        match value {
            Value::String(s) => Some(s.clone()),
            Value::Int(i) => Some(i.to_string()),
            Value::Float(f) => Some(f.to_string()),
            Value::Bool(b) => Some(b.to_string()),
            Value::Empty => Some(String::new()),
            Value::Null(_) => None,
            _ => None,  // Other types may require more complex conversion
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache() {
        let mut cache = LruCache::new(2);

        cache.put(1, "one");
        cache.put(2, "two");

        assert_eq!(cache.get(&1), Some(&"one")); // Access 1
        cache.put(3, "three"); // This should evict 2

        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_object_pool() {
        let mut pool: ObjectPool<Vec<i32>> = ObjectPool::new(10);

        let mut obj = pool.get();
        obj.push(42);

        assert_eq!(obj, vec![42]);

        pool.put(obj);

        let obj2 = pool.get();
        assert_eq!(obj2, vec![42]); // Should reuse the same object
    }

    #[test]
    fn test_kv_builder() {
        use crate::core::{Value, NullType};

        let pairs = vec![
            ("name", Value::String("Alice".to_string())),
            ("age", Value::Int(30)),
            ("active", Value::Bool(true)),
        ];

        let map = kv_builder::build_key_value_map(pairs);
        assert_eq!(map.get("name"), Some(&Value::String("Alice".to_string())));
        assert_eq!(map.get("age"), Some(&Value::Int(30)));
        assert_eq!(map.get("active"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_string_utils() {
        let original = "Hello\nWorld\t\"Test\"";
        let escaped = string_utils::escape_for_query(original);
        assert_eq!(escaped, "Hello\\nWorld\\t\\\"Test\\\"");

        let unescaped = string_utils::unescape_for_query(&escaped);
        assert_eq!(unescaped, original);
    }

    #[test]
    fn test_type_utils() {
        use crate::core::Value;

        let int_val = Value::Int(42);
        assert_eq!(type_utils::value_to_i64(&int_val), Some(42));
        assert_eq!(type_utils::value_to_f64(&int_val), Some(42.0));
        assert_eq!(type_utils::value_to_bool(&int_val), Some(true));
        assert_eq!(type_utils::value_to_string(&int_val), Some("42".to_string()));

        let str_val = Value::String("123".to_string());
        assert_eq!(type_utils::value_to_i64(&str_val), Some(123));

        let float_val = Value::Float(12.3);
        assert_eq!(type_utils::value_to_f64(&float_val), Some(12.3));
        assert_eq!(type_utils::value_to_i64(&float_val), Some(12));
    }
}