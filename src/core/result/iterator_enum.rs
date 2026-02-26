//! 结果迭代器枚举 - 使用静态分发替代 Arc<dyn ResultIterator>
//!
//! 提供 ResultIteratorEnum 枚举，包含所有具体迭代器类型
//! 避免动态分发的性能开销

use crate::core::value::Value;
use crate::core::DBResult;
use crate::core::result::result_iterator::ResultIterator;
use crate::core::result::iterator::{DefaultIterator, GetNeighborsIterator, PropIterator};

/// 结果迭代器枚举 - 使用静态分发
///
/// 包含所有可能的结果迭代器类型，通过 match 实现静态分发
#[derive(Debug)]
pub enum ResultIteratorEnum {
    /// 默认迭代器
    Default(DefaultIterator),
    /// 邻居查询迭代器
    GetNeighbors(GetNeighborsIterator),
    /// 属性迭代器
    Prop(PropIterator),
    /// 空迭代器
    Empty,
}

impl ResultIteratorEnum {
    /// 创建默认迭代器
    pub fn default_iterator(rows: Vec<Vec<Value>>) -> Self {
        ResultIteratorEnum::Default(DefaultIterator::new(rows))
    }

    /// 创建邻居查询迭代器
    pub fn get_neighbors(vertices: Vec<Value>, edges: Vec<Vec<Value>>) -> Self {
        ResultIteratorEnum::GetNeighbors(GetNeighborsIterator::new(vertices, edges))
    }

    /// 创建属性迭代器
    pub fn prop(props: Vec<Vec<Value>>) -> Self {
        ResultIteratorEnum::Prop(PropIterator::new(props))
    }

    /// 创建空迭代器
    pub fn empty() -> Self {
        ResultIteratorEnum::Empty
    }

    /// 获取下一行
    pub fn next(&mut self) -> DBResult<Option<Vec<Value>>> {
        match self {
            ResultIteratorEnum::Default(iter) => iter.next(),
            ResultIteratorEnum::GetNeighbors(iter) => iter.next(),
            ResultIteratorEnum::Prop(iter) => iter.next(),
            ResultIteratorEnum::Empty => Ok(None),
        }
    }

    /// 偷看下一行（不移动迭代器）
    pub fn peek(&self) -> DBResult<Option<&Vec<Value>>> {
        match self {
            ResultIteratorEnum::Default(iter) => iter.peek(),
            ResultIteratorEnum::GetNeighbors(iter) => iter.peek(),
            ResultIteratorEnum::Prop(iter) => iter.peek(),
            ResultIteratorEnum::Empty => Ok(None),
        }
    }

    /// 获取大小提示
    pub fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            ResultIteratorEnum::Default(iter) => iter.size_hint(),
            ResultIteratorEnum::GetNeighbors(iter) => iter.size_hint(),
            ResultIteratorEnum::Prop(iter) => iter.size_hint(),
            ResultIteratorEnum::Empty => (0, Some(0)),
        }
    }

    /// 跳过 n 个元素
    pub fn nth(&mut self, n: usize) -> DBResult<Option<Vec<Value>>> {
        match self {
            ResultIteratorEnum::Default(iter) => iter.nth(n),
            ResultIteratorEnum::GetNeighbors(iter) => iter.nth(n),
            ResultIteratorEnum::Prop(iter) => iter.nth(n),
            ResultIteratorEnum::Empty => Ok(None),
        }
    }

    /// 获取最后一个元素
    pub fn last(&mut self) -> DBResult<Option<Vec<Value>>> {
        match self {
            ResultIteratorEnum::Default(iter) => iter.last(),
            ResultIteratorEnum::GetNeighbors(iter) => iter.last(),
            ResultIteratorEnum::Prop(iter) => iter.last(),
            ResultIteratorEnum::Empty => Ok(None),
        }
    }

    /// 收集所有元素
    pub fn collect(&mut self) -> DBResult<Vec<Vec<Value>>> {
        let mut results = Vec::new();
        while let Some(row) = self.next()? {
            results.push(row);
        }
        Ok(results)
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        match self {
            ResultIteratorEnum::Default(iter) => iter.is_empty(),
            ResultIteratorEnum::GetNeighbors(iter) => iter.is_empty(),
            ResultIteratorEnum::Prop(iter) => iter.is_empty(),
            ResultIteratorEnum::Empty => true,
        }
    }

    /// 获取大小
    pub fn size(&self) -> usize {
        match self {
            ResultIteratorEnum::Default(iter) => iter.size(),
            ResultIteratorEnum::GetNeighbors(iter) => iter.size(),
            ResultIteratorEnum::Prop(iter) => iter.size(),
            ResultIteratorEnum::Empty => 0,
        }
    }

    /// 重置迭代器
    pub fn reset(&mut self) {
        match self {
            ResultIteratorEnum::Default(iter) => iter.reset(),
            ResultIteratorEnum::GetNeighbors(iter) => iter.reset(),
            ResultIteratorEnum::Prop(iter) => iter.reset(),
            ResultIteratorEnum::Empty => {},
        }
    }
}

