//! 嵌入式 API 模块
//!
//! 提供单机使用的嵌入式 GraphDB 接口，类似 SQLite 的使用方式
//!
//! # 快速开始
//!
//! ```rust
//! use graphdb::api::embedded::{GraphDatabase, DatabaseConfig};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // 打开数据库
//! let db = GraphDatabase::open("my_database")?;
//!
//! // 创建会话
//! let mut session = db.session()?;
//!
//! // 切换图空间
//! session.use_space("test_space")?;
//!
//! // 执行查询
//! let result = session.execute("MATCH (n) RETURN n")?;
//!
//! // 使用事务
//! let txn = session.begin_transaction()?;
//! txn.execute("CREATE TAG user(name string)")?;
//! txn.commit()?;
//!
//! // 数据库在 db 离开作用域时自动关闭
//! # Ok(())
//! # }
//! ```

// 子模块
pub mod batch;
pub mod busy_handler;
pub mod config;
pub mod database;
pub mod result;
pub mod session;
pub mod statistics;
pub mod transaction;

// C API 模块（条件编译）
#[cfg(feature = "c-api")]
pub mod c_api;

// 重新导出主要类型
pub use batch::{BatchConfig, BatchError, BatchInserter, BatchItemType, BatchResult};
pub use busy_handler::{BusyConfig, BusyHandler, BusyResult};
pub use config::{DatabaseConfig, SyncMode};
pub use database::GraphDatabase;
pub use result::{QueryResult, ResultMetadata, Row};
pub use session::Session;
pub use statistics::{QueryStatistics, SessionStatistics};
pub use transaction::{Transaction, TransactionConfig, TransactionInfo};

// C API 重新导出
#[cfg(feature = "c-api")]
pub use c_api::{
    error::graphdb_error_code_t,
    types::{
        graphdb_batch_t, graphdb_config_t, graphdb_result_t, graphdb_session_t, graphdb_string_t,
        graphdb_t, graphdb_txn_t, graphdb_value_data_t, graphdb_value_t, graphdb_value_type_t,
    },
};

// 错误类型
pub use crate::api::core::CoreError as EmbeddedError;
