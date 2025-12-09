# Context 模块实现路线图

## 核心功能依赖树

```
查询执行流程：
  Parser → ValidateContext ┐
                           ├→ SymbolTable
  Planner → ExecutionContext
                           ├→ QueryExpressionContext
  Optimizer → SymbolTable
                           │
  Executor → Iterator ◄────┴→ Result
             QueryExpressionContext
                │
            执行器评估表达式
                │
            返回 Result (包含 Iterator)
```

## 实现依赖关系

### 模块级依赖

```
独立模块 (无依赖)
  ├─ Result 和 ResultBuilder
  ├─ Iterator 基类
  └─ DefaultIter

一级依赖
  ├─ ExecutionContext (依赖: Value)
  ├─ SequentialIter (依赖: Iterator基类)
  └─ SymbolTable (依赖: Variable定义)

二级依赖
  ├─ GetNeighborsIter (依赖: Iterator, Value结构)
  ├─ PropIter (依赖: Iterator)
  ├─ QueryExpressionContext (依赖: ExecutionContext, Iterator)
  └─ ValidateContext (依赖: ColsDef, SpaceInfo)

三级依赖
  └─ QueryContext (依赖: 以上所有)
```

## 详细实现步骤

### 步骤 1: 基础类型定义 (0.5 天)

**文件**: `src/core/result.rs` (新建)

```rust
//! 执行结果定义

use crate::core::Value;

/// 执行结果的状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultState {
    /// 未执行
    Unexecuted,
    /// 部分成功（有警告或可恢复的错误）
    PartialSuccess,
    /// 成功
    Success,
}

/// 执行结果 - 包装 Value 和执行状态
#[derive(Debug, Clone)]
pub struct Result {
    value: Arc<Value>,
    state: ResultState,
    message: String,
    // Iterator 将在后续步骤添加
}

impl Result {
    pub fn new(value: Value) -> Self {
        Self {
            value: Arc::new(value),
            state: ResultState::Success,
            message: String::new(),
        }
    }

    pub fn with_state(mut self, state: ResultState) -> Self {
        self.state = state;
        self
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = message;
        self
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn state(&self) -> ResultState {
        self.state
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn empty() -> Self {
        Self {
            value: Arc::new(Value::Empty),
            state: ResultState::Success,
            message: String::new(),
        }
    }
}

/// 结果构建器
pub struct ResultBuilder {
    value: Option<Value>,
    state: ResultState,
    message: String,
}

impl ResultBuilder {
    pub fn new() -> Self {
        Self {
            value: None,
            state: ResultState::Success,
            message: String::new(),
        }
    }

    pub fn value(mut self, value: Value) -> Self {
        self.value = Some(value);
        self
    }

    pub fn state(mut self, state: ResultState) -> Self {
        self.state = state;
        self
    }

    pub fn message(mut self, message: String) -> Self {
        self.message = message;
        self
    }

    pub fn build(self) -> Result {
        Result {
            value: Arc::new(self.value.unwrap_or(Value::Empty)),
            state: self.state,
            message: self.message,
        }
    }
}

impl Default for ResultBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_creation() {
        let result = Result::new(Value::Int(42));
        assert_eq!(result.state(), ResultState::Success);
        assert!(matches!(result.value(), Value::Int(42)));
    }

    #[test]
    fn test_result_builder() {
        let result = ResultBuilder::new()
            .value(Value::String("test".to_string()))
            .state(ResultState::PartialSuccess)
            .message("有警告".to_string())
            .build();

        assert_eq!(result.state(), ResultState::PartialSuccess);
        assert_eq!(result.message(), "有警告");
    }
}
```

### 步骤 2: Iterator 基类 (1 天)

**文件**: `src/storage/iterator/mod.rs` 和子模块

