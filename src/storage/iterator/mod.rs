//! 迭代器模块 - 支持各种数据遍历
//!
//! 对应原C++中的Iterator.h/cpp
//! 提供：
//! - IteratorCore: 零成本抽象的迭代器核心trait
//! - DefaultIter: 单值迭代器
//! - SequentialIter: 顺序迭代器（DataSet行级）
//! - GetNeighborsIter: 邻居查询迭代器（图遍历）
//! - PropIter: 属性查询迭代器

pub mod default_iter;
pub mod get_neighbors_iter;
pub mod prop_iter;
pub mod sequential_iter;

pub use default_iter::DefaultIter;
pub use get_neighbors_iter::GetNeighborsIter;
pub use prop_iter::PropIter;
pub use sequential_iter::SequentialIter;

use crate::core::Value;
use std::fmt::Debug;

/// 行定义 - Vec<Value> 表示一行数据
pub type Row = Vec<Value>;

/// 迭代器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IteratorKind {
    /// 默认常量迭代器
    Default,
    /// 顺序迭代器（用于 DataSet）
    Sequential,
    /// 邻居迭代器（用于图遍历结果）
    GetNeighbors,
    /// 属性迭代器
    Prop,
}

/// 零成本抽象的迭代器核心trait
///
/// 使用泛型实现编译时多态，消除动态分发开销
/// 支持以下操作：
/// 1. 基本迭代：next、valid、reset
/// 2. 删除操作：erase、unstable_erase、clear、erase_range
/// 3. 范围操作：select、sample
/// 4. 行访问：row、move_row、size、is_empty
/// 5. 列访问：get_column、get_column_by_index、get_column_index、get_col_names
/// 6. 图特定：get_tag_prop、get_edge_prop、get_vertex、get_edge
/// 7. 复制：copy（返回具体类型）
pub trait Iterator: Send + Sync + Debug + Clone {
    /// 返回迭代器类型
    fn kind(&self) -> IteratorKind;

    /// 检查当前位置是否有效
    fn valid(&self) -> bool;

    /// 移动到下一行
    fn next(&mut self);

    /// 删除当前行（有序）
    fn erase(&mut self);

    /// 快速删除当前行（破坏顺序，用于优化）
    fn unstable_erase(&mut self);

    /// 清空所有行
    fn clear(&mut self);

    /// 重置到指定位置（默认 0）
    fn reset(&mut self, pos: usize);

    /// 获取总行数
    fn size(&self) -> usize;

    /// 检查是否为空
    fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// 获取当前行（如果有效）
    fn row(&self) -> Option<&Row>;

    /// 移动当前行（消费所有权）
    fn move_row(&mut self) -> Option<Row>;

    /// 选择范围内的行 [offset, offset + count)
    fn select(&mut self, offset: usize, count: usize);

    /// 采样指定数量的行
    fn sample(&mut self, count: i64);

    /// 删除范围 [first, last)
    fn erase_range(&mut self, first: usize, last: usize);

    /// 按列名获取值
    fn get_column(&self, col: &str) -> Option<&Value>;

    /// 按列索引获取值
    fn get_column_by_index(&self, index: i32) -> Option<&Value>;

    /// 获取列索引
    fn get_column_index(&self, col: &str) -> Option<usize>;

    /// 获取所有列名
    fn get_col_names(&self) -> Vec<String>;

    /// 深拷贝迭代器（返回具体类型，零成本抽象）
    fn copy(&self) -> Self;

    /// 类型检查方法
    fn is_default_iter(&self) -> bool {
        self.kind() == IteratorKind::Default
    }

    fn is_sequential_iter(&self) -> bool {
        self.kind() == IteratorKind::Sequential
    }

    fn is_get_neighbors_iter(&self) -> bool {
        self.kind() == IteratorKind::GetNeighbors
    }

    fn is_prop_iter(&self) -> bool {
        self.kind() == IteratorKind::Prop
    }

    // 图特定的方法（可选实现）
    /// 获取标签属性值
    fn get_tag_prop(&self, _tag: &str, _prop: &str) -> Option<Value> {
        None
    }

    /// 获取边属性值
    fn get_edge_prop(&self, _edge: &str, _prop: &str) -> Option<Value> {
        None
    }

    /// 获取顶点
    fn get_vertex(&self, _name: &str) -> Option<Value> {
        None
    }

    /// 获取边
    fn get_edge(&self) -> Option<Value> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iterator_kind_equality() {
        assert_eq!(IteratorKind::Default, IteratorKind::Default);
        assert_ne!(IteratorKind::Default, IteratorKind::Sequential);
    }
}
