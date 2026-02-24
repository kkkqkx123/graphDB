//! 容器操作函数实现
//!
//! 提供列表和映射的操作函数，包括 head, last, tail, size, range, keys

use crate::core::error::ExpressionError;
use crate::core::value::dataset::List;
use crate::core::value::NullType;
use crate::core::Value;
use std::collections::BTreeSet;

/// 容器函数枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerFunction {
    Head,
    Last,
    Tail,
    Size,
    Range,
    Keys,
    ReverseList,
    ToSet,
}

impl ContainerFunction {
    /// 获取函数名称
    pub fn name(&self) -> &str {
        match self {
            Self::Head => "head",
            Self::Last => "last",
            Self::Tail => "tail",
            Self::Size => "size",
            Self::Range => "range",
            Self::Keys => "keys",
            Self::ReverseList => "reverse",
            Self::ToSet => "toset",
        }
    }

    /// 获取参数数量
    pub fn arity(&self) -> usize {
        match self {
            Self::Head => 1,
            Self::Last => 1,
            Self::Tail => 1,
            Self::Size => 1,
            Self::Range => 2,
            Self::Keys => 1,
            Self::ReverseList => 1,
            Self::ToSet => 1,
        }
    }

    /// 是否为可变参数函数
    pub fn is_variadic(&self) -> bool {
        match self {
            Self::Range => true,
            _ => false,
        }
    }

    /// 获取函数描述
    pub fn description(&self) -> &str {
        match self {
            Self::Head => "获取列表的第一个元素",
            Self::Last => "获取列表的最后一个元素",
            Self::Tail => "获取列表除第一个元素外的所有元素",
            Self::Size => "获取字符串、列表、映射或集合的大小",
            Self::Range => "生成一个整数范围列表",
            Self::Keys => "获取顶点、边或映射的所有键",
            Self::ReverseList => "反转列表",
            Self::ToSet => "将列表转换为集合",
        }
    }

    pub fn execute(&self, args: &[Value]) -> Result<Value, ExpressionError> {
        match self {
            Self::Head => execute_head(args),
            Self::Last => execute_last(args),
            Self::Tail => execute_tail(args),
            Self::Size => execute_size(args),
            Self::Range => execute_range(args),
            Self::Keys => execute_keys(args),
            Self::ReverseList => execute_reverse_list(args),
            Self::ToSet => execute_toset(args),
        }
    }
}

fn execute_head(args: &[Value]) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::type_error("head函数需要1个参数"));
    }
    match &args[0] {
        Value::List(list) => Ok(list
            .values
            .first()
            .cloned()
            .unwrap_or(Value::Null(NullType::Null))),
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("head函数需要列表类型")),
    }
}

fn execute_last(args: &[Value]) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::type_error("last函数需要1个参数"));
    }
    match &args[0] {
        Value::List(list) => Ok(list
            .values
            .last()
            .cloned()
            .unwrap_or(Value::Null(NullType::Null))),
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("last函数需要列表类型")),
    }
}

fn execute_tail(args: &[Value]) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::type_error("tail函数需要1个参数"));
    }
    match &args[0] {
        Value::List(list) => {
            if list.values.is_empty() {
                Ok(Value::List(List { values: vec![] }))
            } else {
                Ok(Value::List(List {
                    values: list.values[1..].to_vec(),
                }))
            }
        }
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("tail函数需要列表类型")),
    }
}

fn execute_size(args: &[Value]) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::type_error("size函数需要1个参数"));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Int(s.len() as i64)),
        Value::List(list) => Ok(Value::Int(list.values.len() as i64)),
        Value::Map(map) => Ok(Value::Int(map.len() as i64)),
        Value::Set(set) => Ok(Value::Int(set.len() as i64)),
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error(
            "size函数需要字符串、列表、映射或集合类型",
        )),
    }
}

fn execute_range(args: &[Value]) -> Result<Value, ExpressionError> {
    if args.len() < 2 || args.len() > 3 {
        return Err(ExpressionError::type_error("range函数需要2或3个参数"));
    }
    let start = match &args[0] {
        Value::Int(i) => *i,
        Value::Null(_) => return Ok(Value::Null(NullType::Null)),
        _ => return Err(ExpressionError::type_error("range函数需要整数参数")),
    };
    let end = match &args[1] {
        Value::Int(i) => *i,
        Value::Null(_) => return Ok(Value::Null(NullType::Null)),
        _ => return Err(ExpressionError::type_error("range函数需要整数参数")),
    };
    let step = if args.len() > 2 {
        match &args[2] {
            Value::Int(i) => *i,
            Value::Null(_) => return Ok(Value::Null(NullType::Null)),
            _ => return Err(ExpressionError::type_error("range函数的step需要整数")),
        }
    } else {
        1
    };

    if step == 0 {
        return Err(ExpressionError::new(
            crate::core::error::ExpressionErrorType::InvalidOperation,
            "range函数的step不能为0".to_string(),
        ));
    }

    let mut result = Vec::new();
    if step > 0 {
        let mut i = start;
        while i <= end {
            result.push(Value::Int(i));
            i += step;
        }
    } else {
        let mut i = start;
        while i >= end {
            result.push(Value::Int(i));
            i += step;
        }
    }

    Ok(Value::List(List { values: result }))
}

