//! C API 集成测试
//!
//! 测试范围:
//! - 数据库生命周期管理
//! - 会话管理
//! - 查询执行
//! - 结果处理
//! - 事务管理
//! - 预编译语句
//! - 批量操作
//! - 错误处理

mod common;

use std::ffi::CString;
use std::ptr;

use graphdb::api::embedded::c_api::error::graphdb_error_code_t;

use common::c_api_helpers::{
    CApiTestBatch, CApiTestDatabase, CApiTestResult, CApiTestSession, CApiTestTransaction,
};

// ==================== 数据库生命周期测试 ====================

#[test]
fn test_c_api_database_open_close() {
    let test_db = CApiTestDatabase::new();
    let db = test_db.handle();

    assert!(!db.is_null());

    // 数据库会在 Drop 时自动关闭
}

#[test]
fn test_c_api_libversion() {
    let version = unsafe {
        std::ffi::CStr::from_ptr(graphdb::api::embedded::c_api::database::graphdb_libversion())
    };

    let version_str = version.to_str().expect("版本字符串无效");
    assert!(!version_str.is_empty());
}

#[test]
fn test_c_api_database_null_params() {
    let rc = unsafe {
        graphdb::api::embedded::c_api::database::graphdb_open(ptr::null(), ptr::null_mut())
    };
    assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as i32);
}

#[test]
fn test_c_api_database_multiple_open_close() {
    let test_db1 = CApiTestDatabase::new();
    let test_db2 = CApiTestDatabase::new();

    assert!(!test_db1.handle().is_null());
    assert!(!test_db2.handle().is_null());

    // 验证两个数据库句柄不同
    assert_ne!(test_db1.handle(), test_db2.handle());
}

// ==================== 会话管理测试 ====================

#[test]
fn test_c_api_session_create_close() {
    let test_db = CApiTestDatabase::new();
    let session = CApiTestSession::from_db(&test_db);

    assert!(!session.handle().is_null());
}

#[test]
fn test_c_api_session_autocommit() {
    let test_db = CApiTestDatabase::new();
    let session = CApiTestSession::from_db(&test_db);

    // 默认自动提交
    let autocommit = unsafe {
        graphdb::api::embedded::c_api::session::graphdb_session_get_autocommit(session.handle())
    };
    assert!(autocommit);

    // 关闭自动提交
    let rc = unsafe {
        graphdb::api::embedded::c_api::session::graphdb_session_set_autocommit(
            session.handle(),
            false,
        )
    };
    assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as i32);

    let autocommit = unsafe {
        graphdb::api::embedded::c_api::session::graphdb_session_get_autocommit(session.handle())
    };
    assert!(!autocommit);
}

#[test]
fn test_c_api_session_null_params() {
    let rc = unsafe {
        graphdb::api::embedded::c_api::session::graphdb_session_create(
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as i32);
}

#[test]
fn test_c_api_session_multiple_sessions() {
    let test_db = CApiTestDatabase::new();
    let session1 = CApiTestSession::from_db(&test_db);
    let session2 = CApiTestSession::from_db(&test_db);

    // 验证两个会话句柄不同
    assert_ne!(session1.handle(), session2.handle());
}

// ==================== 查询执行测试 ====================

#[test]
fn test_c_api_execute_simple_query() {
    let test_db = CApiTestDatabase::new();
    let session = CApiTestSession::from_db(&test_db);

    let query = CString::new("SHOW SPACES").expect("创建CString失败");
    let mut result: *mut graphdb::api::embedded::c_api::types::graphdb_result_t = ptr::null_mut();

    let rc = unsafe {
        graphdb::api::embedded::c_api::query::graphdb_execute(
            session.handle(),
            query.as_ptr(),
            &mut result,
        )
    };

    // 打印错误信息用于调试
    if rc != graphdb_error_code_t::GRAPHDB_OK as i32 {
        let error_msg = graphdb::api::embedded::c_api::error::graphdb_get_last_error_message();
        if !error_msg.is_null() {
            let msg = unsafe {
                std::ffi::CStr::from_ptr(error_msg)
                    .to_string_lossy()
                    .to_string()
            };
            eprintln!("错误码: {}, 错误信息: {}", rc, msg);
        }
    }

    assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as i32);
    assert!(!result.is_null());

    // 清理结果
    unsafe {
        graphdb::api::embedded::c_api::result::graphdb_result_free(result);
    }
}

