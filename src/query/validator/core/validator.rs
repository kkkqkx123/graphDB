//! 验证器核心 trait 和枚举定义
//!
//! 定义 StatementValidator trait 和 Validator 枚举，
//! 提供统一的验证器接口

use crate::core::error::ValidationError;
use crate::query::context::validate::ValidationContext;

use super::types::{ColumnDef, StatementType};

/// 语句验证器 trait
///
/// 所有具体验证器必须实现此 trait，提供统一的验证接口
///
/// # 示例
///
/// ```rust,ignore
/// pub struct MatchValidator {
///     // ...
/// }
///
/// impl StatementValidator for MatchValidator {
///     fn validate(&mut self, ctx: &mut ValidationContext) -> Result<(), ValidationError> {
///         // 执行验证逻辑
///     }
///
///     fn statement_type(&self) -> StatementType {
///         StatementType::Match
///     }
/// }
/// ```
pub trait StatementValidator {
    /// 执行验证
    ///
    /// 执行具体的验证逻辑，将错误添加到上下文中
    ///
    /// # 参数
    ///
    /// * `ctx` - 验证上下文，包含 schema、变量等信息
    ///
    /// # 返回
    ///
    /// 验证成功返回 Ok，失败返回 ValidationError
    fn validate(&mut self, ctx: &mut ValidationContext) -> Result<(), ValidationError>;

    /// 获取语句类型
    ///
    /// 返回此验证器处理的语句类型
    fn statement_type(&self) -> StatementType;

    /// 获取输入列定义
    ///
    /// 返回验证器期望的输入列
    fn inputs(&self) -> &[ColumnDef] {
        &[]
    }

    /// 获取输出列定义
    ///
    /// 返回验证器产生的输出列
    fn outputs(&self) -> &[ColumnDef] {
        &[]
    }

    /// 添加输入列
    ///
    /// 添加一个输入列定义
    fn add_input(&mut self, _col: ColumnDef) {}

    /// 添加输出列
    ///
    /// 添加一个输出列定义
    fn add_output(&mut self, _col: ColumnDef) {}

    /// 检查是否需要选择图空间
    ///
    /// 默认根据语句类型判断
    fn requires_space(&self) -> bool {
        self.statement_type().requires_space()
    }
}

/// 验证器枚举
///
/// 包装所有具体验证器类型，实现统一接口
///
/// 使用枚举而非 trait object 实现静态分发，
/// 避免虚函数调用开销
#[derive(Debug)]
pub enum Validator {
    /// MATCH 验证器
    Match(super::super::match_validator::MatchValidator),
    /// GO 验证器
    Go(super::super::go_validator::GoValidator),
    /// FETCH VERTICES 验证器
    FetchVertices(super::super::fetch_vertices_validator::FetchVerticesValidator),
    /// FETCH EDGES 验证器
    FetchEdges(super::super::fetch_edges_validator::FetchEdgesValidator),
    /// LOOKUP 验证器
    Lookup(super::super::lookup_validator::LookupValidator),
    /// FIND PATH 验证器
    FindPath(super::super::find_path_validator::FindPathValidator),
    /// GET SUBGRAPH 验证器
    GetSubgraph(super::super::get_subgraph_validator::GetSubgraphValidator),
    /// INSERT 验证器
    InsertVertices(super::super::insert_vertices_validator::InsertVerticesValidator),
    InsertEdges(super::super::insert_edges_validator::InsertEdgesValidator),
    /// UPDATE 验证器
    Update(super::super::update_validator::UpdateValidator),
    /// DELETE 验证器
    Delete(super::super::delete_validator::DeleteValidator),
    /// CREATE 验证器
    Create(super::super::create_validator::CreateValidator),
    /// USE 验证器
    Use(super::super::use_validator::UseValidator),
    /// PIPE 验证器
    Pipe(super::super::pipe_validator::PipeValidator),
    /// YIELD 验证器
    Yield(super::super::yield_validator::YieldValidator),
    /// UNWIND 验证器
    Unwind(super::super::unwind_validator::UnwindValidator),
    /// SET 验证器
    Set(super::super::set_validator::SetValidator),
    /// SEQUENTIAL 验证器
    Sequential(super::super::sequential_validator::SequentialValidator),
}

