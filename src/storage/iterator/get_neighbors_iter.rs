//! 邻居查询迭代器 - 用于处理图邻居查询结果
//!
//! GetNeighborsIter用于遍历GetNeighbors查询的结果
//! 结果是一个树状结构：srcVertex -> edges -> dstVertices
//!
//! 这是一个复杂的迭代器，支持四层嵌套遍历：
//! 1. 数据集列表层：dsIndices_（可能多个分片返回多个数据集）
//! 2. 数据行层：currentRow_（每个顶点一行）
//! 3. 边列层：colIdx_（每个边类型一列）
//! 4. 边列表层：edgeIdx_（每个邻接边一条记录）

use super::{Iterator, IteratorKind, Row};
use crate::core::{DataSet, Value};
use std::collections::HashMap;
use std::sync::Arc;

/// 属性索引结构
#[derive(Debug, Clone)]
struct PropIndex {
    col_idx: usize,
    prop_list: Vec<String>,
    prop_indices: HashMap<String, usize>,
}

/// 数据集索引结构
#[derive(Debug, Clone)]
struct DataSetIndex {
    ds: Arc<DataSet>,
    // 列名到索引的映射：{ "_vid": 0, "_stats": 1, "_tag:player:name": 2, ... }
    col_indices: HashMap<String, usize>,
    // 列索引到标签/边名的映射：{ 2: "player", 3: "follow" }
    tag_edge_name_indices: HashMap<usize, String>,
    // 标签属性映射："player" -> {col_idx: 2, prop_indices: {"name":0, "age":1}}
    tag_props_map: HashMap<String, PropIndex>,
    // 边属性映射："follow" -> {col_idx: 3, prop_indices: {"weight":0}}
    edge_props_map: HashMap<String, PropIndex>,
    col_lower_bound: i64,
    col_upper_bound: i64,
}

/// 邻居查询迭代器
/// 
/// 用于遍历邻居查询的复杂结果结构
/// 支持多层次的遍历：顶点 -> 边 -> 邻接顶点
#[derive(Debug, Clone)]
pub struct GetNeighborsIter {
    data: Arc<Value>,
    ds_indices: Vec<DataSetIndex>,
    current_ds_index: usize,
    current_row: usize,
    col_idx: i64,
    edge_idx: i64,
    edge_idx_upper_bound: i64,
    valid: bool,
    no_edge: bool,
}

impl GetNeighborsIter {
    /// 创建新的邻居迭代器
    pub fn new(data: Arc<Value>) -> Result<Self, String> {
        let mut iter = Self {
            data: data.clone(),
            ds_indices: Vec::new(),
            current_ds_index: 0,
            current_row: 0,
            col_idx: -1,
            edge_idx: -1,
            edge_idx_upper_bound: -1,
            valid: false,
            no_edge: false,
        };

        iter.process_list()?;
        iter.go_to_first_edge();
        
        Ok(iter)
    }

    /// 处理列表数据
    fn process_list(&mut self) -> Result<(), String> {
        match &*self.data {
            Value::List(list) => {
                for val in &list.values {
                    if let Value::DataSet(dataset) = val {
                        let ds_index = self.make_dataset_index(dataset)?;
                        self.ds_indices.push(ds_index);
                    } else {
                        return Err("GetNeighborsIter 只支持 DataSet 列表".to_string());
                    }
                }
                Ok(())
            }
            _ => Err("GetNeighborsIter 需要 List 类型的数据".to_string()),
        }
    }

    /// 创建数据集索引
    fn make_dataset_index(&self, ds: &DataSet) -> Result<DataSetIndex, String> {
        let mut ds_index = DataSetIndex {
            ds: Arc::new(ds.clone()),
            col_indices: HashMap::new(),
            tag_edge_name_indices: HashMap::new(),
            tag_props_map: HashMap::new(),
            edge_props_map: HashMap::new(),
            col_lower_bound: -1,
            col_upper_bound: -1,
        };

        self.build_index(&mut ds_index)?;
        Ok(ds_index)
    }

