//! 验证器枚举
//! 使用枚举统一管理所有验证器类型
//! 这是新验证器体系的核心组件，替代 Box<dyn> 的动态分发
//!
//! 设计原则：
//! 1. 保留 base_validator.rs 的完整功能
//! 2. 使用枚举避免动态分发开销
//! 3. 统一接口，便于管理和扩展

use crate::core::error::ValidationError;
use crate::query::context::ast::AstContext;
use crate::query::validator::validator_trait::{
    StatementType, StatementValidator, ValidationResult, ColumnDef, ExpressionProps,
};

// 导入具体验证器
use crate::query::validator::create_validator::CreateValidator;
use crate::query::validator::delete_validator::DeleteValidator;
use crate::query::validator::fetch_edges_validator::FetchEdgesValidator;
use crate::query::validator::fetch_vertices_validator::FetchVerticesValidator;
use crate::query::validator::find_path_validator::FindPathValidator;
use crate::query::validator::get_subgraph_validator::GetSubgraphValidator;
use crate::query::validator::go_validator::GoValidator;
use crate::query::validator::insert_edges_validator::InsertEdgesValidator;
use crate::query::validator::insert_vertices_validator::InsertVerticesValidator;
use crate::query::validator::limit_validator::LimitValidator;
use crate::query::validator::lookup_validator::LookupValidator;
use crate::query::validator::match_validator::MatchValidator;
use crate::query::validator::order_by_validator::OrderByValidator;
use crate::query::validator::pipe_validator::PipeValidator;
use crate::query::validator::sequential_validator::SequentialValidator;
use crate::query::validator::set_validator::SetValidator;
use crate::query::validator::unwind_validator::UnwindValidator;
use crate::query::validator::update_validator::UpdateValidator;
use crate::query::validator::use_validator::UseValidator;
use crate::query::validator::yield_validator::YieldValidator;

/// 统一验证器枚举
///
/// 设计优势：
/// 1. 编译期确定类型，避免动态分发开销
/// 2. 统一接口，便于管理和扩展
/// 3. 模式匹配支持，便于针对特定验证器处理
/// 4. 保留完整的验证生命周期功能
#[derive(Debug)]
pub enum Validator {
    /// CREATE 语句验证器
    Create(CreateValidator),
    /// DELETE 语句验证器
    Delete(DeleteValidator),
    /// FETCH EDGES 语句验证器
    FetchEdges(FetchEdgesValidator),
    /// FETCH VERTICES 语句验证器
    FetchVertices(FetchVerticesValidator),
    /// FIND PATH 语句验证器
    FindPath(FindPathValidator),
    /// GET SUBGRAPH 语句验证器
    GetSubgraph(GetSubgraphValidator),
    /// GO 语句验证器
    Go(GoValidator),
    /// INSERT EDGES 语句验证器
    InsertEdges(InsertEdgesValidator),
    /// INSERT VERTICES 语句验证器
    InsertVertices(InsertVerticesValidator),
    /// LIMIT 子句验证器
    Limit(LimitValidator),
    /// LOOKUP 语句验证器
    Lookup(LookupValidator),
    /// MATCH 语句验证器
    Match(MatchValidator),
    /// ORDER BY 子句验证器
    OrderBy(OrderByValidator),
    /// 管道操作验证器
    Pipe(PipeValidator),
    /// Sequential 语句验证器
    Sequential(SequentialValidator),
    /// SET 语句验证器
    Set(SetValidator),
    /// UPDATE 语句验证器
    Update(UpdateValidator),
    /// UNWIND 子句验证器
    Unwind(UnwindValidator),
    /// USE 语句验证器
    Use(UseValidator),
    /// YIELD 子句验证器
    Yield(YieldValidator),
}

