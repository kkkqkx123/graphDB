//! 新的规划器注册机制
//! 使用类型安全的枚举替代字符串匹配

use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::planner::plan::ExecutionPlan;
use crate::query::planner::plan::SubPlan;
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
    max_size: usize,
}

impl PlanCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(max_size).unwrap())),
            max_size,
        }
    }

    pub fn get(&self, key: &PlanCacheKey) -> Option<ExecutionPlan> {
        let mut cache = self.cache.lock().unwrap();
        cache.get(key).cloned()
    }

    pub fn insert(&self, key: PlanCacheKey, plan: ExecutionPlan) {
        let mut cache = self.cache.lock().unwrap();
        cache.push(key, plan);
    }

    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    pub fn size(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
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
}

impl SentenceKind {
    /// 从字符串解析语句类型
    pub fn from_str(s: &str) -> Result<Self, PlannerError> {
        match s.to_uppercase().as_str() {
            "MATCH" => Ok(SentenceKind::Match),
            "GO" => Ok(SentenceKind::Go),
            "LOOKUP" => Ok(SentenceKind::Lookup),
            "PATH" => Ok(SentenceKind::Path),
            "SUBGRAPH" => Ok(SentenceKind::Subgraph),
            "FETCH VERTICES" => Ok(SentenceKind::FetchVertices),
            "FETCH EDGES" => Ok(SentenceKind::FetchEdges),
            "MAINTAIN" => Ok(SentenceKind::Maintain),
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
        }
    }
}

/// 匹配函数类型
pub type MatchFunc = fn(&AstContext) -> bool;

/// 规划器实例化函数类型
pub type PlannerInstantiateFunc = fn() -> Box<dyn Planner>;

/// 匹配和实例化结构
#[derive(Debug, Clone)]
pub struct MatchAndInstantiate {
    pub match_func: MatchFunc,
    pub instantiate_func: PlannerInstantiateFunc,
    pub priority: i32, // 优先级，用于匹配冲突时选择
}

impl MatchAndInstantiate {
    pub fn new(
        match_func: MatchFunc,
        instantiate_func: PlannerInstantiateFunc,
        priority: i32,
    ) -> Self {
        Self {
            match_func,
            instantiate_func,
            priority,
        }
    }
}

/// 新的规划器注册表
#[derive(Debug)]
pub struct PlannerRegistry {
    planners: HashMap<SentenceKind, Vec<MatchAndInstantiate>>,
}

impl PlannerRegistry {
    pub fn new() -> Self {
        Self {
            planners: HashMap::new(),
        }
    }

    /// 注册规划器
    pub fn register_planner(
        &mut self,
        sentence_kind: SentenceKind,
        match_func: MatchFunc,
        instantiate_func: PlannerInstantiateFunc,
        priority: i32,
    ) {
        let match_and_instantiate = MatchAndInstantiate {
            match_func,
            instantiate_func,
            priority,
        };

        self.planners
            .entry(sentence_kind)
            .or_default()
            .push(match_and_instantiate);

        // 按优先级排序
        if let Some(planners) = self.planners.get_mut(&sentence_kind) {
            planners.sort_by_key(|p| -p.priority);
        }
    }

    /// 批量注册 MATCH 规划器
    pub fn register_match_planners(&mut self) {
        // 注册新的 MATCH 语句规划器
        self.register_planner(
            SentenceKind::Match,
            crate::query::planner::statements::match_statement_planner::MatchStatementPlanner::match_ast_ctx,
            || Box::new(crate::query::planner::statements::match_statement_planner::MatchStatementPlanner::new()) as Box<dyn Planner>,
            100,
        );
    }