```rust
//! 迭代器模块 - 支持各种数据遍历

use crate::core::Value;
use std::sync::Arc;

/// 迭代器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// 迭代器基类接口
pub trait Iterator: Send + Sync {
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
    fn reset(&mut self, pos: usize = 0);

    /// 获取总行数
    fn size(&self) -> usize;

    /// 检查是否为空
    fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// 获取当前行
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

    /// 深拷贝迭代器（用于保存状态）
    fn copy(&self) -> Box<dyn Iterator>;

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
    fn get_tag_prop(&self, _tag: &str, _prop: &str) -> Option<Value> {
        None
    }

    fn get_edge_prop(&self, _edge: &str, _prop: &str) -> Option<Value> {
        None
    }

    fn get_vertex(&self, _name: &str) -> Option<Value> {
        None
    }

    fn get_edge(&self) -> Option<Value> {
        None
    }
}

/// 行定义
pub type Row = Vec<Value>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iterator_trait() {
        // 后续实现具体的迭代器后进行测试
    }
}
```

### 步骤 3: DefaultIter 实现 (0.5 天)

**文件**: `src/storage/iterator/default_iter.rs`

```rust
//! 默认迭代器 - 用于单个值

use super::{Iterator, IteratorKind, Row};
use crate::core::Value;
use std::sync::Arc;

pub struct DefaultIter {
    value: Arc<Value>,
    valid: bool,
}

impl DefaultIter {
    pub fn new(value: Arc<Value>) -> Self {
        Self {
            value,
            valid: true,
        }
    }
}

impl Iterator for DefaultIter {
    fn kind(&self) -> IteratorKind {
        IteratorKind::Default
    }

    fn valid(&self) -> bool {
        self.valid
    }

    fn next(&mut self) {
        self.valid = false;
    }

    fn erase(&mut self) {
        self.valid = false;
    }

    fn unstable_erase(&mut self) {
        self.valid = false;
    }

    fn clear(&mut self) {
        self.valid = false;
    }

    fn reset(&mut self, _pos: usize) {
        self.valid = true;
    }

    fn size(&self) -> usize {
        1
    }

    fn row(&self) -> Option<&Row> {
        // DefaultIter 只有一行（整个值）
        // 返回 None，因为我们不能将 Value 转成 Row
        None
    }

    fn move_row(&mut self) -> Option<Row> {
        None
    }

    fn select(&mut self, offset: usize, count: usize) {
        if offset > 0 || count == 0 {
            self.clear();
        }
    }

    fn sample(&mut self, count: i64) {
        if count == 0 {
            self.clear();
        }
    }

    fn erase_range(&mut self, _first: usize, _last: usize) {
        // DefaultIter 只有一行
        self.clear();
    }

    fn get_column(&self, _col: &str) -> Option<&Value> {
        Some(&self.value)
    }

    fn get_column_by_index(&self, _index: i32) -> Option<&Value> {
        Some(&self.value)
    }

    fn get_column_index(&self, _col: &str) -> Option<usize> {
        Some(0)
    }

    fn get_col_names(&self) -> Vec<String> {
        vec![]
    }

    fn copy(&self) -> Box<dyn Iterator> {
        Box::new(DefaultIter {
            value: self.value.clone(),
            valid: self.valid,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_iter() {
        let value = Arc::new(Value::Int(42));
        let mut iter = DefaultIter::new(value.clone());

        assert!(iter.valid());
        assert_eq!(iter.size(), 1);

        iter.next();
        assert!(!iter.valid());
    }

    #[test]
    fn test_default_iter_reset() {
        let value = Arc::new(Value::Int(42));
        let mut iter = DefaultIter::new(value);

        iter.next();
        assert!(!iter.valid());

        iter.reset();
        assert!(iter.valid());
    }
}
```

### 步骤 4: SequentialIter 实现 (1.5 天)

**文件**: `src/storage/iterator/sequential_iter.rs`

