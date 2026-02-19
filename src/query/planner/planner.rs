//! 规划器注册机制
//! 使用类型安全的枚举实现静态注册，完全消除动态分发

use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::planner::plan::ExecutionPlan;
use crate::query::planner::plan::SubPlan;
use crate::query::validator::validation_factory::StatementType;
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::time::Duration;
use parking_lot::Mutex;

use crate::query::planner::statements::fetch_edges_planner::FetchEdgesPlanner;
use crate::query::planner::statements::fetch_vertices_planner::FetchVerticesPlanner;
use crate::query::planner::statements::go_planner::GoPlanner;
use crate::query::planner::statements::insert_planner::InsertPlanner;
use crate::query::planner::statements::lookup_planner::LookupPlanner;
use crate::query::planner::statements::maintain_planner::MaintainPlanner;
use crate::query::planner::statements::match_statement_planner::MatchStatementPlanner;
use crate::query::planner::statements::path_planner::PathPlanner;
use crate::query::planner::statements::subgraph_planner::SubgraphPlanner;
use crate::query::planner::statements::user_management_planner::UserManagementPlanner;

/// 规划器配置
#[derive(Debug, Clone)]
pub struct PlannerConfig {
    pub enable_caching: bool,
    pub max_plan_depth: usize,
    pub enable_parallel_planning: bool,
    pub default_timeout: Duration,
    pub cache_size: usize,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            max_plan_depth: 100,
            enable_parallel_planning: false,
            default_timeout: Duration::from_secs(30),
            cache_size: 1000,
        }
    }
}

/// 计划缓存键
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PlanCacheKey {
    query_text: String,
    space_id: Option<i32>,
    statement_type: String,
}

impl PlanCacheKey {
    pub fn new(query_text: String, space_id: Option<i32>, statement_type: String) -> Self {
        Self {
            query_text,
            space_id,
            statement_type,
        }
    }
}

/// 计划缓存
#[derive(Debug)]
pub struct PlanCache {
    cache: Mutex<LruCache<PlanCacheKey, ExecutionPlan>>,
}

impl PlanCache {
    pub fn new(max_size: usize) -> Result<Self, PlannerError> {
        if max_size == 0 {
            return Err(PlannerError::InvalidOperation(
                "Plan cache size must be greater than 0".to_string(),
            ));
        }
        let cache_size = NonZeroUsize::new(max_size)
            .ok_or_else(|| PlannerError::InvalidOperation(
                "Failed to create plan cache with size".to_string(),
            ))?;
        Ok(Self {
            cache: Mutex::new(LruCache::new(cache_size)),
        })
    }

    pub fn get(&self, key: &PlanCacheKey) -> Result<Option<ExecutionPlan>, PlannerError> {
        let mut cache = self.cache.lock();
        Ok(cache.get(key).cloned())
    }

    pub fn insert(&self, key: PlanCacheKey, plan: ExecutionPlan) -> Result<(), PlannerError> {
        let mut cache = self.cache.lock();
        cache.push(key, plan);
        Ok(())
    }

    pub fn clear(&self) -> Result<(), PlannerError> {
        let mut cache = self.cache.lock();
        cache.clear();
        Ok(())
    }

    pub fn size(&self) -> Result<usize, PlannerError> {
        let cache = self.cache.lock();
        Ok(cache.len())
    }
}

/// 语句类型枚举（替代字符串）
#[derive(Debug, Clone, PartialEq, Hash, Eq, Copy)]
pub enum SentenceKind {
    Match,
    Go,
    Lookup,
    Path,
    Subgraph,
    FetchVertices,
    FetchEdges,
    Maintain,
    UserManagement,
    Create,
    Drop,
    Use,
    Show,
    Desc,
    Insert,
}