impl StatementValidator for Validator {
    fn validate(&mut self, ctx: &mut ValidationContext) -> Result<(), ValidationError> {
        match self {
            Validator::Match(v) => v.validate(ctx),
            Validator::Go(v) => v.validate(ctx),
            Validator::FetchVertices(v) => v.validate(ctx),
            Validator::FetchEdges(v) => v.validate(ctx),
            Validator::Lookup(v) => v.validate(ctx),
            Validator::FindPath(v) => v.validate(ctx),
            Validator::GetSubgraph(v) => v.validate(ctx),
            Validator::InsertVertices(v) => v.validate(ctx),
            Validator::InsertEdges(v) => v.validate(ctx),
            Validator::Update(v) => v.validate(ctx),
            Validator::Delete(v) => v.validate(ctx),
            Validator::Create(v) => v.validate(ctx),
            Validator::Use(v) => v.validate(ctx),
            Validator::Pipe(v) => v.validate(ctx),
            Validator::Yield(v) => v.validate(ctx),
            Validator::Unwind(v) => v.validate(ctx),
            Validator::Set(v) => v.validate(ctx),
            Validator::Sequential(v) => v.validate(ctx),
        }
    }

    fn statement_type(&self) -> StatementType {
        match self {
            Validator::Match(_) => StatementType::Match,
            Validator::Go(_) => StatementType::Go,
            Validator::FetchVertices(_) => StatementType::FetchVertices,
            Validator::FetchEdges(_) => StatementType::FetchEdges,
            Validator::Lookup(_) => StatementType::Lookup,
            Validator::FindPath(_) => StatementType::FindPath,
            Validator::GetSubgraph(_) => StatementType::GetSubgraph,
            Validator::InsertVertices(_) => StatementType::Insert,
            Validator::InsertEdges(_) => StatementType::Insert,
            Validator::Update(_) => StatementType::Update,
            Validator::Delete(_) => StatementType::Delete,
            Validator::Create(_) => StatementType::Create,
            Validator::Use(_) => StatementType::Use,
            Validator::Pipe(_) => StatementType::Pipe,
            Validator::Yield(_) => StatementType::Yield,
            Validator::Unwind(_) => StatementType::Unwind,
            Validator::Set(_) => StatementType::Set,
            Validator::Sequential(_) => StatementType::Sequential,
        }
    }

    fn inputs(&self) -> &[ColumnDef] {
        match self {
            Validator::Match(v) => v.inputs(),
            Validator::Go(v) => v.inputs(),
            Validator::FetchVertices(v) => v.inputs(),
            Validator::FetchEdges(v) => v.inputs(),
            Validator::Lookup(v) => v.inputs(),
            Validator::FindPath(v) => v.inputs(),
            Validator::GetSubgraph(v) => v.inputs(),
            Validator::InsertVertices(v) => v.inputs(),
            Validator::InsertEdges(v) => v.inputs(),
            Validator::Update(v) => v.inputs(),
            Validator::Delete(v) => v.inputs(),
            Validator::Create(v) => v.inputs(),
            Validator::Use(v) => v.inputs(),
            Validator::Pipe(v) => v.inputs(),
            Validator::Yield(v) => v.inputs(),
            Validator::Unwind(v) => v.inputs(),
            Validator::Set(v) => v.inputs(),
            Validator::Sequential(v) => v.inputs(),
        }
    }

    fn outputs(&self) -> &[ColumnDef] {
        match self {
            Validator::Match(v) => v.outputs(),
            Validator::Go(v) => v.outputs(),
            Validator::FetchVertices(v) => v.outputs(),
            Validator::FetchEdges(v) => v.outputs(),
            Validator::Lookup(v) => v.outputs(),
            Validator::FindPath(v) => v.outputs(),
            Validator::GetSubgraph(v) => v.outputs(),
            Validator::InsertVertices(v) => v.outputs(),
            Validator::InsertEdges(v) => v.outputs(),
            Validator::Update(v) => v.outputs(),
            Validator::Delete(v) => v.outputs(),
            Validator::Create(v) => v.outputs(),
            Validator::Use(v) => v.outputs(),
            Validator::Pipe(v) => v.outputs(),
            Validator::Yield(v) => v.outputs(),
            Validator::Unwind(v) => v.outputs(),
            Validator::Set(v) => v.outputs(),
            Validator::Sequential(v) => v.outputs(),
        }
    }

