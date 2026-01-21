//! Function module for GraphDB
//!
//! This module provides built-in functions similar to NebulaGraph's FunctionManager system

use crate::core::error::DBError;
use crate::core::Value;
use crate::utils::safe_lock;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Type signature for function arguments and return types
#[derive(Debug, Clone)]
pub struct TypeSignature {
    pub args_type: Vec<ValueType>,
    pub return_type: ValueType,
}

/// Represents different value types for type checking
#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Null,
    Bool,
    Int,
    Float,
    String,
    Date,
    Time,
    DateTime,
    Vertex,
    Edge,
    Path,
    List(Box<ValueType>),
    Map(Box<ValueType>, Box<ValueType>), // (key_type, value_type)
    Set(Box<ValueType>),
    Geography,
    Duration,
    Empty,
}

impl From<&Value> for ValueType {
    fn from(value: &Value) -> Self {
        match value {
            Value::Empty => ValueType::Empty,
            Value::Null(_) => ValueType::Null,
            Value::Bool(_) => ValueType::Bool,
            Value::Int(_) => ValueType::Int,
            Value::Float(_) => ValueType::Float,
            Value::String(_) => ValueType::String,
            Value::Date(_) => ValueType::Date,
            Value::Time(_) => ValueType::Time,
            Value::DateTime(_) => ValueType::DateTime,
            Value::Vertex(_) => ValueType::Vertex,
            Value::Edge(_) => ValueType::Edge,
            Value::Path(_) => ValueType::Path,
            Value::List(_) => ValueType::List(Box::new(ValueType::Empty)), // Generic list
            Value::Map(_) => {
                ValueType::Map(Box::new(ValueType::String), Box::new(ValueType::Empty))
            } // Generic map
            Value::Set(_) => ValueType::Set(Box::new(ValueType::Empty)),   // Generic set
            Value::Geography(_) => ValueType::Geography,
            Value::Duration(_) => ValueType::Duration,
            Value::DataSet(_) => ValueType::Empty, // For now, treat DataSet as Empty
        }
    }
}

/// Function attributes including arity, purity, and implementation
#[derive(Clone)]
pub struct FunctionAttributes {
    pub min_arity: usize,
    pub max_arity: usize,
    pub is_pure: bool,
    pub body: Arc<dyn Fn(&[Value]) -> Value + Send + Sync>,
    pub type_signature: Vec<TypeSignature>,
}

impl std::fmt::Debug for FunctionAttributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionAttributes")
            .field("min_arity", &self.min_arity)
            .field("max_arity", &self.max_arity)
            .field("is_pure", &self.is_pure)
            .field("type_signature", &self.type_signature)
            .finish()
    }
}

/// Function manager for managing built-in and user-defined functions
pub struct FunctionManager {
    functions: Arc<Mutex<HashMap<String, FunctionAttributes>>>,
}

