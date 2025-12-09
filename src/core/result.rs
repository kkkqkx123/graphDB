//! 执行结果模块 - 表示查询执行的结果
//! 对应原C++中的Result.h/cpp

use std::sync::Arc;
use crate::core::{Value, NullType};

/// 查询执行结果状态
#[derive(Debug, Clone, PartialEq)]
pub enum ResultState {
    UnExecuted,
    PartialSuccess,
    Success,
}

/// 执行结果
///
/// 一个执行器将产生一个结果
/// 对应原C++中的Result类
pub struct ResultCore {
    pub check_memory: bool,
    pub state: ResultState,
    pub msg: String,
    pub value: Arc<Value>,
    pub iterator: Arc<dyn ResultIterator>,
}

// 新的Result结构体，使用新的Iterator系统
#[derive(Debug, Clone)]
pub struct NewResult {
    pub check_memory: bool,
    pub state: ResultState,
    pub msg: String,
    pub value: Arc<Value>,
    pub iterator: Option<Box<dyn Iterator>>,
}

impl NewResult {
    pub fn new(value: Value, state: ResultState) -> Self {
        Self {
            check_memory: false,
            state,
            msg: String::new(),
            value: Arc::new(value),
            iterator: None,
        }
    }

    pub fn with_iterator(mut self, iterator: Box<dyn Iterator>) -> Self {
        self.iterator = Some(iterator);
        self
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn state(&self) -> ResultState {
        self.state
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }

    pub fn iterator(&self) -> Option<&Box<dyn Iterator>> {
        self.iterator.as_ref()
    }

    pub fn iterator_mut(&mut self) -> Option<&mut Box<dyn Iterator>> {
        self.iterator.as_mut()
    }

    pub fn size(&self) -> usize {
        self.iterator.as_ref().map(|iter| iter.size()).unwrap_or(0)
    }
}

impl std::fmt::Debug for ResultCore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResultCore")
            .field("check_memory", &self.check_memory)
            .field("state", &self.state)
            .field("msg", &self.msg)
            .field("value", &self.value)
            .finish()
    }
}

impl Clone for ResultCore {
    fn clone(&self) -> Self {
        ResultCore {
            check_memory: self.check_memory,
            state: self.state.clone(),
            msg: self.msg.clone(),
            value: self.value.clone(),
            iterator: self.iterator.clone(), // This will work because Arc implements Clone
        }
    }
}

#[derive(Clone)]
pub struct Result {
    core: ResultCore,
}

impl PartialEq for Result {
    fn eq(&self, other: &Self) -> bool {
        self.core.check_memory == other.core.check_memory &&
        self.core.state == other.core.state &&
        self.core.msg == other.core.msg &&
        self.core.value == other.core.value
        // Note: We're not comparing the iterator because it's not PartialEq
    }
}

impl std::fmt::Debug for Result {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Result")
            .field("check_memory", &self.core.check_memory)
            .field("state", &self.core.state)
            .field("msg", &self.core.msg)
            .field("value", &self.core.value)
            .finish()
    }
}

impl Result {
    /// 创建空结果
    pub fn empty() -> Self {
        Self {
            core: ResultCore {
                check_memory: false,
                state: ResultState::UnExecuted,
                msg: String::new(),
                value: Arc::new(Value::Null(NullType::Null)),
                iterator: Arc::new(SequentialIterator::new(Box::new(DefaultIter::new(Arc::new(Value::Null(NullType::Null)))))), // 使用默认迭代器
            },
        }
    }

    /// 创建新的结果
    pub fn new(value: Value, state: ResultState) -> Self {
        Self {
            core: ResultCore {
                check_memory: false,
                state,
                msg: String::new(),
                value: Arc::new(value),
                iterator: Arc::new(SequentialIterator::new(Box::new(DefaultIter::new(Arc::new(Value::Null(NullType::Null)))))), // 使用默认迭代器
            },
        }
    }

    /// 创建带消息的结果
    pub fn with_message(value: Value, state: ResultState, msg: String) -> Self {
        Self {
            core: ResultCore {
                check_memory: false,
                state,
                msg,
                value: Arc::new(value),
                iterator: Arc::new(SequentialIterator::new(Box::new(DefaultIter::new(Arc::new(Value::Null(NullType::Null)))))), // 使用默认迭代器
            },
        }
    }

