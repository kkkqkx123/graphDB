//! 规划器注册机制
//! 使用类型安全的枚举实现静态注册，完全消除动态分发
//!

use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::query::QueryContext;
use crate::query::parser::ast::Stmt;
use crate::query::planner::plan::ExecutionPlan;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::template_extractor::TemplateExtractor;
use crate::query::validator::StatementType;
use lru::LruCache;
use parking_lot::Mutex;

use crate::query::planner::statements::delete_planner::DeletePlanner;
use crate::query::planner::statements::fetch_edges_planner::FetchEdgesPlanner;
use crate::query::planner::statements::fetch_vertices_planner::FetchVerticesPlanner;
use crate::query::planner::statements::go_planner::GoPlanner;
use crate::query::planner::statements::group_by_planner::GroupByPlanner;
use crate::query::planner::statements::insert_planner::InsertPlanner;
use crate::query::planner::statements::lookup_planner::LookupPlanner;
use crate::query::planner::statements::maintain_planner::MaintainPlanner;
use crate::query::planner::statements::match_statement_planner::MatchStatementPlanner;
use crate::query::planner::statements::path_planner::PathPlanner;
use crate::query::planner::statements::set_operation_planner::SetOperationPlanner;
use crate::query::planner::statements::subgraph_planner::SubgraphPlanner;
use crate::query::planner::statements::update_planner::UpdatePlanner;
use crate::query::planner::statements::use_planner::UsePlanner;
use crate::query::planner::statements::user_management_planner::UserManagementPlanner;
use crate::query::planner::rewrite::{rewrite_plan, RewriteError};

/// 规划器配置
#[derive(Debug, Clone)]
pub struct PlannerConfig {
    pub enable_caching: bool,
    pub max_plan_depth: usize,
    pub enable_parallel_planning: bool,
    pub default_timeout: Duration,
    pub cache_size: usize,
    /// 启用计划重写优化
    pub enable_rewrite: bool,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            max_plan_depth: 100,
            enable_parallel_planning: false,
            default_timeout: Duration::from_secs(30),
            cache_size: 1000,
            enable_rewrite: true,
        }
    }
}

/// 计划缓存键
/// 支持参数化查询缓存，将具体参数值替换为占位符
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PlanCacheKey {
    /// 查询模板（参数化后的 SQL）
    query_template: String,
    /// 图空间 ID
    space_id: Option<i32>,
    /// 语句类型
    statement_type: SentenceKind,
    /// 模式指纹（用于 MATCH 查询的结构识别）
    pattern_fingerprint: Option<String>,
}

impl PlanCacheKey {
    /// 创建新的缓存键
    pub fn new(
        query_template: String,
        space_id: Option<i32>,
        statement_type: SentenceKind,
        pattern_fingerprint: Option<String>,
    ) -> Self {
        Self {
            query_template,
            space_id,
            statement_type,
            pattern_fingerprint,
        }
    }

    /// 从语句创建缓存键
    pub fn from_stmt(stmt: &Stmt, space_id: Option<i32>) -> Result<Self, PlannerError> {
        let statement_type = SentenceKind::from_stmt(stmt)?;
        let query_template = Self::extract_template(stmt);
        let pattern_fingerprint = Self::generate_fingerprint(stmt);

        Ok(Self {
            query_template,
            space_id,
            statement_type,
            pattern_fingerprint,
        })
    }

    /// 提取查询模板（参数化）
    /// 将具体参数值替换为占位符，使相似查询共享缓存
    fn extract_template(stmt: &Stmt) -> String {
        TemplateExtractor::extract(stmt)
    }

    /// 生成模式指纹
    /// 用于识别查询的结构特征
    fn generate_fingerprint(stmt: &Stmt) -> Option<String> {
        match stmt {
            Stmt::Match(m) => {
                // 提取模式结构作为指纹
                let pattern_count = m.patterns.len();
                let has_where = m.where_clause.is_some();
                let has_return = m.return_clause.is_some();
                Some(format!("M:{}:W{}:R{}", pattern_count, has_where as u8, has_return as u8))
            }
            Stmt::Go(g) => {
                // 提取步数特征
                let step_str = match &g.steps {
                    crate::query::parser::ast::Steps::Fixed(n) => format!("F{}", n),
                    crate::query::parser::ast::Steps::Range { min, max } => format!("R{}-{}", min, max),
                    crate::query::parser::ast::Steps::Variable(_) => "V".to_string(),
                };
                Some(format!("G:{}:S{}", step_str, g.over.as_ref().map(|_| "E").unwrap_or("N")))
            }
            _ => None,
        }
    }
}

