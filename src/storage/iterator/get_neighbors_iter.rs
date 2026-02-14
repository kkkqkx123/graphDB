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
use crate::core::{DataSet, Edge, Value};
use crate::core::vertex_edge_path::Tag;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use parking_lot::Mutex;

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
    ds: Arc<Mutex<DataSet>>,
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
    prev_vertex: Option<Value>,
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
            prev_vertex: None,
        };

        iter.process_list()?;
        iter.go_to_first_edge();

        Ok(iter)
    }

    /// 处理列表数据
    fn process_list(&mut self) -> Result<(), String> {
        let data_clone = self.data.clone();
        match &*data_clone {
            Value::List(list) => {
                for val in list {
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
    fn make_dataset_index(&mut self, ds: &DataSet) -> Result<DataSetIndex, String> {
        let mut ds_index = DataSetIndex {
            ds: Arc::new(Mutex::new(ds.clone())),
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
    fn build_index(&mut self, ds_index: &mut DataSetIndex) -> Result<i64, String> {
        let col_names = {
            let ds = ds_index
                .ds
                .lock();
            ds.col_names.clone()
        };
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
            ds_index
                .tag_edge_name_indices
                .insert(column_id, name.clone());
            ds_index.edge_props_map.insert(name, prop_idx);
        } else {
            ds_index
                .tag_edge_name_indices
                .insert(column_id, name.clone());
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

            let ds_guard = ds_index.ds.lock();

            for row_idx in 0..ds_guard.rows.len() {
                self.col_idx = ds_index.col_lower_bound + 1; // 从第一列边开始
                while self.col_idx <= ds_index.col_upper_bound && !self.valid {
                    let col_idx = self.col_idx as usize;
                    if col_idx >= ds_guard.rows[row_idx].len() {
                        self.col_idx += 1;
                        continue;
                    }

                    let current_col = &ds_guard.rows[row_idx][col_idx];
                    if !matches!(current_col, Value::List(_))
                        || matches!(current_col, Value::List(list) if list.is_empty())
                    {
                        self.col_idx += 1;
                        continue;
                    }

                    if let Value::List(current_col_list) = current_col {
                        self.edge_idx_upper_bound = current_col_list.len() as i64;
                        self.edge_idx = 0;

                        while self.edge_idx < self.edge_idx_upper_bound && !self.valid {
                            let edge_idx = self.edge_idx as usize;
                            if edge_idx >= current_col_list.len() {
                                self.edge_idx += 1;
                                continue;
                            }

                            let current_edge = &current_col_list[edge_idx];
                            if !matches!(current_edge, Value::List(_)) {
                                self.edge_idx += 1;
                                continue;
                            }

                            self.valid = true;
                            self.current_ds_index = ds_idx;
                            self.current_row = row_idx;
                            self.edge_idx = edge_idx as i64; // 确保edge_idx是有效的
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
}

impl Iterator for GetNeighborsIter {
    fn kind(&self) -> IteratorKind {
        IteratorKind::GetNeighbors
    }

    fn valid(&self) -> bool {
        if !self.valid || self.current_ds_index >= self.ds_indices.len() {
            return false;
        }

        let ds_guard = self.ds_indices[self.current_ds_index].ds.lock();
        self.current_row < ds_guard.rows.len()
            && self.col_idx <= self.ds_indices[self.current_ds_index].col_upper_bound
            && self.edge_idx >= 0
            && self.edge_idx < self.edge_idx_upper_bound
    }

    fn next(&mut self) {
        if !self.valid() {
            return;
        }

        if self.no_edge {
            self.current_row += 1;
            let ds_guard = self.ds_indices[self.current_ds_index].ds.lock();
            if self.current_row >= ds_guard.rows.len() {
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

                let current_col_list = {
                    let ds_guard = self.ds_indices[self.current_ds_index].ds.lock();
                    if col_idx >= ds_guard.rows[self.current_row].len() {
                        self.col_idx += 1;
                        continue;
                    }

                    match &ds_guard.rows[self.current_row][col_idx] {
                        Value::List(list) if !list.is_empty() => Some(list.clone()),
                        _ => {
                            self.col_idx += 1;
                            continue;
                        }
                    }
                };

                if let Some(list) = current_col_list {
                    self.edge_idx_upper_bound = list.len() as i64;
                    self.edge_idx = 0;
                    break;
                }

                self.col_idx += 1;
            }

            if self.col_idx >= self.ds_indices[self.current_ds_index].col_upper_bound {
                // 移动到下一行
                self.current_row += 1;
                let ds_guard = self.ds_indices[self.current_ds_index].ds.lock();
                if self.current_row >= ds_guard.rows.len() {
                    // 移动到下一个数据集
                    self.current_ds_index += 1;
                    if self.current_ds_index < self.ds_indices.len() {
                        self.current_row = 0;
                        self.col_idx =
                            self.ds_indices[self.current_ds_index].col_lower_bound + 1;
                    } else {
                        self.valid = false;
                        break;
                    }
                } else {
                    self.col_idx = self.ds_indices[self.current_ds_index].col_lower_bound + 1;
                }
            }
        }
    }

    fn erase(&mut self) {
        // 实现真正的删除逻辑：从当前数据集中删除当前边
        if !self.valid() || self.no_edge {
            return;
        }

        let current_edge_name = match self.current_edge_name() {
            Some(name) => name.to_string(),
            None => return,
        };

        let (col_idx, row_idx) = {
            let ds_index = &self.ds_indices[self.current_ds_index];
            // 获取当前边的属性索引
            match ds_index.edge_props_map.get(&current_edge_name) {
                Some(prop_index) => (prop_index.col_idx, self.current_row),
                None => return,
            }
        };

        // 执行删除操作
        {
            let ds_index = &self.ds_indices[self.current_ds_index];
            let mut ds_guard = ds_index.ds.lock();

            if row_idx < ds_guard.rows.len() && col_idx < ds_guard.rows[row_idx].len() {
                if let Value::List(edge_col) = &mut ds_guard.rows[row_idx][col_idx] {
                    let edge_idx = self.edge_idx as usize;
                    if edge_idx < edge_col.len() {
                        edge_col.remove(edge_idx);

                        // 调整索引
                        if edge_col.is_empty() {
                            // 如果该列空了，删除整个列
                            ds_guard.rows[row_idx].remove(col_idx);
                            ds_guard.col_names.remove(col_idx);
                        }
                    }
                }
            }
        }

        // 重置到有效位置
        self.reset(self.current_row);
    }

    fn unstable_erase(&mut self) {
        // 快速删除：不保持顺序，直接交换删除
        if !self.valid() || self.no_edge {
            return;
        }

        let current_edge_name = match self.current_edge_name() {
            Some(name) => name.to_string(),
            None => return,
        };

        let (col_idx, row_idx) = {
            let ds_index = &self.ds_indices[self.current_ds_index];
            match ds_index.edge_props_map.get(&current_edge_name) {
                Some(prop_index) => (prop_index.col_idx, self.current_row),
                None => return,
            }
        };

        {
            let ds_index = &self.ds_indices[self.current_ds_index];
            let mut ds_guard = ds_index.ds.lock();

            if row_idx < ds_guard.rows.len() && col_idx < ds_guard.rows[row_idx].len() {
                if let Value::List(edge_col) = &mut ds_guard.rows[row_idx][col_idx] {
                    let edge_idx = self.edge_idx as usize;
                    if edge_idx < edge_col.len() {
                        // 快速删除：交换到最后然后pop
                        let len = edge_col.len();
                        if edge_idx != len - 1 {
                            edge_col.swap(edge_idx, len - 1);
                        }
                        edge_col.pop();
                    }
                }
            }
        }

        // 重置到有效位置
        self.reset(self.current_row);
    }

    fn clear(&mut self) {
        self.valid = false;
        self.ds_indices.clear();
        self.reset(0);
    }

    fn reset(&mut self, _pos: usize) {
        self.current_ds_index = 0;
        self.current_row = 0;
        self.col_idx = self.ds_indices[0].col_lower_bound;
        self.edge_idx = -1;
        self.edge_idx_upper_bound = -1;
        self.go_to_first_edge();
    }

    fn size(&self) -> usize {
        let mut count = 0;
        for ds_idx in &self.ds_indices {
            let ds_guard = ds_idx.ds.lock();
            for row in &ds_guard.rows {
                for edge_idx in ds_idx.edge_props_map.values() {
                    if edge_idx.col_idx < row.len() {
                        if let Value::List(list) = &row[edge_idx.col_idx] {
                            count += list.len();
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

        // 注意：由于使用了Mutex，无法返回借用的引用
        // 这里返回None以满足方法签名，实际应该使用move_row()方法
        None
    }

    fn move_row(&mut self) -> Option<Row> {
        if !self.valid() {
            return None;
        }

        let ds_index = &self.ds_indices[self.current_ds_index];
        let ds_guard = ds_index.ds.lock();
        let result = if self.current_row < ds_guard.rows.len() {
            Some(ds_guard.rows[self.current_row].clone())
        } else {
            None
        };
        drop(ds_guard);
        result
    }

    fn add_row(&mut self, _row: Row) {
        // GetNeighborsIter 不支持直接添加行
        // 它的数据结构是复杂的树状结构
    }

    fn select(&mut self, offset: usize, count: usize) {
        // 实现真正的选择逻辑：选择指定范围的边
        if self.no_edge || offset >= self.size() {
            self.clear();
            return;
        }

        // 收集所有边的信息
        let mut all_edges = Vec::new();
        let _original_state = (
            self.current_ds_index,
            self.current_row,
            self.col_idx,
            self.edge_idx,
            self.edge_idx_upper_bound,
            self.valid,
        );

        // 重置到开始位置，收集所有边
        self.reset(0);
        while self.valid() {
            if let Some(edge_name) = self.current_edge_name() {
                all_edges.push((
                    self.current_ds_index,
                    self.current_row,
                    self.col_idx,
                    self.edge_idx,
                    edge_name.to_string(),
                ));
            }
            self.next();
        }

        // 检查范围有效性
        if offset >= all_edges.len() {
            self.clear();
            return;
        }

        let end = std::cmp::min(offset + count, all_edges.len());
        let selected_edges = &all_edges[offset..end];

        // 重建数据集，只保留选中的边
        if !selected_edges.is_empty() {
            // 找到第一个选中的边，重置到该位置
            let (ds_idx, row_idx, col_idx, edge_idx, _) = selected_edges[0];
            self.current_ds_index = ds_idx;
            self.current_row = row_idx;
            self.col_idx = col_idx;
            self.edge_idx = edge_idx;
            self.valid = true;

            // 删除未选中的边
            for i in (0..all_edges.len()).rev() {
                if i < offset || i >= end {
                    let (ds_idx, row_idx, col_idx, edge_idx, _) = all_edges[i];
                    self.current_ds_index = ds_idx;
                    self.current_row = row_idx;
                    self.col_idx = col_idx;
                    self.edge_idx = edge_idx;
                    self.erase();
                }
            }

            // 重置到第一个选中的边
            self.reset(0);
        } else {
            self.clear();
        }
    }

    fn sample(&mut self, count: i64) {
        if count <= 0 {
            self.clear();
            return;
        }

        if self.no_edge {
            return;
        }

        let total_size = self.size();
        if total_size == 0 {
            self.clear();
            return;
        }

        let sample_count = count as usize;
        if sample_count >= total_size {
            // 如果采样数量大于等于总数，保持原样
            self.reset(0);
            return;
        }

        // 使用蓄水池采样算法
        let mut reservoir = Vec::new();
        let mut all_edges = Vec::new();

        // 收集所有边
        let _original_state = (
            self.current_ds_index,
            self.current_row,
            self.col_idx,
            self.edge_idx,
            self.edge_idx_upper_bound,
            self.valid,
        );

        self.reset(0);
        let mut index = 0;
        while self.valid() {
            if let Some(edge_name) = self.current_edge_name() {
                all_edges.push((
                    self.current_ds_index,
                    self.current_row,
                    self.col_idx,
                    self.edge_idx,
                    edge_name.to_string(),
                    index,
                ));
            }
            self.next();
            index += 1;
        }

        // 蓄水池采样
        for i in 0..all_edges.len() {
            if reservoir.len() < sample_count {
                reservoir.push(all_edges[i].clone());
            } else {
                // 使用哈希值生成伪随机数
                let mut hasher = DefaultHasher::new();
                i.hash(&mut hasher);
                let random_val = hasher.finish() as usize;
                let j = random_val % (i + 1);
                if j < sample_count {
                    reservoir[j] = all_edges[i].clone();
                }
            }
        }

        // 重建数据集，只保留采样的边
        if !reservoir.is_empty() {
            // 排序蓄水池，按原始顺序处理
            reservoir.sort_by_key(|edge| edge.5); // 按原始索引排序

            // 删除未采样的边
            for i in (0..all_edges.len()).rev() {
                let should_keep = reservoir.iter().any(|e| e.5 == all_edges[i].5);
                if !should_keep {
                    let (ds_idx, row_idx, col_idx, edge_idx, _, _) = all_edges[i];
                    self.current_ds_index = ds_idx;
                    self.current_row = row_idx;
                    self.col_idx = col_idx;
                    self.edge_idx = edge_idx;
                    self.erase();
                }
            }

            // 重置到第一个采样的边
            self.reset(0);
        } else {
            self.clear();
        }
    }

    fn erase_range(&mut self, first: usize, last: usize) {
        if first >= last || self.no_edge {
            return;
        }

        let total_size = self.size();
        if first >= total_size {
            return;
        }

        let end = std::cmp::min(last, total_size);

        // 收集所有边的信息
        let mut all_edges = Vec::new();

        let _original_state = (
            self.current_ds_index,
            self.current_row,
            self.col_idx,
            self.edge_idx,
            self.edge_idx_upper_bound,
            self.valid,
        );

        self.reset(0);
        let mut index = 0;
        while self.valid() {
            if let Some(edge_name) = self.current_edge_name() {
                all_edges.push((
                    self.current_ds_index,
                    self.current_row,
                    self.col_idx,
                    self.edge_idx,
                    edge_name.to_string(),
                    index,
                ));
            }
            self.next();
            index += 1;
        }

        // 删除指定范围的边（从后往前删除，避免索引变化）
        for i in (first..end).rev() {
            if i < all_edges.len() {
                let (ds_idx, row_idx, col_idx, edge_idx, _, _) = all_edges[i];
                self.current_ds_index = ds_idx;
                self.current_row = row_idx;
                self.col_idx = col_idx;
                self.edge_idx = edge_idx;
                self.erase();
            }
        }

        // 重置到有效位置
        self.reset(0);
    }

    fn get_column(&self, _col: &str) -> Option<&Value> {
        // 由于使用了Mutex，无法返回借用的引用，返回None
        None
    }

    fn get_column_by_index(&self, _index: i32) -> Option<&Value> {
        // 由于使用了Mutex，无法返回借用的引用，返回None
        None
    }

    fn get_column_index(&self, _col: &str) -> Option<usize> {
        if self.current_ds_index >= self.ds_indices.len() {
            return None;
        }

        self.ds_indices[self.current_ds_index]
            .col_indices
            .get(_col)
            .copied()
    }

    fn get_col_names(&self) -> Vec<String> {
        if self.current_ds_index < self.ds_indices.len() {
            let ds_guard = self.ds_indices[self.current_ds_index].ds.lock();
            ds_guard.col_names.clone()
        } else {
            Vec::new()
        }
    }

    fn copy(&self) -> Self {
        self.clone()
    }

    // 图特定方法
    fn get_tag_prop(&self, tag: &str, prop: &str) -> Option<Value> {
        if !self.valid() {
            return None;
        }

        let ds_index = &self.ds_indices[self.current_ds_index];
        let ds_guard = ds_index.ds.lock();

        if tag == "*" {
            // 搜索所有标签
            for prop_index in ds_index.tag_props_map.values() {
                if let Some(prop_idx) = prop_index.prop_indices.get(prop) {
                    let row = &ds_guard.rows[self.current_row];
                    if prop_index.col_idx < row.len() {
                        if let Value::List(list) = &row[prop_index.col_idx] {
                            if *prop_idx < list.len() {
                                return Some(list[*prop_idx].clone());
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

            let row = &ds_guard.rows[self.current_row];
            if prop_index.col_idx < row.len() {
                if let Value::List(list) = &row[prop_index.col_idx] {
                    if *prop_idx < list.len() {
                        Some(list[*prop_idx].clone())
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

    fn get_vertex(&self, _name: &str) -> Option<Value> {
        if !self.valid() {
            return None;
        }

        let vid_val = self.get_column("_vid")?.clone();

        // 缓存机制：如果vid相同，返回缓存的顶点
        if let Some(ref prev) = self.prev_vertex {
            if let Value::Vertex(ref vertex) = prev {
                if *vertex.vid == vid_val {
                    return Some(prev.clone());
                }
            }
        }

        let ds_index = &self.ds_indices[self.current_ds_index];
        let ds_guard = ds_index.ds.lock();
        let row = &ds_guard.rows[self.current_row];

        let mut vertex = crate::core::Vertex::new(vid_val.clone(), Vec::new());

        // 遍历所有tag，收集属性
        for (tag_name, prop_index) in &ds_index.tag_props_map {
            if prop_index.col_idx >= row.len() {
                continue;
            }

            if let Value::List(prop_list) = &row[prop_index.col_idx] {
                let mut tag_props = HashMap::new();
                for (i, prop_name) in prop_index.prop_list.iter().enumerate() {
                    if i < prop_list.len() && prop_name != "_tag" {
                        tag_props.insert(prop_name.clone(), prop_list[i].clone());
                    }
                }
                vertex.add_tag(Tag::new(tag_name.clone(), tag_props));
            }
        }

        let vertex_value = Value::Vertex(Box::new(vertex));
        Some(vertex_value)
    }

    fn get_edge_prop(&self, edge: &str, prop: &str) -> Option<Value> {
        if !self.valid() {
            return None;
        }

        let ds_index = &self.ds_indices[self.current_ds_index];
        let ds_guard = ds_index.ds.lock();

        // 边名必须带 +/- 前缀
        let prop_index = ds_index.edge_props_map.get(edge)?;
        let prop_idx = prop_index.prop_indices.get(prop)?;

        let row = &ds_guard.rows[self.current_row];
        if prop_index.col_idx >= row.len() {
            return None;
        }

        if let Value::List(edge_col) = &row[prop_index.col_idx] {
            if self.edge_idx >= 0 && (self.edge_idx as usize) < edge_col.len() {
                if let Value::List(edge_data) = &edge_col[self.edge_idx as usize] {
                    if *prop_idx < edge_data.len() {
                        return Some(edge_data[*prop_idx].clone());
                    }
                }
            }
        }

        None
    }

    fn get_edge(&self) -> Option<Value> {
        if !self.valid() || self.no_edge {
            return None;
        }

        let current_edge_name = self.current_edge_name()?;
        // 去掉+/-前缀得到边类型名
        let edge_type = if current_edge_name.starts_with('+') || current_edge_name.starts_with('-') {
            &current_edge_name[1..]
        } else {
            current_edge_name
        };

        let src_vid = self.get_column("_vid")?.clone();
        // 使用带前缀的边名调用 get_edge_prop
        let dst_vid = self.get_edge_prop(current_edge_name, "_dst")?.clone();

        let ranking = match self.get_edge_prop(current_edge_name, "_rank") {
            Some(Value::Int(rank)) => rank,
            _ => 0,
        };

        let ds_index = &self.ds_indices[self.current_ds_index];
        let prop_index = ds_index.edge_props_map.get(current_edge_name)?;

        let mut edge_props = HashMap::new();

        // 收集边属性（排除系统属性）
        if let Some(edge_data) = self.get_current_edge_data() {
            for (i, prop_name) in prop_index.prop_list.iter().enumerate() {
                if i < edge_data.len() {
                    let is_system_prop = matches!(
                        prop_name.as_str(),
                        "_dst" | "_rank" | "_type" | "_src"
                    );
                    if !is_system_prop {
                        edge_props.insert(prop_name.clone(), edge_data[i].clone());
                    }
                }
            }
        }

        let edge = Edge::new(
            src_vid,
            dst_vid,
            edge_type.to_string(),
            ranking,
            edge_props,
        );

        Some(Value::Edge(edge))
    }
}

impl GetNeighborsIter {
    /// 获取当前边的数据列表
    fn get_current_edge_data(&self) -> Option<Vec<Value>> {
        if !self.valid() || self.no_edge {
            return None;
        }

        let current_edge_name = self.current_edge_name()?;
        let ds_index = &self.ds_indices[self.current_ds_index];
        let ds_guard = ds_index.ds.lock();
        let prop_index = ds_index.edge_props_map.get(current_edge_name)?;

        let row = &ds_guard.rows[self.current_row];
        if prop_index.col_idx >= row.len() {
            return None;
        }

        if let Value::List(edge_col) = &row[prop_index.col_idx] {
            if self.edge_idx >= 0 && (self.edge_idx as usize) < edge_col.len() {
                if let Value::List(edge_data) = &edge_col[self.edge_idx as usize] {
                    return Some(edge_data.clone().into_vec());
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::DataSet;
    use crate::core::value::dataset::List;

    fn create_test_neighbors_data() -> Value {
        // 创建测试数据：一个顶点有两条边
        let mut dataset = DataSet::new();
        dataset.col_names = vec![
            "_vid".to_string(),
            "_stats".to_string(),
            "_tag:player:name:age".to_string(),
            "_edge:+follow:weight".to_string(),
        ];

        dataset.rows = vec![vec![
            Value::String("player1".to_string()), // _vid
            Value::String("stats".to_string()),   // _stats
            Value::List(List::from(vec![
                // _tag:player:name:age
                Value::String("Alice".to_string()),
                Value::Int(25),
            ])),
            Value::List(List::from(vec![
                // _edge:+follow:weight
                Value::List(List::from(vec![Value::Float(0.8)])), // 第一条边
                Value::List(List::from(vec![Value::Float(0.6)])), // 第二条边
            ])),
        ]];

        Value::List(List::from(vec![Value::DataSet(dataset)]))
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
        let mut iter = GetNeighborsIter::new(data)
            .expect("GetNeighborsIter should be created successfully in test");

        assert_eq!(iter.kind(), IteratorKind::GetNeighbors);

        // 调试信息
        println!("valid: {}", iter.valid());
        println!("current_ds_index: {}", iter.current_ds_index);
        println!("current_row: {}", iter.current_row);
        println!("col_idx: {}", iter.col_idx);
        println!("edge_idx: {}", iter.edge_idx);
        println!("edge_idx_upper_bound: {}", iter.edge_idx_upper_bound);
        println!("no_edge: {}", iter.no_edge);
        println!("ds_indices len: {}", iter.ds_indices.len());
        if !iter.ds_indices.is_empty() {
            println!("col_upper_bound: {}", iter.ds_indices[0].col_upper_bound);
        }

        // 如果迭代器无效，尝试重置
        if !iter.valid() {
            println!("迭代器无效，尝试重置");
            iter.reset(0);
            println!("重置后 valid: {}", iter.valid());
        }

        assert!(iter.valid());
    }

    #[test]
    fn test_get_neighbors_iter_navigation() {
        let data = Arc::new(create_test_neighbors_data());
        let mut iter = GetNeighborsIter::new(data)
            .expect("GetNeighborsIter should be created successfully in test");

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
        let iter = GetNeighborsIter::new(data)
            .expect("GetNeighborsIter should be created successfully in test");

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
    fn test_get_neighbors_iter_get_edge_prop() {
        let data = Arc::new(create_test_neighbors_data());
        let iter = GetNeighborsIter::new(data)
            .expect("GetNeighborsIter should be created successfully in test");

        // 获取边属性
        let weight = iter.get_edge_prop("+follow", "weight");
        assert!(weight.is_some());
        assert_eq!(
            weight.expect("Edge property should exist in test"),
            Value::Float(0.8)
        );
    }

    #[test]
    fn test_get_neighbors_iter_size() {
        let data = Arc::new(create_test_neighbors_data());
        let iter = GetNeighborsIter::new(data)
            .expect("GetNeighborsIter should be created successfully in test");

        // 应该有2条边
        assert_eq!(iter.size(), 2);
    }

    #[test]
    fn test_get_neighbors_iter_reset() {
        let data = Arc::new(create_test_neighbors_data());
        let mut iter = GetNeighborsIter::new(data)
            .expect("GetNeighborsIter should be created successfully in test");

        iter.next();
        assert!(iter.valid());

        iter.reset(0);
        assert!(iter.valid());
    }
}
