//! 结果迭代器枚举 - 使用静态分发替代 Arc<dyn ResultIterator>
//!
//! 提供 ResultIteratorEnum 枚举，包含所有具体迭代器类型
//! 避免动态分发的性能开销

use crate::core::query_result::iterator::{DefaultIterator, GetNeighborsIterator, PropIterator};
use crate::core::query_result::result_iterator::ResultIterator;
use crate::core::value::Value;
use crate::core::DBResult;

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

    /// 收集所有元素
    pub fn collect(&mut self) -> DBResult<Vec<Vec<Value>>> {
        Iterator::collect(self)
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
            ResultIteratorEnum::Empty => {}
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

impl Iterator for ResultIteratorEnum {
    type Item = DBResult<Vec<Value>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ResultIteratorEnum::Default(iter) => iter.next().transpose(),
            ResultIteratorEnum::GetNeighbors(iter) => iter.next().transpose(),
            ResultIteratorEnum::Prop(iter) => iter.next().transpose(),
            ResultIteratorEnum::Empty => None,
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

        let row1 = iter
            .next()
            .expect("获取第一行失败")
            .expect("第一行不应为空");
        assert_eq!(row1[0], Value::Int(1));

        let row2 = iter
            .next()
            .expect("获取第二行失败")
            .expect("第二行不应为空");
        assert_eq!(row2[0], Value::Int(2));

        assert!(iter.next().is_none());
    }

    #[test]
    fn test_empty_iterator_enum() {
        let mut iter = ResultIteratorEnum::empty();
        assert!(iter.is_empty());
        assert_eq!(iter.size(), 0);
        assert!(iter.next().is_none());
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

        let row1 = iter
            .next()
            .expect("获取第一行失败")
            .expect("第一行不应为空");
        assert_eq!(row1[0], Value::Int(1));

        let row2 = iter
            .next()
            .expect("获取第二行失败")
            .expect("第二行不应为空");
        assert_eq!(row2[0], Value::Int(2));
    }

    #[test]
    fn test_prop_iterator_enum() {
        let props = vec![
            vec![
                Value::String("name".to_string()),
                Value::String("Alice".to_string()),
            ],
            vec![Value::String("age".to_string()), Value::Int(25)],
        ];

        let mut iter = ResultIteratorEnum::prop(props);

        assert_eq!(iter.size(), 2);

        let prop1 = iter
            .next()
            .expect("获取第一个属性失败")
            .expect("第一个属性不应为空");
        assert_eq!(prop1[0], Value::String("name".to_string()));

        let prop2 = iter
            .next()
            .expect("获取第二个属性失败")
            .expect("第二个属性不应为空");
        assert_eq!(prop2[0], Value::String("age".to_string()));
    }

    #[test]
    fn test_iterator_enum_clone() {
        let rows = vec![vec![Value::Int(1)]];
        let mut iter1 = ResultIteratorEnum::default_iterator(rows);

        let _ = iter1.next().transpose().expect("迭代不应失败");

        let mut iter2 = iter1.clone();
        let row = iter2
            .next()
            .expect("克隆后迭代不应失败")
            .expect("克隆后第一行不应为空");
        assert_eq!(row[0], Value::Int(1));
    }

    #[test]
    fn test_iterator_trait_impl() {
        let rows = vec![
            vec![Value::Int(1), Value::String("Alice".to_string())],
            vec![Value::Int(2), Value::String("Bob".to_string())],
        ];

        let mut iter = ResultIteratorEnum::default_iterator(rows);

        // 测试标准的 Iterator trait
        let collected = iter.by_ref().collect().expect("收集结果失败");
        assert_eq!(collected.len(), 2);

        // 验证收集到的结果
        let row1 = &collected[0];
        assert_eq!(row1[0], Value::Int(1));

        let row2 = &collected[1];
        assert_eq!(row2[0], Value::Int(2));
    }

    #[test]
    fn test_iterator_trait_empty() {
        let mut iter = ResultIteratorEnum::empty();

        // 测试空迭代器
        let item = iter.next();
        assert!(item.is_none());
    }

    #[test]
    fn test_iterator_trait_error_handling() {
        let mut iter = ResultIteratorEnum::empty();

        // 使用 next() 应该返回 None
        assert!(iter.next().is_none());
    }
}