    /// 获取值的引用
    pub fn value(&self) -> &Value {
        &self.core.value
    }

    /// 获取值的Arc引用
    pub fn value_arc(&self) -> Arc<Value> {
        self.core.value.clone()
    }

    /// 获取结果状态
    pub fn state(&self) -> &ResultState {
        &self.core.state
    }

    /// 获取结果消息
    pub fn msg(&self) -> &str {
        &self.core.msg
    }

    /// 获取迭代器的引用
    pub fn iterator(&self) -> &Arc<dyn ResultIterator> {
        &self.core.iterator
    }

    /// 获取迭代器的可变引用
    pub fn iterator_mut(&mut self) -> &mut Arc<dyn ResultIterator> {
        &mut self.core.iterator
    }

    /// 检查内存
    pub fn check_memory(&self) -> bool {
        self.core.check_memory
    }

    /// 设置内存检查标志
    pub fn set_check_memory(&mut self, check_memory: bool) {
        self.core.check_memory = check_memory;
    }

    /// 获取结果大小
    pub fn size(&self) -> usize {
        self.core.iterator.size()
    }

    /// 获取列名（如果值是数据集）
    pub fn get_col_names(&self) -> Vec<String> {
        match &*self.core.value {
            Value::DataSet(dataset) => dataset.col_names.clone(),
            _ => vec![],
        }
    }
}

impl Default for Result {
    fn default() -> Self {
        Self::empty()
    }
}

/// 结果构建器
///
/// 用于构建Result对象
/// 对应原C++中的ResultBuilder类
#[derive(Debug)]
pub struct ResultBuilder {
    core: ResultCore,
}

impl ResultBuilder {
    /// 创建新的结果构建器
    pub fn new() -> Self {
        Self {
            core: ResultCore {
                check_memory: false,
                state: ResultState::Success,
                msg: String::new(),
                value: Arc::new(Value::Null(NullType::Null)),
                iterator: Arc::new(SequentialIterator::new(Box::new(DefaultIter::new(Arc::new(Value::Null(NullType::Null)))))), // 使用默认迭代器
            },
        }
    }

    /// 设置值
    pub fn value(mut self, value: Value) -> Self {
        self.core.value = Arc::new(value);
        self
    }

    /// 设置状态
    pub fn state(mut self, state: ResultState) -> Self {
        self.core.state = state;
        self
    }

    /// 设置消息
    pub fn msg(mut self, msg: String) -> Self {
        self.core.msg = msg;
        self
    }

    /// 设置迭代器
    pub fn iterator(mut self, iterator: Arc<dyn ResultIterator>) -> Self {
        self.core.iterator = iterator.clone();
        // 如果迭代器存在，更新值为迭代器的值
        if !iterator.is_empty() {
            self.core.value = iterator.value_ptr();
        }
        self
    }

    /// 设置新的迭代器（使用新的Iterator系统）
    pub fn new_iterator(mut self, iterator: Box<dyn Iterator>) -> Self {
        let adapter = IteratorAdapter::new(iterator);
        self.core.iterator = Arc::new(adapter);
        self
    }

    /// 设置内存检查标志
    pub fn check_memory(mut self, check_memory: bool) -> Self {
        self.core.check_memory = check_memory;
        self
    }

    /// 构建结果
    pub fn build(self) -> Result {
        Result {
            core: self.core,
        }
    }
}

impl Default for ResultBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// 定义ResultIterator trait
pub trait ResultIterator: Send + Sync {
    /// 获取值
    fn value_ptr(&self) -> Arc<Value>;

    /// 是否有效
    fn is_valid(&self) -> bool;

    /// 移动到下一个
    fn next(&mut self);

    /// 当前位置的行
    fn current_row(&self) -> Option<&Value>;

    /// 大小
    fn size(&self) -> usize;

    /// 是否为空
    fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// 重置位置
    fn reset(&mut self);

    /// 清空
    fn clear(&mut self);

    /// 获取列值
    fn get_column(&self, col_name: &str) -> &Value;

    /// 获取列值（通过索引）
    fn get_column_by_index(&self, index: usize) -> &Value;
}

// 使用新的Iterator系统
use crate::storage::iterator::{Iterator, DefaultIter};