/// 为 Validator 枚举实现方法
/// 使用宏减少重复代码
macro_rules! forward_to_validator {
    ($self:ident, $method:ident) => {
        match $self {
            Validator::Create(v) => v.$method(),
            Validator::Delete(v) => v.$method(),
            Validator::FetchEdges(v) => v.$method(),
            Validator::FetchVertices(v) => v.$method(),
            Validator::FindPath(v) => v.$method(),
            Validator::GetSubgraph(v) => v.$method(),
            Validator::Go(v) => v.$method(),
            Validator::InsertEdges(v) => v.$method(),
            Validator::InsertVertices(v) => v.$method(),
            Validator::Limit(v) => v.$method(),
            Validator::Lookup(v) => v.$method(),
            Validator::Match(v) => v.$method(),
            Validator::OrderBy(v) => v.$method(),
            Validator::Pipe(v) => v.$method(),
            Validator::Sequential(v) => v.$method(),
            Validator::Set(v) => v.$method(),
            Validator::Update(v) => v.$method(),
            Validator::Unwind(v) => v.$method(),
            Validator::Use(v) => v.$method(),
            Validator::Yield(v) => v.$method(),
        }
    };
    ($self:ident, $method:ident, $arg:expr) => {
        match $self {
            Validator::Create(v) => v.$method($arg),
            Validator::Delete(v) => v.$method($arg),
            Validator::FetchEdges(v) => v.$method($arg),
            Validator::FetchVertices(v) => v.$method($arg),
            Validator::FindPath(v) => v.$method($arg),
            Validator::GetSubgraph(v) => v.$method($arg),
            Validator::Go(v) => v.$method($arg),
            Validator::InsertEdges(v) => v.$method($arg),
            Validator::InsertVertices(v) => v.$method($arg),
            Validator::Limit(v) => v.$method($arg),
            Validator::Lookup(v) => v.$method($arg),
            Validator::Match(v) => v.$method($arg),
            Validator::OrderBy(v) => v.$method($arg),
            Validator::Pipe(v) => v.$method($arg),
            Validator::Sequential(v) => v.$method($arg),
            Validator::Set(v) => v.$method($arg),
            Validator::Update(v) => v.$method($arg),
            Validator::Unwind(v) => v.$method($arg),
            Validator::Use(v) => v.$method($arg),
            Validator::Yield(v) => v.$method($arg),
        }
    };
}

impl Validator {
    /// 创建默认验证器（使用 SequentialValidator 作为默认）
    pub fn new() -> Self {
        Validator::Sequential(SequentialValidator::new())
    }

    /// 创建 CREATE 验证器
    pub fn create(validator: CreateValidator) -> Self {
        Validator::Create(validator)
    }

    /// 创建 DELETE 验证器
    pub fn delete(validator: DeleteValidator) -> Self {
        Validator::Delete(validator)
    }

    /// 创建 FETCH EDGES 验证器
    pub fn fetch_edges(validator: FetchEdgesValidator) -> Self {
        Validator::FetchEdges(validator)
    }

    /// 创建 FETCH VERTICES 验证器
    pub fn fetch_vertices(validator: FetchVerticesValidator) -> Self {
        Validator::FetchVertices(validator)
    }

    /// 创建 FIND PATH 验证器
    pub fn find_path(validator: FindPathValidator) -> Self {
        Validator::FindPath(validator)
    }

    /// 创建 GET SUBGRAPH 验证器
    pub fn get_subgraph(validator: GetSubgraphValidator) -> Self {
        Validator::GetSubgraph(validator)
    }

    /// 创建 GO 验证器
    pub fn go(validator: GoValidator) -> Self {
        Validator::Go(validator)
    }

    /// 创建 INSERT EDGES 验证器
    pub fn insert_edges(validator: InsertEdgesValidator) -> Self {
        Validator::InsertEdges(validator)
    }

    /// 创建 INSERT VERTICES 验证器
    pub fn insert_vertices(validator: InsertVerticesValidator) -> Self {
        Validator::InsertVertices(validator)
    }

    /// 创建 LIMIT 验证器
    pub fn limit(validator: LimitValidator) -> Self {
        Validator::Limit(validator)
    }

    /// 创建 LOOKUP 验证器
    pub fn lookup(validator: LookupValidator) -> Self {
        Validator::Lookup(validator)
    }

    /// 创建 MATCH 验证器
    pub fn match_(validator: MatchValidator) -> Self {
        Validator::Match(validator)
    }

    /// 创建 ORDER BY 验证器
    pub fn order_by(validator: OrderByValidator) -> Self {
        Validator::OrderBy(validator)
    }

    /// 创建 Pipe 验证器
    pub fn pipe(validator: PipeValidator) -> Self {
        Validator::Pipe(validator)
    }

