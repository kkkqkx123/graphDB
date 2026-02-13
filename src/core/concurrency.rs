//! 优化的并发模型模块
//!
//! 提供适合单节点场景的并发原语和优化数据结构。
//! 在单查询执行模式下，使用更轻量的数据结构。
//!
//! # 策略
//!
//! - 单查询模式：使用 `RefCell` + `HashMap`
//! - 多查询模式：使用 `RwLock` + `HashMap`
//! - 跨查询共享：使用 `Arc<RwLock>`

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::cell::{RefCell, Ref, RefMut};

/// 单查询模式符号表
///
/// 使用 `RefCell` 提供内部可变性，避免不必要的锁开销。
/// 适用于单线程查询执行场景。
///
/// # 示例
///
/// ```ignore
/// let mut table = LocalSymbolTable::new();
/// table.insert("var1", value);
/// let value = table.get("var1");
/// ```
#[derive(Debug, Clone)]
pub struct LocalSymbolTable<K: std::hash::Hash + Eq, V> {
    symbols: RefCell<HashMap<K, V>>,
}

impl<K, V> LocalSymbolTable<K, V>
where
    K: std::hash::Hash + Eq + Clone + std::fmt::Debug,
    V: Clone + std::fmt::Debug,
{
    /// 创建新的本地符号表
    pub fn new() -> Self {
        Self {
            symbols: RefCell::new(HashMap::new()),
        }
    }

    /// 插入键值对
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        self.symbols.borrow_mut().insert(key, value)
    }

    /// 获取值（克隆）
    pub fn get(&self, key: &K) -> Option<V> {
        self.symbols.borrow().get(key).cloned()
    }

    /// 获取所有键值对
    pub fn entries(&self) -> Vec<(K, V)> {
        self.symbols.borrow().iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// 检查键是否存在
    pub fn contains_key(&self, key: &K) -> bool {
        self.symbols.borrow().contains_key(key)
    }

    /// 移除键
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.symbols.borrow_mut().remove(key)
    }

    /// 获取大小
    pub fn len(&self) -> usize {
        self.symbols.borrow().len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.symbols.borrow().is_empty()
    }

    /// 清空所有条目
    pub fn clear(&mut self) {
        self.symbols.borrow_mut().clear()
    }

    /// 获取所有键
    pub fn keys(&self) -> Vec<K> {
        self.symbols.borrow().keys().cloned().collect()
    }

    /// 获取所有值
    pub fn values(&self) -> Vec<V> {
        self.symbols.borrow().values().cloned().collect()
    }

    /// 批量插入
    pub fn extend(&mut self, other: &HashMap<K, V>) {
        self.symbols.borrow_mut().extend(other.clone());
    }

    /// 检查是否存在，不存在则插入
    pub fn get_or_insert(&self, key: K, default: V) -> V {
        let mut symbols = self.symbols.borrow_mut();
        symbols.entry(key).or_insert(default).clone()
    }
}

impl<K, V> Default for LocalSymbolTable<K, V>
where
    K: std::hash::Hash + Eq + Clone + std::fmt::Debug,
    V: Clone + std::fmt::Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

/// 多查询模式符号表
///
/// 使用 `RwLock` 提供线程安全，适合多查询并发执行场景。
/// 读写性能优于 DashMap。
///
/// # 示例
///
/// ```ignore
/// let table = SharedSymbolTable::new();
/// let value = table.get("var1");
/// table.insert("var2", value);
/// ```
#[derive(Debug, Clone)]
pub struct SharedSymbolTable<K: std::hash::Hash + Eq, V> {
    symbols: Arc<RwLock<HashMap<K, V>>>,
}