impl FunctionManager {
    /// Get a singleton instance of the FunctionManager
    pub fn instance() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(FunctionManager::new()))
    }

    fn new() -> Self {
        let mut manager = FunctionManager {
            functions: Arc::new(Mutex::new(HashMap::new())),
        };

        // Register built-in functions
        manager.register_builtin_functions();
        manager
    }

    /// Register all built-in functions
    fn register_builtin_functions(&mut self) {
        // String functions
        let _ = self.register_function(
            "strlen",
            FunctionAttributes {
                min_arity: 1,
                max_arity: 1,
                is_pure: true,
                body: Arc::new(|args| {
                    if let Value::String(s) = &args[0] {
                        Value::Int(s.len() as i64)
                    } else {
                        Value::Null(crate::core::NullType::BadType)
                    }
                }),
                type_signature: vec![TypeSignature {
                    args_type: vec![ValueType::String],
                    return_type: ValueType::Int,
                }],
            },
        );

        let _ = self.register_function(
            "upper",
            FunctionAttributes {
                min_arity: 1,
                max_arity: 1,
                is_pure: true,
                body: Arc::new(|args| {
                    if let Value::String(s) = &args[0] {
                        Value::String(s.to_uppercase())
                    } else {
                        Value::Null(crate::core::NullType::BadType)
                    }
                }),
                type_signature: vec![TypeSignature {
                    args_type: vec![ValueType::String],
                    return_type: ValueType::String,
                }],
            },
        );

        let _ = self.register_function(
            "lower",
            FunctionAttributes {
                min_arity: 1,
                max_arity: 1,
                is_pure: true,
                body: Arc::new(|args| {
                    if let Value::String(s) = &args[0] {
                        Value::String(s.to_lowercase())
                    } else {
                        Value::Null(crate::core::NullType::BadType)
                    }
                }),
                type_signature: vec![TypeSignature {
                    args_type: vec![ValueType::String],
                    return_type: ValueType::String,
                }],
            },
        );

        let _ = self.register_function(
            "trim",
            FunctionAttributes {
                min_arity: 1,
                max_arity: 1,
                is_pure: true,
                body: Arc::new(|args| {
                    if let Value::String(s) = &args[0] {
                        Value::String(s.trim().to_string())
                    } else {
                        Value::Null(crate::core::NullType::BadType)
                    }
                }),
                type_signature: vec![TypeSignature {
                    args_type: vec![ValueType::String],
                    return_type: ValueType::String,
                }],
            },
        );

        // Math functions
        let _ = self.register_function(
            "abs",
            FunctionAttributes {
                min_arity: 1,
                max_arity: 1,
                is_pure: true,
                body: Arc::new(|args| match &args[0] {
                    Value::Int(n) => Value::Int(n.abs()),
                    Value::Float(f) => Value::Float(f.abs()),
                    _ => Value::Null(crate::core::NullType::BadType),
                }),
                type_signature: vec![
                    TypeSignature {
                        args_type: vec![ValueType::Int],
                        return_type: ValueType::Int,
                    },
                    TypeSignature {
                        args_type: vec![ValueType::Float],
                        return_type: ValueType::Float,
                    },
                ],
            },
        );

        let _ = self.register_function(
            "ceil",
            FunctionAttributes {
                min_arity: 1,
                max_arity: 1,
                is_pure: true,
                body: Arc::new(|args| {
                    if let Value::Float(f) = &args[0] {
                        Value::Float(f.ceil())
                    } else if let Value::Int(i) = &args[0] {
                        Value::Float(*i as f64)
                    } else {
                        Value::Null(crate::core::NullType::BadType)
                    }
                }),
                type_signature: vec![
                    TypeSignature {
                        args_type: vec![ValueType::Int],
                        return_type: ValueType::Float,
                    },
                    TypeSignature {
                        args_type: vec![ValueType::Float],
                        return_type: ValueType::Float,
                    },
                ],
            },
        );

        let _ = self.register_function(
            "floor",
            FunctionAttributes {
                min_arity: 1,
                max_arity: 1,
                is_pure: true,
                body: Arc::new(|args| {
                    if let Value::Float(f) = &args[0] {
                        Value::Float(f.floor())
                    } else if let Value::Int(i) = &args[0] {
                        Value::Float(*i as f64)
                    } else {
                        Value::Null(crate::core::NullType::BadType)
                    }
                }),
                type_signature: vec![
                    TypeSignature {
                        args_type: vec![ValueType::Int],
                        return_type: ValueType::Float,
                    },
                    TypeSignature {
                        args_type: vec![ValueType::Float],
                        return_type: ValueType::Float,
                    },
                ],
            },
        );

        // Type conversion functions
        let _ = self.register_function(
            "to_string",
            FunctionAttributes {
                min_arity: 1,
                max_arity: 1,
                is_pure: true,
                body: Arc::new(|args| Value::String(format!("{}", args[0]))),
                type_signature: vec![
                    TypeSignature {
                        args_type: vec![ValueType::Int],
                        return_type: ValueType::String,
                    },
                    TypeSignature {
                        args_type: vec![ValueType::Float],
                        return_type: ValueType::String,
                    },
                    TypeSignature {
                        args_type: vec![ValueType::Bool],
                        return_type: ValueType::String,
                    },
                    TypeSignature {
                        args_type: vec![ValueType::String],
                        return_type: ValueType::String,
                    },
                ],
            },
        );

        let _ = self.register_function(
            "to_int",
            FunctionAttributes {
                min_arity: 1,
                max_arity: 1,
                is_pure: true,
                body: Arc::new(|args| match &args[0] {
                    Value::Int(i) => Value::Int(*i),
                    Value::Float(f) => Value::Int(f.trunc() as i64),
                    Value::String(s) => {
                        if let Ok(i) = s.parse::<i64>() {
                            Value::Int(i)
                        } else {
                            Value::Null(crate::core::NullType::BadType)
                        }
                    }
                    Value::Bool(b) => Value::Int(if *b { 1 } else { 0 }),
                    _ => Value::Null(crate::core::NullType::BadType),
                }),
                type_signature: vec![
                    TypeSignature {
                        args_type: vec![ValueType::String],
                        return_type: ValueType::Int,
                    },
                    TypeSignature {
                        args_type: vec![ValueType::Float],
                        return_type: ValueType::Int,
                    },
                    TypeSignature {
                        args_type: vec![ValueType::Bool],
                        return_type: ValueType::Int,
                    },
                ],
            },
        );

        // Utility functions
        let _ = self.register_function(
            "hash",
            FunctionAttributes {
                min_arity: 1,
                max_arity: 1,
                is_pure: true,
                body: Arc::new(|args| {
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};

                    let mut hasher = DefaultHasher::new();
                    args[0].hash(&mut hasher);
                    Value::Int(hasher.finish() as i64)
                }),
                type_signature: vec![TypeSignature {
                    args_type: vec![ValueType::String],
                    return_type: ValueType::Int,
                }],
            },
        );

        let _ = self.register_function(
            "range",
            FunctionAttributes {
                min_arity: 2,
                max_arity: 3, // range(start, end, step)
                is_pure: true,
                body: Arc::new(|args| {
                    let start = match &args[0] {
                        Value::Int(i) => *i,
                        _ => return Value::List(vec![]),
                    };

                    let end = match &args[1] {
                        Value::Int(i) => *i,
                        _ => return Value::List(vec![]),
                    };

                    let step = if args.len() > 2 {
                        match &args[2] {
                            Value::Int(i) => *i,
                            _ => 1,
                        }
                    } else {
                        1
                    };

                    let mut result = Vec::new();
                    let mut current = start;

                    if step > 0 {
                        while current <= end {
                            result.push(Value::Int(current));
                            current += step;
                        }
                    } else if step < 0 {
                        while current >= end {
                            result.push(Value::Int(current));
                            current += step;
                        }
                    }

                    Value::List(result)
                }),
                type_signature: vec![TypeSignature {
                    args_type: vec![ValueType::Int, ValueType::Int, ValueType::Int],
                    return_type: ValueType::List(Box::new(ValueType::Int)),
                }],
            },
        );

        // Time functions
        let _ = self.register_function(
            "now",
            FunctionAttributes {
                min_arity: 0,
                max_arity: 0,
                is_pure: false, // Not pure as it returns different values over time
                body: Arc::new(|_| {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards");
                    Value::Int(now.as_secs() as i64)
                }),
                type_signature: vec![TypeSignature {
                    args_type: vec![],
                    return_type: ValueType::Int,
                }],
            },
        );

        let _ = self.register_function(
            "timestamp",
            FunctionAttributes {
                min_arity: 0,
                max_arity: 1,   // timestamp() or timestamp(date)
                is_pure: false, // Not pure when called without arguments
                body: Arc::new(|args| {
                    if args.is_empty() {
                        use std::time::{SystemTime, UNIX_EPOCH};
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("Time went backwards");
                        Value::Int(now.as_secs() as i64)
                    } else {
                        // For now, just return the input timestamp if provided
                        args[0].clone()
                    }
                }),
                type_signature: vec![TypeSignature {
                    args_type: vec![],
                    return_type: ValueType::Int,
                }],
            },
        );
    }

    /// Register a new function
    pub fn register_function(
        &mut self,
        name: &str,
        attributes: FunctionAttributes,
    ) -> Result<(), DBError> {
        let mut functions = safe_lock(&self.functions)?;
        functions.insert(name.to_string(), attributes);
        Ok(())
    }

    /// Get a function by name and arity
    pub fn get(&self, func: &str, arity: usize) -> Result<Option<FunctionAttributes>, DBError> {
        let functions = safe_lock(&self.functions)?;
        if let Some(attr) = functions.get(func) {
            if arity >= attr.min_arity && arity <= attr.max_arity {
                Ok(Some(attr.clone()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Find if a function exists with the given name and arity
    pub fn find(&self, func: &str, arity: usize) -> Result<bool, DBError> {
        if let Some(attr) = self.get(func, arity)? {
            Ok(arity >= attr.min_arity && arity <= attr.max_arity)
        } else {
            Ok(false)
        }
    }

    /// Get the return type of a function
    pub fn get_return_type(
        &self,
        func_name: &str,
        arg_types: &[ValueType],
    ) -> Result<Option<ValueType>, DBError> {
        let functions = safe_lock(&self.functions)?;
        if let Some(attr) = functions.get(func_name) {
            for sig in &attr.type_signature {
                if sig.args_type.len() == arg_types.len() {
                    let mut matches = true;
                    for (i, arg_type) in arg_types.iter().enumerate() {
                        if !self.type_matches(&sig.args_type[i], arg_type) {
                            matches = false;
                            break;
                        }
                    }
                    if matches {
                        return Ok(Some(sig.return_type.clone()));
                    }
                }
            }
        }
        Ok(None)
    }

    /// Check if two types match (with some flexibility for generic types)
    fn type_matches(&self, expected: &ValueType, actual: &ValueType) -> bool {
        match (expected, actual) {
            (ValueType::Int, ValueType::Int) => true,
            (ValueType::Float, ValueType::Float) => true,
            (ValueType::String, ValueType::String) => true,
            (ValueType::Bool, ValueType::Bool) => true,
            (ValueType::Date, ValueType::Date) => true,
            (ValueType::Time, ValueType::Time) => true,
            (ValueType::DateTime, ValueType::DateTime) => true,
            (ValueType::List(expected_elem), ValueType::List(actual_elem)) => {
                self.type_matches(expected_elem, actual_elem)
            }
            (ValueType::Map(expected_k, expected_v), ValueType::Map(actual_k, actual_v)) => {
                self.type_matches(expected_k, actual_k) && self.type_matches(expected_v, actual_v)
            }
            _ => false,
        }
    }
}

/// Execute a function with given arguments
pub fn execute_function(func_name: &str, args: &[Value]) -> Result<Option<Value>, DBError> {
    let manager = FunctionManager::instance();
    let manager_guard = safe_lock(&manager)?;

    if let Some(attr) = manager_guard.get(func_name, args.len())? {
        Ok(Some((attr.body)(args)))
    } else {
        Ok(None)
    }
}

/// Check if a function is pure (given the same inputs, always returns the same output)
pub fn is_function_pure(func_name: &str, arity: usize) -> Result<bool, DBError> {
    let manager = FunctionManager::instance();
    let manager_guard = safe_lock(&manager)?;

    if let Some(attr) = manager_guard.get(func_name, arity)? {
        Ok(attr.is_pure)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_functions() {
        let result = execute_function("upper", &[Value::String("hello".to_string())])
            .expect("Failed to execute function in test");
        assert_eq!(result, Some(Value::String("HELLO".to_string())));

        let result = execute_function("strlen", &[Value::String("test".to_string())])
            .expect("Failed to execute function in test");
        assert_eq!(result, Some(Value::Int(4)));
    }

    #[test]
    fn test_math_functions() {
        let result =
            execute_function("abs", &[Value::Int(-5)]).expect("Failed to execute function in test");
        assert_eq!(result, Some(Value::Int(5)));

        let result = execute_function("ceil", &[Value::Float(3.2)])
            .expect("Failed to execute function in test");
        assert_eq!(result, Some(Value::Float(4.0)));
    }

    #[test]
    fn test_type_conversion() {
        let result = execute_function("to_int", &[Value::String("123".to_string())])
            .expect("Failed to execute function in test");
        assert_eq!(result, Some(Value::Int(123)));
    }

    #[test]
    fn test_range_function() {
        let result = execute_function("range", &[Value::Int(1), Value::Int(5)])
            .expect("Failed to execute function in test");
        let expected = Value::List(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
            Value::Int(5),
        ]);
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn test_function_purity() {
        assert!(is_function_pure("upper", 1).expect("Failed to check function purity in test")); // Pure function
        assert!(!is_function_pure("now", 0).expect("Failed to check function purity in test"));
        // Non-pure function
    }
}
