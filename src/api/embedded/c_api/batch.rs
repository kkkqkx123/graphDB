//! C API 批量操作模块
//!
//! 提供批量操作功能，支持批量插入、批量更新和批量删除

use crate::api::embedded::c_api::error::{graphdb_error_code_t, set_last_error_message};
use crate::api::embedded::c_api::session::GraphDbSessionHandle;
use crate::api::embedded::c_api::types::graphdb_value_type_t;
use crate::api::embedded::c_api::types::{graphdb_batch_t, graphdb_session_t, graphdb_value_t};
use crate::core::{Edge, Value, Vertex};
use crate::storage::StorageClient;
use std::collections::HashMap;
use std::ffi::{c_char, c_int, CStr, CString};

/// 批量操作项类型
enum BatchItem {
    Vertex(Vertex),
    Edge(Edge),
}

/// 批量操作句柄内部结构
///
/// 注意：此结构体持有会话指针，但不拥有会话的所有权。
/// 调用者必须确保在批量操作句柄被释放之前不关闭会话。
pub struct GraphDbBatchHandle {
    /// 关联的会话指针（用于验证会话有效性）
    session_ptr: *mut GraphDbSessionHandle,
    /// 批次大小
    batch_size: usize,
    /// 缓冲区
    buffer: Vec<BatchItem>,
    /// 已插入的顶点数量
    vertices_inserted: usize,
    /// 已插入的边数量
    edges_inserted: usize,
    /// 错误列表
    errors: Vec<String>,
    /// 最后错误
    pub(crate) last_error: Option<CString>,
}

impl GraphDbBatchHandle {
    /// 检查会话是否仍然有效
    fn is_session_valid(&self) -> bool {
        !self.session_ptr.is_null()
    }

    /// 获取会话引用（如果有效）
    fn get_session(&self) -> Option<&GraphDbSessionHandle> {
        if self.is_session_valid() {
            Some(unsafe { &*self.session_ptr })
        } else {
            None
        }
    }

    /// 刷新顶点缓冲区
    fn flush_vertices(&mut self) -> Result<(), String> {
        // 分离顶点和边
        let mut vertices = Vec::new();
        let mut remaining = Vec::new();

        for item in self.buffer.drain(..) {
            match item {
                BatchItem::Vertex(v) => vertices.push(v),
                _ => remaining.push(item),
            }
        }

        // 将边放回缓冲区
        self.buffer.extend(remaining);

        if vertices.is_empty() {
            return Ok(());
        }

        let vertex_count = vertices.len();

        // 将会话相关操作放在独立作用域中，避免借用冲突
        let result = {
            let session = self
                .get_session()
                .ok_or_else(|| "会话无效或已关闭".to_string())?;

            let space_name = session
                .inner
                .space_name()
                .ok_or_else(|| "未选择图空间".to_string())?;

            // 调用存储层的批量插入接口
            let mut storage = session.inner.storage();
            storage.batch_insert_vertices(space_name, vertices)
        };

        match result {
            Ok(_) => {
                self.vertices_inserted += vertex_count;
                Ok(())
            }
            Err(e) => {
                let err_msg = format!("批量插入顶点失败: {}", e);
                self.errors.push(err_msg.clone());
                Err(err_msg)
            }
        }
    }

    /// 刷新边缓冲区
    fn flush_edges(&mut self) -> Result<(), String> {
        // 分离边和顶点
        let mut edges = Vec::new();
        let mut remaining = Vec::new();

        for item in self.buffer.drain(..) {
            match item {
                BatchItem::Edge(e) => edges.push(e),
                _ => remaining.push(item),
            }
        }

        // 将顶点放回缓冲区
        self.buffer.extend(remaining);

        if edges.is_empty() {
            return Ok(());
        }

        let edge_count = edges.len();

        // 将会话相关操作放在独立作用域中，避免借用冲突
        let result = {
            let session = self
                .get_session()
                .ok_or_else(|| "会话无效或已关闭".to_string())?;

            let space_name = session
                .inner
                .space_name()
                .ok_or_else(|| "未选择图空间".to_string())?;

            // 调用存储层的批量插入接口
            let mut storage = session.inner.storage();
            storage.batch_insert_edges(space_name, edges)
        };

        match result {
            Ok(_) => {
                self.edges_inserted += edge_count;
                Ok(())
            }
            Err(e) => {
                let err_msg = format!("批量插入边失败: {}", e);
                self.errors.push(err_msg.clone());
                Err(err_msg)
            }
        }
    }