#[test]
fn test_c_api_execute_with_wrapper() {
    let test_db = CApiTestDatabase::new();
    let session = CApiTestSession::from_db(&test_db);

    let result = CApiTestResult::from_query(&session, "SHOW SPACES");

    assert!(result.column_count() >= 0);
    assert!(result.row_count() >= 0);
}

#[test]
fn test_c_api_execute_null_params() {
    let rc = unsafe {
        graphdb::api::embedded::c_api::query::graphdb_execute(
            ptr::null_mut(),
            ptr::null(),
            ptr::null_mut(),
        )
    };
    assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as i32);
}

// ==================== 结果处理测试 ====================

#[test]
fn test_c_api_result_metadata() {
    let test_db = CApiTestDatabase::new();
    let session = CApiTestSession::from_db(&test_db);

    let query = CString::new("SHOW SPACES").expect("创建CString失败");
    let mut result: *mut graphdb::api::embedded::c_api::types::graphdb_result_t = ptr::null_mut();

    let rc = unsafe {
        graphdb::api::embedded::c_api::query::graphdb_execute(
            session.handle(),
            query.as_ptr(),
            &mut result,
        )
    };
    assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as i32);

    // 获取列数
    let col_count = unsafe { graphdb::api::embedded::c_api::result::graphdb_column_count(result) };
    assert!(col_count >= 0);

    // 获取行数
    let row_count = unsafe { graphdb::api::embedded::c_api::result::graphdb_row_count(result) };
    assert!(row_count >= 0);

    // 清理结果
    unsafe {
        graphdb::api::embedded::c_api::result::graphdb_result_free(result);
    }
}

#[test]
fn test_c_api_result_column_name() {
    let test_db = CApiTestDatabase::new();
    let session = CApiTestSession::from_db(&test_db);

    let query = CString::new("SHOW SPACES").expect("创建CString失败");
    let mut result: *mut graphdb::api::embedded::c_api::types::graphdb_result_t = ptr::null_mut();

    let rc = unsafe {
        graphdb::api::embedded::c_api::query::graphdb_execute(
            session.handle(),
            query.as_ptr(),
            &mut result,
        )
    };
    assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as i32);

    // 获取列数
    let col_count = unsafe { graphdb::api::embedded::c_api::result::graphdb_column_count(result) };

    if col_count > 0 {
        // 获取第一列的名称
        let col_name =
            unsafe { graphdb::api::embedded::c_api::result::graphdb_column_name(result, 0) };

        if !col_name.is_null() {
            let name = unsafe { std::ffi::CStr::from_ptr(col_name) };
            let name_str = name.to_str().expect("列名无效");
            assert!(!name_str.is_empty());

            // 释放列名字符串
            unsafe {
                graphdb::api::embedded::c_api::database::graphdb_free_string(col_name);
            }
        }
    }

    // 清理结果
    unsafe {
        graphdb::api::embedded::c_api::result::graphdb_result_free(result);
    }
}

#[test]
fn test_c_api_result_null_params() {
    let count =
        unsafe { graphdb::api::embedded::c_api::result::graphdb_column_count(ptr::null_mut()) };
    assert_eq!(count, -1);

    let count =
        unsafe { graphdb::api::embedded::c_api::result::graphdb_row_count(ptr::null_mut()) };
    assert_eq!(count, -1);

    let name =
        unsafe { graphdb::api::embedded::c_api::result::graphdb_column_name(ptr::null_mut(), 0) };
    assert!(name.is_null());
}

// ==================== 事务管理测试 ====================