/// 缓存的计划项
/// 包含执行计划和元数据
#[derive(Debug, Clone)]
pub struct CachedPlan {
    /// 执行计划
    pub plan: ExecutionPlan,
    /// 创建时间
    pub created_at: Instant,
    /// 最后访问时间
    pub last_accessed: Instant,
    /// 访问次数
    pub access_count: u64,
    /// 计划成本估算
    pub estimated_cost: f64,
}

impl CachedPlan {
    /// 创建新的缓存项
    pub fn new(plan: ExecutionPlan, estimated_cost: f64) -> Self {
        let now = Instant::now();
        Self {
            plan,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            estimated_cost,
        }
    }

    /// 记录访问
    pub fn record_access(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }

    /// 计算缓存价值分数（用于淘汰策略）
    /// 分数越高越应该保留
    pub fn value_score(&self) -> f64 {
        let age_secs = self.created_at.elapsed().as_secs() as f64;
        let recency = 1.0 / (1.0 + age_secs / 3600.0); // 1小时内衰减

        let frequency = (self.access_count as f64).ln_1p();
        let cost_savings = self.estimated_cost.max(1.0);

        recency * frequency * cost_savings
    }
}

/// 计划缓存统计
#[derive(Debug, Clone, Default)]
pub struct PlanCacheStats {
    /// 命中次数
    pub hits: u64,
    /// 未命中次数
    pub misses: u64,
    /// 插入次数
    pub inserts: u64,
    /// 淘汰次数
    pub evictions: u64,
}

impl PlanCacheStats {
    /// 总查询次数
    pub fn total_queries(&self) -> u64 {
        self.hits + self.misses
    }

    /// 命中率
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_queries();
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// 计划缓存配置
#[derive(Debug, Clone)]
pub struct PlanCacheConfig {
    /// 最大缓存条目数
    pub max_entries: usize,
    /// 条目最大存活时间（秒）
    pub ttl_seconds: u64,
    /// 启用统计
    pub enable_stats: bool,
    /// 最小执行时间阈值（微秒）- 低于此值不缓存
    pub min_execution_time_us: u64,
    /// 最大计划复杂度（节点数）
    pub max_plan_nodes: usize,
    /// 最小节点数才缓存（太简单的查询不缓存）
    pub min_plan_nodes: usize,
}

impl Default for PlanCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            ttl_seconds: 3600, // 1小时
            enable_stats: true,
            min_execution_time_us: 1000, // 1ms
            max_plan_nodes: 100,
            min_plan_nodes: 3,
        }
    }
}

/// 计划缓存
#[derive(Debug)]
pub struct PlanCache {
    cache: Mutex<LruCache<PlanCacheKey, CachedPlan>>,
    stats: Mutex<PlanCacheStats>,
    config: PlanCacheConfig,
}

impl PlanCache {
    /// 创建新的计划缓存
    pub fn new(config: PlanCacheConfig) -> Result<Self, PlannerError> {
        if config.max_entries == 0 {
            return Err(PlannerError::InvalidOperation(
                "Plan cache size must be greater than 0".to_string(),
            ));
        }
        let cache_size = NonZeroUsize::new(config.max_entries)
            .ok_or_else(|| PlannerError::InvalidOperation(
                "Failed to create plan cache with size".to_string(),
            ))?;
        Ok(Self {
            cache: Mutex::new(LruCache::new(cache_size)),
            stats: Mutex::new(PlanCacheStats::default()),
            config,
        })
    }

    /// 使用默认配置创建
    pub fn with_default_config() -> Result<Self, PlannerError> {
        Self::new(PlanCacheConfig::default())
    }

    /// 获取缓存的计划
    pub fn get(&self, key: &PlanCacheKey) -> Result<Option<ExecutionPlan>, PlannerError> {
        let mut cache = self.cache.lock();
        
        if let Some(cached) = cache.get_mut(key) {
            // 检查 TTL
            if self.is_expired(cached) {
                cache.pop(key);
                drop(cache);
                self.record_miss();
                return Ok(None);
            }
            
            cached.record_access();
            let plan = cached.plan.clone();
            drop(cache);
            self.record_hit();
            Ok(Some(plan))
        } else {
            drop(cache);
            self.record_miss();
            Ok(None)
        }
    }