    /// 批量注册 NGQL 规划器
    pub fn register_ngql_planners(&mut self) {
        self.register_planner(
            SentenceKind::Go,
            crate::query::planner::statements::go_planner::GoPlanner::match_ast_ctx,
            || Box::new(crate::query::planner::statements::go_planner::GoPlanner::new()) as Box<dyn Planner>,
            100,
        );

        self.register_planner(
            SentenceKind::Lookup,
            crate::query::planner::statements::lookup_planner::LookupPlanner::match_ast_ctx,
            || Box::new(crate::query::planner::statements::lookup_planner::LookupPlanner::new()) as Box<dyn Planner>,
            100,
        );

        self.register_planner(
            SentenceKind::Path,
            crate::query::planner::statements::path_planner::PathPlanner::match_ast_ctx,
            || Box::new(crate::query::planner::statements::path_planner::PathPlanner::new()) as Box<dyn Planner>,
            100,
        );

        self.register_planner(
            SentenceKind::Subgraph,
            crate::query::planner::statements::subgraph_planner::SubgraphPlanner::match_ast_ctx,
            || Box::new(crate::query::planner::statements::subgraph_planner::SubgraphPlanner::new()) as Box<dyn Planner>,
            100,
        );

        self.register_planner(
            SentenceKind::FetchVertices,
            crate::query::planner::statements::fetch_vertices_planner::FetchVerticesPlanner::match_ast_ctx,
            || Box::new(crate::query::planner::statements::fetch_vertices_planner::FetchVerticesPlanner::new()) as Box<dyn Planner>,
            100,
        );

        self.register_planner(
            SentenceKind::FetchEdges,
            crate::query::planner::statements::fetch_edges_planner::FetchEdgesPlanner::match_ast_ctx,
            || Box::new(crate::query::planner::statements::fetch_edges_planner::FetchEdgesPlanner::new()) as Box<dyn Planner>,
            100,
        );

        self.register_planner(
            SentenceKind::Maintain,
            crate::query::planner::statements::maintain_planner::MaintainPlanner::match_ast_ctx,
            || Box::new(crate::query::planner::statements::maintain_planner::MaintainPlanner::new()) as Box<dyn Planner>,
            100,
        );
    }

    /// 创建执行计划
    pub fn create_plan(&self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let sentence_kind = self.extract_sentence_kind(ast_ctx)?;

        let planners = self.planners.get(&sentence_kind).ok_or_else(|| {
            PlannerError::NoSuitablePlanner(format!(
                "No planners registered for sentence kind: {:?}",
                sentence_kind
            ))
        })?;

        for planner_info in planners {
            if (planner_info.match_func)(ast_ctx) {
                let mut planner = (planner_info.instantiate_func)();
                return planner.transform(ast_ctx);
            }
        }

        Err(PlannerError::NoSuitablePlanner(
            "No suitable planner found for the given AST context".to_string(),
        ))
    }

    /// 从 AST 上下文提取语句类型
    fn extract_sentence_kind(&self, ast_ctx: &AstContext) -> Result<SentenceKind, PlannerError> {
        // 直接从 AST 节点获取语句类型
        if let Some(sentence) = ast_ctx.sentence() {
            SentenceKind::from_str(sentence.kind())
        } else {
            Err(PlannerError::InvalidAstContext(
                "Missing sentence in AST context".to_string(),
            ))
        }
    }

    /// 获取已注册的规划器数量
    pub fn planner_count(&self) -> usize {
        self.planners.values().map(|v| v.len()).sum()
    }

    /// 检查是否有指定类型的规划器
    pub fn has_planners_for(&self, sentence_kind: &SentenceKind) -> bool {
        self.planners.contains_key(sentence_kind)
    }

    /// 获取指定类型的规划器数量
    pub fn planner_count_for(&self, sentence_kind: &SentenceKind) -> usize {
        self.planners
            .get(sentence_kind)
            .map(|v| v.len())
            .unwrap_or(0)
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

/// 顺序规划器（使用新的注册机制）
#[derive(Debug)]
pub struct SequentialPlanner {}

impl SequentialPlanner {
    pub fn new() -> Self {
        Self {}
    }

    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    pub fn match_ast_ctx(_ast_ctx: &AstContext) -> bool {
        // 对于顺序规划器，通常匹配任何语句
        true
    }

    /// 转换 AST 上下文为计划（类似于原始的 toPlan 方法）
    pub fn to_plan(ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let mut registry = PlannerRegistry::new();
        Self::register_planners(&mut registry);
        registry.create_plan(ast_ctx)
    }

    /// 注册所有可用的规划器
    pub fn register_planners(registry: &mut PlannerRegistry) {
        registry.register_match_planners();
        registry.register_ngql_planners();
    }

    /// 移除左侧尾部起始节点
    ///
    /// 当追加计划时，需要移除左侧尾部 Start 节点。
    /// 这是因为左侧尾部的 Start 节点需要被移除，并保留一个位置用于添加依赖关系。
    ///
    /// TODO: 这是临时解决方案，在实现逐个执行多个序列后应移除
    pub fn rm_left_tail_start_node(plan: &mut SubPlan) {
        let tail = match &plan.tail {
            Some(t) => t,
            None => return,
        };

        if !tail.is_start() {
            return;
        }

        let root = match &plan.root {
            Some(r) => r,
            None => return,
        };

        let mut current = root.clone();
        let mut found = false;

        while let Some(first_dep) = current.first_dependency() {
            if first_dep.dependencies().is_empty() {
                found = true;
                break;
            }
            current = first_dep;
        }

        if found {
            plan.tail = Some(current);
        }
    }
}

impl Planner for SequentialPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        Self::to_plan(ast_ctx)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

/// 可配置的规划器注册表
#[derive(Debug)]
pub struct ConfigurablePlannerRegistry {
    planners: HashMap<SentenceKind, Vec<MatchAndInstantiate>>,
    config: PlannerConfig,
    cache: Option<PlanCache>,
}

impl ConfigurablePlannerRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            planners: HashMap::new(),
            config: PlannerConfig::default(),
            cache: None,
        };

