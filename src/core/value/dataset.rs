//! 数据集类型模块
//!
//! 本模块定义了数据集和列表类型及其相关操作。

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// 简单列表表示
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Encode, Decode)]
pub struct List {
    pub values: Vec<super::types::Value>,
}

impl List {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, super::types::Value> {
        self.values.iter()
    }

    pub fn push(&mut self, value: super::types::Value) {
        self.values.push(value);
    }

    pub fn remove(&mut self, index: usize) -> super::types::Value {
        self.values.remove(index)
    }

    pub fn pop(&mut self) -> Option<super::types::Value> {
        self.values.pop()
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        self.values.swap(a, b);
    }

    pub fn get(&self, index: usize) -> Option<&super::types::Value> {
        self.values.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut super::types::Value> {
        self.values.get_mut(index)
    }

    pub fn insert(&mut self, index: usize, value: super::types::Value) {
        self.values.insert(index, value);
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub fn capacity(&self) -> usize {
        self.values.capacity()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.values.reserve(additional);
    }

    pub fn shrink_to_fit(&mut self) {
        self.values.shrink_to_fit();
    }

    pub fn truncate(&mut self, len: usize) {
        self.values.truncate(len);
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&super::types::Value) -> bool,
    {
        self.values.retain(f);
    }

    pub fn dedup(&mut self)
    where
        super::types::Value: PartialEq,
    {
        self.values.dedup();
    }

    pub fn sort(&mut self)
    where
        super::types::Value: Ord,
    {
        self.values.sort();
    }

    pub fn sort_by<F>(&mut self, f: F)
    where
        F: FnMut(&super::types::Value, &super::types::Value) -> std::cmp::Ordering,
    {
        self.values.sort_by(f);
    }

    pub fn reverse(&mut self) {
        self.values.reverse();
    }

    pub fn contains(&self, value: &super::types::Value) -> bool
    where
        super::types::Value: PartialEq,
    {
        self.values.contains(value)
    }

    pub fn append(&mut self, other: &mut Self) {
        self.values.append(&mut other.values);
    }

    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = super::types::Value>,
    {
        self.values.extend(iter);
    }

    pub fn split_off(&mut self, at: usize) -> Self {
        Self {
            values: self.values.split_off(at),
        }
    }

    pub fn as_slice(&self) -> &[super::types::Value] {
        self.values.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [super::types::Value] {
        self.values.as_mut_slice()
    }

    pub fn into_vec(self) -> Vec<super::types::Value> {
        self.values
    }

    pub fn from_vec(values: Vec<super::types::Value>) -> Self {
        Self { values }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
        }
    }
}

impl std::ops::Index<usize> for List {
    type Output = super::types::Value;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}

impl std::ops::IndexMut<usize> for List {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.values[index]
    }
}

impl std::ops::Index<std::ops::Range<usize>> for List {
    type Output = [super::types::Value];

    fn index(&self, range: std::ops::Range<usize>) -> &Self::Output {
        &self.values[range]
    }
}

impl std::ops::Index<std::ops::RangeFrom<usize>> for List {
    type Output = [super::types::Value];

    fn index(&self, range: std::ops::RangeFrom<usize>) -> &Self::Output {
        &self.values[range]
    }
}

impl std::ops::Index<std::ops::RangeTo<usize>> for List {
    type Output = [super::types::Value];

    fn index(&self, range: std::ops::RangeTo<usize>) -> &Self::Output {
        &self.values[range]
    }
}

impl std::ops::Index<std::ops::RangeFull> for List {
    type Output = [super::types::Value];

