//! 枚举迭代器 - 替代动态分发的迭代器系统
//!
//! 使用枚举模式替代 `Box<dyn Iterator>`，消除动态分发开销

use super::{Iterator, IteratorKind, Row};
use crate::core::Value;

/// 迭代器枚举类型
/// 
/// 替代 `Box<dyn Iterator>` 的枚举实现
/// 消除动态分发开销，提供更好的性能
#[derive(Debug, Clone)]
pub enum IteratorEnum {
    /// 默认常量迭代器
    Default(DefaultIter),
    /// 顺序迭代器（用于 DataSet）
    Sequential(SequentialIter),
    /// 邻居迭代器（用于图遍历结果）
    GetNeighbors(GetNeighborsIter),
    /// 属性迭代器
    Prop(PropIter),
}

impl IteratorEnum {
    /// 获取迭代器类型
    pub fn kind(&self) -> IteratorKind {
        match self {
            IteratorEnum::Default(_) => IteratorKind::Default,
            IteratorEnum::Sequential(_) => IteratorKind::Sequential,
            IteratorEnum::GetNeighbors(_) => IteratorKind::GetNeighbors,
            IteratorEnum::Prop(_) => IteratorKind::Prop,
        }
    }

    /// 检查当前位置是否有效
    pub fn valid(&self) -> bool {
        match self {
            IteratorEnum::Default(iter) => iter.valid(),
            IteratorEnum::Sequential(iter) => iter.valid(),
            IteratorEnum::GetNeighbors(iter) => iter.valid(),
            IteratorEnum::Prop(iter) => iter.valid(),
        }
    }

    /// 移动到下一行
    pub fn next(&mut self) {
        match self {
            IteratorEnum::Default(iter) => iter.next(),
            IteratorEnum::Sequential(iter) => iter.next(),
            IteratorEnum::GetNeighbors(iter) => iter.next(),
            IteratorEnum::Prop(iter) => iter.next(),
        }
    }

    /// 删除当前行（有序）
    pub fn erase(&mut self) {
        match self {
            IteratorEnum::Default(iter) => iter.erase(),
            IteratorEnum::Sequential(iter) => iter.erase(),
            IteratorEnum::GetNeighbors(iter) => iter.erase(),
            IteratorEnum::Prop(iter) => iter.erase(),
        }
    }

    /// 快速删除当前行（破坏顺序，用于优化）
    pub fn unstable_erase(&mut self) {
        match self {
            IteratorEnum::Default(iter) => iter.unstable_erase(),
            IteratorEnum::Sequential(iter) => iter.unstable_erase(),
            IteratorEnum::GetNeighbors(iter) => iter.unstable_erase(),
            IteratorEnum::Prop(iter) => iter.unstable_erase(),
        }
    }

    /// 清空所有行
    pub fn clear(&mut self) {
        match self {
            IteratorEnum::Default(iter) => iter.clear(),
            IteratorEnum::Sequential(iter) => iter.clear(),
            IteratorEnum::GetNeighbors(iter) => iter.clear(),
            IteratorEnum::Prop(iter) => iter.clear(),
        }
    }

    /// 重置到指定位置（默认 0）
    pub fn reset(&mut self, pos: usize) {
        match self {
            IteratorEnum::Default(iter) => iter.reset(pos),
            IteratorEnum::Sequential(iter) => iter.reset(pos),
            IteratorEnum::GetNeighbors(iter) => iter.reset(pos),
            IteratorEnum::Prop(iter) => iter.reset(pos),
        }
    }