        registry.cache = Some(PlanCache::new(registry.config.cache_size));

        registry
    }

    pub fn with_config(config: PlannerConfig) -> Self {
        let mut registry = Self {
            planners: HashMap::new(),
            config: config.clone(),
            cache: None,
        };

        if config.enable_caching {
            registry.cache = Some(PlanCache::new(config.cache_size));
        }

        registry
    }

    pub fn register_planner(
        &mut self,
        sentence_kind: SentenceKind,
        match_func: MatchFunc,
        instantiate_func: PlannerInstantiateFunc,
        priority: i32,
    ) {
        let match_and_instantiate = MatchAndInstantiate {
            match_func,
            instantiate_func,
            priority,
        };

        self.planners
            .entry(sentence_kind)
            .or_default()
            .push(match_and_instantiate);

        if let Some(planners) = self.planners.get_mut(&sentence_kind) {
            planners.sort_by_key(|p| -p.priority);
        }
    }

    pub fn unregister_planners(&mut self, sentence_kind: &SentenceKind) {
        self.planners.remove(sentence_kind);
    }

    pub fn set_config(&mut self, config: PlannerConfig) {
        self.config = config.clone();
        if config.enable_caching && self.cache.is_none() {
            self.cache = Some(PlanCache::new(config.cache_size));
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

        let planners = self.planners.get(&sentence_kind).ok_or_else(|| {
            PlannerError::NoSuitablePlanner(format!(
                "No planners registered for sentence kind: {:?}",
                sentence_kind
            ))
        })?;

        let cache_key = self.generate_cache_key(ast_ctx);

        if self.config.enable_caching {
            if let Some(ref cache) = self.cache {
                if let Some(cached_plan) = cache.get(&cache_key) {
                    return Ok(cached_plan.clone());
                }
            }
        }

        for planner_info in planners {
            if (planner_info.match_func)(ast_ctx) {
                let mut planner = (planner_info.instantiate_func)();
                let plan = planner.transform_with_full_context(query_context, ast_ctx)?;

                if self.config.enable_caching {
                    if let Some(ref mut cache) = self.cache {
                        cache.insert(cache_key.clone(), plan.clone());
                    }
                }

                return Ok(plan);
            }
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
            SentenceKind::from_str(sentence.kind())
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
        self.cache.as_ref().map_or(0, |c| c.size())
    }

    pub fn clear_cache(&mut self) {
        if let Some(ref mut cache) = self.cache {
            cache.clear();
        }
    }

    pub fn is_caching_enabled(&self) -> bool {
        self.config.enable_caching
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

    #[error("Not implemented: {0}")]
    NotImplemented(String),
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
        fn dummy_match(_ast_ctx: &AstContext) -> bool {
            true
        }

        fn dummy_instantiate() -> Box<dyn Planner> {
            SequentialPlanner::make()
        }

        let mi = MatchAndInstantiate::new(dummy_match, dummy_instantiate, 100);
        assert_eq!(mi.priority, 100);
    }

    #[test]
    fn test_planner_registry() {
        let mut registry = PlannerRegistry::new();
        assert_eq!(registry.planner_count(), 0);

        // 测试注册规划器
        fn dummy_match(_ast_ctx: &AstContext) -> bool {
            true
        }

        fn dummy_instantiate() -> Box<dyn Planner> {
            SequentialPlanner::make()
        }

        registry.register_planner(SentenceKind::Match, dummy_match, dummy_instantiate, 100);

        assert_eq!(registry.planner_count(), 1);
        assert!(registry.has_planners_for(&SentenceKind::Match));
        assert_eq!(registry.planner_count_for(&SentenceKind::Match), 1);
        assert!(!registry.has_planners_for(&SentenceKind::Go));
        assert_eq!(registry.planner_count_for(&SentenceKind::Go), 0);
    }

    #[test]
    fn test_sequential_planner() {
        let _planner = SequentialPlanner::new();
        assert!(SequentialPlanner::match_ast_ctx(&AstContext::from_strings(
            "test", "test"
        )));
    }
}