impl SentenceKind {
    /// 从字符串解析语句类型
    pub fn from_str(s: &str) -> Result<Self, PlannerError> {
        match s.to_uppercase().as_str() {
            "MATCH" => Ok(SentenceKind::Match),
            "GO" => Ok(SentenceKind::Go),
            "LOOKUP" => Ok(SentenceKind::Lookup),
            "PATH" | "FIND PATH" => Ok(SentenceKind::Path),
            "SUBGRAPH" => Ok(SentenceKind::Subgraph),
            "FETCH VERTICES" => Ok(SentenceKind::FetchVertices),
            "FETCH EDGES" => Ok(SentenceKind::FetchEdges),
            "MAINTAIN" => Ok(SentenceKind::Maintain),
            "CREATE_USER" | "ALTER_USER" | "DROP_USER" | "CHANGE_PASSWORD" |
            "CREATE USER" | "ALTER USER" | "DROP USER" | "CHANGE PASSWORD" => {
                Ok(SentenceKind::UserManagement)
            }
            "CREATE" => Ok(SentenceKind::Create),
            "DROP" => Ok(SentenceKind::Drop),
            "USE" => Ok(SentenceKind::Use),
            "SHOW" => Ok(SentenceKind::Show),
            "DESC" => Ok(SentenceKind::Desc),
            "INSERT" | "INSERT VERTEX" | "INSERT EDGE" => Ok(SentenceKind::Insert),
            _ => Err(PlannerError::UnsupportedOperation(format!(
                "Unsupported statement type: {}",
                s
            ))),
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            SentenceKind::Match => "MATCH",
            SentenceKind::Go => "GO",
            SentenceKind::Lookup => "LOOKUP",
            SentenceKind::Path => "PATH",
            SentenceKind::Subgraph => "SUBGRAPH",
            SentenceKind::FetchVertices => "FETCH VERTICES",
            SentenceKind::FetchEdges => "FETCH EDGES",
            SentenceKind::Maintain => "MAINTAIN",
            SentenceKind::UserManagement => "USER_MANAGEMENT",
            SentenceKind::Create => "CREATE",
            SentenceKind::Drop => "DROP",
            SentenceKind::Use => "USE",
            SentenceKind::Show => "SHOW",
            SentenceKind::Desc => "DESC",
            SentenceKind::Insert => "INSERT",
        }
    }

    /// 从 StatementType 转换到 SentenceKind
    /// 建立验证层和规划层之间的显式映射关系
    pub fn from_statement_type(stmt_type: &StatementType) -> Option<Self> {
        match stmt_type {
            StatementType::Match => Some(SentenceKind::Match),
            StatementType::Go => Some(SentenceKind::Go),
            StatementType::Lookup => Some(SentenceKind::Lookup),
            StatementType::FindPath => Some(SentenceKind::Path),
            StatementType::GetSubgraph => Some(SentenceKind::Subgraph),
            StatementType::FetchVertices => Some(SentenceKind::FetchVertices),
            StatementType::FetchEdges => Some(SentenceKind::FetchEdges),
            // INSERT 语句映射到 Insert
            StatementType::InsertVertices |
            StatementType::InsertEdges => Some(SentenceKind::Insert),
            // 其他DDL和DML操作映射到 Maintain
            StatementType::Update |
            StatementType::Delete |
            StatementType::CreateSpace |
            StatementType::CreateTag |
            StatementType::CreateEdge |
            StatementType::AlterTag |
            StatementType::AlterEdge |
            StatementType::DropSpace |
            StatementType::DropTag |
            StatementType::DropEdge |
            StatementType::DescribeSpace |
            StatementType::DescribeTag |
            StatementType::DescribeEdge |
            StatementType::ShowSpaces |
            StatementType::ShowTags |
            StatementType::ShowEdges => Some(SentenceKind::Maintain),
            // 以下类型没有对应的规划器，返回 None
            StatementType::Unwind |
            StatementType::Yield |
            StatementType::OrderBy |
            StatementType::Limit |
            StatementType::GroupBy |
            StatementType::Use |
            StatementType::Assignment |
            StatementType::Set |
            StatementType::Pipe |
            StatementType::Sequential |
            StatementType::Explain => None,
        }
    }
}

/// 匹配函数类型
pub type MatchFunc = fn(&AstContext) -> bool;

