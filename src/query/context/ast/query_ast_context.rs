//! 查询AST上下文 - 用于查询计划生成的AST上下文

use super::base::AstContext;
use std::collections::HashMap;

/// 查询AST上下文
/// 
/// 专门用于查询计划生成和优化的AST上下文
/// 包含查询执行计划相关的信息
#[derive(Debug, Clone)]
pub struct QueryAstContext {
    base: AstContext,
    query_plan: QueryPlan,              // 查询计划
    optimization_hints: Vec<OptimizationHint>, // 优化提示
    execution_stats: ExecutionStats,     // 执行统计
    dependencies: HashMap<String, Vec<String>>, // 依赖关系
}

/// 查询计划
#[derive(Debug, Clone)]
pub struct QueryPlan {
    pub plan_type: String,               // "sequential", "parallel", "distributed"
    pub steps: Vec<QueryStep>,           // 查询步骤
    pub estimated_cost: f64,             // 预估成本
    pub estimated_rows: usize,          // 预估行数
}

/// 查询步骤
#[derive(Debug, Clone)]
pub struct QueryStep {
    pub step_type: String,              // "scan", "filter", "join", "aggregate"
    pub description: String,             // 步骤描述
    pub dependencies: Vec<String>,      // 依赖的步骤
    pub estimated_cost: f64,            // 预估成本
    pub estimated_rows: usize,          // 预估行数
}

/// 优化提示
#[derive(Debug, Clone)]
pub struct OptimizationHint {
    pub hint_type: String,               // "index", "join_order", "predicate_pushdown"
    pub target: String,                  // 目标对象
    pub parameters: HashMap<String, String>, // 参数
}

/// 执行统计
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    pub total_time: f64,                 // 总执行时间
    pub memory_usage: usize,             // 内存使用量
    pub rows_processed: usize,           // 处理的行数
    pub cache_hits: usize,               // 缓存命中数
    pub cache_misses: usize,             // 缓存未命中数
}

impl QueryAstContext {
    /// 创建新的查询AST上下文
    pub fn new(query_text: &str) -> Self {
        Self {
            base: AstContext::new("QUERY", query_text),
            query_plan: QueryPlan {
                plan_type: "sequential".to_string(),
                steps: Vec::new(),
                estimated_cost: 0.0,
                estimated_rows: 0,
            },
            optimization_hints: Vec::new(),
            execution_stats: ExecutionStats {
                total_time: 0.0,
                memory_usage: 0,
                rows_processed: 0,
                cache_hits: 0,
                cache_misses: 0,
            },
            dependencies: HashMap::new(),
        }
    }

    /// 添加查询步骤
    pub fn add_step(&mut self, step: QueryStep) {
        self.query_plan.steps.push(step);
    }

    /// 添加优化提示
    pub fn add_optimization_hint(&mut self, hint: OptimizationHint) {
        self.optimization_hints.push(hint);
    }

    /// 添加依赖关系
    pub fn add_dependency(&mut self, step_name: String, dependencies: Vec<String>) {
        self.dependencies.insert(step_name, dependencies);
    }

    /// 更新执行统计
    pub fn update_execution_stats(&mut self, stats: ExecutionStats) {
        self.execution_stats = stats;
    }

    /// 获取查询计划
    pub fn query_plan(&self) -> &QueryPlan {
        &self.query_plan
    }

    /// 获取优化提示
    pub fn optimization_hints(&self) -> &[OptimizationHint] {
        &self.optimization_hints
    }

    /// 获取执行统计
    pub fn execution_stats(&self) -> &ExecutionStats {
        &self.execution_stats
    }

    /// 获取依赖关系
    pub fn dependencies(&self) -> &HashMap<String, Vec<String>> {
        &self.dependencies
    }

    /// 获取基础AST上下文
    pub fn base_context(&self) -> &AstContext {
        &self.base
    }

    /// 计算查询计划的总预估成本
    pub fn total_estimated_cost(&self) -> f64 {
        self.query_plan.steps.iter().map(|step| step.estimated_cost).sum()
    }

    /// 检查是否包含特定类型的优化提示
    pub fn has_optimization_hint(&self, hint_type: &str) -> bool {
        self.optimization_hints.iter().any(|h| h.hint_type == hint_type)
    }

    /// 获取特定类型的优化提示
    pub fn get_optimization_hints_by_type(&self, hint_type: &str) -> Vec<&OptimizationHint> {
        self.optimization_hints.iter().filter(|h| h.hint_type == hint_type).collect()
    }

    /// 获取步骤的依赖关系
    pub fn get_step_dependencies(&self, step_name: &str) -> Option<&Vec<String>> {
        self.dependencies.get(step_name)
    }

    /// 检查查询计划是否为空
    pub fn is_plan_empty(&self) -> bool {
        self.query_plan.steps.is_empty()
    }
}

impl Default for QueryAstContext {
    fn default() -> Self {
        Self {
            base: AstContext::default(),
            query_plan: QueryPlan {
                plan_type: "sequential".to_string(),
                steps: Vec::new(),
                estimated_cost: 0.0,
                estimated_rows: 0,
            },
            optimization_hints: Vec::new(),
            execution_stats: ExecutionStats {
                total_time: 0.0,
                memory_usage: 0,
                rows_processed: 0,
                cache_hits: 0,
                cache_misses: 0,
            },
            dependencies: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_ast_context_creation() {
        let query = "SELECT * FROM users WHERE age > 30";
        let context = QueryAstContext::new(query);
        
        assert_eq!(context.base_context().statement_type(), "QUERY");
        assert!(context.is_plan_empty());
        assert_eq!(context.total_estimated_cost(), 0.0);
    }

    #[test]
    fn test_query_ast_context_add_step() {
        let mut context = QueryAstContext::new("SELECT * FROM users");
        
        let step = QueryStep {
            step_type: "scan".to_string(),
            description: "Scan users table".to_string(),
            dependencies: Vec::new(),
            estimated_cost: 10.0,
            estimated_rows: 1000,
        };
        
        context.add_step(step);
        assert!(!context.is_plan_empty());
        assert_eq!(context.total_estimated_cost(), 10.0);
    }

    #[test]
    fn test_query_ast_context_add_optimization_hint() {
        let mut context = QueryAstContext::new("SELECT * FROM users WHERE id = 1");
        
        let hint = OptimizationHint {
            hint_type: "index".to_string(),
            target: "users.id".to_string(),
            parameters: HashMap::new(),
        };
        
        context.add_optimization_hint(hint);
        assert!(context.has_optimization_hint("index"));
        assert_eq!(context.get_optimization_hints_by_type("index").len(), 1);
    }

    #[test]
    fn test_query_ast_context_add_dependency() {
        let mut context = QueryAstContext::new("SELECT * FROM users JOIN orders ON users.id = orders.user_id");
        
        let dependencies = vec!["scan_users".to_string(), "scan_orders".to_string()];
        context.add_dependency("join".to_string(), dependencies.clone());
        
        assert_eq!(context.get_step_dependencies("join"), Some(&dependencies));
    }

    #[test]
    fn test_query_ast_context_update_execution_stats() {
        let mut context = QueryAstContext::new("SELECT * FROM users");
        
        let stats = ExecutionStats {
            total_time: 0.5,
            memory_usage: 1024,
            rows_processed: 100,
            cache_hits: 10,
            cache_misses: 5,
        };
        
        context.update_execution_stats(stats);
        
        assert_eq!(context.execution_stats().total_time, 0.5);
        assert_eq!(context.execution_stats().memory_usage, 1024);
        assert_eq!(context.execution_stats().rows_processed, 100);
    }
}