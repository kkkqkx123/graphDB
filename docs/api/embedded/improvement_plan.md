# GraphDB 嵌入式 API 改进方案

## 概述

本文档基于与 SQLite API 的对比分析，提供详细的改进方案。按优先级分为高、中、低三个级别。

---

## 高优先级改进

### 1. 忙等待/超时机制

#### 问题描述
当前实现缺少多线程环境下的并发控制机制。当多个线程同时访问数据库时，可能导致冲突或死锁。

#### 解决方案

**1.1 添加忙等待超时配置**

修改 `DatabaseConfig`：

```rust
// src/api/embedded/config.rs
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    // ... 现有字段
    /// 忙等待超时（毫秒），0 表示不等待
    pub busy_timeout_ms: u32,
}

impl DatabaseConfig {
    pub fn with_busy_timeout(mut self, timeout_ms: u32) -> Self {
        self.busy_timeout_ms = timeout_ms;
        self
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            // ... 现有默认值
            busy_timeout_ms: 5000,  // 默认 5 秒
        }
    }
}
```

**1.2 实现忙等待处理器**

```rust
// src/api/embedded/busy_handler.rs
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

/// 忙等待处理器
pub struct BusyHandler {
    timeout_ms: u32,
    retry_count: AtomicU32,
}

impl BusyHandler {
    pub fn new(timeout_ms: u32) -> Self {
        Self {
            timeout_ms,
            retry_count: AtomicU32::new(0),
        }
    }

    /// 处理忙状态
    /// 返回 true 表示继续等待，false 表示放弃
    pub fn handle_busy(&self) -> bool {
        let count = self.retry_count.fetch_add(1, Ordering::SeqCst);
        
        if self.timeout_ms == 0 {
            return false;  // 不等待
        }
        
        let wait_time = Self::calculate_wait_time(count);
        let total_wait = wait_time * (count as u64 + 1);
        
        if total_wait > self.timeout_ms as u64 {
            return false;  // 超时
        }
        
        std::thread::sleep(Duration::from_millis(wait_time));
        true
    }

    /// 计算等待时间（指数退避）
    fn calculate_wait_time(retry_count: u32) -> u64 {
        let base = 1u64;
        let max = 100u64;  // 最大 100ms
        std::cmp::min(base << retry_count, max)
    }

    pub fn reset(&self) {
        self.retry_count.store(0, Ordering::SeqCst);
    }
}
```

**1.3 C API 接口**

```rust
// src/api/embedded/c_api/session.rs

/// 设置忙等待超时
#[no_mangle]
pub extern "C" fn graphdb_busy_timeout(
    session: *mut graphdb_session_t,
    timeout_ms: c_int,
) -> c_int {
    if session.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let handle = &mut *(session as *mut GraphDbSessionHandle);
        // 存储超时设置
        handle.busy_timeout_ms = timeout_ms.max(0) as u32;
        graphdb_error_code_t::GRAPHDB_OK as c_int
    }
}

/// 获取忙等待超时
#[no_mangle]
pub extern "C" fn graphdb_busy_timeout_get(
    session: *mut graphdb_session_t,
) -> c_int {
    if session.is_null() {
        return 0;
    }

    unsafe {
        let handle = &*(session as *mut GraphDbSessionHandle);
        handle.busy_timeout_ms as c_int
    }
}
```

**1.4 在存储层集成忙等待**

```rust
// src/storage/mod.rs 或相关文件

pub trait StorageClient {
    // ... 现有方法
    
    /// 尝试获取锁，支持忙等待
    fn try_lock_with_timeout(&self, timeout_ms: u32) -> Result<LockGuard, StorageError>;
}
```

---

### 2. 变更统计功能

#### 问题描述
当前无法获取查询影响的行数、最后插入的 ID 等统计信息，不利于应用程序进行后续处理。

#### 解决方案

**2.1 添加变更统计结构体**