    /// 获取总行数
    pub fn size(&self) -> usize {
        match self {
            IteratorEnum::Default(iter) => iter.size(),
            IteratorEnum::Sequential(iter) => iter.size(),
            IteratorEnum::GetNeighbors(iter) => iter.size(),
            IteratorEnum::Prop(iter) => iter.size(),
        }
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// 获取当前行（如果有效）
    pub fn row(&self) -> Option<&Row> {
        match self {
            IteratorEnum::Default(iter) => iter.row(),
            IteratorEnum::Sequential(iter) => iter.row(),
            IteratorEnum::GetNeighbors(iter) => iter.row(),
            IteratorEnum::Prop(iter) => iter.row(),
        }
    }

    /// 移动当前行（消费所有权）
    pub fn move_row(&mut self) -> Option<Row> {
        match self {
            IteratorEnum::Default(iter) => iter.move_row(),
            IteratorEnum::Sequential(iter) => iter.move_row(),
            IteratorEnum::GetNeighbors(iter) => iter.move_row(),
            IteratorEnum::Prop(iter) => iter.move_row(),
        }
    }

    /// 选择范围内的行 [offset, offset + count)
    pub fn select(&mut self, offset: usize, count: usize) {
        match self {
            IteratorEnum::Default(iter) => iter.select(offset, count),
            IteratorEnum::Sequential(iter) => iter.select(offset, count),
            IteratorEnum::GetNeighbors(iter) => iter.select(offset, count),
            IteratorEnum::Prop(iter) => iter.select(offset, count),
        }
    }

    /// 采样指定数量的行
    pub fn sample(&mut self, count: i64) {
        match self {
            IteratorEnum::Default(iter) => iter.sample(count),
            IteratorEnum::Sequential(iter) => iter.sample(count),
            IteratorEnum::GetNeighbors(iter) => iter.sample(count),
            IteratorEnum::Prop(iter) => iter.sample(count),
        }
    }

    /// 删除范围 [first, last)
    pub fn erase_range(&mut self, first: usize, last: usize) {
        match self {
            IteratorEnum::Default(iter) => iter.erase_range(first, last),
            IteratorEnum::Sequential(iter) => iter.erase_range(first, last),
            IteratorEnum::GetNeighbors(iter) => iter.erase_range(first, last),
            IteratorEnum::Prop(iter) => iter.erase_range(first, last),
        }
    }

    /// 按列名获取值
    pub fn get_column(&self, col: &str) -> Option<&Value> {
        match self {
            IteratorEnum::Default(iter) => iter.get_column(col),
            IteratorEnum::Sequential(iter) => iter.get_column(col),
            IteratorEnum::GetNeighbors(iter) => iter.get_column(col),
            IteratorEnum::Prop(iter) => iter.get_column(col),
        }
    }

    /// 按列索引获取值
    pub fn get_column_by_index(&self, index: i32) -> Option<&Value> {
        match self {
            IteratorEnum::Default(iter) => iter.get_column_by_index(index),
            IteratorEnum::Sequential(iter) => iter.get_column_by_index(index),
            IteratorEnum::GetNeighbors(iter) => iter.get_column_by_index(index),
            IteratorEnum::Prop(iter) => iter.get_column_by_index(index),
        }
    }

    /// 获取列索引
    pub fn get_column_index(&self, col: &str) -> Option<usize> {
        match self {
            IteratorEnum::Default(iter) => iter.get_column_index(col),
            IteratorEnum::Sequential(iter) => iter.get_column_index(col),
            IteratorEnum::GetNeighbors(iter) => iter.get_column_index(col),
            IteratorEnum::Prop(iter) => iter.get_column_index(col),
        }
    }

    /// 获取所有列名
    pub fn get_col_names(&self) -> Vec<String> {
        match self {
            IteratorEnum::Default(iter) => iter.get_col_names(),
            IteratorEnum::Sequential(iter) => iter.get_col_names(),
            IteratorEnum::GetNeighbors(iter) => iter.get_col_names(),
            IteratorEnum::Prop(iter) => iter.get_col_names(),
        }
    }

    /// 深拷贝迭代器（无动态分发）
    pub fn copy(&self) -> IteratorEnum {
        self.clone()
    }

    /// 类型检查方法
    pub fn is_default_iter(&self) -> bool {
        self.kind() == IteratorKind::Default
    }

    pub fn is_sequential_iter(&self) -> bool {
        self.kind() == IteratorKind::Sequential
    }

    pub fn is_get_neighbors_iter(&self) -> bool {
        self.kind() == IteratorKind::GetNeighbors
    }

    pub fn is_prop_iter(&self) -> bool {
        self.kind() == IteratorKind::Prop
    }

    // 图特定的方法
    /// 获取标签属性值
    pub fn get_tag_prop(&self, tag: &str, prop: &str) -> Option<Value> {
        match self {
            IteratorEnum::Default(iter) => iter.get_tag_prop(tag, prop),
            IteratorEnum::Sequential(iter) => iter.get_tag_prop(tag, prop),
            IteratorEnum::GetNeighbors(iter) => iter.get_tag_prop(tag, prop),
            IteratorEnum::Prop(iter) => iter.get_tag_prop(tag, prop),
        }
    }

    /// 获取边属性值
    pub fn get_edge_prop(&self, edge: &str, prop: &str) -> Option<Value> {
        match self {
            IteratorEnum::Default(iter) => iter.get_edge_prop(edge, prop),
            IteratorEnum::Sequential(iter) => iter.get_edge_prop(edge, prop),
            IteratorEnum::GetNeighbors(iter) => iter.get_edge_prop(edge, prop),
            IteratorEnum::Prop(iter) => iter.get_edge_prop(edge, prop),
        }
    }

    /// 获取顶点
    pub fn get_vertex(&self, name: &str) -> Option<Value> {
        match self {
            IteratorEnum::Default(iter) => iter.get_vertex(name),
            IteratorEnum::Sequential(iter) => iter.get_vertex(name),
            IteratorEnum::GetNeighbors(iter) => iter.get_vertex(name),
            IteratorEnum::Prop(iter) => iter.get_vertex(name),
        }
    }

    /// 获取边
    pub fn get_edge(&self) -> Option<Value> {
        match self {
            IteratorEnum::Default(iter) => iter.get_edge(),
            IteratorEnum::Sequential(iter) => iter.get_edge(),
            IteratorEnum::GetNeighbors(iter) => iter.get_edge(),
            IteratorEnum::Prop(iter) => iter.get_edge(),
        }
    }

    /// 从 Box<dyn Iterator> 转换到 IteratorEnum
    pub fn from_boxed(iter: Box<dyn Iterator>) -> Option<Self> {
        // 由于 Box<dyn Iterator> 不支持 downcast，我们无法安全地进行转换
        // 这个方法暂时无法实现，需要保持向后兼容性
        None
    }

    /// 转换为 Box<dyn Iterator>（向后兼容）
    pub fn to_boxed(self) -> Box<dyn Iterator> {
        match self {
            IteratorEnum::Default(iter) => Box::new(iter),
            IteratorEnum::Sequential(iter) => Box::new(iter),
            IteratorEnum::GetNeighbors(iter) => Box::new(iter),
            IteratorEnum::Prop(iter) => Box::new(iter),
        }
    }
}

// 导入具体的迭代器类型
use super::default_iter::DefaultIter;
use super::get_neighbors_iter::GetNeighborsIter;
use super::prop_iter::PropIter;
use super::sequential_iter::SequentialIter;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_iterator_enum_creation() {
        let value = Value::Int(42);
        let default_iter = DefaultIter::from_value(value);
        let enum_iter = IteratorEnum::Default(default_iter);

        assert_eq!(enum_iter.kind(), IteratorKind::Default);
        assert!(enum_iter.valid());
        assert_eq!(enum_iter.size(), 1);
    }