```rust
//! 顺序迭代器 - 用于 DataSet 行级迭代

use super::{Iterator, IteratorKind, Row};
use crate::core::Value;
use std::sync::Arc;

pub struct SequentialIter {
    data: Arc<Value>,  // 应该是 DataSet
    rows: Vec<Row>,
    curr_pos: usize,
    col_names: Vec<String>,
}

impl SequentialIter {
    pub fn new(data: Arc<Value>) -> Result<Self, String> {
        // 验证是否是 DataSet
        match &*data {
            Value::DataSet(dataset) => {
                let col_names = dataset.column_names.clone();
                let rows = dataset.rows.clone();
                Ok(Self {
                    data,
                    rows,
                    curr_pos: 0,
                    col_names,
                })
            }
            _ => Err("SequentialIter 只支持 DataSet".to_string()),
        }
    }

    pub fn curr_row(&self) -> Option<&Row> {
        if self.curr_pos < self.rows.len() {
            Some(&self.rows[self.curr_pos])
        } else {
            None
        }
    }
}

impl Iterator for SequentialIter {
    fn kind(&self) -> IteratorKind {
        IteratorKind::Sequential
    }

    fn valid(&self) -> bool {
        self.curr_pos < self.rows.len()
    }

    fn next(&mut self) {
        self.curr_pos += 1;
    }

    fn erase(&mut self) {
        if self.curr_pos < self.rows.len() {
            self.rows.remove(self.curr_pos);
        }
    }

    fn unstable_erase(&mut self) {
        if self.curr_pos < self.rows.len() {
            // 快速删除：交换最后一行到当前位置，然后 pop
            self.rows.swap(self.curr_pos, self.rows.len() - 1);
            self.rows.pop();
        }
    }

    fn clear(&mut self) {
        self.rows.clear();
        self.curr_pos = 0;
    }

    fn reset(&mut self, pos: usize) {
        self.curr_pos = pos;
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
        }
    }

    fn erase_range(&mut self, first: usize, last: usize) {
        if first < self.rows.len() {
            let end = std::cmp::min(last, self.rows.len());
            self.rows.drain(first..end);
            // 调整当前位置
            if self.curr_pos >= first {
                self.curr_pos = first;
            }
        }
    }

    fn get_column(&self, col: &str) -> Option<&Value> {
        let col_idx = self.col_names.iter().position(|c| c == col)?;
        self.curr_row()?.get(col_idx)
    }

    fn get_column_by_index(&self, index: i32) -> Option<&Value> {
        let row = self.curr_row()?;
        let size = row.len() as i32;
        let idx = if index >= 0 {
            index as usize
        } else {
            ((size + index) % size) as usize
        };
        row.get(idx)
    }

    fn get_column_index(&self, col: &str) -> Option<usize> {
        self.col_names.iter().position(|c| c == col)
    }

    fn get_col_names(&self) -> Vec<String> {
        self.col_names.clone()
    }

    fn copy(&self) -> Box<dyn Iterator> {
        Box::new(SequentialIter {
            data: self.data.clone(),
            rows: self.rows.clone(),
            curr_pos: self.curr_pos,
            col_names: self.col_names.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_dataset() -> Value {
        // 创建测试数据集
        // TODO: 实现具体的 DataSet 创建逻辑
        Value::DataSet(Default::default())
    }

    #[test]
    fn test_sequential_iter_creation() {
        let data = Arc::new(create_test_dataset());
        let iter = SequentialIter::new(data);
        assert!(iter.is_ok());
    }

    #[test]
    fn test_sequential_iter_navigation() {
        // TODO: 实现完整测试
    }
}
```

### 步骤 5: ExecutionContext 增强 (1 天)

**增强现有的 `src/core/execution_context.rs`**

```rust
// 添加 Result 支持
use crate::core::result::{Result as ExecResult, ResultState};

pub struct ExecutionContext {
    // ... 现有字段 ...
    
    // 添加：结果历史
    result_map: Arc<RwLock<HashMap<String, Vec<ExecResult>>>>,
}

impl ExecutionContext {
    // 添加方法
    pub fn set_result(&self, name: &str, result: ExecResult) -> Result<()> {
        let mut map = self.result_map.write().unwrap();
        let hist = map.entry(name.to_string()).or_insert_with(Vec::new);
        hist.push(result);
        Ok(())
    }

    pub fn get_result(&self, name: &str) -> Result<ExecResult> {
        let map = self.result_map.read().unwrap();
        Ok(map
            .get(name)
            .and_then(|hist| hist.last())
            .cloned()
            .unwrap_or_else(ExecResult::empty))
    }

    pub fn get_history(&self, name: &str) -> Result<Vec<ExecResult>> {
        let map = self.result_map.read().unwrap();
        Ok(map.get(name).cloned().unwrap_or_default())
    }

    pub fn trunc_history(&self, name: &str, num_versions_to_keep: usize) -> Result<()> {
        let mut map = self.result_map.write().unwrap();
        if let Some(hist) = map.get_mut(name) {
            if hist.len() > num_versions_to_keep {
                let start = hist.len() - num_versions_to_keep;
                hist.drain(0..start);
            }
        }
        Ok(())
    }
}
```