    /// 插入计划到缓存
    pub fn insert(
        &self,
        key: PlanCacheKey,
        plan: ExecutionPlan,
        estimated_cost: f64,
    ) -> Result<(), PlannerError> {
        // 检查是否应该缓存
        if !self.should_cache(&plan) {
            return Ok(());
        }

        let cached = CachedPlan::new(plan, estimated_cost);
        let mut cache = self.cache.lock();
        
        // 检查是否发生淘汰
        if cache.len() >= self.config.max_entries && !cache.contains(&key) {
            self.record_eviction();
        }
        
        cache.push(key, cached);
        drop(cache);
        
        self.record_insert();
        Ok(())
    }

    /// 从语句和计划创建缓存项
    pub fn insert_from_stmt(
        &self,
        stmt: &Stmt,
        space_id: Option<i32>,
        plan: ExecutionPlan,
        estimated_cost: f64,
    ) -> Result<(), PlannerError> {
        let key = PlanCacheKey::from_stmt(stmt, space_id)?;
        self.insert(key, plan, estimated_cost)
    }

    /// 移除缓存项
    pub fn remove(&self, key: &PlanCacheKey) -> Result<(), PlannerError> {
        let mut cache = self.cache.lock();
        cache.pop(key);
        Ok(())
    }

    /// 清空缓存
    pub fn clear(&self) -> Result<(), PlannerError> {
        let mut cache = self.cache.lock();
        cache.clear();
        
        let mut stats = self.stats.lock();
        *stats = PlanCacheStats::default();
        
        Ok(())
    }

    /// 获取缓存大小
    pub fn size(&self) -> usize {
        let cache = self.cache.lock();
        cache.len()
    }

    /// 获取统计信息
    pub fn stats(&self) -> PlanCacheStats {
        let stats = self.stats.lock();
        stats.clone()
    }

    /// 检查计划是否应该被缓存
    fn should_cache(&self, plan: &ExecutionPlan) -> bool {
        let node_count = plan.node_count();
        
        // 太简单的查询不缓存
        if node_count < self.config.min_plan_nodes {
            return false;
        }
        
        // 太复杂的查询可能是 adhoc，不缓存
        if node_count > self.config.max_plan_nodes {
            return false;
        }
        
        true
    }

    /// 检查缓存项是否过期
    fn is_expired(&self, cached: &CachedPlan) -> bool {
        cached.created_at.elapsed().as_secs() > self.config.ttl_seconds
    }

    /// 记录命中
    fn record_hit(&self) {
        if self.config.enable_stats {
            let mut stats = self.stats.lock();
            stats.hits += 1;
        }
    }

    /// 记录未命中
    fn record_miss(&self) {
        if self.config.enable_stats {
            let mut stats = self.stats.lock();
            stats.misses += 1;
        }
    }

    /// 记录插入
    fn record_insert(&self) {
        if self.config.enable_stats {
            let mut stats = self.stats.lock();
            stats.inserts += 1;
        }
    }