/// 静态匹配和实例化枚举 - 完全消除动态分发
#[derive(Debug, Clone)]
pub enum MatchAndInstantiateEnum {
    Match(MatchStatementPlanner),
    Go(GoPlanner),
    Lookup(LookupPlanner),
    Path(PathPlanner),
    Subgraph(SubgraphPlanner),
    FetchVertices(FetchVerticesPlanner),
    FetchEdges(FetchEdgesPlanner),
    Maintain(MaintainPlanner),
    UserManagement(UserManagementPlanner),
    Insert(InsertPlanner),
}

impl MatchAndInstantiateEnum {
    pub fn priority(&self) -> i32 {
        match self {
            MatchAndInstantiateEnum::Match(_) => 100,
            MatchAndInstantiateEnum::Go(_) => 100,
            MatchAndInstantiateEnum::Lookup(_) => 100,
            MatchAndInstantiateEnum::Path(_) => 100,
            MatchAndInstantiateEnum::Subgraph(_) => 100,
            MatchAndInstantiateEnum::FetchVertices(_) => 100,
            MatchAndInstantiateEnum::FetchEdges(_) => 100,
            MatchAndInstantiateEnum::Maintain(_) => 100,
            MatchAndInstantiateEnum::UserManagement(_) => 100,
            MatchAndInstantiateEnum::Insert(_) => 100,
        }
    }

    pub fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        match self {
            MatchAndInstantiateEnum::Match(planner) => planner.transform(ast_ctx),
            MatchAndInstantiateEnum::Go(planner) => planner.transform(ast_ctx),
            MatchAndInstantiateEnum::Lookup(planner) => planner.transform(ast_ctx),
            MatchAndInstantiateEnum::Path(planner) => planner.transform(ast_ctx),
            MatchAndInstantiateEnum::Subgraph(planner) => planner.transform(ast_ctx),
            MatchAndInstantiateEnum::FetchVertices(planner) => planner.transform(ast_ctx),
            MatchAndInstantiateEnum::FetchEdges(planner) => planner.transform(ast_ctx),
            MatchAndInstantiateEnum::Maintain(planner) => planner.transform(ast_ctx),
            MatchAndInstantiateEnum::UserManagement(planner) => planner.transform(ast_ctx),
            MatchAndInstantiateEnum::Insert(planner) => planner.transform(ast_ctx),
        }
    }

    pub fn transform_with_full_context(
        &mut self,
        _query_context: &mut QueryContext,
        ast_ctx: &AstContext,
    ) -> Result<ExecutionPlan, PlannerError> {
        let sub_plan = self.transform(ast_ctx)?;
        Ok(ExecutionPlan::new(sub_plan.root().clone()))
    }
}

/// 规划器特征（保持与原有接口兼容）
pub trait Planner: std::fmt::Debug {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError>;
    fn match_planner(&self, ast_ctx: &AstContext) -> bool;

