//! 属性查询迭代器 - 用于处理顶点/边属性查询
//!
//! PropIter用于遍历属性查询的结果
//! 类似SequentialIter，但针对属性数据优化
//! 支持顶点和边的属性访问

use super::{Iterator, IteratorKind, Row};
use crate::core::{DataSet, Value};
use std::collections::HashMap;
use std::sync::Arc;

/// 属性查询迭代器
///
/// 用于遍历属性查询结果
/// 支持顶点和边的属性访问
#[derive(Debug, Clone)]
pub struct PropIter {
    data: Arc<Value>,
    rows: Vec<Row>,
    col_names: Vec<String>,
    curr_pos: usize,
    ds_index: DataSetIndex,
}

/// 数据集索引结构
#[derive(Debug, Clone)]
struct DataSetIndex {
    ds: Arc<DataSet>,
    // 列名到索引的映射
    col_indices: HashMap<String, usize>,
    // 属性映射：{tag_name: {prop_name: col_index}}
    props_map: HashMap<String, HashMap<String, usize>>,
}

impl PropIter {
    /// 创建新的属性迭代器
    pub fn new(data: Arc<Value>) -> Result<Self, String> {
        match &*data {
            Value::DataSet(dataset) => {
                let col_names = dataset.col_names.clone();
                let rows = dataset.rows.clone();

                let mut iter = Self {
                    data: data.clone(),
                    rows,
                    col_names,
                    curr_pos: 0,
                    ds_index: DataSetIndex {
                        ds: Arc::new(dataset.clone()),
                        col_indices: HashMap::new(),
                        props_map: HashMap::new(),
                    },
                };

                iter.make_dataset_index()?;
                Ok(iter)
            }
            _ => Err("PropIter 只支持 DataSet".to_string()),
        }
    }

    /// 创建数据集索引
    fn make_dataset_index(&mut self) -> Result<(), String> {
        let ds = self.ds_index.ds.clone();

        // 构建列索引
        for (i, col_name) in ds.col_names.iter().enumerate() {
            self.ds_index.col_indices.insert(col_name.clone(), i);

            // 构建属性索引（如果列名包含点号）
            if col_name.contains('.') {
                self.build_prop_index(col_name, i)?;
            }
        }

        Ok(())
    }

    /// 构建属性索引
    fn build_prop_index(&mut self, props: &str, column_id: usize) -> Result<(), String> {
        let pieces: Vec<&str> = props.split('.').collect();
        if pieces.len() != 2 {
            return Err(format!("错误的属性列名格式: {}", props));
        }

        let name = pieces[0].to_string();
        let prop = pieces[1].to_string();

        let prop_map = self
            .ds_index
            .props_map
            .entry(name)
            .or_insert(HashMap::new());
        prop_map.insert(prop, column_id);

        Ok(())
    }

    /// 获取当前行的引用
    fn curr_row(&self) -> Option<&Row> {
        if self.curr_pos < self.rows.len() {
            Some(&self.rows[self.curr_pos])
        } else {
            None
        }
    }

    /// 获取属性值
    fn get_prop(&self, name: &str, prop: &str) -> Option<&Value> {
        if !self.valid() {
            return None;
        }

        let row = self.curr_row()?;

        if name == "*" {
            // 搜索所有属性
            for prop_map in self.ds_index.props_map.values() {
                if let Some(&col_id) = prop_map.get(prop) {
                    if col_id < row.len() {
                        let val = &row[col_id];
                        if !matches!(val, Value::Empty) {
                            return Some(val);
                        }
                    }
                }
            }
            None
        } else {
            // 搜索特定属性
            let prop_map = self.ds_index.props_map.get(name)?;
            let col_id = prop_map.get(prop)?;

            if *col_id < row.len() {
                Some(&row[*col_id])
            } else {
                None
            }
        }
    }

    /// 获取列索引映射
    pub fn get_col_indices(&self) -> &HashMap<String, usize> {
        &self.ds_index.col_indices
    }

    /// 获取当前位置
    pub fn curr_pos(&self) -> usize {
        self.curr_pos
    }
}

impl Iterator for PropIter {
    fn kind(&self) -> IteratorKind {
        IteratorKind::Prop
    }

    fn valid(&self) -> bool {
        self.curr_pos < self.rows.len()
    }

    fn next(&mut self) {
        if self.curr_pos < self.rows.len() {
            self.curr_pos += 1;
        }
    }

