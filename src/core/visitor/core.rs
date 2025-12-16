//! 访问者模式核心定义
//!
//! 这个模块提供了访问者模式的核心 trait 和基础实现

use crate::core::value::{
    DataSet, DateTimeValue, DateValue, DurationValue, GeographyValue, NullType, TimeValue, Value,
};
use crate::core::vertex_edge_path::{Edge, Path, Vertex};
use std::collections::HashMap;

/// Value 访问者 trait
///
/// 这个 trait 定义了访问者模式的核心接口，允许对 Value 类型进行操作而不修改其结构
///
/// # 示例
/// ```
/// use graphdb::core::visitor::{ValueVisitor, ValueAcceptor};
///
/// struct MyVisitor {
///     count: usize,
/// }
///
/// impl ValueVisitor for MyVisitor {
///     type Result = ();
///     
///     fn visit_int(&mut self, _value: i64) -> Self::Result {
///         self.count += 1;
///     }
///     
///     fn visit_string(&mut self, _value: &str) -> Self::Result {
///         self.count += 1;
///     }
///     
///     // ... 其他 visit 方法
/// }
///
/// let value = Value::Int(42);
/// let mut visitor = MyVisitor { count: 0 };
/// value.accept(&mut visitor);
/// assert_eq!(visitor.count, 1);
/// ```
pub trait ValueVisitor {
    type Result;

    fn visit_bool(&mut self, value: bool) -> Self::Result;
    fn visit_int(&mut self, value: i64) -> Self::Result;
    fn visit_float(&mut self, value: f64) -> Self::Result;
    fn visit_string(&mut self, value: &str) -> Self::Result;
    fn visit_date(&mut self, value: &DateValue) -> Self::Result;
    fn visit_time(&mut self, value: &TimeValue) -> Self::Result;
    fn visit_datetime(&mut self, value: &DateTimeValue) -> Self::Result;
    fn visit_vertex(&mut self, value: &Vertex) -> Self::Result;
    fn visit_edge(&mut self, value: &Edge) -> Self::Result;
    fn visit_path(&mut self, value: &Path) -> Self::Result;
    fn visit_list(&mut self, value: &[Value]) -> Self::Result;
    fn visit_map(&mut self, value: &HashMap<String, Value>) -> Self::Result;
    fn visit_set(&mut self, value: &std::collections::HashSet<Value>) -> Self::Result;
    fn visit_geography(&mut self, value: &GeographyValue) -> Self::Result;
    fn visit_duration(&mut self, value: &DurationValue) -> Self::Result;
    fn visit_dataset(&mut self, value: &DataSet) -> Self::Result;
    fn visit_null(&mut self, null_type: &NullType) -> Self::Result;
    fn visit_empty(&mut self) -> Self::Result;
}

/// Value 访问者接受器 trait
///
/// 这个 trait 为 Value 类型提供了接受访问者的能力，实现了访问者模式的"可访问性"部分
pub trait ValueAcceptor {
    /// 接受访问者进行访问
    fn accept<V: ValueVisitor>(&self, visitor: &mut V) -> V::Result;
}

impl ValueAcceptor for Value {
    fn accept<V: ValueVisitor>(&self, visitor: &mut V) -> V::Result {
        match self {
            Value::Bool(b) => visitor.visit_bool(*b),
            Value::Int(i) => visitor.visit_int(*i),
            Value::Float(f) => visitor.visit_float(*f),
            Value::String(s) => visitor.visit_string(s),
            Value::Date(d) => visitor.visit_date(d),
            Value::Time(t) => visitor.visit_time(t),
            Value::DateTime(dt) => visitor.visit_datetime(dt),
            Value::Vertex(v) => visitor.visit_vertex(v),
            Value::Edge(e) => visitor.visit_edge(e),
            Value::Path(p) => visitor.visit_path(p),
            Value::List(l) => visitor.visit_list(l),
            Value::Map(m) => visitor.visit_map(m),
            Value::Set(s) => visitor.visit_set(s),
            Value::Geography(g) => visitor.visit_geography(g),
            Value::Duration(d) => visitor.visit_duration(d),
            Value::DataSet(ds) => visitor.visit_dataset(ds),
            Value::Null(nt) => visitor.visit_null(nt),
            Value::Empty => visitor.visit_empty(),
        }
    }
}

/// 访问者模式辅助工具
pub mod utils {
    use super::*;