    /// 创建 Sequential 验证器
    pub fn sequential(validator: SequentialValidator) -> Self {
        Validator::Sequential(validator)
    }

    /// 创建 Set 验证器
    pub fn set(validator: SetValidator) -> Self {
        Validator::Set(validator)
    }

    /// 创建 UPDATE 验证器
    pub fn update(validator: UpdateValidator) -> Self {
        Validator::Update(validator)
    }

    /// 创建 Unwind 验证器
    pub fn unwind(validator: UnwindValidator) -> Self {
        Validator::Unwind(validator)
    }

    /// 创建 Use 验证器
    pub fn use_(validator: UseValidator) -> Self {
        Validator::Use(validator)
    }

    /// 创建 Yield 验证器
    pub fn yield_(validator: YieldValidator) -> Self {
        Validator::Yield(validator)
    }

    /// 获取语句类型
    pub fn statement_type(&self) -> StatementType {
        match self {
            Validator::Create(_) => StatementType::Create,
            Validator::Delete(_) => StatementType::Delete,
            Validator::FetchEdges(_) => StatementType::FetchEdges,
            Validator::FetchVertices(_) => StatementType::FetchVertices,
            Validator::FindPath(_) => StatementType::FindPath,
            Validator::GetSubgraph(_) => StatementType::GetSubgraph,
            Validator::Go(_) => StatementType::Go,
            Validator::InsertEdges(_) => StatementType::InsertEdges,
            Validator::InsertVertices(_) => StatementType::InsertVertices,
            Validator::Limit(_) => StatementType::Limit,
            Validator::Lookup(_) => StatementType::Lookup,
            Validator::Match(_) => StatementType::Match,
            Validator::OrderBy(_) => StatementType::OrderBy,
            Validator::Pipe(_) => StatementType::Pipe,
            Validator::Sequential(_) => StatementType::Sequential,
            Validator::Set(_) => StatementType::Set,
            Validator::Update(_) => StatementType::Update,
            Validator::Unwind(_) => StatementType::Unwind,
            Validator::Use(_) => StatementType::Use,
            Validator::Yield(_) => StatementType::Yield,
        }
    }

    /// 根据 AST 语句创建对应的验证器
    pub fn from_stmt(stmt: &crate::query::parser::ast::Stmt) -> Option<Self> {
        use crate::query::parser::ast::Stmt;
        match stmt {
            Stmt::Match(_) => Some(Validator::Match(MatchValidator::new())),
            Stmt::Go(_) => Some(Validator::Go(GoValidator::new())),
            Stmt::Fetch(fetch_stmt) => {
                // 根据 FetchTarget 类型选择对应的验证器
                use crate::query::parser::ast::stmt::FetchTarget;
                match &fetch_stmt.target {
                    FetchTarget::Vertices { .. } => Some(Validator::FetchVertices(FetchVerticesValidator::new())),
                    FetchTarget::Edges { .. } => Some(Validator::FetchEdges(FetchEdgesValidator::new())),
                }
            }
            Stmt::Lookup(_) => Some(Validator::Lookup(LookupValidator::new())),
            Stmt::Subgraph(_) => Some(Validator::GetSubgraph(GetSubgraphValidator::new())),
            Stmt::FindPath(_) => Some(Validator::FindPath(FindPathValidator::new())),
            Stmt::Insert(insert_stmt) => {
                // 根据 InsertTarget 类型选择对应的验证器
                use crate::query::parser::ast::stmt::InsertTarget;
                match &insert_stmt.target {
                    InsertTarget::Vertices { .. } => Some(Validator::InsertVertices(InsertVerticesValidator::new())),
                    InsertTarget::Edge { .. } => Some(Validator::InsertEdges(InsertEdgesValidator::new())),
                }
            }
            Stmt::Update(_) => Some(Validator::Update(UpdateValidator::new())),
            Stmt::Delete(_) => Some(Validator::Delete(DeleteValidator::new())),
            Stmt::Create(_) => Some(Validator::Create(CreateValidator::new())),
            Stmt::Use(_) => Some(Validator::Use(UseValidator::new())),
            Stmt::Pipe(_) => Some(Validator::Pipe(PipeValidator::new())),
            Stmt::Yield(_) => Some(Validator::Yield(YieldValidator::new())),
            Stmt::Set(_) => Some(Validator::Set(SetValidator::new())),
            Stmt::Unwind(_) => Some(Validator::Unwind(UnwindValidator::new())),
            // 以下语句类型暂未实现专门的验证器，使用 SequentialValidator 作为占位
            // 后续应该为每种语句类型实现专门的验证器
            Stmt::Query(_) | 
            Stmt::Show(_) | 
            Stmt::Explain(_) | 
            Stmt::Profile(_) | 
            Stmt::GroupBy(_) | 
            Stmt::Merge(_) | 
            Stmt::Return(_) | 
            Stmt::With(_) | 
            Stmt::Remove(_) | 
            Stmt::Drop(_) | 
            Stmt::Desc(_) | 
            Stmt::Alter(_) |
            Stmt::CreateUser(_) |
            Stmt::AlterUser(_) |
            Stmt::DropUser(_) |
            Stmt::ChangePassword(_) |
            Stmt::Grant(_) |
            Stmt::Revoke(_) |
            Stmt::DescribeUser(_) |
            Stmt::ShowUsers(_) |
            Stmt::ShowRoles(_) |
            Stmt::ShowCreate(_) |
            Stmt::ShowSessions(_) |
            Stmt::ShowQueries(_) |
            Stmt::KillQuery(_) |
            Stmt::ShowConfigs(_) |
            Stmt::UpdateConfigs(_) |
            Stmt::Assignment(_) |
            Stmt::SetOperation(_) => {
                Some(Validator::Sequential(SequentialValidator::new()))
            }
        }
    }

