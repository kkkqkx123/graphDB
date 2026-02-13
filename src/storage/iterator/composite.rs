//! 组合迭代器 - 支持迭代器链式操作
//!
//! 提供迭代器的组合操作：
//! - FilterIter: 过滤迭代器
//! - MapIter: 映射迭代器
//! - TakeIter: 限制数量迭代器
//! - SkipIter: 跳过迭代器
//! - ChainIter: 链式迭代器

use super::{Iterator, IteratorKind, Row};
use crate::core::Value;
use std::fmt;
use std::sync::Arc;

/// 过滤迭代器 - 根据谓词过滤元素
///
/// 包装另一个迭代器，只返回满足条件的元素
pub struct FilterIter<I: Iterator> {
    inner: I,
    predicate: Arc<dyn Fn(&Row) -> bool + Send + Sync>,
}

impl<I: Iterator> fmt::Debug for FilterIter<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilterIter")
            .field("inner", &self.inner)
            .field("predicate", &"Arc<dyn Fn(&Row) -> bool + Send + Sync>")
            .finish()
    }
}

impl<I: Iterator> Clone for FilterIter<I> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            predicate: self.predicate.clone(),
        }
    }
}

impl<I: Iterator> FilterIter<I> {
    pub fn new(inner: I, predicate: impl Fn(&Row) -> bool + Send + Sync + 'static) -> Self {
        Self {
            inner,
            predicate: Arc::new(predicate),
        }
    }

    fn find_next_valid(&mut self) -> bool {
        while self.inner.valid() {
            if let Some(row) = self.inner.row() {
                if (self.predicate)(row) {
                    return true;
                }
            }
            self.inner.next();
        }
        false
    }
}

impl<I: Iterator> Iterator for FilterIter<I> {
    fn kind(&self) -> IteratorKind {
        IteratorKind::Default
    }

    fn valid(&self) -> bool {
        self.inner.valid()
    }

    fn next(&mut self) {
        self.inner.next();
        self.find_next_valid();
    }

    fn erase(&mut self) {
        self.inner.erase();
        self.find_next_valid();
    }

    fn unstable_erase(&mut self) {
        self.inner.unstable_erase();
        self.find_next_valid();
    }

    fn clear(&mut self) {
        self.inner.clear();
    }

    fn reset(&mut self, pos: usize) {
        self.inner.reset(pos);
        self.find_next_valid();
    }

    fn size(&self) -> usize {
        let mut count = 0;
        let mut temp = self.inner.copy();
        temp.reset(0);
        while temp.valid() {
            if let Some(row) = temp.row() {
                if (self.predicate)(row) {
                    count += 1;
                }
            }
            temp.next();
        }
        count
    }

    fn row(&self) -> Option<&Row> {
        self.inner.row()
    }

    fn move_row(&mut self) -> Option<Row> {
        self.inner.move_row()
    }

    fn add_row(&mut self, row: Row) {
        self.inner.add_row(row)
    }

    fn select(&mut self, offset: usize, count: usize) {
        let mut temp = Vec::new();
        let mut offset_remaining = offset;
        let mut count_remaining = count;

        self.inner.reset(0);
        while self.inner.valid() {
            if let Some(row) = self.inner.row() {
                if (self.predicate)(row) {
                    if offset_remaining > 0 {
                        offset_remaining -= 1;
                    } else if count_remaining > 0 {
                        temp.push(row.clone());
                        count_remaining -= 1;
                    }
                }
            }
            self.inner.next();
        }

        self.inner.clear();
        self.inner.reset(0);
        for row in temp {
            self.inner.add_row(row);
        }
    }

    fn sample(&mut self, count: i64) {
        let target = if count < 0 { 0 } else { count as usize };
        let total = self.size();
        if target >= total {
            return;
        }

        let step = total / target;
        let mut idx = 0;
        self.inner.reset(0);
        while self.inner.valid() {
            if let Some(row) = self.inner.row() {
                if (self.predicate)(row) {
                    if idx % step != 0 {
                        self.inner.erase();
                    }
                    idx += 1;
                }
            }
            self.inner.next();
        }
    }