    fn transform_with_full_context(
        &mut self,
        _query_context: &mut QueryContext,
        _ast_ctx: &AstContext,
    ) -> Result<ExecutionPlan, PlannerError> {
        let sub_plan = self.transform(_ast_ctx)?;
        Ok(ExecutionPlan::new(sub_plan.root().clone()))
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// 可配置的规划器注册表（静态版本）
#[derive(Debug)]
pub struct StaticConfigurablePlannerRegistry {
    planners: HashMap<SentenceKind, Vec<MatchAndInstantiateEnum>>,
    config: PlannerConfig,
    cache: Option<PlanCache>,
}

impl StaticConfigurablePlannerRegistry {
    pub fn new() -> Self {
        let cache = PlanCache::new(100)
            .unwrap_or_else(|_| PlanCache::new(1).expect("Failed to create plan cache with minimum size"));
        Self {
            planners: HashMap::new(),
            config: PlannerConfig::default(),
            cache: Some(cache),
        }
    }

    pub fn with_config(config: PlannerConfig) -> Self {
        let cache = if config.enable_caching {
            Some(PlanCache::new(config.cache_size)
                .unwrap_or_else(|_| PlanCache::new(100)
                    .expect("Failed to create plan cache with default size")))
        } else {
            None
        };
        Self {
            planners: HashMap::new(),
            config: config.clone(),
            cache,
        }
    }

    pub fn register(
        &mut self,
        sentence_kind: SentenceKind,
        planner: MatchAndInstantiateEnum,
    ) {
        self.planners
            .entry(sentence_kind)
            .or_default()
            .push(planner);

        if let Some(planners) = self.planners.get_mut(&sentence_kind) {
            planners.sort_by_key(|p| -p.priority());
        }
    }

    pub fn unregister_planners(&mut self, sentence_kind: &SentenceKind) {
        self.planners.remove(sentence_kind);
    }

    pub fn set_config(&mut self, config: PlannerConfig) {
        self.config = config.clone();
        if config.enable_caching && self.cache.is_none() {
            self.cache = Some(PlanCache::new(config.cache_size)
                .unwrap_or_else(|_| PlanCache::new(100)
                    .expect("Failed to create plan cache in set_config")));
        } else if !config.enable_caching {
            self.cache = None;
        }
    }

    pub fn config(&self) -> &PlannerConfig {
        &self.config
    }

    pub fn create_plan(
        &mut self,
        query_context: &mut QueryContext,
        ast_ctx: &AstContext,
    ) -> Result<ExecutionPlan, PlannerError> {
        let sentence_kind = self.extract_sentence_kind(ast_ctx)?;

        let cache_key = self.generate_cache_key(ast_ctx);

        if self.config.enable_caching {
            if let Some(ref cache) = self.cache {
                if let Ok(Some(cached_plan)) = cache.get(&cache_key) {
                    return Ok(cached_plan.clone());
                }
            }
        }

        let planners = self.planners.get_mut(&sentence_kind).ok_or_else(|| {
            PlannerError::NoSuitablePlanner(format!(
                "No planners registered for sentence kind: {:?}",
                sentence_kind
            ))
        })?;

        if let Some(first_planner) = planners.first_mut() {
            let plan = first_planner.transform_with_full_context(query_context, ast_ctx)?;

            if self.config.enable_caching {
                if let Some(ref cache) = self.cache {
                    let _ = cache.insert(cache_key.clone(), plan.clone());
                }
            }

            return Ok(plan);
        }

        Err(PlannerError::NoSuitablePlanner(
            "No suitable planner found for the given AST context".to_string(),
        ))
    }

    fn generate_cache_key(&self, ast_ctx: &AstContext) -> PlanCacheKey {
        let query_text = ast_ctx.query_text();
        let space_id = ast_ctx.space().space_id.map(|id| id as i32);
        let statement_type = ast_ctx.statement_type().to_string();

        PlanCacheKey::new(query_text, space_id, statement_type)
    }

    fn extract_sentence_kind(&self, ast_ctx: &AstContext) -> Result<SentenceKind, PlannerError> {
        if let Some(sentence) = ast_ctx.sentence() {
            let kind = SentenceKind::from_str(sentence.kind())?;
            // 将新的 SentenceKind 变体映射到 Maintain，以便使用相同的规划器
            match kind {
                SentenceKind::Create | SentenceKind::Drop | SentenceKind::Use | SentenceKind::Show | SentenceKind::Desc => {
                    Ok(SentenceKind::Maintain)
                }
                _ => Ok(kind),
            }
        } else {
            Err(PlannerError::InvalidAstContext(
                "Missing sentence in AST context".to_string(),
            ))
        }
    }

    pub fn planner_count(&self) -> usize {
        self.planners.values().map(|v| v.len()).sum()
    }

    pub fn has_planners_for(&self, sentence_kind: &SentenceKind) -> bool {
        self.planners.contains_key(sentence_kind)
    }

    pub fn cache_size(&self) -> usize {
        self.cache.as_ref().map_or(0, |c| c.size().unwrap_or(0))
    }

    pub fn clear_cache(&mut self) {
        if let Some(ref mut cache) = self.cache {
            let _ = cache.clear();
        }
    }

    pub fn is_caching_enabled(&self) -> bool {
        self.config.enable_caching
    }
}

// ============================================================================
// 静态注册实现 - 完全消除动态分发
// ============================================================================

/// 规划器枚举 - 静态分发核心
/// 完全消除 Box<dyn Planner> 动态分发，使用编译时多态
#[derive(Debug, Clone)]
pub enum PlannerEnum {
    Match(MatchStatementPlanner),
    Go(GoPlanner),
    Lookup(LookupPlanner),
    Path(PathPlanner),
    Subgraph(SubgraphPlanner),
    FetchVertices(FetchVerticesPlanner),
    FetchEdges(FetchEdgesPlanner),
    Maintain(MaintainPlanner),
    UserManagement(UserManagementPlanner),
    Insert(InsertPlanner),
}

impl PlannerEnum {
    /// 根据语句类型创建规划器
    pub fn from_sentence_kind(kind: SentenceKind) -> Option<Self> {
        match kind {
            SentenceKind::Match => Some(PlannerEnum::Match(MatchStatementPlanner::new())),
            SentenceKind::Go => Some(PlannerEnum::Go(GoPlanner::new())),
            SentenceKind::Lookup => Some(PlannerEnum::Lookup(LookupPlanner::new())),
            SentenceKind::Path => Some(PlannerEnum::Path(PathPlanner::new())),
            SentenceKind::Subgraph => Some(PlannerEnum::Subgraph(SubgraphPlanner::new())),
            SentenceKind::FetchVertices => Some(PlannerEnum::FetchVertices(FetchVerticesPlanner::new())),
            SentenceKind::FetchEdges => Some(PlannerEnum::FetchEdges(FetchEdgesPlanner::new())),
            SentenceKind::Maintain => Some(PlannerEnum::Maintain(MaintainPlanner::new())),
            SentenceKind::UserManagement => Some(PlannerEnum::UserManagement(UserManagementPlanner::new())),
            SentenceKind::Insert => Some(PlannerEnum::Insert(InsertPlanner::new())),
            // DDL/DML 操作使用 Maintain 规划器
            SentenceKind::Create | SentenceKind::Drop | SentenceKind::Use | SentenceKind::Show | SentenceKind::Desc => {
                Some(PlannerEnum::Maintain(MaintainPlanner::new()))
            }
        }
    }

    /// 将 AST 上下文转换为执行计划
    pub fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        match self {
            PlannerEnum::Match(planner) => planner.transform(ast_ctx),
            PlannerEnum::Go(planner) => planner.transform(ast_ctx),
            PlannerEnum::Lookup(planner) => planner.transform(ast_ctx),
            PlannerEnum::Path(planner) => planner.transform(ast_ctx),
            PlannerEnum::Subgraph(planner) => planner.transform(ast_ctx),
            PlannerEnum::FetchVertices(planner) => planner.transform(ast_ctx),
            PlannerEnum::FetchEdges(planner) => planner.transform(ast_ctx),
            PlannerEnum::Maintain(planner) => planner.transform(ast_ctx),
            PlannerEnum::UserManagement(planner) => planner.transform(ast_ctx),
            PlannerEnum::Insert(planner) => planner.transform(ast_ctx),
        }
    }

    /// 获取规划器名称
    pub fn name(&self) -> &'static str {
        match self {
            PlannerEnum::Match(_) => "MatchPlanner",
            PlannerEnum::Go(_) => "GoPlanner",
            PlannerEnum::Lookup(_) => "LookupPlanner",
            PlannerEnum::Path(_) => "PathPlanner",
            PlannerEnum::Subgraph(_) => "SubgraphPlanner",
            PlannerEnum::FetchVertices(_) => "FetchVerticesPlanner",
            PlannerEnum::FetchEdges(_) => "FetchEdgesPlanner",
            PlannerEnum::Maintain(_) => "MaintainPlanner",
            PlannerEnum::UserManagement(_) => "UserManagementPlanner",
            PlannerEnum::Insert(_) => "InsertPlanner",
        }
    }