```rust
// src/api/embedded/statistics.rs

/// 会话级变更统计
#[derive(Debug, Default, Clone)]
pub struct SessionStatistics {
    /// 上次操作影响的行数
    pub last_changes: usize,
    /// 总会话变更数
    pub total_changes: u64,
    /// 最后插入的顶点 ID
    pub last_insert_vertex_id: Option<i64>,
    /// 最后插入的边 ID
    pub last_insert_edge_id: Option<i64>,
}

impl SessionStatistics {
    pub fn record_changes(&mut self, count: usize) {
        self.last_changes = count;
        self.total_changes += count as u64;
    }

    pub fn record_vertex_insert(&mut self, id: i64) {
        self.last_insert_vertex_id = Some(id);
        self.last_changes = 1;
        self.total_changes += 1;
    }

    pub fn record_edge_insert(&mut self, id: i64) {
        self.last_insert_edge_id = Some(id);
        self.last_changes = 1;
        self.total_changes += 1;
    }

    pub fn reset_last(&mut self) {
        self.last_changes = 0;
        self.last_insert_vertex_id = None;
        self.last_insert_edge_id = None;
    }
}
```

**2.2 在 Session 中集成统计**

```rust
// src/api/embedded/session.rs

pub struct Session<S: StorageClient + Clone + 'static> {
    // ... 现有字段
    statistics: Mutex<SessionStatistics>,
}

impl<S: StorageClient + Clone + 'static> Session<S> {
    // ... 现有方法

    /// 获取上次操作影响的行数
    pub fn changes(&self) -> usize {
        self.statistics.lock().last_changes
    }

    /// 获取总会话变更数
    pub fn total_changes(&self) -> u64 {
        self.statistics.lock().total_changes
    }

    /// 获取最后插入的顶点 ID
    pub fn last_insert_vertex_id(&self) -> Option<i64> {
        self.statistics.lock().last_insert_vertex_id
    }

    /// 获取最后插入的边 ID
    pub fn last_insert_edge_id(&self) -> Option<i64> {
        self.statistics.lock().last_insert_edge_id
    }
}
```

**2.3 C API 接口**

```rust
// src/api/embedded/c_api/statistics.rs（新建文件）

/// 获取上次操作影响的行数
#[no_mangle]
pub extern "C" fn graphdb_changes(session: *mut graphdb_session_t) -> c_int {
    if session.is_null() {
        return 0;
    }

    unsafe {
        let handle = &*(session as *mut GraphDbSessionHandle);
        handle.inner.changes() as c_int
    }
}

/// 获取总会话变更数
#[no_mangle]
pub extern "C" fn graphdb_total_changes(session: *mut graphdb_session_t) -> i64 {
    if session.is_null() {
        return 0;
    }

    unsafe {
        let handle = &*(session as *mut GraphDbSessionHandle);
        handle.inner.total_changes() as i64
    }
}

/// 获取最后插入的顶点 ID
#[no_mangle]
pub extern "C" fn graphdb_last_insert_vertex_id(
    session: *mut graphdb_session_t,
) -> i64 {
    if session.is_null() {
        return -1;
    }

    unsafe {
        let handle = &*(session as *mut GraphDbSessionHandle);
        handle.inner.last_insert_vertex_id().unwrap_or(-1)
    }
}

/// 获取最后插入的边 ID
#[no_mangle]
pub extern "C" fn graphdb_last_insert_edge_id(
    session: *mut graphdb_session_t,
) -> i64 {
    if session.is_null() {
        return -1;
    }

    unsafe {
        let handle = &*(session as *mut GraphDbSessionHandle);
        handle.inner.last_insert_edge_id().unwrap_or(-1)
    }
}
```

**2.4 在查询执行中更新统计**