impl<K, V> SharedSymbolTable<K, V>
where
    K: std::hash::Hash + Eq + Clone + std::fmt::Debug + Send + 'static,
    V: Clone + std::fmt::Debug + Send + 'static,
{
    /// 创建新的共享符号表
    pub fn new() -> Self {
        Self {
            symbols: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 读取数据
    pub fn get(&self, key: &K) -> Result<Option<V>, String> {
        Ok(self.symbols.read()
            .map_err(|e| format!("Failed to acquire symbols read lock: {}", e))?
            .get(key)
            .cloned())
    }

    /// 写入数据
    pub fn insert(&self, key: K, value: V) -> Result<Option<V>, String> {
        Ok(self.symbols.write()
            .map_err(|e| format!("Failed to acquire symbols write lock: {}", e))?
            .insert(key, value))
    }

    /// 批量读取所有数据
    pub fn get_all(&self) -> Result<HashMap<K, V>, String> {
        Ok(self.symbols.read()
            .map_err(|e| format!("Failed to acquire symbols read lock: {}", e))?
            .clone())
    }

    /// 批量写入数据
    pub fn extend(&self, other: &HashMap<K, V>) -> Result<(), String> {
        let mut map = self.symbols.write()
            .map_err(|e| format!("Failed to acquire symbols write lock: {}", e))?;
        for (k, v) in other.iter() {
            map.insert(k.clone(), v.clone());
        }
        Ok(())
    }

    /// 检查是否存在
    pub fn contains_key(&self, key: &K) -> Result<bool, String> {
        Ok(self.symbols.read()
            .map_err(|e| format!("Failed to acquire symbols read lock: {}", e))?
            .contains_key(key))
    }

    /// 获取大小
    pub fn len(&self) -> Result<usize, String> {
        Ok(self.symbols.read()
            .map_err(|e| format!("Failed to acquire symbols read lock: {}", e))?
            .len())
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> Result<bool, String> {
        Ok(self.symbols.read()
            .map_err(|e| format!("Failed to acquire symbols read lock: {}", e))?
            .is_empty())
    }

    /// 清空
    pub fn clear(&self) -> Result<(), String> {
        self.symbols.write()
            .map_err(|e| format!("Failed to acquire symbols write lock: {}", e))?
            .clear();
        Ok(())
    }

    /// 移除键
    pub fn remove(&self, key: &K) -> Result<Option<V>, String> {
        Ok(self.symbols.write()
            .map_err(|e| format!("Failed to acquire symbols write lock: {}", e))?
            .remove(key))
    }
}

impl<K, V> Default for SharedSymbolTable<K, V>
where
    K: std::hash::Hash + Eq + Clone + std::fmt::Debug + Send + 'static,
    V: Clone + std::fmt::Debug + Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

/// 优化的请求上下文
///
/// 在单节点场景下，使用 `RefCell` 替代 `Arc<RwLock>`。
/// 仅在需要跨线程共享时才使用原子引用计数。
///
/// # 策略对比
///
/// | 场景 | 数据结构 | 开销 |
/// |------|----------|------|
/// | 单查询 | `RefCell<HashMap>` | 低 |
/// | 多查询 | `RwLock<HashMap>` | 中 |
/// | 跨线程 | `Arc<RwLock<HashMap>>` | 高 |
#[derive(Debug, Clone)]
pub struct OptimizedRequestContext<T: Clone + std::fmt::Debug> {
    data: RefCell<T>,
}

impl<T> OptimizedRequestContext<T>
where
    T: Clone + std::fmt::Debug,
{
    /// 创建新的请求上下文
    pub fn new(data: T) -> Self {
        Self {
            data: RefCell::new(data),
        }
    }

    /// 获取不可变引用
    pub fn get(&self) -> Ref<'_, T> {
        self.data.borrow()
    }

    /// 获取可变引用
    pub fn get_mut(&mut self) -> RefMut<'_, T> {
        self.data.borrow_mut()
    }

    /// 更新数据
    pub fn set(&self, data: T) {
        *self.data.borrow_mut() = data;
    }

    /// 克隆内部数据
    pub fn clone_data(&self) -> T {
        self.data.borrow().clone()
    }
}

impl<T> Default for OptimizedRequestContext<T>
where
    T: Clone + std::fmt::Debug + Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// 线程安全的请求上下文包装器
///
/// 当需要在多线程环境中共享请求上下文时使用。
#[derive(Debug, Clone)]
pub struct SharedRequestContext<T: Clone + std::fmt::Debug> {
    inner: Arc<RwLock<T>>,
}

impl<T> SharedRequestContext<T>
where
    T: Clone + std::fmt::Debug,
{
    /// 创建新的共享请求上下文
    pub fn new(data: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(data)),
        }
    }

    /// 从上下文克隆数据
    pub fn get(&self) -> Result<T, String> {
        Ok(self.inner.read()
            .map_err(|e| format!("Failed to acquire inner read lock: {}", e))?
            .clone())
    }

    /// 更新上下文数据
    pub fn set(&self, data: T) -> Result<(), String> {
        *self.inner.write()
            .map_err(|e| format!("Failed to acquire inner write lock: {}", e))? = data;
        Ok(())
    }

    /// 读取并修改数据
    pub fn update<F>(&self, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut T),
    {
        let mut data = self.inner.write()
            .map_err(|e| format!("Failed to acquire inner write lock: {}", e))?;
        f(&mut data);
        Ok(())
    }

    /// 检查是否满足条件
    pub fn check<F>(&self, f: F) -> Result<bool, String>
    where
        F: FnOnce(&T) -> bool,
    {
        let guard = self.inner.read()
            .map_err(|e| format!("Failed to acquire inner read lock: {}", e))?;
        Ok(f(&*guard))
    }
}

impl<T> Default for SharedRequestContext<T>
where
    T: Clone + std::fmt::Debug + Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// 并发模式枚举
///
/// 根据使用场景选择合适的并发策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConcurrencyMode {
    /// 单查询模式 - 使用 RefCell
    SingleQuery,
    /// 多查询模式 - 使用 RwLock
    MultiQuery,
    /// 跨线程共享模式 - 使用 Arc<RwLock>
    ThreadSafe,
}

impl Default for ConcurrencyMode {
    fn default() -> Self {
        ConcurrencyMode::SingleQuery
    }
}

/// 并发配置
///
/// 提供灵活的并发配置选项。
#[derive(Debug, Clone)]
pub struct ConcurrencyConfig {
    mode: ConcurrencyMode,
    max_readers: usize,
    write_batch_size: usize,
}

impl ConcurrencyConfig {
    /// 创建单查询配置
    pub fn single_query() -> Self {
        Self {
            mode: ConcurrencyMode::SingleQuery,
            max_readers: 1,
            write_batch_size: 100,
        }
    }

    /// 创建多查询配置
    pub fn multi_query() -> Self {
        Self {
            mode: ConcurrencyMode::MultiQuery,
            max_readers: 10,
            write_batch_size: 50,
        }
    }

    /// 创建线程安全配置
    pub fn thread_safe() -> Self {
        Self {
            mode: ConcurrencyMode::ThreadSafe,
            max_readers: 100,
            write_batch_size: 10,
        }
    }

    /// 获取当前模式
    pub fn mode(&self) -> ConcurrencyMode {
        self.mode
    }

    /// 获取最大读者数
    pub fn max_readers(&self) -> usize {
        self.max_readers
    }

    /// 获取写入批次大小
    pub fn write_batch_size(&self) -> usize {
        self.write_batch_size
    }
}

impl Default for ConcurrencyConfig {
    fn default() -> Self {
        Self::single_query()
    }
}