    /// 构建索引
    fn build_index(&self, ds_index: &mut DataSetIndex) -> Result<i64, String> {
        let col_names = &ds_index.ds.col_names;
        if col_names.len() < 3 {
            return Err("列名数量不足".to_string());
        }

        let mut edge_start_index = -1;
        for (i, col_name) in col_names.iter().enumerate() {
            ds_index.col_indices.insert(col_name.clone(), i);

            if col_name.starts_with("_tag") {
                self.build_prop_index(col_name, i, false, ds_index)?;
            } else if col_name.starts_with("_edge") {
                self.build_prop_index(col_name, i, true, ds_index)?;
                if edge_start_index < 0 {
                    edge_start_index = i as i64;
                }
            }
        }

        if edge_start_index == -1 {
            self.no_edge = true;
        }

        ds_index.col_lower_bound = edge_start_index - 1;
        ds_index.col_upper_bound = col_names.len() as i64 - 1;
        
        Ok(edge_start_index)
    }

    /// 构建属性索引
    fn build_prop_index(
        &self,
        props: &str,
        column_id: usize,
        is_edge: bool,
        ds_index: &mut DataSetIndex,
    ) -> Result<(), String> {
        let pieces: Vec<&str> = props.split(':').collect();
        if pieces.len() < 2 {
            return Err(format!("错误的列名格式: {}", props));
        }

        let mut prop_idx = PropIndex {
            col_idx: column_id,
            prop_list: Vec::new(),
            prop_indices: HashMap::new(),
        };

        // 如果有属性列表，构建属性索引
        if pieces.len() > 2 {
            for (i, prop) in pieces.iter().skip(2).enumerate() {
                prop_idx.prop_indices.insert(prop.to_string(), i);
                prop_idx.prop_list.push(prop.to_string());
            }
        }

        let name = pieces[1].to_string();
        if is_edge {
            // 边名以+/-开头
            if name.is_empty() || (!name.starts_with('+') && !name.starts_with('-')) {
                return Err(format!("错误的边名: {}", name));
            }
            ds_index.tag_edge_name_indices.insert(column_id, name.clone());
            ds_index.edge_props_map.insert(name, prop_idx);
        } else {
            ds_index.tag_edge_name_indices.insert(column_id, name.clone());
            ds_index.tag_props_map.insert(name, prop_idx);
        }

        Ok(())
    }

    /// 移动到第一条边
    fn go_to_first_edge(&mut self) {
        self.valid = false;

        for (ds_idx, ds_index) in self.ds_indices.iter().enumerate() {
            if self.no_edge {
                self.current_ds_index = ds_idx;
                self.current_row = 0;
                self.valid = true;
                break;
            }

            for row_idx in 0..ds_index.ds.rows.len() {
                self.col_idx = ds_index.col_lower_bound + 1;
                while self.col_idx < ds_index.col_upper_bound && !self.valid {
                    let col_idx = self.col_idx as usize;
                    if col_idx >= ds_index.ds.rows[row_idx].len() {
                        self.col_idx += 1;
                        continue;
                    }

                    let current_col = &ds_index.ds.rows[row_idx][col_idx];
                    if !matches!(current_col, Value::List(_)) || 
                       matches!(current_col, Value::List(list) if list.values.is_empty()) {
                        self.col_idx += 1;
                        continue;
                    }

                    if let Value::List(current_col_list) = current_col {
                        self.edge_idx_upper_bound = current_col_list.values.len() as i64;
                        self.edge_idx = 0;
                        
                        while self.edge_idx < self.edge_idx_upper_bound && !self.valid {
                            let edge_idx = self.edge_idx as usize;
                            if edge_idx >= current_col_list.values.len() {
                                self.edge_idx += 1;
                                continue;
                            }

                            let current_edge = &current_col_list.values[edge_idx];
                            if !matches!(current_edge, Value::List(_)) {
                                self.edge_idx += 1;
                                continue;
                            }

                            self.valid = true;
                            self.current_ds_index = ds_idx;
                            self.current_row = row_idx;
                            break;
                        }
                    }
                    
                    if !self.valid {
                        self.col_idx += 1;
                    }
                }
                
                if self.valid {
                    break;
                }
            }
            
            if self.valid {
                break;
            }
        }
    }