```rust
// 在 QueryApi 或 Session 的 execute 方法中

pub fn execute(&self, query: &str) -> CoreResult<QueryResult> {
    // ... 现有代码
    
    let result = {
        let mut query_api = self.db.query_api.lock();
        query_api.execute(query, ctx)?
    };
    
    // 更新统计信息
    if let Some(ref metadata) = result.metadata {
        self.statistics.lock().record_changes(metadata.rows_affected);
    }
    
    Ok(QueryResult::from_core(result))
}
```

---

### 3. 二进制数据(BLOB)支持

#### 问题描述
当前 C API 只支持基本类型（null, bool, int, float, string），缺少二进制数据支持。

#### 解决方案

**3.1 扩充值类型枚举**

```rust
// src/api/embedded/c_api/types.rs

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum graphdb_value_type_t {
    // ... 现有类型
    /// 二进制数据
    GRAPHDB_BLOB = 10,
}

/// 二进制数据结构
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct graphdb_blob_t {
    /// 数据指针
    pub data: *const u8,
    /// 数据长度
    pub len: usize,
    /// 是否需要释放（由 GraphDB 分配）
    pub owned: bool,
}

/// 扩展值数据联合体
#[repr(C)]
#[derive(Clone, Copy)]
pub union graphdb_value_data_t {
    // ... 现有字段
    /// 二进制数据
    pub blob: graphdb_blob_t,
}
```

**3.2 添加 BLOB 绑定函数**

```rust
// src/api/embedded/c_api/statement.rs

/// 绑定二进制数据（按索引）
#[no_mangle]
pub extern "C" fn graphdb_bind_blob(
    stmt: *mut graphdb_stmt_t,
    index: c_int,
    data: *const u8,
    len: c_int,
    destructor: Option<extern "C" fn(*mut c_void)>,
) -> c_int {
    if stmt.is_null() || index < 1 || data.is_null() || len < 0 {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let blob_data = unsafe {
        std::slice::from_raw_parts(data, len as usize).to_vec()
    };

    unsafe {
        let handle = &mut *(stmt as *mut GraphDbStmtHandle);
        let param_name = format!("param_{}", index - 1);

        // 存储 destructor 以便后续释放
        if let Some(dtor) = destructor {
            // 需要存储在 handle 中，在 finalize 时调用
            handle.blob_destructors.insert(index as u32, dtor);
            handle.blob_pointers.insert(index as u32, data as *mut c_void);
        }

        match handle.inner.bind(&param_name, Value::Blob(blob_data)) {
            Ok(_) => graphdb_error_code_t::GRAPHDB_OK as c_int,
            Err(e) => {
                let error_code = error_code_from_core_error(&e);
                error_code
            }
        }
    }
}
```

**3.3 添加 BLOB 获取函数**

```rust
// src/api/embedded/c_api/result.rs

/// 获取二进制数据
#[no_mangle]
pub extern "C" fn graphdb_get_blob(
    result: *mut graphdb_result_t,
    row: c_int,
    col: *const c_char,
    len: *mut c_int,
) -> *const u8 {
    if result.is_null() || col.is_null() || len.is_null() {
        if !len.is_null() {
            unsafe { *len = -1; }
        }
        return ptr::null();
    }

    let col_str = unsafe {
        match CStr::from_ptr(col).to_str() {
            Ok(s) => s,
            Err(_) => {
                unsafe { *len = -1; }
                return ptr::null();
            }
        }
    };

    unsafe {
        let handle = &*(result as *mut GraphDbResultHandle);
        
        match handle.inner.get(row as usize) {
            Some(row_data) => {
                match row_data.get(col_str) {
                    Some(crate::core::Value::Blob(blob)) => {
                        *len = blob.len() as c_int;
                        // 注意：返回的指针生命周期与 result 绑定
                        blob.as_ptr()
                    }
                    Some(_) => {
                        *len = -1;
                        ptr::null()
                    }
                    None => {
                        *len = -1;
                        ptr::null()
                    }
                }
            }
            None => {
                *len = -1;
                ptr::null()
            }
        }
    }
}
```

**3.4 在核心 Value 类型中添加 Blob**

