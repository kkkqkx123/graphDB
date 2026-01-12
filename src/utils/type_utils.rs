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
                None // Cannot convert string to bool without more context
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
        _ => None, // Other types may require more complex conversion
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{NullType, Value};

    #[test]
    fn test_type_utils() {
        let int_val = Value::Int(42);
        assert_eq!(value_to_i64(&int_val), Some(42));
        assert_eq!(value_to_f64(&int_val), Some(42.0));
        assert_eq!(value_to_bool(&int_val), Some(true));
        assert_eq!(value_to_string(&int_val), Some("42".to_string()));

        let str_val = Value::String("123".to_string());
        assert_eq!(value_to_i64(&str_val), Some(123));

        let float_val = Value::Float(12.3);
        assert_eq!(value_to_f64(&float_val), Some(12.3));
        assert_eq!(value_to_i64(&float_val), Some(12));
    }

    #[test]
    fn test_value_to_bool() {
        assert_eq!(value_to_bool(&Value::Bool(true)), Some(true));
        assert_eq!(value_to_bool(&Value::Bool(false)), Some(false));
        assert_eq!(value_to_bool(&Value::Int(1)), Some(true));
        assert_eq!(value_to_bool(&Value::Int(0)), Some(false));
        assert_eq!(value_to_bool(&Value::Int(-1)), Some(true));
        assert_eq!(value_to_bool(&Value::Float(1.0)), Some(true));
        assert_eq!(value_to_bool(&Value::Float(0.0)), Some(false));
        assert_eq!(value_to_bool(&Value::Float(f64::NAN)), Some(false));
        assert_eq!(
            value_to_bool(&Value::String("true".to_string())),
            Some(true)
        );
        assert_eq!(
            value_to_bool(&Value::String("TRUE".to_string())),
            Some(true)
        );
        assert_eq!(
            value_to_bool(&Value::String("false".to_string())),
            Some(false)
        );
        assert_eq!(
            value_to_bool(&Value::String("FALSE".to_string())),
            Some(false)
        );
        assert_eq!(value_to_bool(&Value::String("yes".to_string())), None);
        assert_eq!(value_to_bool(&Value::Empty), Some(false));
        assert_eq!(value_to_bool(&Value::Null(NullType::Null)), None);
    }

    #[test]
    fn test_value_to_i64() {
        assert_eq!(value_to_i64(&Value::Int(42)), Some(42));
        assert_eq!(value_to_i64(&Value::Int(-42)), Some(-42));
        assert_eq!(value_to_i64(&Value::Float(12.7)), Some(12));
        assert_eq!(value_to_i64(&Value::Float(-12.7)), Some(-12));
        assert_eq!(value_to_i64(&Value::String("123".to_string())), Some(123));
        assert_eq!(value_to_i64(&Value::String("-123".to_string())), Some(-123));
        assert_eq!(value_to_i64(&Value::String("abc".to_string())), None);
        assert_eq!(value_to_i64(&Value::Bool(true)), Some(1));
        assert_eq!(value_to_i64(&Value::Bool(false)), Some(0));
        assert_eq!(value_to_i64(&Value::Empty), Some(0));
        assert_eq!(value_to_i64(&Value::Null(NullType::Null)), None);
    }

    #[test]
    fn test_value_to_f64() {
        assert_eq!(value_to_f64(&Value::Float(12.3)), Some(12.3));
        assert_eq!(value_to_f64(&Value::Float(-12.3)), Some(-12.3));
        assert_eq!(value_to_f64(&Value::Int(42)), Some(42.0));
        assert_eq!(value_to_f64(&Value::Int(-42)), Some(-42.0));
        assert_eq!(value_to_f64(&Value::String("12.3".to_string())), Some(12.3));
        assert_eq!(
            value_to_f64(&Value::String("-12.3".to_string())),
            Some(-12.3)
        );
        assert_eq!(value_to_f64(&Value::String("abc".to_string())), None);
        assert_eq!(value_to_f64(&Value::Bool(true)), Some(1.0));
        assert_eq!(value_to_f64(&Value::Bool(false)), Some(0.0));
        assert_eq!(value_to_f64(&Value::Empty), Some(0.0));
        assert_eq!(value_to_f64(&Value::Null(NullType::Null)), None);
    }

    #[test]
    fn test_value_to_string() {
        assert_eq!(
            value_to_string(&Value::String("hello".to_string())),
            Some("hello".to_string())
        );
        assert_eq!(value_to_string(&Value::Int(42)), Some("42".to_string()));
        assert_eq!(value_to_string(&Value::Int(-42)), Some("-42".to_string()));
        assert_eq!(
            value_to_string(&Value::Float(12.3)),
            Some("12.3".to_string())
        );
        assert_eq!(
            value_to_string(&Value::Float(-12.3)),
            Some("-12.3".to_string())
        );
        assert_eq!(
            value_to_string(&Value::Bool(true)),
            Some("true".to_string())
        );
        assert_eq!(
            value_to_string(&Value::Bool(false)),
            Some("false".to_string())
        );
        assert_eq!(value_to_string(&Value::Empty), Some("".to_string()));
        assert_eq!(value_to_string(&Value::Null(NullType::Null)), None);
    }
}