// 适配器：将新的Iterator适配到ResultIterator接口
pub struct IteratorAdapter {
    iter: Box<dyn Iterator>,
}

impl IteratorAdapter {
    pub fn new(iter: Box<dyn Iterator>) -> Self {
        Self { iter }
    }
}

impl ResultIterator for IteratorAdapter {
    fn value_ptr(&self) -> Arc<Value> {
        // 从迭代器获取当前行的第一个值作为默认值
        if let Some(row) = self.iter.row() {
            if !row.is_empty() {
                Arc::new(row[0].clone())
            } else {
                Arc::new(Value::Null(NullType::Null))
            }
        } else {
            Arc::new(Value::Null(NullType::Null))
        }
    }

    fn is_valid(&self) -> bool {
        self.iter.valid()
    }

    fn next(&mut self) {
        self.iter.next();
    }

    fn current_row(&self) -> Option<&Value> {
        // Iterator返回Row，我们返回行的第一个值
        self.iter.row().and_then(|row| row.first())
    }

    fn size(&self) -> usize {
        self.iter.size()
    }

    fn reset(&mut self) {
        self.iter.reset(0);
    }

    fn clear(&mut self) {
        self.iter.clear();
    }

    fn get_column(&self, col_name: &str) -> &Value {
        self.iter.get_column(col_name).unwrap_or(&Value::Null(NullType::Null))
    }

    fn get_column_by_index(&self, index: usize) -> &Value {
        self.iter.get_column_by_index(index as i32).unwrap_or(&Value::Null(NullType::Null))
    }
}

// SequentialIterator适配器
pub struct SequentialIterator {
    iter: Box<dyn Iterator>,
}

impl SequentialIterator {
    pub fn new(iter: Box<dyn Iterator>) -> Self {
        Self { iter }
    }
}

impl ResultIterator for SequentialIterator {
    fn value_ptr(&self) -> Arc<Value> {
        // 从迭代器获取当前行的第一个值作为默认值
        if let Some(row) = self.iter.row() {
            if !row.is_empty() {
                Arc::new(row[0].clone())
            } else {
                Arc::new(Value::Null(NullType::Null))
            }
        } else {
            Arc::new(Value::Null(NullType::Null))
        }
    }

    fn is_valid(&self) -> bool {
        self.iter.valid()
    }

    fn next(&mut self) {
        self.iter.next();
    }

    fn current_row(&self) -> Option<&Value> {
        // Iterator返回Row，我们返回行的第一个值
        self.iter.row().and_then(|row| row.first())
    }

    fn size(&self) -> usize {
        self.iter.size()
    }

    fn reset(&mut self) {
        self.iter.reset(0);
    }

    fn clear(&mut self) {
        self.iter.clear();
    }

    fn get_column(&self, col_name: &str) -> &Value {
        self.iter.get_column(col_name).unwrap_or(&Value::Null(NullType::Null))
    }

    fn get_column_by_index(&self, index: usize) -> &Value {
        self.iter.get_column_by_index(index as i32).unwrap_or(&Value::Null(NullType::Null))
    }
}

impl std::fmt::Debug for SequentialIterator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SequentialIterator")
            .field("iter", &self.iter)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_creation() {
        let value = Value::String("test_value".to_string());
        let result = Result::new(value.clone(), ResultState::Success);
        
        assert_eq!(result.value(), &value);
        assert_eq!(result.state(), &ResultState::Success);
    }

    #[test]
    fn test_result_builder() {
        let value = Value::Int(42);
        let result = ResultBuilder::new()
            .value(value.clone())
            .state(ResultState::Success)
            .msg("Test message".to_string())
            .build();
        
        assert_eq!(result.value(), &value);
        assert_eq!(result.state(), &ResultState::Success);
        assert_eq!(result.msg(), "Test message");
    }

    #[test]
    fn test_sequential_iterator() {
        let values = vec![Value::Int(1), Value::Int(2), Value::Int(3)];
        let mut iter = SequentialIterator::new(values.clone());
        
        assert_eq!(iter.size(), 3);
        assert_eq!(iter.current_row(), Some(&values[0]));
        
        iter.next();
        assert_eq!(iter.current_row(), Some(&values[1]));
        
        iter.next();
        assert_eq!(iter.current_row(), Some(&values[2]));
        
        iter.next();
        assert_eq!(iter.current_row(), None);
    }
}