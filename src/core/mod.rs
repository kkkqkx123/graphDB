pub mod allocator;
pub mod collect_n_succeeded;
pub mod cord;
pub mod error;
pub mod expressions;
pub mod murmur;
pub mod result;
pub mod schema;
pub mod signal_handler;
pub mod symbol;
pub mod type_utils;
pub mod value;
pub mod vertex_edge_path;
pub mod visitor;
pub mod visitor_state_enum;

// 新增的子模块
pub mod context;
pub mod context_traits;
pub mod evaluator;
pub mod types;

// 查询处理模块
pub mod executor_factory;
pub mod query_pipeline_manager;

// 错误和结果类型
pub use error::{
    DBError, DBResult, ExpressionError, ExpressionErrorType, ExpressionPosition, QueryError,
};
pub use result::*;

// 核心数据类型
pub use value::*;
pub use vertex_edge_path::{Direction, Edge, Path, Step, Tag, Vertex};

// 表达式系统类型
pub use types::expression::{
    DataType, Expression, ExpressionType, LiteralValue,
};

// 操作符系统类型
pub use types::operators::{
    AggregateFunction, BinaryOperator, Operator, OperatorCategory, OperatorInstance, OperatorRegistry, UnaryOperator,
};

// 其他核心类型
pub use symbol::*;
pub use type_utils::TypeUtils;
pub use visitor::*;
pub use visitor_state_enum::*;

// 表达式上下文
pub use expressions::*;

// 上下文特征
pub use context_traits::*;

// 上下文和求值器
pub use context::*;
pub use evaluator::*;

// 查询处理
pub use executor_factory::*;
pub use query_pipeline_manager::*;