    fn erase_range(&mut self, first: usize, last: usize) {
        let mut removed = 0;
        self.inner.reset(0);
        while self.inner.valid() && removed < last {
            if let Some(row) = self.inner.row() {
                if (self.predicate)(row) {
                    if removed >= first {
                        self.inner.erase();
                    }
                    removed += 1;
                }
            }
            self.inner.next();
        }
    }

    fn get_column(&self, col: &str) -> Option<&Value> {
        self.inner.get_column(col)
    }

    fn get_column_by_index(&self, index: i32) -> Option<&Value> {
        self.inner.get_column_by_index(index)
    }

    fn get_column_index(&self, col: &str) -> Option<usize> {
        self.inner.get_column_index(col)
    }

    fn get_col_names(&self) -> Vec<String> {
        self.inner.get_col_names()
    }

    fn copy(&self) -> Self {
        Self {
            inner: self.inner.copy(),
            predicate: self.predicate.clone(),
        }
    }
}

/// 映射迭代器 - 对每个元素应用转换函数
///
/// 包装另一个迭代器，对返回的每行进行转换
pub struct MapIter<I: Iterator> {
    inner: I,
    mapper: Arc<dyn Fn(Row) -> Row + Send + Sync>,
    current_row: Option<Row>,
}

impl<I: Iterator> fmt::Debug for MapIter<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MapIter")
            .field("inner", &self.inner)
            .field("mapper", &"Arc<dyn Fn(Row) -> Row + Send + Sync>")
            .finish()
    }
}

impl<I: Iterator> Clone for MapIter<I> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            mapper: self.mapper.clone(),
            current_row: self.current_row.clone(),
        }
    }
}

impl<I: Iterator> MapIter<I> {
    pub fn new(inner: I, mapper: impl Fn(Row) -> Row + Send + Sync + 'static) -> Self {
        Self {
            inner,
            mapper: Arc::new(mapper),
            current_row: None,
        }
    }
}

impl<I: Iterator> Iterator for MapIter<I> {
    fn kind(&self) -> IteratorKind {
        IteratorKind::Default
    }

    fn valid(&self) -> bool {
        self.inner.valid()
    }

    fn next(&mut self) {
        self.inner.next();
        self.current_row = None;
    }

    fn erase(&mut self) {
        self.inner.erase();
        self.current_row = None;
    }

    fn unstable_erase(&mut self) {
        self.inner.unstable_erase();
        self.current_row = None;
    }

    fn clear(&mut self) {
        self.inner.clear();
        self.current_row = None;
    }

    fn reset(&mut self, pos: usize) {
        self.inner.reset(pos);
        self.current_row = None;
    }

    fn size(&self) -> usize {
        self.inner.size()
    }

    fn row(&self) -> Option<&Row> {
        self.current_row.as_ref()
    }

    fn move_row(&mut self) -> Option<Row> {
        if let Some(row) = self.inner.move_row() {
            let mapped = (self.mapper)(row);
            self.current_row = Some(mapped.clone());
            Some(mapped)
        } else {
            None
        }
    }

    fn add_row(&mut self, row: Row) {
        self.inner.add_row(row)
    }

    fn select(&mut self, offset: usize, count: usize) {
        self.inner.select(offset, count);
    }

    fn sample(&mut self, count: i64) {
        self.inner.sample(count);
    }

    fn erase_range(&mut self, first: usize, last: usize) {
        self.inner.erase_range(first, last);
    }

    fn get_column(&self, col: &str) -> Option<&Value> {
        if let Some(ref row) = self.current_row {
            let col_idx = self.inner.get_column_index(col)?;
            row.get(col_idx)
        } else {
            self.inner.get_column(col)
        }
    }