    fn add_input(&mut self, col: ColumnDef) {
        match self {
            Validator::Match(v) => v.add_input(col),
            Validator::Go(v) => v.add_input(col),
            Validator::FetchVertices(v) => v.add_input(col),
            Validator::FetchEdges(v) => v.add_input(col),
            Validator::Lookup(v) => v.add_input(col),
            Validator::FindPath(v) => v.add_input(col),
            Validator::GetSubgraph(v) => v.add_input(col),
            Validator::InsertVertices(v) => v.add_input(col),
            Validator::InsertEdges(v) => v.add_input(col),
            Validator::Update(v) => v.add_input(col),
            Validator::Delete(v) => v.add_input(col),
            Validator::Create(v) => v.add_input(col),
            Validator::Use(v) => v.add_input(col),
            Validator::Pipe(v) => v.add_input(col),
            Validator::Yield(v) => v.add_input(col),
            Validator::Unwind(v) => v.add_input(col),
            Validator::Set(v) => v.add_input(col),
            Validator::Sequential(v) => v.add_input(col),
        }
    }

    fn add_output(&mut self, col: ColumnDef) {
        match self {
            Validator::Match(v) => v.add_output(col),
            Validator::Go(v) => v.add_output(col),
            Validator::FetchVertices(v) => v.add_output(col),
            Validator::FetchEdges(v) => v.add_output(col),
            Validator::Lookup(v) => v.add_output(col),
            Validator::FindPath(v) => v.add_output(col),
            Validator::GetSubgraph(v) => v.add_output(col),
            Validator::InsertVertices(v) => v.add_output(col),
            Validator::InsertEdges(v) => v.add_output(col),
            Validator::Update(v) => v.add_output(col),
            Validator::Delete(v) => v.add_output(col),
            Validator::Create(v) => v.add_output(col),
            Validator::Use(v) => v.add_output(col),
            Validator::Pipe(v) => v.add_output(col),
            Validator::Yield(v) => v.add_output(col),
            Validator::Unwind(v) => v.add_output(col),
            Validator::Set(v) => v.add_output(col),
            Validator::Sequential(v) => v.add_output(col),
        }
    }
}

/// 验证器构建器
///
/// 用于从 AST 创建对应的验证器
pub struct ValidatorBuilder;

impl ValidatorBuilder {
    /// 从 AST 语句创建验证器
    ///
    /// # 参数
    ///
    /// * `stmt` - AST 语句
    ///
    /// # 返回
    ///
    /// 成功返回对应的验证器，失败返回错误
    pub fn from_ast(stmt: &crate::query::parser::ast::Stmt) -> Result<Validator, ValidationError> {
        use crate::query::parser::ast::Stmt;

        match stmt {
            Stmt::Match(_) => Ok(Validator::Match(super::super::match_validator::MatchValidator::new())),
            Stmt::Go(_) => Ok(Validator::Go(super::super::go_validator::GoValidator::new())),
            Stmt::Fetch(_) => Ok(Validator::FetchVertices(super::super::fetch_vertices_validator::FetchVerticesValidator::new())),
            Stmt::Lookup(_) => Ok(Validator::Lookup(super::super::lookup_validator::LookupValidator::new())),
            Stmt::FindPath(_) => Ok(Validator::FindPath(super::super::find_path_validator::FindPathValidator::new())),
            Stmt::Subgraph(_) => Ok(Validator::GetSubgraph(super::super::get_subgraph_validator::GetSubgraphValidator::new())),
            Stmt::Insert(_) => Ok(Validator::InsertVertices(super::super::insert_vertices_validator::InsertVerticesValidator::new())),
            Stmt::Update(_) => Ok(Validator::Update(super::super::update_validator::UpdateValidator::new())),
            Stmt::Delete(_) => Ok(Validator::Delete(super::super::delete_validator::DeleteValidator::new())),
            Stmt::Create(_) => Ok(Validator::Create(super::super::create_validator::CreateValidator::new())),
            Stmt::Use(_) => Ok(Validator::Use(super::super::use_validator::UseValidator::new())),
            Stmt::Pipe(_) => Ok(Validator::Pipe(super::super::pipe_validator::PipeValidator::new())),
            Stmt::Yield(_) => Ok(Validator::Yield(super::super::yield_validator::YieldValidator::new())),
            Stmt::Unwind(_) => Ok(Validator::Unwind(super::super::unwind_validator::UnwindValidator::new())),
            Stmt::Set(_) => Ok(Validator::Set(super::super::set_validator::SetValidator::new())),
            _ => Err(ValidationError::new(
                format!("不支持的语句类型: {:?}", stmt),
                crate::core::error::ValidationErrorType::SyntaxError,
            )),
        }
    }
}