#[test]
fn test_c_api_transaction_begin_commit() {
    let test_db = CApiTestDatabase::new();
    let session = CApiTestSession::from_db(&test_db);

    let mut txn: *mut graphdb::api::embedded::c_api::types::graphdb_txn_t = ptr::null_mut();

    // 开始事务
    let rc = unsafe {
        graphdb::api::embedded::c_api::transaction::graphdb_txn_begin(session.handle(), &mut txn)
    };
    assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as i32);
    assert!(!txn.is_null());

    // 打印事务句柄的地址（用于调试）
    eprintln!("事务句柄地址: {:?}", txn);

    // 提交事务
    let rc = unsafe { graphdb::api::embedded::c_api::transaction::graphdb_txn_commit(txn) };

    // 打印错误信息用于调试
    if rc != graphdb_error_code_t::GRAPHDB_OK as i32 {
        let error_msg = graphdb::api::embedded::c_api::error::graphdb_get_last_error_message();
        if !error_msg.is_null() {
            let msg = unsafe {
                std::ffi::CStr::from_ptr(error_msg)
                    .to_string_lossy()
                    .to_string()
            };
            eprintln!("事务提交失败 - 错误码: {}, 错误信息: {}", rc, msg);
        }
    }

    assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as i32);

    // 清理事务句柄
    unsafe {
        graphdb::api::embedded::c_api::transaction::graphdb_txn_free(txn);
    }
}

#[test]
fn test_c_api_transaction_begin_rollback() {
    let test_db = CApiTestDatabase::new();
    let session = CApiTestSession::from_db(&test_db);

    let mut txn: *mut graphdb::api::embedded::c_api::types::graphdb_txn_t = ptr::null_mut();

    // 开始事务
    let rc = unsafe {
        graphdb::api::embedded::c_api::transaction::graphdb_txn_begin(session.handle(), &mut txn)
    };
    assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as i32);
    assert!(!txn.is_null());

    // 回滚事务
    let rc = unsafe { graphdb::api::embedded::c_api::transaction::graphdb_txn_rollback(txn) };
    assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as i32);

    // 清理事务句柄
    unsafe {
        graphdb::api::embedded::c_api::transaction::graphdb_txn_free(txn);
    }
}

#[test]
fn test_c_api_transaction_with_wrapper() {
    let test_db = CApiTestDatabase::new();
    let session = CApiTestSession::from_db(&test_db);

    let txn = CApiTestTransaction::from_session(&session);
    assert!(!txn.handle().is_null());

    // 提交事务
    txn.commit();
}

#[test]
fn test_c_api_transaction_rollback_with_wrapper() {
    let test_db = CApiTestDatabase::new();
    let session = CApiTestSession::from_db(&test_db);

    let txn = CApiTestTransaction::from_session(&session);
    assert!(!txn.handle().is_null());

    // 回滚事务
    txn.rollback();
}

#[test]
fn test_c_api_transaction_null_params() {
    let rc = unsafe {
        graphdb::api::embedded::c_api::transaction::graphdb_txn_begin(
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as i32);
}

// ==================== 批量操作测试 ====================

#[test]
fn test_c_api_batch_inserter_create_free() {
    let test_db = CApiTestDatabase::new();
    let session = CApiTestSession::from_db(&test_db);

    let mut batch: *mut graphdb::api::embedded::c_api::types::graphdb_batch_t = ptr::null_mut();

    // 创建批量插入器
    let rc = unsafe {
        graphdb::api::embedded::c_api::batch::graphdb_batch_inserter_create(
            session.handle(),
            100,
            &mut batch,
        )
    };
    assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as i32);
    assert!(!batch.is_null());

    // 释放批量插入器
    let rc = unsafe { graphdb::api::embedded::c_api::batch::graphdb_batch_free(batch) };
    assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as i32);
}

#[test]
fn test_c_api_batch_with_wrapper() {
    let test_db = CApiTestDatabase::new();
    let session = CApiTestSession::from_db(&test_db);

    let batch = CApiTestBatch::from_session(&session, 100);
    assert!(!batch.handle().is_null());
}

#[test]
fn test_c_api_batch_null_params() {
    let rc = unsafe {
        graphdb::api::embedded::c_api::batch::graphdb_batch_inserter_create(
            ptr::null_mut(),
            100,
            ptr::null_mut(),
        )
    };
    assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as i32);
}