    fn index(&self, range: std::ops::RangeFull) -> &Self::Output {
        &self.values[range]
    }
}

impl IntoIterator for List {
    type Item = super::types::Value;
    type IntoIter = std::vec::IntoIter<super::types::Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl<'a> IntoIterator for &'a List {
    type Item = &'a super::types::Value;
    type IntoIter = std::slice::Iter<'a, super::types::Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.iter()
    }
}

impl From<Vec<super::types::Value>> for List {
    fn from(values: Vec<super::types::Value>) -> Self {
        Self { values }
    }
}

impl From<List> for Vec<super::types::Value> {
    fn from(list: List) -> Self {
        list.values
    }
}

impl Default for List {
    fn default() -> Self {
        Self::new()
    }
}

/// 简单数据集表示
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Encode, Decode)]
pub struct DataSet {
    pub col_names: Vec<String>,
    pub rows: Vec<Vec<super::types::Value>>,
}

impl DataSet {
    pub fn new() -> Self {
        Self {
            col_names: Vec::new(),
            rows: Vec::new(),
        }
    }

    /// 创建带列名的数据集
    pub fn with_columns(col_names: Vec<String>) -> Self {
        Self {
            col_names,
            rows: Vec::new(),
        }
    }

    /// 添加行
    pub fn add_row(&mut self, row: Vec<super::types::Value>) {
        self.rows.push(row);
    }