    /// 检查是否匹配
    pub fn matches(&self, ast_ctx: &AstContext) -> bool {
        match self {
            PlannerEnum::Match(planner) => planner.match_planner(ast_ctx),
            PlannerEnum::Go(planner) => planner.match_planner(ast_ctx),
            PlannerEnum::Lookup(planner) => planner.match_planner(ast_ctx),
            PlannerEnum::Path(planner) => planner.match_planner(ast_ctx),
            PlannerEnum::Subgraph(planner) => planner.match_planner(ast_ctx),
            PlannerEnum::FetchVertices(planner) => planner.match_planner(ast_ctx),
            PlannerEnum::FetchEdges(planner) => planner.match_planner(ast_ctx),
            PlannerEnum::Maintain(planner) => planner.match_planner(ast_ctx),
            PlannerEnum::UserManagement(planner) => planner.match_planner(ast_ctx),
            PlannerEnum::Insert(planner) => planner.match_planner(ast_ctx),
        }
    }

    /// 转换为动态分发类型（用于向后兼容）
    pub fn into_dynamic(self) -> Box<dyn Planner> {
        match self {
            PlannerEnum::Match(planner) => Box::new(planner),
            PlannerEnum::Go(planner) => Box::new(planner),
            PlannerEnum::Lookup(planner) => Box::new(planner),
            PlannerEnum::Path(planner) => Box::new(planner),
            PlannerEnum::Subgraph(planner) => Box::new(planner),
            PlannerEnum::FetchVertices(planner) => Box::new(planner),
            PlannerEnum::FetchEdges(planner) => Box::new(planner),
            PlannerEnum::Maintain(planner) => Box::new(planner),
            PlannerEnum::UserManagement(planner) => Box::new(planner),
            PlannerEnum::Insert(planner) => Box::new(planner),
        }
    }
}