    /// 获取当前边名
    fn current_edge_name(&self) -> Option<&str> {
        if self.current_ds_index >= self.ds_indices.len() {
            return None;
        }
        
        let ds_index = &self.ds_indices[self.current_ds_index];
        ds_index
            .tag_edge_name_indices
            .get(&(self.col_idx as usize))
            .map(|s| s.as_str())
    }

    /// 检查列是否有效
    fn col_valid(&self) -> bool {
        !self.no_edge && self.valid()
    }
}

impl Iterator for GetNeighborsIter {
    fn kind(&self) -> IteratorKind {
        IteratorKind::GetNeighbors
    }

    fn valid(&self) -> bool {
        self.valid && 
        self.current_ds_index < self.ds_indices.len() &&
        self.current_row < self.ds_indices[self.current_ds_index].ds.rows.len() &&
        self.col_idx < self.ds_indices[self.current_ds_index].col_upper_bound
    }

    fn next(&mut self) {
        if !self.valid() {
            return;
        }

        if self.no_edge {
            self.current_row += 1;
            if self.current_row >= self.ds_indices[self.current_ds_index].ds.rows.len() {
                self.current_ds_index += 1;
                if self.current_ds_index < self.ds_indices.len() {
                    self.current_row = 0;
                } else {
                    self.valid = false;
                }
            }
            return;
        }

        self.edge_idx += 1;
        
        while self.edge_idx >= 0 {
            if self.edge_idx < self.edge_idx_upper_bound {
                // 找到有效边
                self.valid = true;
                break;
            }

            // 移动到下一列
            self.col_idx += 1;
            while self.col_idx < self.ds_indices[self.current_ds_index].col_upper_bound {
                let col_idx = self.col_idx as usize;
                if col_idx >= self.ds_indices[self.current_ds_index].ds.rows[self.current_row].len() {
                    self.col_idx += 1;
                    continue;
                }

                let current_col = &self.ds_indices[self.current_ds_index].ds.rows[self.current_row][col_idx];
                if !matches!(current_col, Value::List(_)) || 
                   matches!(current_col, Value::List(list) if list.values.is_empty()) {
                    self.col_idx += 1;
                    continue;
                }

                if let Value::List(current_col_list) = current_col {
                    self.edge_idx_upper_bound = current_col_list.values.len() as i64;
                    self.edge_idx = 0;
                    break;
                }
                
                self.col_idx += 1;
            }

            if self.col_idx >= self.ds_indices[self.current_ds_index].col_upper_bound {
                // 移动到下一行
                self.current_row += 1;
                if self.current_row >= self.ds_indices[self.current_ds_index].ds.rows.len() {
                    // 移动到下一个数据集
                    self.current_ds_index += 1;
                    if self.current_ds_index < self.ds_indices.len() {
                        self.current_row = 0;
                        self.col_idx = self.ds_indices[self.current_ds_index].col_lower_bound;
                    } else {
                        self.valid = false;
                        break;
                    }
                } else {
                    self.col_idx = self.ds_indices[self.current_ds_index].col_lower_bound;
                }
            }
        }
    }

    fn erase(&mut self) {
        // 简单实现：移动到下一行
        self.next();
    }

    fn unstable_erase(&mut self) {
        self.erase();
    }

    fn clear(&mut self) {
        self.valid = false;
        self.ds_indices.clear();
        self.reset(0);
    }

    fn reset(&mut self, pos: usize) {
        self.current_ds_index = 0;
        self.current_row = 0;
        self.col_idx = -1;
        self.edge_idx = -1;
        self.edge_idx_upper_bound = -1;
        self.go_to_first_edge();
    }

    fn size(&self) -> usize {
        let mut count = 0;
        for ds_idx in &self.ds_indices {
            for row in &ds_idx.ds.rows {
                for edge_idx in ds_idx.edge_props_map.values() {
                    if edge_idx.col_idx < row.len() {
                        if let Value::List(list) = &row[edge_idx.col_idx] {
                            count += list.values.len();
                        }
                    }
                }
            }
        }
        count
    }

    fn row(&self) -> Option<&Row> {
        if !self.valid() {
            return None;
        }
        
        if self.current_row < self.ds_indices[self.current_ds_index].ds.rows.len() {
            Some(&self.ds_indices[self.current_ds_index].ds.rows[self.current_row])
        } else {
            None
        }
    }