fn execute_keys(args: &[Value]) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::type_error("keys函数需要1个参数"));
    }
    let mut keys: BTreeSet<String> = BTreeSet::new();

    match &args[0] {
        Value::Vertex(v) => {
            for tag in &v.tags {
                for key in tag.properties.keys() {
                    keys.insert(key.clone());
                }
            }
            for key in v.properties.keys() {
                keys.insert(key.clone());
            }
        }
        Value::Edge(e) => {
            for key in e.props.keys() {
                keys.insert(key.clone());
            }
        }
        Value::Map(m) => {
            for key in m.keys() {
                keys.insert(key.clone());
            }
        }
        Value::Null(_) => return Ok(Value::Null(NullType::Null)),
        _ => return Err(ExpressionError::type_error("keys函数需要顶点、边或映射类型")),
    }

    let result: Vec<Value> = keys.into_iter().map(Value::String).collect();
    Ok(Value::List(List { values: result }))
}

fn execute_reverse_list(args: &[Value]) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::type_error("reverse函数需要1个参数"));
    }
    match &args[0] {
        Value::List(list) => {
            let mut reversed = list.values.clone();
            reversed.reverse();
            Ok(Value::List(List { values: reversed }))
        }
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("reverse函数需要列表类型")),
    }
}

fn execute_toset(args: &[Value]) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::type_error("toset函数需要1个参数"));
    }
    match &args[0] {
        Value::List(list) => {
            let set: std::collections::HashSet<Value> = list.values.iter().cloned().collect();
            Ok(Value::Set(set))
        }
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("toset函数需要列表类型")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_head_function() {
        let list = Value::List(List {
            values: vec![Value::Int(1), Value::Int(2), Value::Int(3)],
        });
        let result = ContainerFunction::Head.execute(&[list]).expect("head函数执行应该成功");
        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_head_empty_list() {
        let list = Value::List(List { values: vec![] });
        let result = ContainerFunction::Head.execute(&[list]).expect("head函数执行应该成功");
        assert_eq!(result, Value::Null(NullType::Null));
    }

    #[test]
    fn test_last_function() {
        let list = Value::List(List {
            values: vec![Value::Int(1), Value::Int(2), Value::Int(3)],
        });
        let result = ContainerFunction::Last.execute(&[list]).expect("last函数执行应该成功");
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_tail_function() {
        let list = Value::List(List {
            values: vec![Value::Int(1), Value::Int(2), Value::Int(3)],
        });
        let result = ContainerFunction::Tail.execute(&[list]).expect("tail函数执行应该成功");
        assert_eq!(
            result,
            Value::List(List {
                values: vec![Value::Int(2), Value::Int(3)]
            })
        );
    }

    #[test]
    fn test_size_string() {
        let result = ContainerFunction::Size
            .execute(&[Value::String("hello".to_string())])
            .expect("size函数执行应该成功");
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_size_list() {
        let list = Value::List(List {
            values: vec![Value::Int(1), Value::Int(2), Value::Int(3)],
        });
        let result = ContainerFunction::Size.execute(&[list]).expect("size函数执行应该成功");
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_range_basic() {
        let result = ContainerFunction::Range
            .execute(&[Value::Int(1), Value::Int(5)])
            .expect("range函数执行应该成功");
        assert_eq!(
            result,
            Value::List(List {
                values: vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4), Value::Int(5)]
            })
        );
    }

    #[test]
    fn test_range_with_step() {
        let result = ContainerFunction::Range
            .execute(&[Value::Int(0), Value::Int(10), Value::Int(2)])
            .expect("range函数执行应该成功");
        assert_eq!(
            result,
            Value::List(List {
                values: vec![Value::Int(0), Value::Int(2), Value::Int(4), Value::Int(6), Value::Int(8), Value::Int(10)]
            })
        );
    }

    #[test]
    fn test_null_handling() {
        let null_value = Value::Null(NullType::Null);

        assert_eq!(
            ContainerFunction::Head.execute(&[null_value.clone()]).expect("head函数应该处理NULL"),
            Value::Null(NullType::Null)
        );
        assert_eq!(
            ContainerFunction::Last.execute(&[null_value.clone()]).expect("last函数应该处理NULL"),
            Value::Null(NullType::Null)
        );
        assert_eq!(
            ContainerFunction::Tail.execute(&[null_value.clone()]).expect("tail函数应该处理NULL"),
            Value::Null(NullType::Null)
        );
        assert_eq!(
            ContainerFunction::Size.execute(&[null_value.clone()]).expect("size函数应该处理NULL"),
            Value::Null(NullType::Null)
        );
    }
}