    fn get_column_by_index(&self, index: i32) -> Option<&Value> {
        if let Some(ref row) = self.current_row {
            let size = row.len() as i32;
            let idx = if index >= 0 {
                index as usize
            } else {
                let adjusted = (size + index) % size;
                if adjusted < 0 {
                    return None;
                }
                adjusted as usize
            };
            row.get(idx)
        } else {
            self.inner.get_column_by_index(index)
        }
    }

    fn get_column_index(&self, col: &str) -> Option<usize> {
        self.inner.get_column_index(col)
    }

    fn get_col_names(&self) -> Vec<String> {
        self.inner.get_col_names()
    }

    fn copy(&self) -> Self {
        Self {
            inner: self.inner.copy(),
            mapper: self.mapper.clone(),
            current_row: None,
        }
    }
}

/// 限制数量迭代器 - 只返回前n个元素
///
/// 包装另一个迭代器，限制返回的元素数量
#[derive(Debug, Clone)]
pub struct TakeIter<I: Iterator> {
    inner: I,
    limit: usize,
    taken: usize,
}

impl<I: Iterator> TakeIter<I> {
    pub fn new(inner: I, limit: usize) -> Self {
        Self {
            inner,
            limit,
            taken: 0,
        }
    }
}

impl<I: Iterator> Iterator for TakeIter<I> {
    fn kind(&self) -> IteratorKind {
        IteratorKind::Default
    }

    fn valid(&self) -> bool {
        self.inner.valid() && self.taken < self.limit
    }

    fn next(&mut self) {
        if self.taken < self.limit {
            self.inner.next();
            self.taken += 1;
        }
    }

    fn erase(&mut self) {
        self.inner.erase();
    }

    fn unstable_erase(&mut self) {
        self.inner.unstable_erase();
    }

    fn clear(&mut self) {
        self.inner.clear();
        self.taken = 0;
    }

    fn reset(&mut self, pos: usize) {
        self.inner.reset(pos);
        self.taken = pos;
    }

    fn size(&self) -> usize {
        std::cmp::min(self.inner.size().saturating_sub(self.taken), self.limit)
    }

    fn row(&self) -> Option<&Row> {
        if self.taken < self.limit {
            self.inner.row()
        } else {
            None
        }
    }

    fn move_row(&mut self) -> Option<Row> {
        if self.taken < self.limit {
            let result = self.inner.move_row();
            self.taken += 1;
            result
        } else {
            None
        }
    }

    fn add_row(&mut self, row: Row) {
        self.inner.add_row(row)
    }

    fn select(&mut self, offset: usize, count: usize) {
        let actual_offset = self.taken + offset;
        self.inner.select(actual_offset, count);
        self.taken += offset + count;
    }

    fn sample(&mut self, count: i64) {
        self.inner.sample(std::cmp::min(count, self.limit as i64 - self.taken as i64));
    }

    fn erase_range(&mut self, first: usize, last: usize) {
        let actual_first = self.taken + first;
        let actual_last = self.taken + last;
        self.inner.erase_range(actual_first, actual_last);
        self.taken += last - first;
    }

    fn get_column(&self, col: &str) -> Option<&Value> {
        if self.taken < self.limit {
            self.inner.get_column(col)
        } else {
            None
        }
    }

    fn get_column_by_index(&self, index: i32) -> Option<&Value> {
        if self.taken < self.limit {
            self.inner.get_column_by_index(index)
        } else {
            None
        }
    }

    fn get_column_index(&self, col: &str) -> Option<usize> {
        self.inner.get_column_index(col)
    }

    fn get_col_names(&self) -> Vec<String> {
        self.inner.get_col_names()
    }

    fn copy(&self) -> Self {
        Self {
            inner: self.inner.copy(),
            limit: self.limit,
            taken: self.taken,
        }
    }
}

/// 跳过迭代器 - 跳过前n个元素
///
/// 包装另一个迭代器，跳过指定数量的元素
#[derive(Debug, Clone)]
pub struct SkipIter<I: Iterator> {
    inner: I,
    skipped: usize,
}

