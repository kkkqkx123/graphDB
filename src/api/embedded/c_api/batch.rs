//! C API 批量操作模块
//!
//! 提供批量操作功能，支持批量插入、批量更新和批量删除

use crate::api::embedded::c_api::error::{error_code_from_core_error, graphdb_error_code_t};
use crate::api::embedded::c_api::session::GraphDbSessionHandle;
use crate::api::embedded::c_api::types::{graphdb_batch_t, graphdb_session_t, graphdb_value_t};
use crate::api::embedded::c_api::types::graphdb_value_type_t;
use crate::core::{Edge, Value, Vertex};
use std::collections::HashMap;
use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::ptr;

/// 批量操作句柄内部结构
pub struct GraphDbBatchHandle {
    pub(crate) inner: crate::api::embedded::batch::BatchInserter<'static, crate::storage::RedbStorage>,
    pub(crate) last_error: Option<CString>,
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
#[no_mangle]
pub extern "C" fn graphdb_batch_inserter_create(
    session: *mut graphdb_session_t,
    batch_size: c_int,
    batch: *mut *mut graphdb_batch_t,
) -> c_int {
    if session.is_null() || batch.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let size = if batch_size <= 0 { 100 } else { batch_size as usize };

    unsafe {
        let handle = &*(session as *mut GraphDbSessionHandle);

        let session_ref: &'static GraphDbSessionHandle = std::mem::transmute(handle);
        let inserter = crate::api::embedded::batch::BatchInserter::new_static(
            &session_ref.inner,
            size,
        );

        let batch_handle = Box::new(GraphDbBatchHandle {
            inner: inserter,
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
#[no_mangle]
pub extern "C" fn graphdb_batch_add_vertex(
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

        let vertex = Vertex::with_vid(Value::Int(vid));
        handle.inner.add_vertex(vertex);
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
#[no_mangle]
pub extern "C" fn graphdb_batch_add_edge(
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

        let edge = Edge::new(
            Value::Int(src_vid),
            Value::Int(dst_vid),
            edge_type_str.to_string(),
            rank,
            HashMap::new(),
        );
        handle.inner.add_edge(edge);
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
#[no_mangle]
pub extern "C" fn graphdb_batch_flush(batch: *mut graphdb_batch_t) -> c_int {
    if batch.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let handle = &mut *(batch as *mut GraphDbBatchHandle);

        let session_ref: &'static GraphDbSessionHandle = std::mem::transmute(&*(batch as *const GraphDbBatchHandle));
        let inserter = std::mem::replace(
            &mut handle.inner,
            crate::api::embedded::batch::BatchInserter::new_static(
                &session_ref.inner,
                100,
            ),
        );

        match inserter.execute() {
            Ok(_) => graphdb_error_code_t::GRAPHDB_OK as c_int,
            Err(e) => {
                let error_code = error_code_from_core_error(&e);
                handle.last_error = Some(CString::new(format!("{}", e)).unwrap_or_default());
                error_code
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
#[no_mangle]
pub extern "C" fn graphdb_batch_buffered_vertices(batch: *mut graphdb_batch_t) -> c_int {
    if batch.is_null() {
        return -1;
    }

    unsafe {
        let handle = &*(batch as *mut GraphDbBatchHandle);
        handle.inner.buffered_vertices() as c_int
    }
}

/// 获取缓冲的边数量
///
/// # 参数
/// - `batch`: 批量操作句柄
///
/// # 返回
/// - 缓冲的边数量
#[no_mangle]
pub extern "C" fn graphdb_batch_buffered_edges(batch: *mut graphdb_batch_t) -> c_int {
    if batch.is_null() {
        return -1;
    }

    unsafe {
        let handle = &*(batch as *mut GraphDbBatchHandle);
        handle.inner.buffered_edges() as c_int
    }
}

/// 释放批量操作句柄
///
/// # 参数
/// - `batch`: 批量操作句柄
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_batch_free(batch: *mut graphdb_batch_t) -> c_int {
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

        let path_cstring = CString::new(db_path.to_str().unwrap()).unwrap();
        let mut db: *mut graphdb_t = ptr::null_mut();

        let rc = graphdb_open(path_cstring.as_ptr(), &mut db);
        if rc != graphdb_error_code_t::GRAPHDB_OK as c_int {
            panic!("打开数据库失败，错误码: {}, 路径: {:?}", rc, db_path);
        }
        assert!(!db.is_null());

        db
    }

    #[test]
    fn test_batch_inserter_create_null_params() {
        let rc = graphdb_batch_inserter_create(ptr::null_mut(), 100, ptr::null_mut());
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as c_int);
    }

    #[test]
    fn test_batch_free_null() {
        let rc = graphdb_batch_free(ptr::null_mut());
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as c_int);
    }

    #[test]
    fn test_batch_inserter_create_and_free() {
        let db = create_test_db();
        let mut session: *mut graphdb_session_t = ptr::null_mut();

        let rc = graphdb_session_create(db, &mut session);
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);

        let mut batch: *mut graphdb_batch_t = ptr::null_mut();
        let rc = graphdb_batch_inserter_create(session, 100, &mut batch);
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);
        assert!(!batch.is_null());

        let rc = graphdb_batch_free(batch);
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);

        graphdb_session_close(session);
        graphdb_close(db);
    }

    #[test]
    fn test_batch_buffered_counts_null() {
        let count = graphdb_batch_buffered_vertices(ptr::null_mut());
        assert_eq!(count, -1);

        let count = graphdb_batch_buffered_edges(ptr::null_mut());
        assert_eq!(count, -1);
    }
}