```rust
// src/core/value/mod.rs

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    // ... 现有类型
    /// 二进制数据
    Blob(Vec<u8>),
}
```

---

### 4. 按列索引访问结果集

#### 问题描述
当前只能通过列名访问结果，某些场景下按索引访问更高效。

#### 解决方案

**4.1 添加按索引访问的 API**

```rust
// src/api/embedded/c_api/result.rs

/// 获取整数值（按列索引）
#[no_mangle]
pub extern "C" fn graphdb_get_int_by_index(
    result: *mut graphdb_result_t,
    row: c_int,
    col: c_int,
    value: *mut i64,
) -> c_int {
    if result.is_null() || value.is_null() || col < 0 {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let handle = &*(result as *mut GraphDbResultHandle);
        
        // 获取列名
        let columns = handle.inner.columns();
        let col_name = match columns.get(col as usize) {
            Some(name) => name.as_str(),
            None => return graphdb_error_code_t::GRAPHDB_NOTFOUND as c_int,
        };
        
        match handle.inner.get(row as usize) {
            Some(row_data) => {
                match row_data.get(col_name) {
                    Some(crate::core::Value::Int(i)) => {
                        *value = *i;
                        graphdb_error_code_t::GRAPHDB_OK as c_int
                    }
                    Some(_) => graphdb_error_code_t::GRAPHDB_MISMATCH as c_int,
                    None => graphdb_error_code_t::GRAPHDB_NOTFOUND as c_int,
                }
            }
            None => graphdb_error_code_t::GRAPHDB_NOTFOUND as c_int,
        }
    }
}

/// 获取字符串值（按列索引）
#[no_mangle]
pub extern "C" fn graphdb_get_string_by_index(
    result: *mut graphdb_result_t,
    row: c_int,
    col: c_int,
    len: *mut c_int,
) -> *const c_char {
    if result.is_null() || col < 0 {
        if !len.is_null() {
            unsafe { *len = -1; }
        }
        return ptr::null();
    }

    unsafe {
        let handle = &*(result as *mut GraphDbResultHandle);
        
        let columns = handle.inner.columns();
        let col_name = match columns.get(col as usize) {
            Some(name) => name.as_str(),
            None => {
                if !len.is_null() {
                    *len = -1;
                }
                return ptr::null();
            }
        };
        
        match handle.inner.get(row as usize) {
            Some(row_data) => {
                match row_data.get(col_name) {
                    Some(crate::core::Value::String(s)) => {
                        if !len.is_null() {
                            *len = s.len() as c_int;
                        }
                        match CString::new(s.as_str()) {
                            Ok(c_str) => c_str.into_raw(),
                            Err(_) => ptr::null(),
                        }
                    }
                    Some(_) => {
                        if !len.is_null() {
                            *len = -1;
                        }
                        ptr::null()
                    }
                    None => {
                        if !len.is_null() {
                            *len = -1;
                        }
                        ptr::null()
                    }
                }
            }
            None => {
                if !len.is_null() {
                    *len = -1;
                }
                ptr::null()
            }
        }
    }
}

/// 获取列类型
#[no_mangle]
pub extern "C" fn graphdb_column_type(
    result: *mut graphdb_result_t,
    col: c_int,
) -> graphdb_value_type_t {
    if result.is_null() || col < 0 {
        return graphdb_value_type_t::GRAPHDB_NULL;
    }

    unsafe {
        let handle = &*(result as *mut GraphDbResultHandle);
        
        // 获取第一行来确定类型
        match handle.inner.first() {
            Some(row) => {
                let columns = handle.inner.columns();
                let col_name = match columns.get(col as usize) {
                    Some(name) => name.as_str(),
                    None => return graphdb_value_type_t::GRAPHDB_NULL,
                };
                
                match row.get(col_name) {
                    Some(value) => match value {
                        crate::core::Value::Null(_) => graphdb_value_type_t::GRAPHDB_NULL,
                        crate::core::Value::Bool(_) => graphdb_value_type_t::GRAPHDB_BOOL,
                        crate::core::Value::Int(_) => graphdb_value_type_t::GRAPHDB_INT,
                        crate::core::Value::Float(_) => graphdb_value_type_t::GRAPHDB_FLOAT,
                        crate::core::Value::String(_) => graphdb_value_type_t::GRAPHDB_STRING,
                        crate::core::Value::Blob(_) => graphdb_value_type_t::GRAPHDB_BLOB,
                        crate::core::Value::List(_) => graphdb_value_type_t::GRAPHDB_LIST,
                        crate::core::Value::Map(_) => graphdb_value_type_t::GRAPHDB_MAP,
                        crate::core::Value::Vertex(_) => graphdb_value_type_t::GRAPHDB_VERTEX,
                        crate::core::Value::Edge(_) => graphdb_value_type_t::GRAPHDB_EDGE,
                        crate::core::Value::Path(_) => graphdb_value_type_t::GRAPHDB_PATH,
                        _ => graphdb_value_type_t::GRAPHDB_NULL,
                    }
                    None => graphdb_value_type_t::GRAPHDB_NULL,
                }
            }
            None => graphdb_value_type_t::GRAPHDB_NULL,
        }
    }
}
```