    #[test]
    fn test_iterator_enum_copy() {
        let value = Value::Int(42);
        let default_iter = DefaultIter::from_value(value);
        let enum_iter = IteratorEnum::Default(default_iter);

        let copy = enum_iter.copy();
        assert_eq!(copy.kind(), IteratorKind::Default);
        assert!(copy.valid());
    }
    
    #[cfg(all(test, not(debug_assertions)))]
    mod benches {
        use super::*;
        use std::sync::Arc;
        use std::time::Instant;
    
        #[test]
        fn bench_iterator_copy_comparison() {
            // 测试枚举迭代器 vs Box<dyn Iterator> 的性能
            let value = Arc::new(Value::Int(42));
            let iter = IteratorEnum::Default(DefaultIter::new(value.clone()));
            
            // 预热
            for _ in 0..1000 {
                let _copy = iter.copy();
            }
            
            // 基准测试：枚举迭代器
            let start = Instant::now();
            for _ in 0..100000 {
                let _copy = iter.copy();
            }
            let enum_duration = start.elapsed();
            
            // 基准测试：Box<dyn Iterator>（通过 to_boxed 方法）
            let boxed = iter.to_boxed();
            let start = Instant::now();
            for _ in 0..100000 {
                let _copy = boxed.copy();
            }
            let boxed_duration = start.elapsed();
            
            println!("枚举迭代器 copy 时间: {:?}", enum_duration);
            println!("Box<dyn Iterator> copy 时间: {:?}", boxed_duration);
            println!("性能提升: {:.2}x", boxed_duration.as_nanos() as f64 / enum_duration.as_nanos() as f64);
            
            // 枚举迭代器应该更快
            assert!(enum_duration < boxed_duration);
        }
    
        #[test]
        fn bench_iterator_enum_creation() {
            let value = Arc::new(Value::Int(42));
            
            let start = Instant::now();
            for _ in 0..100000 {
                let _iter = IteratorEnum::Default(DefaultIter::new(value.clone()));
            }
            let duration = start.elapsed();
            
            println!("创建 100,000 个枚举迭代器: {:?}", duration);
            println!("平均每个: {:?}", duration / 100000);
        }
    }

    #[test]
    fn test_iterator_enum_methods() {
        let value = Value::Int(42);
        let default_iter = DefaultIter::from_value(value);
        let mut enum_iter = IteratorEnum::Default(default_iter);

        // 测试基本方法
        assert!(enum_iter.valid());
        assert_eq!(enum_iter.size(), 1);
        assert!(!enum_iter.is_empty());

        // 测试移动
        enum_iter.next();
        assert!(!enum_iter.valid());

        // 测试重置
        enum_iter.reset(0);
        assert!(enum_iter.valid());
    }

    #[test]
    fn test_iterator_enum_type_checks() {
        let value = Value::Int(42);
        let default_iter = DefaultIter::from_value(value);
        let enum_iter = IteratorEnum::Default(default_iter);

        assert!(enum_iter.is_default_iter());
        assert!(!enum_iter.is_sequential_iter());
        assert!(!enum_iter.is_get_neighbors_iter());
        assert!(!enum_iter.is_prop_iter());
    }
}