/// 静态规划器注册表
/// 编译时确定所有规划器，完全消除动态分发
#[derive(Debug, Default)]
pub struct StaticPlannerRegistry {
    planners: Vec<PlannerEnum>,
}

impl StaticPlannerRegistry {
    /// 创建注册表并注册所有规划器
    pub fn new() -> Self {
        Self {
            planners: vec![
                PlannerEnum::Match(MatchStatementPlanner::new()),
                PlannerEnum::Go(GoPlanner::new()),
                PlannerEnum::Lookup(LookupPlanner::new()),
                PlannerEnum::Path(PathPlanner::new()),
                PlannerEnum::Subgraph(SubgraphPlanner::new()),
                PlannerEnum::FetchVertices(FetchVerticesPlanner::new()),
                PlannerEnum::FetchEdges(FetchEdgesPlanner::new()),
                PlannerEnum::Maintain(MaintainPlanner::new()),
                PlannerEnum::UserManagement(UserManagementPlanner::new()),
            ],
        }
    }

    /// 获取规划器数量
    pub fn len(&self) -> usize {
        self.planners.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.planners.is_empty()
    }

    /// 根据语句类型获取规划器
    pub fn get(&self, kind: SentenceKind) -> Option<&PlannerEnum> {
        self.planners.iter().find(|p| p.name() == kind.as_str())
    }

    /// 获取可变的规划器
    pub fn get_mut(&mut self, kind: SentenceKind) -> Option<&mut PlannerEnum> {
        self.planners.iter_mut().find(|p| p.name() == kind.as_str())
    }

    /// 创建执行计划（使用静态分发）
    pub fn create_plan(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let kind = SentenceKind::from_str(ast_ctx.statement_type())
            .map_err(|_| PlannerError::NoSuitablePlanner("Unknown statement type".to_string()))?;

        if let Some(planner) = self.planners.iter_mut().find(|p| {
            p.name() == kind.as_str() && p.matches(ast_ctx)
        }) {
            return planner.transform(ast_ctx);
        }

        Err(PlannerError::NoSuitablePlanner(
            "No suitable planner found for the given AST context".to_string(),
        ))
    }

    /// 迭代所有规划器
    pub fn iter(&self) -> impl Iterator<Item = &PlannerEnum> {
        self.planners.iter()
    }

    /// 迭代所有规划器（可变）
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PlannerEnum> {
        self.planners.iter_mut()
    }
}

