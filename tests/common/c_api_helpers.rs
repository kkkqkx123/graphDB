//! C API 集成测试辅助工具
//!
//! 提供 C API 测试的公共函数和结构体

#![allow(dead_code)]

use std::ffi::CString;
use std::ptr;

use graphdb::api::embedded::c_api::error::graphdb_error_code_t;

/// C API 测试数据库包装器
///
/// 使用 RAII 模式管理数据库生命周期，确保测试后正确清理资源
pub struct CApiTestDatabase {
    db: *mut graphdb::api::embedded::c_api::types::graphdb_t,
    temp_path: std::path::PathBuf,
}

impl CApiTestDatabase {
    /// 创建新的测试数据库
    ///
    /// 使用临时目录创建独立的数据库文件，确保测试隔离
    pub fn new() -> Self {
        let temp_dir = std::env::temp_dir().join("graphdb_c_api_integration_test");
        std::fs::create_dir_all(&temp_dir).expect("创建临时目录失败");

        let db_path = temp_dir.join(format!("test_{}.db", std::process::id()));

        // 确保数据库文件不存在
        if db_path.exists() {
            std::fs::remove_file(&db_path).expect("删除旧数据库文件失败");
            // 等待文件系统完成删除操作
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        let path_cstring =
            CString::new(db_path.to_str().expect("路径转换为字符串失败")).expect("创建CString失败");
        let mut db: *mut graphdb::api::embedded::c_api::types::graphdb_t = ptr::null_mut();

        let rc = unsafe {
            graphdb::api::embedded::c_api::database::graphdb_open(path_cstring.as_ptr(), &mut db)
        };

        assert_eq!(
            rc,
            graphdb_error_code_t::GRAPHDB_OK as i32,
            "打开数据库失败，错误码: {}, 路径: {:?}",
            rc,
            db_path
        );
        assert!(!db.is_null(), "数据库句柄不应为空");

        Self {
            db,
            temp_path: db_path,
        }
    }

    /// 获取数据库句柄
    pub fn handle(&self) -> *mut graphdb::api::embedded::c_api::types::graphdb_t {
        self.db
    }
}

impl Drop for CApiTestDatabase {
    fn drop(&mut self) {
        if !self.db.is_null() {
            unsafe {
                graphdb::api::embedded::c_api::database::graphdb_close(self.db);
            }
        }

        // 清理临时文件
        if self.temp_path.exists() {
            let _ = std::fs::remove_file(&self.temp_path);
        }
    }
}

/// C API 测试会话包装器
///
/// 使用 RAII 模式管理会话生命周期
pub struct CApiTestSession {
    session: *mut graphdb::api::embedded::c_api::types::graphdb_session_t,
}

impl CApiTestSession {
    /// 从数据库创建会话
    pub fn from_db(db: &CApiTestDatabase) -> Self {
        let mut session: *mut graphdb::api::embedded::c_api::types::graphdb_session_t =
            ptr::null_mut();

        let rc = unsafe {
            graphdb::api::embedded::c_api::session::graphdb_session_create(
                db.handle(),
                &mut session,
            )
        };

        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as i32, "创建会话失败");
        assert!(!session.is_null(), "会话句柄不应为空");

        Self { session }
    }

    /// 获取会话句柄
    pub fn handle(&self) -> *mut graphdb::api::embedded::c_api::types::graphdb_session_t {
        self.session
    }
}

impl Drop for CApiTestSession {
    fn drop(&mut self) {
        if !self.session.is_null() {
            unsafe {
                graphdb::api::embedded::c_api::session::graphdb_session_close(self.session);
            }
        }
    }
}

/// C API 测试事务包装器
///
/// 使用 RAII 模式管理事务生命周期
pub struct CApiTestTransaction {
    txn: *mut graphdb::api::embedded::c_api::types::graphdb_txn_t,
}

impl CApiTestTransaction {
    /// 从会话创建事务
    pub fn from_session(session: &CApiTestSession) -> Self {
        let mut txn: *mut graphdb::api::embedded::c_api::types::graphdb_txn_t = ptr::null_mut();

        let rc = unsafe {
            graphdb::api::embedded::c_api::transaction::graphdb_txn_begin(
                session.handle(),
                &mut txn,
            )
        };

        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as i32, "开始事务失败");
        assert!(!txn.is_null(), "事务句柄不应为空");

        Self { txn }
    }

    /// 获取事务句柄
    pub fn handle(&self) -> *mut graphdb::api::embedded::c_api::types::graphdb_txn_t {
        self.txn
    }

    /// 提交事务
    pub fn commit(self) {
        let rc =
            unsafe { graphdb::api::embedded::c_api::transaction::graphdb_txn_commit(self.txn) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as i32, "提交事务失败");
        // 防止 Drop 时再次释放
        std::mem::forget(self);
    }

    /// 回滚事务
    pub fn rollback(self) {
        let rc =
            unsafe { graphdb::api::embedded::c_api::transaction::graphdb_txn_rollback(self.txn) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as i32, "回滚事务失败");
        // 防止 Drop 时再次释放
        std::mem::forget(self);
    }
}