    /// 执行验证
    pub fn validate(&mut self, ast: &mut AstContext) -> Result<ValidationResult, ValidationError> {
        forward_to_validator!(self, validate, ast)
    }

    /// 获取输入列
    pub fn inputs(&self) -> &[ColumnDef] {
        forward_to_validator!(self, inputs)
    }

    /// 获取输出列
    pub fn outputs(&self) -> &[ColumnDef] {
        forward_to_validator!(self, outputs)
    }

    /// 判断是否为全局语句
    pub fn is_global_statement(&self) -> bool {
        forward_to_validator!(self, is_global_statement)
    }

    /// 获取验证器名称
    pub fn validator_name(&self) -> String {
        forward_to_validator!(self, validator_name)
    }

    /// 获取表达式属性
    pub fn expression_props(&self) -> &ExpressionProps {
        forward_to_validator!(self, expression_props)
    }

    /// 获取用户定义变量列表
    pub fn user_defined_vars(&self) -> &[String] {
        forward_to_validator!(self, user_defined_vars)
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

/// 验证器工厂
/// 用于创建不同类型的验证器
pub struct ValidatorFactory;

impl ValidatorFactory {
    /// 根据语句类型创建对应的验证器
    pub fn create(stmt_type: StatementType) -> Option<Validator> {
        match stmt_type {
            StatementType::Create => Some(Validator::Create(CreateValidator::new())),
            StatementType::Delete => Some(Validator::Delete(DeleteValidator::new())),
            StatementType::FetchEdges => Some(Validator::FetchEdges(FetchEdgesValidator::new())),
            StatementType::FetchVertices => Some(Validator::FetchVertices(FetchVerticesValidator::new())),
            StatementType::FindPath => Some(Validator::FindPath(FindPathValidator::new())),
            StatementType::GetSubgraph => Some(Validator::GetSubgraph(GetSubgraphValidator::new())),
            StatementType::Go => Some(Validator::Go(GoValidator::new())),
            StatementType::InsertEdges => Some(Validator::InsertEdges(InsertEdgesValidator::new())),
            StatementType::InsertVertices => Some(Validator::InsertVertices(InsertVerticesValidator::new())),
            StatementType::Limit => Some(Validator::Limit(LimitValidator::new())),
            StatementType::Lookup => Some(Validator::Lookup(LookupValidator::new())),
            StatementType::Match => Some(Validator::Match(MatchValidator::new())),
            StatementType::OrderBy => Some(Validator::OrderBy(OrderByValidator::new())),
            StatementType::Pipe => Some(Validator::Pipe(PipeValidator::new())),
            StatementType::Sequential => Some(Validator::Sequential(SequentialValidator::new())),
            StatementType::Set => Some(Validator::Set(SetValidator::new())),
            StatementType::Update => Some(Validator::Update(UpdateValidator::new())),
            StatementType::Unwind => Some(Validator::Unwind(UnwindValidator::new())),
            StatementType::Use => Some(Validator::Use(UseValidator::new())),
            StatementType::Yield => Some(Validator::Yield(YieldValidator::new())),
            _ => None,
        }
    }

    /// 获取支持的语句类型列表
    pub fn supported_types() -> Vec<StatementType> {
        vec![
            StatementType::Create,
            StatementType::Delete,
            StatementType::FetchEdges,
            StatementType::FetchVertices,
            StatementType::FindPath,
            StatementType::GetSubgraph,
            StatementType::Go,
            StatementType::InsertEdges,
            StatementType::InsertVertices,
            StatementType::Limit,
            StatementType::Lookup,
            StatementType::Match,
            StatementType::OrderBy,
            StatementType::Pipe,
            StatementType::Sequential,
            StatementType::Set,
            StatementType::Update,
            StatementType::Unwind,
            StatementType::Use,
            StatementType::Yield,
        ]
    }
}

/// 验证器集合
/// 用于管理多个验证器
#[derive(Debug, Default)]
pub struct ValidatorCollection {
    validators: Vec<Validator>,
}

impl ValidatorCollection {
    /// 创建空的验证器集合
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    /// 添加验证器
    pub fn add(&mut self, validator: Validator) {
        self.validators.push(validator);
    }

    /// 获取验证器数量
    pub fn len(&self) -> usize {
        self.validators.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.validators.is_empty()
    }

    /// 获取指定索引的验证器
    pub fn get(&self, index: usize) -> Option<&Validator> {
        self.validators.get(index)
    }

    /// 获取指定索引的可变验证器
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Validator> {
        self.validators.get_mut(index)
    }

    /// 迭代器
    pub fn iter(&self) -> impl Iterator<Item = &Validator> {
        self.validators.iter()
    }

    /// 可变迭代器
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Validator> {
        self.validators.iter_mut()
    }

    /// 清空验证器集合
    pub fn clear(&mut self) {
        self.validators.clear();
    }

    /// 验证所有验证器
    pub fn validate_all(&mut self, ast: &mut AstContext) -> Result<Vec<ValidationResult>, ValidationError> {
        let mut results = Vec::new();
        for validator in &mut self.validators {
            let result = validator.validate(ast)?;
            results.push(result);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_factory_create() {
        assert!(ValidatorFactory::create(StatementType::Create).is_some());
        assert!(ValidatorFactory::create(StatementType::Match).is_some());
        assert!(ValidatorFactory::create(StatementType::Go).is_some());
        assert!(ValidatorFactory::create(StatementType::Pipe).is_some());
        assert!(ValidatorFactory::create(StatementType::Sequential).is_some());
    }

    #[test]
    fn test_validator_statement_type() {
        let create_validator = Validator::create(CreateValidator::new());
        assert_eq!(create_validator.statement_type(), StatementType::Create);

        let match_validator = Validator::match_(MatchValidator::new());
        assert_eq!(match_validator.statement_type(), StatementType::Match);

        let go_validator = Validator::go(GoValidator::new());
        assert_eq!(go_validator.statement_type(), StatementType::Go);

        let pipe_validator = Validator::pipe(PipeValidator::new());
        assert_eq!(pipe_validator.statement_type(), StatementType::Pipe);

        let sequential_validator = Validator::sequential(SequentialValidator::new());
        assert_eq!(sequential_validator.statement_type(), StatementType::Sequential);
    }

    #[test]
    fn test_validator_collection() {
        let mut collection = ValidatorCollection::new();
        assert!(collection.is_empty());

        collection.add(Validator::create(CreateValidator::new()));
        collection.add(Validator::match_(MatchValidator::new()));

        assert_eq!(collection.len(), 2);
        assert!(!collection.is_empty());

        let validator = collection.get(0);
        assert!(validator.is_some());
        assert_eq!(validator.unwrap().statement_type(), StatementType::Create);
    }
}