---

### 5. 数据库打开选项

#### 问题描述
当前只支持基本的打开方式，缺少只读模式、创建标志等选项。

#### 解决方案

**5.1 添加打开标志**

```rust
// src/api/embedded/c_api/types.rs

/// 数据库打开标志
pub const GRAPHDB_OPEN_READONLY: c_int = 0x00000001;
pub const GRAPHDB_OPEN_READWRITE: c_int = 0x00000002;
pub const GRAPHDB_OPEN_CREATE: c_int = 0x00000004;
pub const GRAPHDB_OPEN_NOMUTEX: c_int = 0x00008000;
pub const GRAPHDB_OPEN_FULLMUTEX: c_int = 0x00010000;
pub const GRAPHDB_OPEN_SHAREDCACHE: c_int = 0x00020000;
pub const GRAPHDB_OPEN_PRIVATECACHE: c_int = 0x00040000;
```

**5.2 实现 graphdb_open_v2**

```rust
// src/api/embedded/c_api/database.rs

/// 使用标志打开数据库
#[no_mangle]
pub extern "C" fn graphdb_open_v2(
    path: *const c_char,
    db: *mut *mut graphdb_t,
    flags: c_int,
    vfs: *const c_char,  // 保留参数，当前未使用
) -> c_int {
    if path.is_null() || db.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let path_str = unsafe {
        match CStr::from_ptr(path).to_str() {
            Ok(s) => s,
            Err(_) => return graphdb_error_code_t::GRAPHDB_MISUSE as c_int,
        }
    };

    // 解析标志
    let read_only = (flags & GRAPHDB_OPEN_READONLY) != 0;
    let read_write = (flags & GRAPHDB_OPEN_READWRITE) != 0;
    let create = (flags & GRAPHDB_OPEN_CREATE) != 0;
    
    // 验证标志组合
    if read_only && read_write {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }
    
    if !read_only && !read_write {
        // 默认读写模式
    }

    let mut config = if read_only {
        DatabaseConfig::file(path_str).with_read_only(true)
    } else {
        DatabaseConfig::file(path_str)
    };
    
    if create {
        config = config.with_create_if_missing(true);
    }

    // 打开数据库
    match GraphDatabase::open_with_config(config) {
        Ok(graphdb) => {
            let handle = Box::new(GraphDbHandle {
                inner: Arc::new(graphdb),
                last_error: None,
            });
            unsafe {
                *db = Box::into_raw(handle) as *mut graphdb_t;
            }
            graphdb_error_code_t::GRAPHDB_OK as c_int
        }
        Err(e) => {
            let error_code = error_code_from_core_error(&e);
            unsafe {
                *db = ptr::null_mut();
            }
            error_code
        }
    }
}
```