impl<I: Iterator> SkipIter<I> {
    pub fn new(inner: I, n: usize) -> Self {
        let mut iter = Self {
            inner,
            skipped: 0,
        };
        // 预先跳过n个元素
        while iter.skipped < n && iter.inner.valid() {
            iter.inner.next();
            iter.skipped += 1;
        }
        iter
    }
}

impl<I: Iterator> Iterator for SkipIter<I> {
    fn kind(&self) -> IteratorKind {
        IteratorKind::Default
    }

    fn valid(&self) -> bool {
        self.inner.valid()
    }

    fn next(&mut self) {
        self.inner.next();
    }

    fn erase(&mut self) {
        self.inner.erase();
    }

    fn unstable_erase(&mut self) {
        self.inner.unstable_erase();
    }

    fn clear(&mut self) {
        self.inner.clear();
    }

    fn reset(&mut self, pos: usize) {
        self.inner.reset(pos + self.skipped);
    }

    fn size(&self) -> usize {
        self.inner.size().saturating_sub(self.skipped)
    }

    fn row(&self) -> Option<&Row> {
        self.inner.row()
    }

    fn move_row(&mut self) -> Option<Row> {
        self.inner.move_row()
    }

    fn add_row(&mut self, row: Row) {
        self.inner.add_row(row)
    }

    fn select(&mut self, offset: usize, count: usize) {
        self.inner.select(self.skipped + offset, count);
    }

    fn sample(&mut self, count: i64) {
        self.inner.sample(count);
    }

    fn erase_range(&mut self, first: usize, last: usize) {
        self.inner.erase_range(self.skipped + first, self.skipped + last);
    }

    fn get_column(&self, col: &str) -> Option<&Value> {
        self.inner.get_column(col)
    }

    fn get_column_by_index(&self, index: i32) -> Option<&Value> {
        self.inner.get_column_by_index(index)
    }

    fn get_column_index(&self, col: &str) -> Option<usize> {
        self.inner.get_column_index(col)
    }

    fn get_col_names(&self) -> Vec<String> {
        self.inner.get_col_names()
    }

    fn copy(&self) -> Self {
        Self {
            inner: self.inner.copy(),
            skipped: self.skipped,
        }
    }
}

/// 通用组合迭代器 - 支持链式组合多个操作
///
/// 提供流畅的API来组合多个迭代器操作
#[derive(Debug, Clone)]
pub struct CompositeIter<I: Iterator> {
    inner: I,
}

impl<I: Iterator> CompositeIter<I> {
    pub fn new(inner: I) -> Self {
        Self { inner }
    }

    pub fn filter<F>(self, predicate: F) -> FilterIter<Self>
    where
        F: Fn(&Row) -> bool + Send + Sync + 'static,
    {
        FilterIter::new(self, predicate)
    }

    pub fn map<M>(self, mapper: M) -> MapIter<Self>
    where
        M: Fn(Row) -> Row + Send + Sync + 'static,
    {
        MapIter::new(self, mapper)
    }

    pub fn take(self, limit: usize) -> TakeIter<Self> {
        TakeIter::new(self, limit)
    }

    pub fn skip(self, n: usize) -> SkipIter<Self> {
        SkipIter::new(self, n)
    }
}

impl<I: Iterator> Iterator for CompositeIter<I> {
    fn kind(&self) -> IteratorKind {
        self.inner.kind()
    }

    fn valid(&self) -> bool {
        self.inner.valid()
    }

    fn next(&mut self) {
        self.inner.next()
    }

    fn erase(&mut self) {
        self.inner.erase()
    }

    fn unstable_erase(&mut self) {
        self.inner.unstable_erase()
    }

    fn clear(&mut self) {
        self.inner.clear()
    }

    fn reset(&mut self, pos: usize) {
        self.inner.reset(pos)
    }