    fn move_row(&mut self) -> Option<Row> {
        self.row().cloned()
    }

    fn select(&mut self, offset: usize, count: usize) {
        // 简化实现：重置到指定位置
        self.reset(offset);
        for _ in 0..count {
            if !self.valid() {
                break;
            }
            self.next();
        }
    }

    fn sample(&mut self, count: i64) {
        if count <= 0 {
            self.clear();
        } else {
            // 简化实现：保留前count个结果
            let mut sampled_count = 0;
            self.reset(0);
            while self.valid() && sampled_count < count as usize {
                sampled_count += 1;
                self.next();
            }
            if sampled_count < count as usize {
                self.clear();
            }
        }
    }

    fn erase_range(&mut self, first: usize, last: usize) {
        // 简化实现：重置到first，然后删除到last
        self.reset(first);
        for i in first..last {
            if !self.valid() {
                break;
            }
            if i >= first && i < last {
                self.erase();
            } else {
                self.next();
            }
        }
        self.reset(0);
    }

    fn get_column(&self, col: &str) -> Option<&Value> {
        if !self.valid() {
            return None;
        }
        
        let ds_index = &self.ds_indices[self.current_ds_index];
        let col_idx = ds_index.col_indices.get(col)?;
        
        if *col_idx < self.ds_indices[self.current_ds_index].ds.rows[self.current_row].len() {
            Some(&self.ds_indices[self.current_ds_index].ds.rows[self.current_row][*col_idx])
        } else {
            None
        }
    }

    fn get_column_by_index(&self, index: i32) -> Option<&Value> {
        if !self.valid() {
            return None;
        }
        
        let row = &self.ds_indices[self.current_ds_index].ds.rows[self.current_row];
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
        if self.current_ds_index >= self.ds_indices.len() {
            return None;
        }
        
        self.ds_indices[self.current_ds_index]
            .col_indices
            .get(col)
            .copied()
    }

    fn get_col_names(&self) -> Vec<String> {
        if self.current_ds_index < self.ds_indices.len() {
            self.ds_indices[self.current_ds_index].ds.col_names.clone()
        } else {
            Vec::new()
        }
    }

    fn copy(&self) -> Box<dyn Iterator> {
        Box::new(self.clone())
    }