**5.3 更新 DatabaseConfig**

```rust
// src/api/embedded/config.rs

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    // ... 现有字段
    /// 是否只读
    pub read_only: bool,
    /// 不存在时是否创建
    pub create_if_missing: bool,
}

impl DatabaseConfig {
    pub fn with_read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn with_create_if_missing(mut self, create: bool) -> Self {
        self.create_if_missing = create;
        self
    }
}
```

---

## 中优先级改进

### 6. SQL 错误位置

#### 解决方案

```rust
// src/api/embedded/c_api/error.rs

/// 获取 SQL 错误位置（字符偏移量）
#[no_mangle]
pub extern "C" fn graphdb_error_offset(session: *mut graphdb_session_t) -> c_int {
    if session.is_null() {
        return -1;
    }

    unsafe {
        let handle = &*(session as *mut GraphDbSessionHandle);
        handle.last_error_offset.unwrap_or(-1) as c_int
    }
}

// 在 GraphDbSessionHandle 中添加
pub struct GraphDbSessionHandle {
    // ... 现有字段
    pub last_error_offset: Option<usize>,
}
```

需要在 parser 层支持错误位置信息的传递。

---

### 7. 数据库备份 API

#### 解决方案

```rust
// src/api/embedded/c_api/backup.rs（新建文件）

/// 备份句柄
pub struct GraphDbBackupHandle {
    source: Arc<GraphDatabase<RedbStorage>>,
    destination: Arc<GraphDatabase<RedbStorage>>,
    progress: AtomicU64,
}

/// 初始化备份
#[no_mangle]
pub extern "C" fn graphdb_backup_init(
    dest_db: *mut graphdb_t,
    src_db: *mut graphdb_t,
) -> *mut graphdb_backup_t {
    if dest_db.is_null() || src_db.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let src_handle = &*(src_db as *mut GraphDbHandle);
        let dest_handle = &*(dest_db as *mut GraphDbHandle);

        let backup_handle = Box::new(GraphDbBackupHandle {
            source: src_handle.inner.clone(),
            destination: dest_handle.inner.clone(),
            progress: AtomicU64::new(0),
        });

        Box::into_raw(backup_handle) as *mut graphdb_backup_t
    }
}

/// 执行备份步骤
#[no_mangle]
pub extern "C" fn graphdb_backup_step(
    backup: *mut graphdb_backup_t,
    pages: c_int,
) -> c_int {
    if backup.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    // 实现备份逻辑
    // 返回 GRAPHDB_OK, GRAPHDB_BUSY, GRAPHDB_DONE 等
    graphdb_error_code_t::GRAPHDB_OK as c_int
}

/// 完成备份
#[no_mangle]
pub extern "C" fn graphdb_backup_finish(backup: *mut graphdb_backup_t) -> c_int {
    if backup.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let _ = Box::from_raw(backup as *mut GraphDbBackupHandle);
    }

    graphdb_error_code_t::GRAPHDB_OK as c_int
}

/// 获取备份剩余页数
#[no_mangle]
pub extern "C" fn graphdb_backup_remaining(backup: *mut graphdb_backup_t) -> c_int {
    if backup.is_null() {
        return -1;
    }

    // 返回剩余页数
    0
}

/// 获取备份总页数
#[no_mangle]
pub extern "C" fn graphdb_backup_pagecount(backup: *mut graphdb_backup_t) -> c_int {
    if backup.is_null() {
        return -1;
    }

    // 返回总页数
    0
}
```

---

### 8. 扩展错误码

#### 解决方案

