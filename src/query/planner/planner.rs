//! 新的规划器注册机制
//! 使用类型安全的枚举替代字符串匹配

use crate::query::context::ast::AstContext;
use crate::query::planner::plan::SubPlan;
use std::collections::HashMap;

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
        // 注册新的 MATCH 规划器
        self.register_planner(
            SentenceKind::Match,
            crate::query::planner::statements::MatchPlanner::match_ast_ctx,
            crate::query::planner::statements::MatchPlanner::make,
            100,
        );
    }

    /// 批量注册 NGQL 规划器
    pub fn register_ngql_planners(&mut self) {
        // 暂时注释掉，因为现有的规划器还没有实现新的接口
        // self.register_planner(
        //     SentenceKind::Go,
        //     crate::query::planner::statements::GoPlanner::match_ast_ctx,
        //     crate::query::planner::statements::GoPlanner::make,
        //     100,
        // );

        // self.register_planner(
        //     SentenceKind::Lookup,
        //     crate::query::planner::statements::LookupPlanner::match_ast_ctx,
        //     crate::query::planner::statements::LookupPlanner::make,
        //     100,
        // );

        // self.register_planner(
        //     SentenceKind::Path,
        //     crate::query::planner::statements::PathPlanner::match_ast_ctx,
        //     crate::query::planner::statements::PathPlanner::make,
        //     100,
        // );

        // self.register_planner(
        //     SentenceKind::Subgraph,
        //     crate::query::planner::statements::SubgraphPlanner::match_ast_ctx,
        //     crate::query::planner::statements::SubgraphPlanner::make,
        //     100,
        // );

        // self.register_planner(
        //     SentenceKind::FetchVertices,
        //     crate::query::planner::statements::FetchVerticesPlanner::match_ast_ctx,
        //     crate::query::planner::statements::FetchVerticesPlanner::make,
        //     100,
        // );

        // self.register_planner(
        //     SentenceKind::FetchEdges,
        //     crate::query::planner::statements::FetchEdgesPlanner::match_ast_ctx,
        //     crate::query::planner::statements::FetchEdgesPlanner::make,
        //     100,
        // );

        // self.register_planner(
        //     SentenceKind::Maintain,
        //     crate::query::planner::statements::MaintainPlanner::match_ast_ctx,
        //     crate::query::planner::statements::MaintainPlanner::make,
        //     100,
        // );
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
        // 这里需要根据实际的 AST 上下文结构来提取语句类型
        // 暂时使用一个假设的方法
        let statement_type = ast_ctx.statement_type();
        SentenceKind::from_str(&statement_type)
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
}

/// 顺序规划器（使用新的注册机制）
#[derive(Debug)]
pub struct SequentialPlanner {
    planners: Vec<MatchAndInstantiate>,
}

impl SequentialPlanner {
    pub fn new() -> Self {
        Self {
            planners: Vec::new(),
        }
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
}

impl Planner for SequentialPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        Self::to_plan(ast_ctx)
    }

    fn match_planner(&self, _ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(_ast_ctx)
    }
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