    fn erase(&mut self) {
        if self.curr_pos < self.rows.len() {
            self.rows.remove(self.curr_pos);
            // curr_pos 不变，指向下一行
        }
    }

    fn unstable_erase(&mut self) {
        if self.curr_pos < self.rows.len() {
            // 快速删除：交换最后一行到当前位置，然后 pop
            let len = self.rows.len();
            self.rows.swap(self.curr_pos, len - 1);
            self.rows.pop();
            // curr_pos 不变，指向原来的最后一行
        }
    }

    fn clear(&mut self) {
        self.rows.clear();
        self.curr_pos = 0;
    }

    fn reset(&mut self, pos: usize) {
        self.curr_pos = if pos <= self.rows.len() {
            pos
        } else {
            self.rows.len()
        };
    }

    fn size(&self) -> usize {
        self.rows.len()
    }

    fn row(&self) -> Option<&Row> {
        self.curr_row()
    }

    fn move_row(&mut self) -> Option<Row> {
        if self.curr_pos < self.rows.len() {
            Some(self.rows[self.curr_pos].clone())
        } else {
            None
        }
    }

    fn add_row(&mut self, row: Row) {
        self.rows.push(row);
    }

    fn select(&mut self, offset: usize, count: usize) {
        if offset >= self.rows.len() {
            self.rows.clear();
        } else {
            let end = std::cmp::min(offset + count, self.rows.len());
            let selected: Vec<_> = self.rows.drain(offset..end).collect();
            self.rows = selected;
            self.curr_pos = 0;
        }
    }

    fn sample(&mut self, count: i64) {
        if count <= 0 {
            self.clear();
        } else {
            let count = count as usize;
            if self.rows.len() > count {
                // 简单采样：保留前 count 行
                self.rows.truncate(count);
            }
            self.curr_pos = 0;
        }
    }

    fn erase_range(&mut self, first: usize, last: usize) {
        if first < self.rows.len() && first < last {
            let end = std::cmp::min(last, self.rows.len());
            self.rows.drain(first..end);
            // 调整当前位置
            if self.curr_pos >= first {
                self.curr_pos = first;
            }
        }
    }

    fn get_column(&self, col: &str) -> Option<&Value> {
        let col_idx = self.ds_index.col_indices.get(col)?;
        self.curr_row()?.get(*col_idx)
    }

    fn get_column_by_index(&self, index: i32) -> Option<&Value> {
        let row = self.curr_row()?;
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

        if idx < row.len() {
            Some(&row[idx])
        } else {
            None
        }
    }

    fn get_column_index(&self, col: &str) -> Option<usize> {
        self.ds_index.col_indices.get(col).copied()
    }

    fn get_col_names(&self) -> Vec<String> {
        self.col_names.clone()
    }

    fn copy(&self) -> Self {
        Self {
            data: self.data.clone(),
            rows: self.rows.clone(),
            col_names: self.col_names.clone(),
            curr_pos: self.curr_pos,
            ds_index: self.ds_index.clone(),
        }
    }

    // 图特定方法
    fn get_tag_prop(&self, tag: &str, prop: &str) -> Option<Value> {
        self.get_prop(tag, prop).cloned()
    }

    fn get_edge_prop(&self, edge: &str, prop: &str) -> Option<Value> {
        self.get_prop(edge, prop).cloned()
    }

    fn get_vertex(&self, _name: &str) -> Option<Value> {
        // 简化实现：返回当前行的第一个列值作为顶点
        self.curr_row()?.first().cloned()
    }