```rust
// src/api/embedded/c_api/error.rs

/// 扩展错误码
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum graphdb_extended_error_code_t {
    // 基础错误码（0-21）
    // ...
    
    // 扩展错误码（1000+）
    GRAPHDB_ERROR_SYNTAX = 1000,        // 语法错误
    GRAPHDB_ERROR_SEMANTIC = 1001,      // 语义错误
    GRAPHDB_ERROR_TYPE_MISMATCH = 1002, // 类型不匹配
    GRAPHDB_ERROR_DIVISION_BY_ZERO = 1003, // 除零错误
    GRAPHDB_ERROR_OUT_OF_RANGE = 1004,  // 超出范围
    GRAPHDB_ERROR_DUPLICATE_KEY = 1005, // 重复键
    GRAPHDB_ERROR_FOREIGN_KEY = 1006,   // 外键约束失败
    GRAPHDB_ERROR_NOT_NULL = 1007,      // 非空约束失败
    GRAPHDB_ERROR_UNIQUE = 1008,        // 唯一约束失败
    GRAPHDB_ERROR_CHECK = 1009,         // CHECK 约束失败
    GRAPHDB_ERROR_CONNECTION_LOST = 1010, // 连接丢失
    GRAPHDB_ERROR_DEADLOCK = 1011,      // 死锁
    GRAPHDB_ERROR_LOCK_TIMEOUT = 1012,  // 锁超时
}

/// 获取扩展错误码
#[no_mangle]
pub extern "C" fn graphdb_extended_errcode(session: *mut graphdb_session_t) -> c_int {
    if session.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let handle = &*(session as *mut GraphDbSessionHandle);
        handle.last_extended_error.map(|e| e as c_int).unwrap_or(0)
    }
}
```

---

## 低优先级改进

### 9. 自定义函数（可选）

```rust
// 注册自定义标量函数
int graphdb_create_function(
    graphdb_session_t* session,
    const char* name,
    int argc,
    void* user_data,
    void (*xFunc)(graphdb_context*, int, graphdb_value**),
    void (*xDestroy)(void*)
);

// 注册自定义聚合函数
int graphdb_create_aggregate(
    graphdb_session_t* session,
    const char* name,
    int argc,
    void* user_data,
    void (*xStep)(graphdb_context*, int, graphdb_value**),
    void (*xFinal)(graphdb_context*),
    void (*xDestroy)(void*)
);
```

### 10. 钩子机制（可选）

```rust
// 提交钩子
void* graphdb_commit_hook(
    graphdb_t* db,
    int (*callback)(void*),
    void* user_data
);

// 回滚钩子
void* graphdb_rollback_hook(
    graphdb_t* db,
    void (*callback)(void*),
    void* user_data
);

// 更新钩子
void* graphdb_update_hook(
    graphdb_t* db,
    void (*callback)(void*, int, const char*, const char*, int64_t),
    void* user_data
);
```

### 11. SQL 追踪（可选）

```rust
// 追踪回调类型
typedef void (*graphdb_trace_callback)(const char* sql, void* user_data);

// 设置追踪回调
int graphdb_trace(
    graphdb_session_t* session,
    graphdb_trace_callback callback,
    void* user_data
);
```

---

## 实施建议

### 阶段一：高优先级（1-2 周）

1. **忙等待/超时机制**
   - 实现 BusyHandler
   - 添加 C API 接口
   - 在存储层集成

2. **变更统计**
   - 添加 SessionStatistics
   - 在查询执行中更新统计
   - 添加 C API 接口

3. **二进制数据支持**
   - 扩展 Value 类型
   - 添加 bind_blob/get_blob

### 阶段二：中优先级（2-3 周）

4. **按列索引访问**
   - 添加索引访问函数
   - 添加类型获取函数

5. **数据库打开选项**
   - 实现 graphdb_open_v2
   - 添加标志定义

6. **SQL 错误位置**
   - 需要 parser 支持

### 阶段三：低优先级（后续版本）

7. 数据库备份 API
8. 扩展错误码
9. 自定义函数
10. 钩子机制
11. SQL 追踪

---

## 兼容性说明

- 所有新增 API 遵循现有命名约定
- 保持与现有代码的向后兼容
- 新增功能通过条件编译标志控制（可选）
