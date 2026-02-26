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
//! // 关闭数据库
//! db.close()?;
//! # Ok(())
//! # }
//! ```

// 子模块
pub mod batch;
pub mod config;
pub mod database;
pub mod result;
pub mod session;
pub mod statement;
pub mod transaction;

// 重新导出主要类型
pub use batch::{BatchConfig, BatchError, BatchInserter, BatchItemType, BatchResult};
pub use config::{DatabaseConfig, SyncMode};
pub use database::GraphDatabase;
pub use result::{QueryResult, ResultMetadata, Row};
pub use session::Session;
pub use statement::PreparedStatement;
pub use transaction::{Transaction, TransactionConfig, TransactionInfo};

// 错误类型
pub use crate::api::core::CoreError as EmbeddedError;