impl Drop for CApiTestTransaction {
    fn drop(&mut self) {
        if !self.txn.is_null() {
            unsafe {
                graphdb::api::embedded::c_api::transaction::graphdb_txn_free(self.txn);
            }
        }
    }
}

/// C API 测试结果包装器
///
/// 使用 RAII 模式管理结果集生命周期
pub struct CApiTestResult {
    result: *mut graphdb::api::embedded::c_api::types::graphdb_result_t,
}

impl CApiTestResult {
    /// 从会话执行查询创建结果
    pub fn from_query(session: &CApiTestSession, query: &str) -> Self {
        let query_cstring = CString::new(query).expect("查询字符串无效");
        let mut result: *mut graphdb::api::embedded::c_api::types::graphdb_result_t =
            ptr::null_mut();

        let rc = unsafe {
            graphdb::api::embedded::c_api::query::graphdb_execute(
                session.handle(),
                query_cstring.as_ptr(),
                &mut result,
            )
        };

        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as i32, "执行查询失败");
        assert!(!result.is_null(), "结果句柄不应为空");

        Self { result }
    }

    /// 获取列数
    pub fn column_count(&self) -> i32 {
        unsafe { graphdb::api::embedded::c_api::result::graphdb_column_count(self.result) }
    }

    /// 获取行数
    pub fn row_count(&self) -> i32 {
        unsafe { graphdb::api::embedded::c_api::result::graphdb_row_count(self.result) }
    }
}

impl Drop for CApiTestResult {
    fn drop(&mut self) {
        if !self.result.is_null() {
            unsafe {
                graphdb::api::embedded::c_api::result::graphdb_result_free(self.result);
            }
        }
    }
}

/// C API 测试预编译语句包装器
///
/// 使用 RAII 模式管理预编译语句生命周期
pub struct CApiTestStatement {
    stmt: *mut graphdb::api::embedded::c_api::types::graphdb_stmt_t,
}

impl CApiTestStatement {
    /// 从会话准备语句
    pub fn from_session(session: &CApiTestSession, query: &str) -> Self {
        let query_cstring = CString::new(query).expect("查询字符串无效");
        let mut stmt: *mut graphdb::api::embedded::c_api::types::graphdb_stmt_t = ptr::null_mut();

        let rc = unsafe {
            graphdb::api::embedded::c_api::statement::graphdb_prepare(
                session.handle(),
                query_cstring.as_ptr(),
                &mut stmt,
            )
        };

        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as i32, "准备语句失败");
        assert!(!stmt.is_null(), "语句句柄不应为空");

        Self { stmt }
    }

    /// 获取语句句柄
    pub fn handle(&self) -> *mut graphdb::api::embedded::c_api::types::graphdb_stmt_t {
        self.stmt
    }
}

impl Drop for CApiTestStatement {
    fn drop(&mut self) {
        if !self.stmt.is_null() {
            unsafe {
                graphdb::api::embedded::c_api::statement::graphdb_finalize(self.stmt);
            }
        }
    }
}

/// C API 测试批量操作包装器
///
/// 使用 RAII 模式管理批量操作生命周期
pub struct CApiTestBatch {
    batch: *mut graphdb::api::embedded::c_api::types::graphdb_batch_t,
}

impl CApiTestBatch {
    /// 从会话创建批量插入器
    pub fn from_session(session: &CApiTestSession, batch_size: i32) -> Self {
        let mut batch: *mut graphdb::api::embedded::c_api::types::graphdb_batch_t = ptr::null_mut();

        let rc = unsafe {
            graphdb::api::embedded::c_api::batch::graphdb_batch_inserter_create(
                session.handle(),
                batch_size,
                &mut batch,
            )
        };

        assert_eq!(
            rc,
            graphdb_error_code_t::GRAPHDB_OK as i32,
            "创建批量插入器失败"
        );
        assert!(!batch.is_null(), "批量操作句柄不应为空");

        Self { batch }
    }

    /// 获取批量操作句柄
    pub fn handle(&self) -> *mut graphdb::api::embedded::c_api::types::graphdb_batch_t {
        self.batch
    }
}

impl Drop for CApiTestBatch {
    fn drop(&mut self) {
        if !self.batch.is_null() {
            unsafe {
                graphdb::api::embedded::c_api::batch::graphdb_batch_free(self.batch);
            }
        }
    }
}

