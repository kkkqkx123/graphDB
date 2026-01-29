//! 类型签名系统
//!
//! 定义函数的类型签名，用于类型检查和函数重载解析

use crate::core::Value;
use std::fmt;

/// 值类型枚举（用于函数签名）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    List,
    Map,
    Set,
    Geography,
    Duration,
    DataSet,
    Any,
}

impl ValueType {
    pub fn from_value(value: &Value) -> Self {
        match value {
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
            Value::List(_) => ValueType::List,
            Value::Map(_) => ValueType::Map,
            Value::Set(_) => ValueType::Set,
            Value::Geography(_) => ValueType::Geography,
            Value::Duration(_) => ValueType::Duration,
            Value::DataSet(_) => ValueType::DataSet,
            Value::Empty => ValueType::Any,
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, ValueType::Int | ValueType::Float)
    }

    pub fn is_string(&self) -> bool {
        matches!(self, ValueType::String)
    }

    pub fn is_collection(&self) -> bool {
        matches!(self, ValueType::List | ValueType::Map | ValueType::Set)
    }

    pub fn compatible_with(&self, other: &ValueType) -> bool {
        self == &ValueType::Any || other == &ValueType::Any || self == other
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::Null => write!(f, "NULL"),
            ValueType::Bool => write!(f, "BOOL"),
            ValueType::Int => write!(f, "INT"),
            ValueType::Float => write!(f, "FLOAT"),
            ValueType::String => write!(f, "STRING"),
            ValueType::Date => write!(f, "DATE"),
            ValueType::Time => write!(f, "TIME"),
            ValueType::DateTime => write!(f, "DATETIME"),
            ValueType::Vertex => write!(f, "VERTEX"),
            ValueType::Edge => write!(f, "EDGE"),
            ValueType::Path => write!(f, "PATH"),
            ValueType::List => write!(f, "LIST"),
            ValueType::Map => write!(f, "MAP"),
            ValueType::Set => write!(f, "SET"),
            ValueType::Geography => write!(f, "GEOGRAPHY"),
            ValueType::Duration => write!(f, "DURATION"),
            ValueType::DataSet => write!(f, "DATASET"),
            ValueType::Any => write!(f, "ANY"),
        }
    }
}

/// 函数签名定义
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub arg_types: Vec<ValueType>,
    pub return_type: ValueType,
    pub min_arity: usize,
    pub max_arity: usize,
    pub is_pure: bool,
    pub description: String,
}

impl FunctionSignature {
    pub fn new(
        name: &str,
        arg_types: Vec<ValueType>,
        return_type: ValueType,
        min_arity: usize,
        max_arity: usize,
        is_pure: bool,
        description: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            arg_types,
            return_type,
            min_arity,
            max_arity,
            is_pure,
            description: description.to_string(),
        }
    }

    pub fn is_variadic(&self) -> bool {
        self.max_arity == usize::MAX
    }

    pub fn check_arity(&self, arity: usize) -> bool {
        arity >= self.min_arity && (self.is_variadic() || arity <= self.max_arity)
    }

    pub fn check_exact_types(&self, args: &[Value]) -> bool {
        if args.len() != self.arg_types.len() {
            return false;
        }
        args.iter().zip(&self.arg_types).all(|(arg, expected)| {
            let actual = ValueType::from_value(arg);
            actual == *expected
        })
    }

    pub fn check_compatible_types(&self, args: &[Value]) -> bool {
        if !self.is_variadic() && args.len() != self.arg_types.len() {
            return false;
        }
        args.iter().zip(&self.arg_types).all(|(arg, expected)| {
            if expected == &ValueType::Any {
                return true;
            }
            let actual = ValueType::from_value(arg);
            actual.compatible_with(expected)
        })
    }

    pub fn type_matching_score(&self, args: &[Value]) -> i32 {
        if !self.check_arity(args.len()) {
            return i32::MIN;
        }
        let mut score = 0;
        for (arg, expected) in args.iter().zip(&self.arg_types) {
            let actual = ValueType::from_value(arg);
            if actual == *expected {
                score += 10;
            } else if *expected == ValueType::Any {
                score += 1;
            } else if actual.compatible_with(expected) {
                score += 5;
            } else {
                return i32::MIN;
            }
        }
        score
    }
}

/// 函数调用主体
pub type FunctionBody = dyn Fn(&[Value]) -> Result<Value, crate::core::error::ExpressionError> + Send + Sync;

/// 注册的函数信息
pub struct RegisteredFunction {
    pub signature: FunctionSignature,
    pub body: Box<FunctionBody>,
}

impl RegisteredFunction {
    pub fn new(signature: FunctionSignature, body: Box<FunctionBody>) -> Self {
        Self { signature, body }
    }
}

impl std::fmt::Debug for RegisteredFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisteredFunction")
            .field("signature", &self.signature)
            .finish()
    }
}