/// 便捷函数 - 创建规划器
pub fn create_planner(kind: SentenceKind) -> Option<PlannerEnum> {
    PlannerEnum::from_sentence_kind(kind)
}

/// 便捷函数 - 执行规划（使用静态注册）
pub fn plan(ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
    let mut registry = StaticPlannerRegistry::new();
    registry.create_plan(ast_ctx)
}

/// 静态注册版本的顺序规划器
#[derive(Debug, Default)]
pub struct StaticSequentialPlanner {
    registry: StaticPlannerRegistry,
}

impl StaticSequentialPlanner {
    pub fn new() -> Self {
        Self {
            registry: StaticPlannerRegistry::new(),
        }
    }

    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        let kind = SentenceKind::from_str(ast_ctx.statement_type());
        kind.is_ok()
    }

    pub fn create_plan(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        self.registry.create_plan(ast_ctx)
    }

    /// 转换 AST 上下文为计划（静态分发版本）
    pub fn to_plan(ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        plan(ast_ctx)
    }
}

/// 错误处理宏
///
/// 类似于 C++ 中的 NG_RETURN_IF_ERROR 宏，用于简化错误传播
#[macro_export]
macro_rules! ng_return_if_error {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => return Err(e.into()),
        }
    };
}

/// 错误处理宏变体，返回默认错误消息
///
/// 当表达式返回错误时，返回一个带有默认消息的 PlannerError
#[macro_export]
macro_rules! ng_ok_or_err {
    ($expr:expr, $msg:expr) => {
        match $expr {
            Ok(val) => val,
            Err(_) => return Err(PlannerError::PlanGenerationFailed($msg.to_string())),
        }
    };
}

/// 规划器错误类型
#[derive(Debug, thiserror::Error)]
pub enum PlannerError {
    #[error("No suitable planner found: {0}")]
    NoSuitablePlanner(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Plan generation failed: {0}")]
    PlanGenerationFailed(String),

    #[error("Join operation failed: {0}")]
    JoinFailed(String),

    #[error("Invalid AST context: {0}")]
    InvalidAstContext(String),

    #[error("Missing input: {0}")]
    MissingInput(String),

    #[error("Missing variable: {0}")]
    MissingVariable(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

// 为 DBError 实现 From 转换，以便在规划器中使用 ? 操作符
impl From<crate::core::error::DBError> for PlannerError {
    fn from(err: crate::core::error::DBError) -> Self {
        PlannerError::PlanGenerationFailed(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentence_kind_from_str() {
        assert_eq!(
            SentenceKind::from_str("MATCH").expect("Expected successful parsing of 'MATCH'"),
            SentenceKind::Match
        );
        assert_eq!(
            SentenceKind::from_str("match").expect("Expected successful parsing of 'match'"),
            SentenceKind::Match
        );
        assert_eq!(
            SentenceKind::from_str("GO").expect("Expected successful parsing of 'GO'"),
            SentenceKind::Go
        );
        assert_eq!(
            SentenceKind::from_str("FETCH VERTICES")
                .expect("Expected successful parsing of 'FETCH VERTICES'"),
            SentenceKind::FetchVertices
        );

        assert!(SentenceKind::from_str("INVALID").is_err());
    }

    #[test]
    fn test_sentence_kind_as_str() {
        assert_eq!(SentenceKind::Match.as_str(), "MATCH");
        assert_eq!(SentenceKind::Go.as_str(), "GO");
        assert_eq!(SentenceKind::FetchVertices.as_str(), "FETCH VERTICES");
    }

    #[test]
    fn test_match_and_instantiate() {
        let mi = MatchAndInstantiateEnum::Match(MatchStatementPlanner::new());
        assert_eq!(mi.priority(), 100);
    }

    #[test]
    fn test_planner_registry() {
        let registry = StaticPlannerRegistry::new();
        assert_eq!(registry.len(), 9);
    }

    #[test]
    fn test_sequential_planner() {
        let registry = StaticPlannerRegistry::new();
        assert_eq!(registry.len(), 9);
    }
}
