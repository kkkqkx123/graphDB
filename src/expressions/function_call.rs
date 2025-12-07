use std::collections::{HashMap, HashSet};
use crate::core::{Value, NullType};
use super::base::EvaluationError;

/// Evaluate function call operation
pub fn eval_function_call(name: &str, args: Vec<Value>) -> Result<Value, EvaluationError> {
    // This is a simplified implementation. A full graph database would have
    // many more built-in functions
    match name {
        "abs" => {
            if args.len() != 1 {
                return Err(EvaluationError::InvalidOperation(
                    "abs function expects 1 argument".to_string()
                ));
            }
            
            match &args[0] {
                Value::Int(i) => Ok(Value::Int(i.abs())),
                Value::Float(f) => Ok(Value::Float(f.abs())),
                _ => Err(EvaluationError::TypeError(
                    "abs function expects numeric argument".to_string()
                )),
            }
        },
        "strlen" => {
            if args.len() != 1 {
                return Err(EvaluationError::InvalidOperation(
                    "strlen function expects 1 argument".to_string()
                ));
            }
            
            match &args[0] {
                Value::String(s) => Ok(Value::Int(s.len() as i64)),
                _ => Err(EvaluationError::TypeError(
                    "strlen function expects string argument".to_string()
                )),
            }
        },
        "upper" => {
            if args.len() != 1 {
                return Err(EvaluationError::InvalidOperation(
                    "upper function expects 1 argument".to_string()
                ));
            }
            
            match &args[0] {
                Value::String(s) => Ok(Value::String(s.to_uppercase())),
                _ => Err(EvaluationError::TypeError(
                    "upper function expects string argument".to_string()
                )),
            }
        },
        "lower" => {
            if args.len() != 1 {
                return Err(EvaluationError::InvalidOperation(
                    "lower function expects 1 argument".to_string()
                ));
            }
            
            match &args[0] {
                Value::String(s) => Ok(Value::String(s.to_lowercase())),
                _ => Err(EvaluationError::TypeError(
                    "lower function expects string argument".to_string()
                )),
            }
        },
        "size" => {
            if args.len() != 1 {
                return Err(EvaluationError::InvalidOperation(
                    "size function expects 1 argument".to_string()
                ));
            }
            
            match &args[0] {
                Value::List(list) => Ok(Value::Int(list.len() as i64)),
                Value::Map(map) => Ok(Value::Int(map.len() as i64)),
                Value::Set(set) => Ok(Value::Int(set.len() as i64)),
                Value::String(s) => Ok(Value::Int(s.len() as i64)),
                _ => Err(EvaluationError::TypeError(
                    "size function expects container or string argument".to_string()
                )),
            }
        },
        "coalesce" => {
            for arg in args {
                if !matches!(arg, Value::Null(_)) {
                    return Ok(arg);
                }
            }
            Ok(Value::Null(NullType::NaN))
        },
        _ => Err(EvaluationError::Other(
            format!("Function '{}' not implemented", name)
        )),
    }
}