    fn size(&self) -> usize {
        self.inner.size()
    }

    fn row(&self) -> Option<&Row> {
        self.inner.row()
    }

    fn move_row(&mut self) -> Option<Row> {
        self.inner.move_row()
    }

    fn add_row(&mut self, row: Row) {
        self.inner.add_row(row)
    }

    fn select(&mut self, offset: usize, count: usize) {
        self.inner.select(offset, count)
    }

    fn sample(&mut self, count: i64) {
        self.inner.sample(count)
    }

    fn erase_range(&mut self, first: usize, last: usize) {
        self.inner.erase_range(first, last)
    }

    fn get_column(&self, col: &str) -> Option<&Value> {
        self.inner.get_column(col)
    }

    fn get_column_by_index(&self, index: i32) -> Option<&Value> {
        self.inner.get_column_by_index(index)
    }

    fn get_column_index(&self, col: &str) -> Option<usize> {
        self.inner.get_column_index(col)
    }

    fn get_col_names(&self) -> Vec<String> {
        self.inner.get_col_names()
    }

    fn copy(&self) -> Self {
        Self {
            inner: self.inner.copy(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::DataSet;
    use std::sync::Arc;

    fn create_test_data() -> Arc<Value> {
        let mut dataset = DataSet::new();
        dataset.col_names = vec!["name".to_string(), "age".to_string()];
        dataset.rows = vec![
            vec![Value::String("Alice".to_string()), Value::Int(25)],
            vec![Value::String("Bob".to_string()), Value::Int(30)],
            vec![Value::String("Charlie".to_string()), Value::Int(35)],
            vec![Value::String("Diana".to_string()), Value::Int(40)],
        ];
        Arc::new(Value::DataSet(dataset))
    }

    #[test]
    fn test_filter_iter() {
        let data = create_test_data();
        let inner = super::super::SequentialIter::new(data).expect("Failed to create sequential iterator");
        let mut iter = FilterIter::new(inner, |row| {
            if let Some(Value::Int(age)) = row.get(1) {
                *age >= 30
            } else {
                false
            }
        });

        assert_eq!(iter.size(), 3);
        assert!(iter.valid());

        iter.next();
        if let Some(Value::String(name)) = iter.get_column("name") {
            assert_eq!(name, "Bob");
        }
    }

    #[test]
    fn test_map_iter() {
        let data = create_test_data();
        let inner = super::super::SequentialIter::new(data).expect("Failed to create sequential iterator");
        let mut iter = MapIter::new(inner, |mut row| {
            if let Some(Value::Int(age)) = row.get(1) {
                row[1] = Value::Int(*age + 1);
            }
            row
        });

        assert!(iter.valid());
        iter.move_row();
        if let Some(Value::Int(age)) = iter.get_column("age") {
            assert_eq!(*age, 26);
        }
    }

    #[test]
    fn test_take_iter() {
        let data = create_test_data();
        let inner = super::super::SequentialIter::new(data).expect("Failed to create sequential iterator");
        let iter = TakeIter::new(inner, 2);

        assert_eq!(iter.size(), 2);
    }

    #[test]
    fn test_skip_iter() {
        let data = create_test_data();
        let inner = super::super::SequentialIter::new(data).expect("Failed to create sequential iterator");
        let iter = SkipIter::new(inner, 2);

        assert_eq!(iter.size(), 2);
    }

    #[test]
    fn test_composite_iter() {
        let data = create_test_data();
        let inner = super::super::SequentialIter::new(data).expect("Failed to create sequential iterator");
        let filtered = FilterIter::new(CompositeIter::new(inner), |row| {
            if let Some(Value::Int(age)) = row.get(1) {
                *age >= 30
            } else {
                false
            }
        });

        assert_eq!(filtered.size(), 3);

        let taken = TakeIter::new(filtered, 1);
        let skipped = SkipIter::new(taken, 0);

        assert_eq!(skipped.size(), 1);
    }
}