### 步骤 6: QueryExpressionContext 实现 (1.5 天)

**文件**: `src/query/context/expression_context.rs` (新建)

```rust
//! 表达式求值上下文

use crate::core::{ExecutionContext, Value};
use crate::storage::iterator::Iterator;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

pub struct QueryExpressionContext {
    // 执行上下文（变量值来源）
    ectx: Arc<ExecutionContext>,

    // 当前迭代器（用于访问行数据）
    iter: Arc<Mutex<Option<Box<dyn Iterator>>>>,

    // 表达式内部变量（例如列表解析中的变量）
    expr_value_map: Arc<RwLock<HashMap<String, Value>>>,
}

impl QueryExpressionContext {
    pub fn new(ectx: Arc<ExecutionContext>) -> Self {
        Self {
            ectx,
            iter: Arc::new(Mutex::new(None)),
            expr_value_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 设置当前迭代器（用于行数据访问）
    pub fn with_iterator(self, iter: Box<dyn Iterator>) -> Self {
        *self.iter.lock().unwrap() = Some(iter);
        self
    }

    /// 获取变量值
    pub fn get_var(&self, var: &str) -> Result<Value, String> {
        self.ectx.get_value(var)
    }

    /// 获取指定版本的变量值
    pub fn get_versioned_var(&self, var: &str, version: i64) -> Result<Value, String> {
        self.ectx.get_versioned_value(var, version)
    }

    /// 设置变量值
    pub fn set_var(&self, var: &str, value: Value) -> Result<(), String> {
        self.ectx.set_value(var, value)
    }

    /// 设置表达式内部变量（不持久化到执行上下文）
    pub fn set_inner_var(&self, var: &str, value: Value) {
        self.expr_value_map.write().unwrap().insert(var.to_string(), value);
    }

    /// 获取表达式内部变量
    pub fn get_inner_var(&self, var: &str) -> Option<Value> {
        self.expr_value_map.read().unwrap().get(var).cloned()
    }

    /// 获取列值（从当前行）
    pub fn get_column(&self, col: &str) -> Result<Value, String> {
        let iter_guard = self.iter.lock().unwrap();
        match iter_guard.as_ref() {
            Some(iter) => iter.get_column(col)
                .ok_or_else(|| format!("列 {} 不存在", col))
                .map(|v| v.clone()),
            None => Err("没有设置迭代器".to_string()),
        }
    }

    /// 获取列索引
    pub fn get_column_index(&self, col: &str) -> Result<usize, String> {
        let iter_guard = self.iter.lock().unwrap();
        match iter_guard.as_ref() {
            Some(iter) => iter.get_column_index(col)
                .ok_or_else(|| format!("列 {} 的索引不存在", col)),
            None => Err("没有设置迭代器".to_string()),
        }
    }

    /// 获取属性值（$a.prop_name）
    pub fn get_var_prop(&self, _var: &str, prop: &str) -> Result<Value, String> {
        // 从当前行获取属性
        self.get_column(prop)
    }

    /// 获取标签属性（tag.prop_name）
    pub fn get_tag_prop(&self, tag: &str, prop: &str) -> Result<Value, String> {
        let iter_guard = self.iter.lock().unwrap();
        match iter_guard.as_ref() {
            Some(iter) => iter.get_tag_prop(tag, prop)
                .ok_or_else(|| format!("标签 {} 的属性 {} 不存在", tag, prop)),
            None => Err("没有设置迭代器".to_string()),
        }
    }

    /// 获取边属性（edge.prop_name）
    pub fn get_edge_prop(&self, edge: &str, prop: &str) -> Result<Value, String> {
        let iter_guard = self.iter.lock().unwrap();
        match iter_guard.as_ref() {
            Some(iter) => iter.get_edge_prop(edge, prop)
                .ok_or_else(|| format!("边 {} 的属性 {} 不存在", edge, prop)),
            None => Err("没有设置迭代器".to_string()),
        }
    }

    /// 获取顶点
    pub fn get_vertex(&self, name: &str) -> Result<Value, String> {
        let iter_guard = self.iter.lock().unwrap();
        match iter_guard.as_ref() {
            Some(iter) => iter.get_vertex(name)
                .ok_or_else(|| format!("顶点 {} 不存在", name)),
            None => Err("没有设置迭代器".to_string()),
        }
    }

    /// 获取边
    pub fn get_edge(&self) -> Result<Value, String> {
        let iter_guard = self.iter.lock().unwrap();
        match iter_guard.as_ref() {
            Some(iter) => iter.get_edge()
                .ok_or_else(|| "边不存在".to_string()),
            None => Err("没有设置迭代器".to_string()),
        }
    }
}

impl Clone for QueryExpressionContext {
    fn clone(&self) -> Self {
        Self {
            ectx: self.ectx.clone(),
            iter: Arc::new(Mutex::new(
                self.iter.lock().unwrap().as_ref().map(|i| i.copy())
            )),
            expr_value_map: Arc::new(
                RwLock::new(self.expr_value_map.read().unwrap().clone())
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inner_var_management() {
        let ectx = Arc::new(ExecutionContext::new());
        let qctx = QueryExpressionContext::new(ectx);

        qctx.set_inner_var("temp", Value::Int(100));
        assert_eq!(qctx.get_inner_var("temp"), Some(Value::Int(100)));
    }

    #[test]
    fn test_var_access() {
        let ectx = Arc::new(ExecutionContext::new());
        ectx.set_value("x", Value::Int(42)).unwrap();

        let qctx = QueryExpressionContext::new(ectx);
        let val = qctx.get_var("x").unwrap();
        assert_eq!(val, Value::Int(42));
    }
}
```