    /// 执行批量插入，刷新所有缓冲的数据
    fn execute(&mut self) -> Result<(), String> {
        // 先刷新顶点
        self.flush_vertices()?;

        // 再刷新边
        self.flush_edges()?;

        Ok(())
    }
}

/// 创建批量插入器
///
/// # 参数
/// - `session`: 会话句柄
/// - `batch_size`: 批次大小
/// - `batch`: 输出参数，批量操作句柄
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
///
/// # Safety
/// - `session` must be a valid session handle created by `graphdb_session_create`
/// - `batch_size` must be a positive integer (if <= 0, defaults to 100)
/// - `batch` must be a valid pointer to store the batch handle
/// - The created batch handle holds a session pointer but does not own the session
/// - The caller must ensure the session is not closed before the batch handle is freed
/// - The caller is responsible for freeing the batch handle using `graphdb_batch_inserter_free` when done
#[no_mangle]
pub unsafe extern "C" fn graphdb_batch_inserter_create(
    session: *mut graphdb_session_t,
    batch_size: c_int,
    batch: *mut *mut graphdb_batch_t,
) -> c_int {
    if session.is_null() || batch.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let size = if batch_size <= 0 {
        100
    } else {
        batch_size as usize
    };

    unsafe {
        let batch_handle = Box::new(GraphDbBatchHandle {
            session_ptr: session as *mut GraphDbSessionHandle,
            batch_size: size,
            buffer: Vec::with_capacity(size),
            vertices_inserted: 0,
            edges_inserted: 0,
            errors: Vec::new(),
            last_error: None,
        });
        *batch = Box::into_raw(batch_handle) as *mut graphdb_batch_t;
        graphdb_error_code_t::GRAPHDB_OK as c_int
    }
}

/// 添加顶点
///
/// # 参数
/// - `batch`: 批量操作句柄
/// - `vid`: 顶点 ID
/// - `tag_name`: 标签名称（UTF-8 编码）
/// - `properties`: 属性数组
/// - `prop_count`: 属性数量
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
///
/// # Safety
/// - `batch` 必须是通过 `graphdb_batch_inserter_create` 创建的有效批量操作句柄
/// - `tag_name` 必须是指向以 null 结尾的 UTF-8 字符串的有效指针
/// - 如果 `properties` 不为 null,则必须指向至少 `prop_count` 个有效的 `graphdb_value_t` 元素
/// - 调用者必须确保在调用此函数时,关联的会话仍然有效
#[no_mangle]
pub unsafe extern "C" fn graphdb_batch_add_vertex(
    batch: *mut graphdb_batch_t,
    vid: i64,
    tag_name: *const c_char,
    properties: *const graphdb_value_t,
    prop_count: usize,
) -> c_int {
    if batch.is_null() || tag_name.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let tag_str = unsafe {
        match CStr::from_ptr(tag_name).to_str() {
            Ok(s) => s,
            Err(_) => return graphdb_error_code_t::GRAPHDB_MISUSE as c_int,
        }
    };

    let mut props = HashMap::new();

    if !properties.is_null() && prop_count > 0 {
        for i in 0..prop_count {
            unsafe {
                let prop = &*properties.add(i);
                let prop_name = format!("prop_{}", i);
                let value = convert_c_value_to_rust(prop);
                props.insert(prop_name, value);
            }
        }
    }

    unsafe {
        let handle = &mut *(batch as *mut GraphDbBatchHandle);

        // 检查会话有效性
        if !handle.is_session_valid() {
            return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
        }

        let mut vertex = Vertex::with_vid(Value::Int(vid));
        let tag = crate::core::vertex_edge_path::Tag::new(tag_str.to_string(), props);
        vertex.add_tag(tag);

        handle.buffer.push(BatchItem::Vertex(vertex));

        // 如果达到批次大小，自动刷新
        if handle.buffer.len() >= handle.batch_size {
            if let Err(e) = handle.flush_vertices() {
                let error_msg = e.to_string();
                set_last_error_message(error_msg.clone());
                handle.last_error = Some(CString::new(error_msg).unwrap_or_default());
                return graphdb_error_code_t::GRAPHDB_ERROR as c_int;
            }
        }

        graphdb_error_code_t::GRAPHDB_OK as c_int
    }
}