    /// 记录淘汰
    fn record_eviction(&self) {
        if self.config.enable_stats {
            let mut stats = self.stats.lock();
            stats.evictions += 1;
        }
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
    Delete,
    Update,
    GroupBy,
    SetOperation,
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
            "DELETE" => Ok(SentenceKind::Delete),
            "UPDATE" => Ok(SentenceKind::Update),
            "GROUP BY" => Ok(SentenceKind::GroupBy),
            "SET OPERATION" | "UNION" | "UNION ALL" | "INTERSECT" | "MINUS" => Ok(SentenceKind::SetOperation),
            "SHOW" => Ok(SentenceKind::Show),
            "DESC" => Ok(SentenceKind::Desc),
            "INSERT" | "INSERT VERTEX" | "INSERT EDGE" => Ok(SentenceKind::Insert),
            _ => Err(PlannerError::UnsupportedOperation(format!(
                "Unsupported statement type: {}",
                s
            ))),
        }
    }

    /// 从语句枚举解析语句类型
    pub fn from_stmt(stmt: &Stmt) -> Result<Self, PlannerError> {
        match stmt.kind().to_uppercase().as_str() {
            "MATCH" => Ok(SentenceKind::Match),
            "GO" => Ok(SentenceKind::Go),
            "LOOKUP" => Ok(SentenceKind::Lookup),
            "FIND PATH" => Ok(SentenceKind::Path),
            "SUBGRAPH" => Ok(SentenceKind::Subgraph),
            "FETCH" => {
                if let Stmt::Fetch(fetch_stmt) = stmt {
                    match &fetch_stmt.target {
                        crate::query::parser::ast::FetchTarget::Vertices { .. } => Ok(SentenceKind::FetchVertices),
                        crate::query::parser::ast::FetchTarget::Edges { .. } => Ok(SentenceKind::FetchEdges),
                    }
                } else {
                    Err(PlannerError::UnsupportedOperation("Invalid FETCH statement".to_string()))
                }
            }
            "CREATE USER" | "ALTER USER" | "DROP USER" | "CHANGE PASSWORD" => Ok(SentenceKind::UserManagement),
            "CREATE" => Ok(SentenceKind::Create),
            "DROP" => Ok(SentenceKind::Drop),
            "USE" => Ok(SentenceKind::Use),
            "DELETE" => Ok(SentenceKind::Delete),
            "UPDATE" => Ok(SentenceKind::Update),
            "GROUP BY" => Ok(SentenceKind::GroupBy),
            "SET OPERATION" => Ok(SentenceKind::SetOperation),
            "SHOW" => Ok(SentenceKind::Show),
            "DESC" => Ok(SentenceKind::Desc),
            "INSERT" => Ok(SentenceKind::Insert),
            _ => Err(PlannerError::UnsupportedOperation(format!(
                "Unsupported statement type: {}",
                stmt.kind()
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
            SentenceKind::Delete => "DELETE",
            SentenceKind::Update => "UPDATE",
            SentenceKind::GroupBy => "GROUP BY",
            SentenceKind::SetOperation => "SET OPERATION",
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
            // DELETE 和 UPDATE 有独立的规划器
            StatementType::Delete => Some(SentenceKind::Delete),
            StatementType::Update => Some(SentenceKind::Update),
            // GROUP BY 有独立的规划器
            StatementType::GroupBy => Some(SentenceKind::GroupBy),
            // USE 有独立的规划器
            StatementType::Use => Some(SentenceKind::Use),
            // 集合操作有独立的规划器
            StatementType::SetOperation => Some(SentenceKind::SetOperation),
            // 其他DDL和DML操作映射到 Maintain
            StatementType::Create |
            StatementType::CreateSpace |
            StatementType::CreateTag |
            StatementType::CreateEdge |
            StatementType::Alter |
            StatementType::AlterTag |
            StatementType::AlterEdge |
            StatementType::Drop |
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
            StatementType::Assignment |
            StatementType::Set |
            StatementType::Pipe |
            StatementType::Sequential |
            StatementType::Explain |
            StatementType::Profile |
            StatementType::Query |
            StatementType::Merge |
            StatementType::Return |
            StatementType::With |
            StatementType::Remove |
            StatementType::UpdateConfigs |
            StatementType::Show |
            StatementType::Desc |
            StatementType::ShowCreate |
            StatementType::ShowConfigs |
            StatementType::ShowSessions |
            StatementType::ShowQueries |
            StatementType::KillQuery |
            StatementType::CreateUser |
            StatementType::DropUser |
            StatementType::AlterUser |
            StatementType::Grant |
            StatementType::Revoke |
            StatementType::ChangePassword |
            StatementType::DescribeUser |
            StatementType::ShowUsers |
            StatementType::ShowRoles => None,
        }
    }
}

/// 匹配函数类型
///
/// # 重构变更
/// - 使用 &Stmt 替代 &AstContext
pub type MatchFunc = fn(&Stmt) -> bool;

/// 规划器特征（重构后接口）
///
/// # 重构变更
/// - transform 方法接收 Arc<QueryContext> 和 &Stmt 替代 &AstContext
/// - match_planner 方法接收 &Stmt 替代 &AstContext
pub trait Planner: std::fmt::Debug {
    fn transform(
        &mut self,
        stmt: &Stmt,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError>;
    fn match_planner(&self, stmt: &Stmt) -> bool;

    fn transform_with_full_context(
        &mut self,
        qctx: Arc<QueryContext>,
        stmt: &Stmt,
    ) -> Result<ExecutionPlan, PlannerError> {
        let sub_plan = self.transform(stmt, qctx)?;
        let plan = ExecutionPlan::new(sub_plan.root().clone());
        
        // 应用计划重写优化
        let plan = rewrite_plan(plan)?;
        
        Ok(plan)
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
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
    Delete(DeletePlanner),
    Update(UpdatePlanner),
    GroupBy(GroupByPlanner),
    SetOperation(SetOperationPlanner),
    Use(UsePlanner),
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
            SentenceKind::Delete => Some(PlannerEnum::Delete(DeletePlanner::new())),
            SentenceKind::Update => Some(PlannerEnum::Update(UpdatePlanner::new())),
            SentenceKind::GroupBy => Some(PlannerEnum::GroupBy(GroupByPlanner::new())),
            SentenceKind::SetOperation => Some(PlannerEnum::SetOperation(SetOperationPlanner::new())),
            SentenceKind::Use => Some(PlannerEnum::Use(UsePlanner::new())),
            // DDL/DML 操作使用 Maintain 规划器
            SentenceKind::Create | SentenceKind::Drop | SentenceKind::Show | SentenceKind::Desc => {
                Some(PlannerEnum::Maintain(MaintainPlanner::new()))
            }
        }
    }

    /// 将语句转换为执行计划
    pub fn transform(&mut self, stmt: &Stmt, qctx: Arc<QueryContext>) -> Result<SubPlan, PlannerError> {
        match self {
            PlannerEnum::Match(planner) => planner.transform(stmt, qctx),
            PlannerEnum::Go(planner) => planner.transform(stmt, qctx),
            PlannerEnum::Lookup(planner) => planner.transform(stmt, qctx),
            PlannerEnum::Path(planner) => planner.transform(stmt, qctx),
            PlannerEnum::Subgraph(planner) => planner.transform(stmt, qctx),
            PlannerEnum::FetchVertices(planner) => planner.transform(stmt, qctx),
            PlannerEnum::FetchEdges(planner) => planner.transform(stmt, qctx),
            PlannerEnum::Maintain(planner) => planner.transform(stmt, qctx),
            PlannerEnum::UserManagement(planner) => planner.transform(stmt, qctx),
            PlannerEnum::Insert(planner) => planner.transform(stmt, qctx),
            PlannerEnum::Delete(planner) => planner.transform(stmt, qctx),
            PlannerEnum::Update(planner) => planner.transform(stmt, qctx),
            PlannerEnum::GroupBy(planner) => planner.transform(stmt, qctx),
            PlannerEnum::SetOperation(planner) => planner.transform(stmt, qctx),
            PlannerEnum::Use(planner) => planner.transform(stmt, qctx),
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
            PlannerEnum::Delete(_) => "DeletePlanner",
            PlannerEnum::Update(_) => "UpdatePlanner",
            PlannerEnum::GroupBy(_) => "GroupByPlanner",
            PlannerEnum::SetOperation(_) => "SetOperationPlanner",
            PlannerEnum::Use(_) => "UsePlanner",
        }
    }

    /// 检查是否匹配
    pub fn matches(&self, stmt: &Stmt) -> bool {
        match self {
            PlannerEnum::Match(planner) => planner.match_planner(stmt),
            PlannerEnum::Go(planner) => planner.match_planner(stmt),
            PlannerEnum::Lookup(planner) => planner.match_planner(stmt),
            PlannerEnum::Path(planner) => planner.match_planner(stmt),
            PlannerEnum::Subgraph(planner) => planner.match_planner(stmt),
            PlannerEnum::FetchVertices(planner) => planner.match_planner(stmt),
            PlannerEnum::FetchEdges(planner) => planner.match_planner(stmt),
            PlannerEnum::Maintain(planner) => planner.match_planner(stmt),
            PlannerEnum::UserManagement(planner) => planner.match_planner(stmt),
            PlannerEnum::Insert(planner) => planner.match_planner(stmt),
            PlannerEnum::Delete(planner) => planner.match_planner(stmt),
            PlannerEnum::Update(planner) => planner.match_planner(stmt),
            PlannerEnum::GroupBy(planner) => planner.match_planner(stmt),
            PlannerEnum::SetOperation(planner) => planner.match_planner(stmt),
            PlannerEnum::Use(planner) => planner.match_planner(stmt),
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
            PlannerEnum::Delete(planner) => Box::new(planner),
            PlannerEnum::Update(planner) => Box::new(planner),
            PlannerEnum::GroupBy(planner) => Box::new(planner),
            PlannerEnum::SetOperation(planner) => Box::new(planner),
            PlannerEnum::Use(planner) => Box::new(planner),
        }
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

// 为 RewriteError 实现 From 转换
impl From<RewriteError> for PlannerError {
    fn from(err: RewriteError) -> Self {
        PlannerError::PlanGenerationFailed(format!("Plan rewrite failed: {}", err))
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
    fn test_planner_enum_from_sentence_kind() {
        let planner = PlannerEnum::from_sentence_kind(SentenceKind::Match);
        assert!(planner.is_some());
    }
}