## 优先级总结表

| 优先级 | 功能 | 工作量 | 依赖 | 影响范围 |
|--------|------|--------|------|---------|
| 🔴 P1 | Result + ResultBuilder | 0.5 天 | Value | ExecutionContext, Iterator |
| 🔴 P1 | Iterator 基类 | 1 天 | Value | 所有迭代器 |
| 🔴 P1 | DefaultIter | 0.5 天 | Iterator 基类 | 常量表达式 |
| 🔴 P1 | SequentialIter | 1.5 天 | Iterator 基类, DataSet | 行级操作 |
| 🔴 P1 | QueryExpressionContext | 1.5 天 | ExecutionContext, Iterator | 表达式求值 |
| 🟡 P2 | GetNeighborsIter | 1.5 天 | Iterator 基类 | 图遍历 |
| 🟡 P2 | PropIter | 1 天 | Iterator 基类 | 属性查询 |
| 🟡 P2 | ExecutionContext 增强 | 1 天 | Result, Iterator | 版本管理 |
| 🟡 P2 | ValidateContext 增强 | 1 天 | ColsDef | 语义分析 |
| 🟡 P2 | SymbolTable 完善 | 1 天 | Variable | 优化 |

**总工作量**: ~10 天（开发 + 测试）

## 下一步行动

1. **立即开始** (今天):
   - 创建 `src/core/result.rs` - Result 和 ResultBuilder
   - 创建 `src/storage/iterator/mod.rs` - Iterator 基类

2. **明天**:
   - 实现 DefaultIter 和 SequentialIter
   - 为 ExecutionContext 添加 Result 支持

3. **后天**:
   - 实现 QueryExpressionContext
   - 编写集成测试

4. **第一周**:
   - 实现 GetNeighborsIter 和 PropIter
   - 完整的单元测试和文档

5. **第二周**:
   - 性能优化和基准测试
   - 与其他模块集成测试