/// 添加边
///
/// # 参数
/// - `batch`: 批量操作句柄
/// - `src_vid`: 源顶点 ID
/// - `dst_vid`: 目标顶点 ID
/// - `edge_type`: 边类型名称（UTF-8 编码）
/// - `rank`: 排名
/// - `properties`: 属性数组
/// - `prop_count`: 属性数量
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
///
/// # Safety
/// - `batch` 必须是通过 `graphdb_batch_inserter_create` 创建的有效批量操作句柄
/// - `edge_type` 必须是指向以 null 结尾的 UTF-8 字符串的有效指针
/// - 如果 `properties` 不为 null,则必须指向至少 `prop_count` 个有效的 `graphdb_value_t` 元素
/// - 调用者必须确保在调用此函数时,关联的会话仍然有效
#[no_mangle]
pub unsafe extern "C" fn graphdb_batch_add_edge(
    batch: *mut graphdb_batch_t,
    src_vid: i64,
    dst_vid: i64,
    edge_type: *const c_char,
    rank: i64,
    properties: *const graphdb_value_t,
    prop_count: usize,
) -> c_int {
    if batch.is_null() || edge_type.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let edge_type_str = unsafe {
        match CStr::from_ptr(edge_type).to_str() {
            Ok(s) => s,
            Err(_) => return graphdb_error_code_t::GRAPHDB_MISUSE as c_int,
        }
    };

    let mut props = HashMap::new();

    if !properties.is_null() && prop_count > 0 {
        for i in 0..prop_count {
            unsafe {
                let prop = &*properties.add(i);
                let prop_name = format!("prop_{}", i);
                let value = convert_c_value_to_rust(prop);
                props.insert(prop_name, value);
            }
        }
    }

    unsafe {
        let handle = &mut *(batch as *mut GraphDbBatchHandle);

        // 检查会话有效性
        if !handle.is_session_valid() {
            return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
        }

        let edge = Edge::new(
            Value::Int(src_vid),
            Value::Int(dst_vid),
            edge_type_str.to_string(),
            rank,
            props,
        );

        handle.buffer.push(BatchItem::Edge(edge));

        // 如果达到批次大小，自动刷新
        if handle.buffer.len() >= handle.batch_size {
            if let Err(e) = handle.flush_edges() {
                let error_msg = e.to_string();
                set_last_error_message(error_msg.clone());
                handle.last_error = Some(CString::new(error_msg).unwrap_or_default());
                return graphdb_error_code_t::GRAPHDB_ERROR as c_int;
            }
        }

        graphdb_error_code_t::GRAPHDB_OK as c_int
    }
}

/// 执行批量插入
///
/// # 参数
/// - `batch`: 批量操作句柄
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
///
/// # Safety
/// - `batch` 必须是通过 `graphdb_batch_inserter_create` 创建的有效批量操作句柄
/// - 调用者必须确保在调用此函数时,关联的会话仍然有效
/// - 此函数会触发实际的数据库写入操作,可能涉及 I/O 操作
#[no_mangle]
pub unsafe extern "C" fn graphdb_batch_flush(batch: *mut graphdb_batch_t) -> c_int {
    if batch.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let handle = &mut *(batch as *mut GraphDbBatchHandle);

        // 检查会话有效性
        if !handle.is_session_valid() {
            return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
        }

        match handle.execute() {
            Ok(_) => graphdb_error_code_t::GRAPHDB_OK as c_int,
            Err(e) => {
                let error_msg = e.to_string();
                set_last_error_message(error_msg.clone());
                handle.last_error = Some(CString::new(error_msg).unwrap_or_default());
                graphdb_error_code_t::GRAPHDB_ERROR as c_int
            }
        }
    }
}

/// 获取缓冲的顶点数量
///
/// # 参数
/// - `batch`: 批量操作句柄
///
/// # 返回
/// - 缓冲的顶点数量
///
/// # Safety
/// - `batch` 必须是通过 `graphdb_batch_inserter_create` 创建的有效批量操作句柄
/// - 调用者必须确保在调用此函数时,关联的会话仍然有效
#[no_mangle]
pub unsafe extern "C" fn graphdb_batch_buffered_vertices(batch: *mut graphdb_batch_t) -> c_int {
    if batch.is_null() {
        return -1;
    }

    unsafe {
        let handle = &*(batch as *mut GraphDbBatchHandle);
        handle
            .buffer
            .iter()
            .filter(|item| matches!(item, BatchItem::Vertex(_)))
            .count() as c_int
    }
}

/// 获取缓冲的边数量
///
/// # Arguments
/// - `batch`: Batch operation handle
///
/// # Returns
/// - Number of buffered edges
///
/// # Safety
/// - `batch` must be a valid batch handle created by `graphdb_batch_inserter_create`
/// - Caller must ensure the associated session is still valid when calling this function
#[no_mangle]
pub unsafe extern "C" fn graphdb_batch_buffered_edges(batch: *mut graphdb_batch_t) -> c_int {
    if batch.is_null() {
        return -1;
    }

    unsafe {
        let handle = &*(batch as *mut GraphDbBatchHandle);
        handle
            .buffer
            .iter()
            .filter(|item| matches!(item, BatchItem::Edge(_)))
            .count() as c_int
    }
}