    /// 获取行数
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// 获取列数
    pub fn col_count(&self) -> usize {
        self.col_names.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// 获取指定列的索引
    pub fn get_col_index(&self, col_name: &str) -> Option<usize> {
        self.col_names.iter().position(|name| name == col_name)
    }

    /// 获取指定列的所有值
    pub fn get_column(&self, col_name: &str) -> Option<Vec<super::types::Value>> {
        if let Some(index) = self.get_col_index(col_name) {
            Some(self.rows.iter().filter_map(|row| row.get(index).cloned()).collect())
        } else {
            None
        }
    }

    /// 过滤数据集
    pub fn filter<F>(&self, predicate: F) -> DataSet
    where
        F: Fn(&Vec<super::types::Value>) -> bool,
    {
        DataSet {
            col_names: self.col_names.clone(),
            rows: self.rows.iter().filter(|row| predicate(row)).cloned().collect(),
        }
    }

    /// 映射数据集
    pub fn map<F>(&self, mapper: F) -> DataSet
    where
        F: Fn(&Vec<super::types::Value>) -> Vec<super::types::Value>,
    {
        DataSet {
            col_names: self.col_names.clone(),
            rows: self.rows.iter().map(|row| mapper(row)).collect(),
        }
    }

    /// 排序数据集
    pub fn sort_by<F>(&mut self, comparator: F)
    where
        F: Fn(&Vec<super::types::Value>, &Vec<super::types::Value>) -> std::cmp::Ordering,
    {
        self.rows.sort_by(comparator);
    }

    /// 连接两个数据集
    pub fn join(&self, other: &DataSet, on: &str) -> Result<DataSet, String> {
        let left_index = self.get_col_index(on)
            .ok_or_else(|| format!("左数据集找不到列: {}", on))?;
        let right_index = other.get_col_index(on)
            .ok_or_else(|| format!("右数据集找不到列: {}", on))?;

        let mut result = DataSet::new();
        result.col_names = self.col_names.iter()
            .chain(other.col_names.iter())
            .filter(|name| *name != on)
            .cloned()
            .collect();

        for left_row in &self.rows {
            if let Some(left_key) = left_row.get(left_index) {
                for right_row in &other.rows {
                    if let Some(right_key) = right_row.get(right_index) {
                        if left_key == right_key {
                            let mut merged_row = left_row.clone();
                            for (i, val) in right_row.iter().enumerate() {
                                if i != right_index {
                                    merged_row.push(val.clone());
                                }
                            }
                            result.add_row(merged_row);
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    /// 分组数据集
    pub fn group_by<F, K>(&self, key_fn: F) -> Vec<(K, DataSet)>
    where
        F: Fn(&Vec<super::types::Value>) -> K,
        K: std::hash::Hash + Eq + Clone,
    {
        use std::collections::HashMap;
        let mut groups: HashMap<K, Vec<Vec<super::types::Value>>> = HashMap::new();

        for row in &self.rows {
            let key = key_fn(row);
            groups.entry(key).or_insert_with(Vec::new).push(row.clone());
        }

        groups.into_iter()
            .map(|(key, rows)| {
                let dataset = DataSet {
                    col_names: self.col_names.clone(),
                    rows,
                };
                (key, dataset)
            })
            .collect()
    }

    /// 聚合数据集
    pub fn aggregate<F, R>(&self, aggregator: F) -> Vec<R>
    where
        F: Fn(&Vec<super::types::Value>) -> R,
    {
        self.rows.iter().map(aggregator).collect()
    }

    /// 限制行数
    pub fn limit(&self, n: usize) -> DataSet {
        DataSet {
            col_names: self.col_names.clone(),
            rows: self.rows.iter().take(n).cloned().collect(),
        }
    }

    /// 跳过前 n 行
    pub fn skip(&self, n: usize) -> DataSet {
        DataSet {
            col_names: self.col_names.clone(),
            rows: self.rows.iter().skip(n).cloned().collect(),
        }
    }

    /// 合并数据集
    pub fn union(&self, other: &DataSet) -> Result<DataSet, String> {
        if self.col_names != other.col_names {
            return Err("列名不匹配，无法合并".to_string());
        }

        Ok(DataSet {
            col_names: self.col_names.clone(),
            rows: self.rows.iter().chain(other.rows.iter()).cloned().collect(),
        })
    }

    /// 计算交集
    pub fn intersect(&self, other: &DataSet) -> DataSet {
        use std::collections::HashSet;
        let other_set: HashSet<&Vec<super::types::Value>> = other.rows.iter().collect();
        DataSet {
            col_names: self.col_names.clone(),
            rows: self.rows.iter()
                .filter(|row| other_set.contains(row))
                .cloned()
                .collect(),
        }
    }

    /// 计算差集
    pub fn except(&self, other: &DataSet) -> DataSet {
        use std::collections::HashSet;
        let other_set: HashSet<&Vec<super::types::Value>> = other.rows.iter().collect();
        DataSet {
            col_names: self.col_names.clone(),
            rows: self.rows.iter()
                .filter(|row| !other_set.contains(row))
                .cloned()
                .collect(),
        }
    }

    /// 转置数据集
    pub fn transpose(&self) -> DataSet {
        if self.rows.is_empty() {
            return DataSet::new();
        }

        let col_count = self.col_names.len();
        let mut transposed = DataSet::new();
        transposed.col_names = (0..self.row_count())
            .map(|i| format!("row_{}", i))
            .collect();

        for col in 0..col_count {
            let mut new_row = Vec::new();
            for row in &self.rows {
                if let Some(val) = row.get(col) {
                    new_row.push(val.clone());
                }
            }
            transposed.add_row(new_row);
        }

        transposed
    }

    /// 获取唯一值
    pub fn distinct(&self, col_name: &str) -> Vec<super::types::Value> {
        use std::collections::HashSet;
        if let Some(index) = self.get_col_index(col_name) {
            let mut unique = HashSet::new();
            for row in &self.rows {
                if let Some(val) = row.get(index) {
                    unique.insert(val.clone());
                }
            }
            unique.into_iter().collect()
        } else {
            Vec::new()
        }
    }

    /// 估算数据集的内存使用大小
    pub fn estimated_size(&self) -> usize {
        let mut size = std::mem::size_of::<Self>();
        
        // 计算 col_names 的容量开销
        size += self.col_names.capacity() * std::mem::size_of::<String>();
        for col_name in &self.col_names {
            size += col_name.capacity();
        }
        
        // 计算 rows 的容量开销
        size += self.rows.capacity() * std::mem::size_of::<Vec<super::types::Value>>();
        for row in &self.rows {
            size += row.capacity() * std::mem::size_of::<super::types::Value>();
            for value in row {
                size += value.estimated_size();
            }
        }
        
        size
    }
}