#[test]
fn test_c_api_batch_buffered_counts_null() {
    let count = unsafe {
        graphdb::api::embedded::c_api::batch::graphdb_batch_buffered_vertices(ptr::null_mut())
    };
    assert_eq!(count, -1);

    let count = unsafe {
        graphdb::api::embedded::c_api::batch::graphdb_batch_buffered_edges(ptr::null_mut())
    };
    assert_eq!(count, -1);
}

// ==================== 错误处理测试 ====================

#[test]
fn test_c_api_error_string() {
    let error_str = unsafe {
        std::ffi::CStr::from_ptr(graphdb::api::embedded::c_api::error::graphdb_error_string(
            graphdb_error_code_t::GRAPHDB_OK as i32,
        ))
    };

    let desc = error_str.to_str().expect("Invalid error description");
    assert_eq!(desc, "OK");
}

#[test]
fn test_c_api_error_codes() {
    let test_cases = vec![
        (graphdb_error_code_t::GRAPHDB_OK as i32, "OK"),
        (graphdb_error_code_t::GRAPHDB_ERROR as i32, "General error"),
        (graphdb_error_code_t::GRAPHDB_MISUSE as i32, "Misuse"),
        (graphdb_error_code_t::GRAPHDB_NOTFOUND as i32, "Not found"),
        (graphdb_error_code_t::GRAPHDB_IOERR as i32, "IO error"),
        (
            graphdb_error_code_t::GRAPHDB_CORRUPT as i32,
            "Data corruption",
        ),
        (graphdb_error_code_t::GRAPHDB_NOMEM as i32, "Out of memory"),
    ];

    for (code, expected_desc) in test_cases {
        let error_str = unsafe {
            std::ffi::CStr::from_ptr(graphdb::api::embedded::c_api::error::graphdb_error_string(
                code,
            ))
        };

        let desc = error_str.to_str().expect("Invalid error description");
        assert_eq!(
            desc, expected_desc,
            "Description mismatch for error code {}",
            code
        );
    }
}

#[test]
fn test_c_api_errmsg() {
    let mut buffer = [0i8; 256];
    let len = unsafe {
        graphdb::api::embedded::c_api::error::graphdb_errmsg(buffer.as_mut_ptr(), buffer.len())
    };

    // 验证返回的长度合理
    assert!(len >= 0);
    assert!((len as usize) < buffer.len());
}

// ==================== 内存管理测试 ====================

#[test]
fn test_c_api_free_string() {
    let test_str = CString::new("test string").expect("创建CString失败");
    let ptr = test_str.into_raw();

    assert!(!ptr.is_null());

    // 释放字符串
    unsafe {
        graphdb::api::embedded::c_api::database::graphdb_free_string(ptr);
    }
}

#[test]
fn test_c_api_free() {
    let test_value = Box::new(42i32);
    let ptr = Box::into_raw(test_value) as *mut std::ffi::c_void;

    assert!(!ptr.is_null());

    // 释放内存
    unsafe {
        graphdb::api::embedded::c_api::database::graphdb_free(ptr);
    }
}

// ==================== 集成场景测试 ====================

#[test]
fn test_c_api_full_workflow() {
    let test_db = CApiTestDatabase::new();
    let session = CApiTestSession::from_db(&test_db);

    // 执行查询
    let result = CApiTestResult::from_query(&session, "SHOW SPACES");

    // 验证结果
    assert!(result.column_count() >= 0);
    assert!(result.row_count() >= 0);

    // 开始事务
    let txn = CApiTestTransaction::from_session(&session);

    // 提交事务
    txn.commit();
    // 所有资源会在 Drop 时自动清理
}

#[test]
fn test_c_api_concurrent_sessions() {
    let test_db = CApiTestDatabase::new();

    let session1 = CApiTestSession::from_db(&test_db);
    let session2 = CApiTestSession::from_db(&test_db);
    let session3 = CApiTestSession::from_db(&test_db);

    // 验证三个会话都有效
    assert!(!session1.handle().is_null());
    assert!(!session2.handle().is_null());
    assert!(!session3.handle().is_null());

    // 验证会话句柄都不同
    assert_ne!(session1.handle(), session2.handle());
    assert_ne!(session2.handle(), session3.handle());
    assert_ne!(session1.handle(), session3.handle());
}