/// 释放批量操作句柄
///
/// # Arguments
/// - `batch`: Batch operation handle
///
/// # Returns
/// - Success: GRAPHDB_OK
/// - Failure: Error code
///
/// # Safety
/// - `batch` must be a valid batch handle created by `graphdb_batch_inserter_create`
/// - After calling this function, the batch handle becomes invalid and must not be used
/// - This function does not close the associated session; the caller must close the session separately
#[no_mangle]
pub unsafe extern "C" fn graphdb_batch_free(batch: *mut graphdb_batch_t) -> c_int {
    if batch.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let _ = Box::from_raw(batch as *mut GraphDbBatchHandle);
    }

    graphdb_error_code_t::GRAPHDB_OK as c_int
}

/// 将 C 值转换为 Rust 值
unsafe fn convert_c_value_to_rust(c_value: &graphdb_value_t) -> Value {
    match c_value.type_ {
        graphdb_value_type_t::GRAPHDB_NULL => Value::Null(crate::core::value::NullType::Null),
        graphdb_value_type_t::GRAPHDB_BOOL => Value::Bool(c_value.data.boolean),
        graphdb_value_type_t::GRAPHDB_INT => Value::Int(c_value.data.integer),
        graphdb_value_type_t::GRAPHDB_FLOAT => Value::Float(c_value.data.floating),
        graphdb_value_type_t::GRAPHDB_STRING => {
            if c_value.data.string.data.is_null() || c_value.data.string.len == 0 {
                Value::String(String::new())
            } else {
                let slice = std::slice::from_raw_parts(
                    c_value.data.string.data as *const u8,
                    c_value.data.string.len,
                );
                let s = String::from_utf8_unchecked(slice.to_vec());
                Value::String(s)
            }
        }
        _ => Value::Null(crate::core::value::NullType::Null),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::embedded::c_api::database::{graphdb_close, graphdb_open};
    use crate::api::embedded::c_api::session::{graphdb_session_close, graphdb_session_create};
    use crate::api::embedded::c_api::types::graphdb_t;
    use std::ptr;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn create_test_db() -> *mut graphdb_t {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_dir = std::env::temp_dir().join("graphdb_c_api_test");
        std::fs::create_dir_all(&temp_dir).ok();
        let db_path = temp_dir.join(format!("test_batch_{}_{}.db", std::process::id(), counter));

        // 确保数据库文件不存在
        if db_path.exists() {
            std::fs::remove_file(&db_path).ok();
            // 等待文件系统完成删除操作
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        let path_cstring = CString::new(db_path.to_str().expect("Invalid path"))
            .expect("Failed to create CString");
        let mut db: *mut graphdb_t = ptr::null_mut();

        let rc = unsafe { graphdb_open(path_cstring.as_ptr(), &mut db) };
        if rc != graphdb_error_code_t::GRAPHDB_OK as c_int {
            panic!("打开数据库失败，错误码: {}, 路径: {:?}", rc, db_path);
        }
        assert!(!db.is_null());

        db
    }

    #[test]
    fn test_batch_inserter_create_null_params() {
        let rc = unsafe { graphdb_batch_inserter_create(ptr::null_mut(), 100, ptr::null_mut()) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as c_int);
    }

    #[test]
    fn test_batch_free_null() {
        let rc = unsafe { graphdb_batch_free(ptr::null_mut()) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as c_int);
    }

    #[test]
    fn test_batch_inserter_create_and_free() {
        let db = create_test_db();
        let mut session: *mut graphdb_session_t = ptr::null_mut();

        let rc = unsafe { graphdb_session_create(db, &mut session) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);

        let mut batch: *mut graphdb_batch_t = ptr::null_mut();
        let rc = unsafe { graphdb_batch_inserter_create(session, 100, &mut batch) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);
        assert!(!batch.is_null());

        let rc = unsafe { graphdb_batch_free(batch) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);

        unsafe { graphdb_session_close(session) };
        unsafe { graphdb_close(db) };
    }

    #[test]
    fn test_batch_buffered_counts_null() {
        let count = unsafe { graphdb_batch_buffered_vertices(ptr::null_mut()) };
        assert_eq!(count, -1);

        let count = unsafe { graphdb_batch_buffered_edges(ptr::null_mut()) };
        assert_eq!(count, -1);
    }
}
