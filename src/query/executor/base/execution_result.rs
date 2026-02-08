//! 执行结果类型
//!
//! 定义执行器执行结果的数据结构，支持多种结果类型。

use crate::core::error::DBError;
use crate::core::result::Result as CoreResult;

/// 执行结果类型
///
/// 统一表示所有执行器的执行结果，支持多种数据格式。
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    /// 成功执行，返回通用值列表
    Values(Vec<crate::core::Value>),
    /// 成功执行，返回顶点数据
    Vertices(Vec<crate::core::Vertex>),
    /// 成功执行，返回边数据
    Edges(Vec<crate::core::Edge>),
    /// 成功执行，返回结构化数据集
    DataSet(crate::core::DataSet),
    /// 成功执行，返回内部 Result 对象
    Result(CoreResult),
    /// 成功执行，无数据返回
    Empty,
    /// 成功执行，无数据返回（别名）
    Success,
    /// 执行错误
    Error(String),
    /// 返回计数
    Count(usize),
    /// 返回路径
    Paths(Vec<crate::core::vertex_edge_path::Path>),
}

impl ExecutionResult {
    /// 获取结果中的元素计数
    pub fn count(&self) -> usize {
        match self {
            ExecutionResult::Values(v) => v.len(),
            ExecutionResult::Vertices(v) => v.len(),
            ExecutionResult::Edges(v) => v.len(),
            ExecutionResult::DataSet(ds) => ds.rows.len(),
            ExecutionResult::Result(r) => r.row_count(),
            ExecutionResult::Count(c) => *c,
            ExecutionResult::Success => 0,
            ExecutionResult::Empty => 0,
            ExecutionResult::Error(_) => 0,
            ExecutionResult::Paths(p) => p.len(),
        }
    }

    /// 从 CoreResult 创建 ExecutionResult
    pub fn from_result(result: CoreResult) -> Self {
        ExecutionResult::Result(result)
    }

    /// 转换为 CoreResult
    pub fn to_result(&self) -> Option<CoreResult> {
        match self {
            ExecutionResult::Result(r) => Some(r.clone()),
            _ => None,
        }
    }
}

/// 结果类型别名
pub type DBResult<T> = Result<T, DBError>;

/// 支持转换为执行结果的 trait
pub trait IntoExecutionResult {
    fn into_execution_result(self) -> ExecutionResult;
}

impl IntoExecutionResult for Vec<crate::core::Value> {
    fn into_execution_result(self) -> ExecutionResult {
        ExecutionResult::Values(self)
    }
}

impl IntoExecutionResult for Vec<crate::core::Vertex> {
    fn into_execution_result(self) -> ExecutionResult {
        ExecutionResult::Vertices(self)
    }
}

impl IntoExecutionResult for Vec<crate::core::Edge> {
    fn into_execution_result(self) -> ExecutionResult {
        ExecutionResult::Edges(self)
    }
}

impl IntoExecutionResult for crate::core::DataSet {
    fn into_execution_result(self) -> ExecutionResult {
        ExecutionResult::DataSet(self)
    }
}

impl IntoExecutionResult for () {
    fn into_execution_result(self) -> ExecutionResult {
        ExecutionResult::Success
    }
}

impl IntoExecutionResult for usize {
    fn into_execution_result(self) -> ExecutionResult {
        ExecutionResult::Count(self)
    }
}