    /// 递归访问辅助函数，避免栈溢出
    pub fn visit_recursive<V: ValueVisitor>(
        value: &Value,
        visitor: &mut V,
        depth: usize,
        max_depth: usize,
    ) -> Result<V::Result, RecursionError> {
        if depth > max_depth {
            return Err(RecursionError::MaxDepthExceeded);
        }

        match value {
            Value::Bool(b) => Ok(visitor.visit_bool(*b)),
            Value::Int(i) => Ok(visitor.visit_int(*i)),
            Value::Float(f) => Ok(visitor.visit_float(*f)),
            Value::String(s) => Ok(visitor.visit_string(s)),
            Value::Date(d) => Ok(visitor.visit_date(d)),
            Value::Time(t) => Ok(visitor.visit_time(t)),
            Value::DateTime(dt) => Ok(visitor.visit_datetime(dt)),
            Value::Vertex(v) => Ok(visitor.visit_vertex(v)),
            Value::Edge(e) => Ok(visitor.visit_edge(e)),
            Value::Path(p) => Ok(visitor.visit_path(p)),
            Value::List(l) => {
                // For lists, we can't call visitor.visit_list directly on the results
                // We need to transform the list first and then call visit_list
                // This is handled by the specific visitor implementation
                Ok(visitor.visit_list(l))
            }
            Value::Map(m) => Ok(visitor.visit_map(m)),
            Value::Set(s) => Ok(visitor.visit_set(s)),
            Value::Geography(g) => Ok(visitor.visit_geography(g)),
            Value::Duration(d) => Ok(visitor.visit_duration(d)),
            Value::DataSet(ds) => Ok(visitor.visit_dataset(ds)),
            Value::Null(nt) => Ok(visitor.visit_null(nt)),
            Value::Empty => Ok(visitor.visit_empty()),
        }
    }

    /// 递归错误类型
    #[derive(Debug, thiserror::Error)]
    pub enum RecursionError {
        #[error("递归深度超过最大限制")]
        MaxDepthExceeded,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::Value;

    #[test]
    fn test_value_acceptor() {
        struct CountVisitor {
            count: usize,
        }

        impl ValueVisitor for CountVisitor {
            type Result = ();

            fn visit_int(&mut self, _value: i64) -> Self::Result {
                self.count += 1;
            }

            fn visit_string(&mut self, _value: &str) -> Self::Result {
                self.count += 1;
            }

            fn visit_bool(&mut self, _value: bool) -> Self::Result {
                self.count += 1;
            }

            fn visit_float(&mut self, _value: f64) -> Self::Result {
                self.count += 1;
            }

            fn visit_date(&mut self, _value: &DateValue) -> Self::Result {
                self.count += 1;
            }

            fn visit_time(&mut self, _value: &TimeValue) -> Self::Result {
                self.count += 1;
            }

            fn visit_datetime(&mut self, _value: &DateTimeValue) -> Self::Result {
                self.count += 1;
            }

            fn visit_vertex(&mut self, _value: &Vertex) -> Self::Result {
                self.count += 1;
            }

            fn visit_edge(&mut self, _value: &Edge) -> Self::Result {
                self.count += 1;
            }

            fn visit_path(&mut self, _value: &Path) -> Self::Result {
                self.count += 1;
            }

            fn visit_list(&mut self, _value: &[Value]) -> Self::Result {
                self.count += 1;
            }

            fn visit_map(&mut self, _value: &HashMap<String, Value>) -> Self::Result {
                self.count += 1;
            }

            fn visit_set(&mut self, _value: &std::collections::HashSet<Value>) -> Self::Result {
                self.count += 1;
            }

            fn visit_geography(&mut self, _value: &GeographyValue) -> Self::Result {
                self.count += 1;
            }

            fn visit_duration(&mut self, _value: &DurationValue) -> Self::Result {
                self.count += 1;
            }

            fn visit_dataset(&mut self, _value: &DataSet) -> Self::Result {
                self.count += 1;
            }

            fn visit_null(&mut self, _null_type: &NullType) -> Self::Result {
                self.count += 1;
            }

            fn visit_empty(&mut self) -> Self::Result {
                self.count += 1;
            }
        }

        let value = Value::Int(42);
        let mut visitor = CountVisitor { count: 0 };
        value.accept(&mut visitor);
        assert_eq!(visitor.count, 1);
    }
}