    // 图特定方法
    fn get_tag_prop(&self, tag: &str, prop: &str) -> Option<Value> {
        if !self.valid() {
            return None;
        }

        let ds_index = &self.ds_indices[self.current_ds_index];
        
        if tag == "*" {
            // 搜索所有标签
            for prop_index in ds_index.tag_props_map.values() {
                if let Some(prop_idx) = prop_index.prop_indices.get(prop) {
                    let row = &ds_index.ds.rows[self.current_row];
                    if prop_index.col_idx < row.len() {
                        if let Value::List(list) = &row[prop_index.col_idx] {
                            if *prop_idx < list.values.len() {
                                return Some(list.values[*prop_idx].clone());
                            }
                        }
                    }
                }
            }
            None
        } else {
            // 搜索特定标签
            let prop_index = ds_index.tag_props_map.get(tag)?;
            let prop_idx = prop_index.prop_indices.get(prop)?;
            
            let row = &ds_index.ds.rows[self.current_row];
            if prop_index.col_idx < row.len() {
                if let Value::List(list) = &row[prop_index.col_idx] {
                    if *prop_idx < list.values.len() {
                        Some(list.values[*prop_idx].clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    fn get_edge_prop(&self, edge: &str, prop: &str) -> Option<Value> {
        if !self.valid() || self.no_edge {
            return None;
        }

        let current_edge_name = self.current_edge_name()?;
        
        if edge != "*" && !current_edge_name.contains(edge) {
            return None;
        }

        let ds_index = &self.ds_indices[self.current_ds_index];
        let prop_index = ds_index.edge_props_map.get(current_edge_name)?;
        let prop_idx = prop_index.prop_indices.get(prop)?;

        // 获取当前边数据
        let row = &ds_index.ds.rows[self.current_row];
        if prop_index.col_idx >= row.len() {
            return None;
        }

        if let Value::List(edge_col) = &row[prop_index.col_idx] {
            if self.edge_idx >= 0 && (self.edge_idx as usize) < edge_col.values.len() {
                if let Value::List(edge_data) = &edge_col.values[self.edge_idx as usize] {
                    if *prop_idx < edge_data.values.len() {
                        return Some(edge_data.values[*prop_idx].clone());
                    }
                }
            }
        }

        None
    }

    fn get_vertex(&self, _name: &str) -> Option<Value> {
        // 简化实现：返回当前顶点的VID
        self.get_column("_vid").cloned()
    }

    fn get_edge(&self) -> Option<Value> {
        if !self.valid() || self.no_edge {
            return None;
        }

        // 简化实现：返回当前边的基本信息
        let src_vid = self.get_column("_vid")?;
        let dst_vid = self.get_edge_prop("*", "_dst")?;
        
        // 创建简单的边表示
        Some(Value::String(format!("{}->{}", src_vid, dst_vid)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::DataSet;

    fn create_test_neighbors_data() -> Value {
        // 创建测试数据：一个顶点有两条边
        let mut dataset = DataSet::new();
        dataset.col_names = vec![
            "_vid".to_string(),
            "_stats".to_string(), 
            "_tag:player:name:age".to_string(),
            "_edge:follow:weight".to_string(),
        ];
        
        dataset.rows = vec![vec![
            Value::String("player1".to_string()), // _vid
            Value::String("stats".to_string()),   // _stats
            Value::List(vec![                       // _tag:player:name:age
                Value::String("Alice".to_string()),
                Value::Int(25),
            ]),
            Value::List(vec![                       // _edge:follow:weight
                Value::List(vec![Value::Float(0.8)]), // 第一条边
                Value::List(vec![Value::Float(0.6)]), // 第二条边
            ]),
        ]];
        
        Value::List(vec![Value::DataSet(dataset)])
    }

    #[test]
    fn test_get_neighbors_iter_creation() {
        let data = Arc::new(create_test_neighbors_data());
        let iter = GetNeighborsIter::new(data);
        assert!(iter.is_ok());
    }

    #[test]
    fn test_get_neighbors_iter_valid() {
        let data = Arc::new(create_test_neighbors_data());
        let iter = GetNeighborsIter::new(data).unwrap();
        
        assert_eq!(iter.kind(), IteratorKind::GetNeighbors);
        assert!(iter.valid());
    }

    #[test]
    fn test_get_neighbors_iter_navigation() {
        let data = Arc::new(create_test_neighbors_data());
        let mut iter = GetNeighborsIter::new(data).unwrap();
        
        assert!(iter.valid());
        
        // 移动到下一条边
        iter.next();
        assert!(iter.valid());
        
        // 移动到第三条边（不存在）
        iter.next();
        assert!(!iter.valid());
    }

    #[test]
    fn test_get_neighbors_iter_get_tag_prop() {
        let data = Arc::new(create_test_neighbors_data());
        let iter = GetNeighborsIter::new(data).unwrap();
        
        // 获取标签属性
        let name = iter.get_tag_prop("player", "name");
        assert!(name.is_some());
        assert_eq!(name.unwrap(), Value::String("Alice".to_string()));
        
        let age = iter.get_tag_prop("player", "age");
        assert!(age.is_some());
        assert_eq!(age.unwrap(), Value::Int(25));
    }

    #[test]
    fn test_get_neighbors_iter_get_edge_prop() {
        let data = Arc::new(create_test_neighbors_data());
        let iter = GetNeighborsIter::new(data).unwrap();
        
        // 获取边属性
        let weight = iter.get_edge_prop("follow", "weight");
        assert!(weight.is_some());
        assert_eq!(weight.unwrap(), Value::Float(0.8));
    }

    #[test]
    fn test_get_neighbors_iter_size() {
        let data = Arc::new(create_test_neighbors_data());
        let iter = GetNeighborsIter::new(data).unwrap();
        
        // 应该有2条边
        assert_eq!(iter.size(), 2);
    }

    #[test]
    fn test_get_neighbors_iter_reset() {
        let data = Arc::new(create_test_neighbors_data());
        let mut iter = GetNeighborsIter::new(data).unwrap();
        
        iter.next();
        assert!(iter.valid());
        
        iter.reset(0);
        assert!(iter.valid());
    }
}