    fn get_edge(&self) -> Option<Value> {
        // 简化实现：返回当前行的字符串表示
        Some(Value::String(format!("Edge at position {}", self.curr_pos)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::DataSet;

    fn create_test_prop_data() -> Value {
        // 创建测试属性数据
        let mut dataset = DataSet::new();
        dataset.col_names = vec![
            "_vid".to_string(),
            "player.name".to_string(),
            "player.age".to_string(),
            "follow.weight".to_string(),
        ];

        dataset.rows = vec![
            vec![
                Value::String("player1".to_string()),
                Value::String("Alice".to_string()),
                Value::Int(25),
                Value::Float(0.8),
            ],
            vec![
                Value::String("player2".to_string()),
                Value::String("Bob".to_string()),
                Value::Int(30),
                Value::Float(0.6),
            ],
        ];

        Value::DataSet(dataset)
    }

    #[test]
    fn test_prop_iter_creation() {
        let data = Arc::new(create_test_prop_data());
        let iter = PropIter::new(data);
        assert!(iter.is_ok());
    }

    #[test]
    fn test_prop_iter_valid() {
        let data = Arc::new(create_test_prop_data());
        let iter = PropIter::new(data).expect("PropIter should be created successfully in test");

        assert_eq!(iter.kind(), IteratorKind::Prop);
        assert!(iter.valid());
        assert_eq!(iter.size(), 2);
    }

    #[test]
    fn test_prop_iter_navigation() {
        let data = Arc::new(create_test_prop_data());
        let mut iter =
            PropIter::new(data).expect("PropIter should be created successfully in test");

        assert_eq!(iter.curr_pos(), 0);
        assert!(iter.valid());

        // 移动到第二行
        iter.next();
        assert_eq!(iter.curr_pos(), 1);
        assert!(iter.valid());

        // 移动到第三行（越界）
        iter.next();
        assert_eq!(iter.curr_pos(), 2);
        assert!(!iter.valid());
    }

    #[test]
    fn test_prop_iter_get_tag_prop() {
        let data = Arc::new(create_test_prop_data());
        let iter = PropIter::new(data).expect("PropIter should be created successfully in test");

        // 获取标签属性
        let name = iter.get_tag_prop("player", "name");
        assert!(name.is_some());
        assert_eq!(
            name.expect("Tag property should exist in test"),
            Value::String("Alice".to_string())
        );

        let age = iter.get_tag_prop("player", "age");
        assert!(age.is_some());
        assert_eq!(
            age.expect("Tag property should exist in test"),
            Value::Int(25)
        );
    }

    #[test]
    fn test_prop_iter_get_edge_prop() {
        let data = Arc::new(create_test_prop_data());
        let iter = PropIter::new(data).expect("PropIter should be created successfully in test");

        // 获取边属性
        let weight = iter.get_edge_prop("follow", "weight");
        assert!(weight.is_some());
        assert_eq!(
            weight.expect("Edge property should exist in test"),
            Value::Float(0.8)
        );
    }

    #[test]
    fn test_prop_iter_get_column() {
        let data = Arc::new(create_test_prop_data());
        let iter = PropIter::new(data).expect("PropIter should be created successfully in test");

        // 按列名获取值
        let vid = iter.get_column("_vid");
        assert!(vid.is_some());
        assert_eq!(
            vid.expect("Column value should exist in test"),
            &Value::String("player1".to_string())
        );

        let name = iter.get_column("player.name");
        assert!(name.is_some());
        assert_eq!(
            name.expect("Column value should exist in test"),
            &Value::String("Alice".to_string())
        );
    }

    #[test]
    fn test_prop_iter_get_column_index() {
        let data = Arc::new(create_test_prop_data());
        let iter = PropIter::new(data).expect("PropIter should be created successfully in test");

        // 获取列索引
        assert_eq!(iter.get_column_index("_vid"), Some(0));
        assert_eq!(iter.get_column_index("player.name"), Some(1));
        assert_eq!(iter.get_column_index("player.age"), Some(2));
        assert_eq!(iter.get_column_index("follow.weight"), Some(3));
        assert_eq!(iter.get_column_index("unknown"), None);
    }

    #[test]
    fn test_prop_iter_reset() {
        let data = Arc::new(create_test_prop_data());
        let mut iter =
            PropIter::new(data).expect("PropIter should be created successfully in test");

        iter.next();
        assert_eq!(iter.curr_pos(), 1);

        iter.reset(0);
        assert_eq!(iter.curr_pos(), 0);
        assert_eq!(
            iter.get_column("player.name"),
            Some(&Value::String("Alice".to_string()))
        );
    }

    #[test]
    fn test_prop_iter_copy() {
        let data = Arc::new(create_test_prop_data());
        let mut iter =
            PropIter::new(data).expect("PropIter should be created successfully in test");

        iter.next();
        let copy = iter.copy();

        // 拷贝应该有相同的状态
        assert_eq!(copy.is_empty(), iter.is_empty());
        assert_eq!(copy.size(), iter.size());

        // 修改原迭代器不应该影响拷贝
        iter.next();
        assert_eq!(copy.size(), iter.size());
        assert_eq!(copy.is_empty(), iter.is_empty());
    }

    #[test]
    fn test_prop_iter_invalid_dataset() {
        let data = Arc::new(Value::Int(42));
        let result = PropIter::new(data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "PropIter 只支持 DataSet");
    }
}