impl Clone for ResultIteratorEnum {
    fn clone(&self) -> Self {
        match self {
            ResultIteratorEnum::Default(iter) => {
                // DefaultIterator 可以通过重新创建来克隆
                let rows: Vec<Vec<Value>> = iter.rows().to_vec();
                ResultIteratorEnum::Default(DefaultIterator::new(rows))
            }
            ResultIteratorEnum::GetNeighbors(iter) => {
                let vertices = iter.vertices().to_vec();
                let edges = iter.edges().to_vec();
                ResultIteratorEnum::GetNeighbors(GetNeighborsIterator::new(vertices, edges))
            }
            ResultIteratorEnum::Prop(iter) => {
                let props = iter.props().to_vec();
                ResultIteratorEnum::Prop(PropIterator::new(props))
            }
            ResultIteratorEnum::Empty => ResultIteratorEnum::Empty,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_iterator_enum() {
        let rows = vec![
            vec![Value::Int(1), Value::String("Alice".to_string())],
            vec![Value::Int(2), Value::String("Bob".to_string())],
        ];

        let mut iter = ResultIteratorEnum::default_iterator(rows);

        assert_eq!(iter.size(), 2);

        let row1 = iter.next().unwrap().unwrap();
        assert_eq!(row1[0], Value::Int(1));

        let row2 = iter.next().unwrap().unwrap();
        assert_eq!(row2[0], Value::Int(2));

        assert!(iter.next().unwrap().is_none());
    }

    #[test]
    fn test_empty_iterator_enum() {
        let mut iter = ResultIteratorEnum::empty();
        assert!(iter.is_empty());
        assert_eq!(iter.size(), 0);
        assert!(iter.next().unwrap().is_none());
    }

    #[test]
    fn test_get_neighbors_iterator_enum() {
        let vertices = vec![Value::Int(1), Value::Int(2)];
        let edges = vec![
            vec![Value::String("edge1".to_string())],
            vec![Value::String("edge2".to_string())],
        ];

        let mut iter = ResultIteratorEnum::get_neighbors(vertices, edges);

        assert_eq!(iter.size(), 2);

        let row1 = iter.next().unwrap().unwrap();
        assert_eq!(row1[0], Value::Int(1));

        let row2 = iter.next().unwrap().unwrap();
        assert_eq!(row2[0], Value::Int(2));
    }

    #[test]
    fn test_prop_iterator_enum() {
        let props = vec![
            vec![Value::String("name".to_string()), Value::String("Alice".to_string())],
            vec![Value::String("age".to_string()), Value::Int(25)],
        ];

        let mut iter = ResultIteratorEnum::prop(props);

        assert_eq!(iter.size(), 2);

        let prop1 = iter.next().unwrap().unwrap();
        assert_eq!(prop1[0], Value::String("name".to_string()));

        let prop2 = iter.next().unwrap().unwrap();
        assert_eq!(prop2[0], Value::String("age".to_string()));
    }

    #[test]
    fn test_iterator_enum_clone() {
        let rows = vec![vec![Value::Int(1)]];
        let mut iter1 = ResultIteratorEnum::default_iterator(rows);

        iter1.next().unwrap();

        let mut iter2 = iter1.clone();
        // 克隆后应该能重新迭代
        let row = iter2.next().unwrap().unwrap();
        assert_eq!(row[0], Value::Int(1));
    }
}
