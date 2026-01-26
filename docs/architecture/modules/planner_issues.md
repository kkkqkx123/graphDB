# Planner 模块问题清单与修改方案

## 问题清单

| 序号 | 问题描述 | 严重程度 | 问题类型 | 状态 |
|------|----------|----------|----------|------|
| 4.1 | 使用 AstContext 而非完整的 QueryAstContext | 高 | 数据丢失 | 待修复 |
| 4.2 | MatchPlanner 实现不完整 | 高 | 功能缺失 | 待修复 |
| 4.3 | 规划器注册机制缺乏动态配置 | 中 | 扩展性问题 | 待修复 |
| 4.4 | 计划节点 ID 生成方式不标准 | 低 | 代码质量问题 | 待修复 |
| 4.5 | 缺乏计划缓存机制 | 低 | 性能问题 | 待修复 |
| 4.6 | 规划器不支持查询重写 | 低 | 功能缺失 | 待修复 |

---

## 详细问题分析

### 问题 4.1: 使用子集上下文

**涉及文件**: 
- `src/query/query_pipeline_manager.rs`
- `src/query/planner/mod.rs`

**当前实现**:
```rust
fn generate_execution_plan(
    &mut self,
    _query_context: &mut QueryContext,
    ast: &crate::query::context::ast::QueryAstContext,
) -> DBResult<crate::query::planner::plan::ExecutionPlan> {
    let ast_ctx = ast.base_context();  // 只使用 AstContext
    match self.planner.transform(ast_ctx) {
        Ok(sub_plan) => {
            // ...
        }
        Err(e) => Err(DBError::Query(QueryError::PlanningError(format!(
            "规划失败: {}",
            e
        )))),
    }
}
```

**问题分析**:
```
QueryAstContext (完整上下文)
├── AstContext (被使用)
│   ├── query_type
│   ├── space
│   └── sentence
└── 额外信息 (未被使用)
    ├── dependencies: HashMap<String, Vec<String>>
    ├── query_variables: HashMap<String, VariableInfo>
    └── expression_contexts: Vec<ExpressionContext>

Planner 只使用 AstContext，丢失了:
- 变量依赖关系
- 表达式上下文
- 子查询信息
```

**影响**:
1. 无法进行跨变量的优化
2. 无法正确处理子查询
3. 表达式求值信息丢失

---

### 问题 4.2: MatchPlanner 实现不完整

**涉及文件**: `src/query/planner/match_planner.rs`

**当前实现**:
```rust
impl Planner for MatchPlanner {
    fn transform(
        &mut self,
        ast_ctx: &AstContext,
    ) -> Result<SubPlan, PlannerError> {
        let stmt = ast_ctx.sentence().ok_or_else(|| {
            PlannerError::InvalidAstContext("AstContext 中缺少语句".to_string())
        })?;

        let space_id = ast_ctx.space.space_id.unwrap_or(1) as i32;

        // 创建起始节点
        let start_node = ScanVerticesNode::new(space_id);
        let mut current_plan = SubPlan::from_root(start_node.into_enum());

        if ast_ctx.query_type() == QueryType::ReadQuery {
            // 空实现：未处理 MATCH 的各个部分
        }

        // 添加 LIMIT
        let limit = 10;
        let limit_node = LimitNode::new(limit, limit);
        current_plan = current_plan.with_root(limit_node.into_enum());

        Ok(current_plan)
    }
}
```

**问题**:
- 未解析和处理 MATCH 的 pattern
- 未处理 WHERE 子句
- 未处理 RETURN 子句
- 未处理 ORDER BY 子句
- 未处理 SKIP/LIMIT 子句
- 生成了过于简单的计划（只有 ScanVertices + Limit）

**缺失的处理**:
```sql
-- 当前实现只能处理:
MATCH (n) RETURN n LIMIT 10

-- 无法处理:
MATCH (n:Player)-[e:PLAY]->(m:Team WHERE m.name STARTS WITH 'L')
WHERE n.age > 25
RETURN n.name, m.name, e.score
ORDER BY n.age
SKIP 5 LIMIT 10
```

---

### 问题 4.3: 规划器注册机制缺乏动态配置

**涉及文件**: `src/query/planner/mod.rs`

**当前实现**:
```rust
impl PlannerRegistry {
    pub fn new() -> Self {
        let mut planner_registry = PlannerRegistry {
            planners: HashMap::new(),
        };
        planner_registry.init();
        planner_registry
    }

    fn init(&mut self) {
        self.planners.insert(
            SentenceKind::Match,
            Box::new(|| Box::new(MatchPlanner::new())),
        );
        self.planners.insert(
            SentenceKind::Go,
            Box::new(|| Box::new(GoPlanner::new())),
        );
        // ... 硬编码注册
    }
}
```

**问题**:
- 无法动态添加/移除规划器
- 无法配置规划器参数
- 无法启用/禁用特定规划器
- 无法设置规划策略

---

### 问题 4.5: 缺乏计划缓存

**当前实现**: 无计划缓存

**问题**:
- 相同查询重复规划
- 无法利用历史规划结果
- 性能开销大

---

## 修改方案

### 修改方案 4.1: 使用完整上下文

**预估工作量**: 3-4 人天

**修改目标**:
- 让规划器能够访问完整的 QueryAstContext
- 传递变量和表达式信息

**修改步骤**:

**步骤 1**: 修改 Planner trait

```rust
// src/query/planner/traits.rs

use crate::query::context::ast::QueryAstContext;
use crate::query::context::execution::QueryContext;

/// 规划器 trait
pub trait Planner {
    /// 使用 AstContext 进行基础规划
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError>;
    
    /// 使用完整上下文进行规划
    ///
    /// 默认实现：提取 AstContext 进行规划
    fn transform_with_full_context(
        &mut self,
        _query_context: &mut QueryContext,
        ast: &QueryAstContext,
    ) -> Result<ExecutionPlan, PlannerError> {
        let ast_ctx = ast.base_context();
        let sub_plan = self.transform(ast_ctx)?;
        Ok(ExecutionPlan::new(sub_plan.root().clone()))
    }
    
    /// 获取规划器的名称
    fn name(&self) -> &'static str;
    
    /// 检查是否可以处理给定的语句类型
    fn can_handle(&self, statement: &Stmt) -> bool;
}
```

**步骤 2**: 更新 MatchPlanner 使用完整上下文

```rust
// src/query/planner/match_planner.rs

impl MatchPlanner {
    /// 使用完整上下文进行 MATCH 规划
    pub fn transform_with_full_context(
        &mut self,
        query_context: &mut QueryContext,
        ast: &QueryAstContext,
    ) -> Result<ExecutionPlan, PlannerError> {
        let stmt = ast.base_context().sentence().ok_or_else(|| {
            PlannerError::InvalidAstContext("AstContext 中缺少语句".to_string())
        })?;

        let space_id = ast.base_context().space.space_id.unwrap_or(1) as i32;

        // 1. 处理 MATCH pattern
        let mut current_plan = self.plan_match_pattern(ast, stmt, space_id)?;

        // 2. 处理 WHERE 子句
        if let Some(where_condition) = self.extract_where_condition(ast, stmt)? {
            current_plan = self.plan_filter(current_plan, where_condition, space_id)?;
        }

        // 3. 处理 RETURN 子句
        if let Some(return_columns) = self.extract_return_columns(ast, stmt)? {
            current_plan = self.plan_project(current_plan, return_columns, space_id)?;
        }

        // 4. 处理 ORDER BY 子句
        if let Some(order_by) = self.extract_order_by(ast, stmt)? {
            current_plan = self.plan_sort(current_plan, order_by, space_id)?;
        }

        // 5. 处理 SKIP/LIMIT 子句
        if let Some(pagination) = self.extract_pagination(ast, stmt)? {
            current_plan = self.plan_limit(current_plan, pagination, space_id)?;
        }

        Ok(ExecutionPlan::from_sub_plan(current_plan))
    }

    fn plan_match_pattern(
        &self,
        ast: &QueryAstContext,
        stmt: &Stmt,
        space_id: i32,
    ) -> Result<SubPlan, PlannerError> {
        match stmt {
            Stmt::Match(match_stmt) => {
                // 解析 pattern，生成扫描或查找节点
                let pattern = &match_stmt.pattern;
                let start_node = self.plan_pattern(pattern, space_id)?;
                Ok(SubPlan::from_root(start_node))
            }
            _ => Err(PlannerError::InvalidOperation(
                "Expected MATCH statement".to_string()
            ))
        }
    }

    fn plan_pattern(
        &self,
        pattern: &MatchPattern,
        space_id: i32,
    ) -> Result<PlanNodeEnum, PlannerError> {
        match pattern {
            MatchPattern::Node { name, labels, properties } => {
                self.plan_node_pattern(name, labels, properties, space_id)
            }
            MatchPattern::Relationship { from, edge, to } => {
                self.plan_relationship_pattern(from, edge, to, space_id)
            }
            MatchPattern::Path { elements } => {
                self.plan_path_pattern(elements, space_id)
            }
        }
    }

    fn plan_node_pattern(
        &self,
        name: &str,
        labels: &[String],
        properties: &Option<HashMap<String, Expression>>,
        space_id: i32,
    ) -> Result<PlanNodeEnum, PlannerError> {
        if labels.is_empty() && properties.is_none() {
            // 全表扫描
            Ok(ScanVerticesNode::new(space_id).into_enum())
        } else if !labels.is_empty() {
            // 标签过滤
            let node = GetVerticesNode::new(space_id);
            node.set_tag_filter(labels.clone());
            Ok(node.into_enum())
        } else {
            // 属性过滤（需要索引）
            let node = GetVerticesNode::new(space_id);
            node.set_expression(properties.clone());
            Ok(node.into_enum())
        }
    }
}
```

**步骤 3**: 更新 QueryPipelineManager

```rust
// src/query/query_pipeline_manager.rs

impl<S: StorageEngine + 'static> QueryPipelineManager<S> {
    fn generate_execution_plan(
        &mut self,
        query_context: &mut QueryContext,
        ast: &QueryAstContext,
    ) -> DBResult<ExecutionPlan> {
        // 使用完整的上下文
        self.planner
            .transform_with_full_context(query_context, ast)
            .map_err(|e| {
                DBError::Query(QueryError::PlanningError(format!(
                    "规划失败: {}",
                    e
                )))
            })
    }
}
```

---

### 修改方案 4.2: 完善 MatchPlanner

**预估工作量**: 5-7 人天

**修改目标**: 实现完整的 MATCH 语句规划

**修改步骤**:

**步骤 1**: 实现 WHERE 子句处理

```rust
impl MatchPlanner {
    fn plan_filter(
        &self,
        input_plan: SubPlan,
        condition: Expression,
        space_id: i32,
    ) -> Result<SubPlan, PlannerError> {
        let filter_node = FilterNode::new(input_plan.root().clone(), condition)?;
        Ok(input_plan.with_root(filter_node.into_enum()))
    }

    fn extract_where_condition(
        &self,
        ast: &QueryAstContext,
        stmt: &Stmt,
    ) -> Result<Option<Expression>, PlannerError> {
        match stmt {
            Stmt::Match(match_stmt) => {
                // 从 QueryAstContext 获取变量信息
                if let Some(where_clause) = &match_stmt.where_clause {
                    // 检查变量引用是否有效
                    self.validate_variable_references(ast, where_clause)?;
                    Ok(Some(where_clause.clone()))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    fn validate_variable_references(
        &self,
        ast: &QueryAstContext,
        expr: &Expression,
    ) -> Result<(), PlannerError> {
        // 遍历表达式，检查所有变量引用
        let mut visitor = VariableReferenceValidator::new(ast);
        visitor.visit_expression(expr);
        
        if let Some(unknown_var) = visitor.unknown_variable() {
            return Err(PlannerError::UnknownVariable(
                format!("Unknown variable: {}", unknown_var)
            ));
        }
        
        Ok(())
    }
}
```

**步骤 2**: 实现 RETURN 子句处理

```rust
impl MatchPlanner {
    fn plan_project(
        &self,
        input_plan: SubPlan,
        columns: Vec<YieldColumn>,
        space_id: i32,
    ) -> Result<SubPlan, PlannerError> {
        let project_node = ProjectNode::new(input_plan.root().clone(), columns)?;
        Ok(input_plan.with_root(project_node.into_enum()))
    }

    fn extract_return_columns(
        &self,
        ast: &QueryAstContext,
        stmt: &Stmt,
    ) -> Result<Option<Vec<YieldColumn>>, PlannerError> {
        match stmt {
            Stmt::Match(match_stmt) => {
                Ok(match_stmt.return_columns.clone())
            }
            _ => Ok(None),
        }
    }
}
```

**步骤 3**: 实现 ORDER BY 和 LIMIT

```rust
impl MatchPlanner {
    fn plan_sort(
        &self,
        input_plan: SubPlan,
        order_by: Vec<OrderByItem>,
        space_id: i32,
    ) -> Result<SubPlan, PlannerError> {
        let sort_node = SortNode::new(input_plan.root().clone(), order_by)?;
        Ok(input_plan.with_root(sort_node.into_enum()))
    }

    fn plan_limit(
        &self,
        input_plan: SubPlan,
        pagination: PaginationInfo,
        space_id: i32,
    ) -> Result<SubPlan, PlannerError> {
        let limit_node = LimitNode::new(pagination.skip, pagination.limit);
        Ok(input_plan.with_root(limit_node.into_enum()))
    }

    fn extract_order_by(
        &self,
        _ast: &QueryAstContext,
        stmt: &Stmt,
    ) -> Result<Option<Vec<OrderByItem>>, PlannerError> {
        match stmt {
            Stmt::Match(match_stmt) => Ok(match_stmt.order_by.clone()),
            _ => Ok(None),
        }
    }

    fn extract_pagination(
        &self,
        _ast: &QueryAstContext,
        stmt: &Stmt,
    ) -> Result<Option<PaginationInfo>, PlannerError> {
        match stmt {
            Stmt::Match(match_stmt) => {
                let skip = match_stmt.skip.unwrap_or(0);
                let limit = match_stmt.limit.unwrap_or(usize::MAX);
                if skip > 0 || limit != usize::MAX {
                    Ok(Some(PaginationInfo { skip, limit }))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PaginationInfo {
    pub skip: usize,
    pub limit: usize,
}
```

---

### 修改方案 4.3: 动态规划器配置

**预估工作量**: 2 人天

**修改代码**:

```rust
// src/query/planner/mod.rs

/// 规划器配置
#[derive(Debug, Clone)]
pub struct PlannerConfig {
    /// 是否启用计划缓存
    pub enable_caching: bool,
    /// 最大计划深度
    pub max_plan_depth: usize,
    /// 是否启用并行规划
    pub enable_parallel_planning: bool,
    /// 默认查询超时
    pub default_timeout: Duration,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            max_plan_depth: 100,
            enable_parallel_planning: false,
            default_timeout: Duration::from_secs(30),
        }
    }
}

/// 可配置的规划器注册表
pub struct ConfigurablePlannerRegistry {
    planners: HashMap<&'static str, Box<dyn PlannerCreator>>,
    config: PlannerConfig,
    cache: Option<PlanCache>,
}

pub trait PlannerCreator: Send {
    fn create(&self) -> Box<dyn Planner>;
    fn can_handle(&self, statement: &Stmt) -> bool;
}

impl ConfigurablePlannerRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            planners: HashMap::new(),
            config: PlannerConfig::default(),
            cache: None,
        };
        
        // 注册默认规划器
        registry.register("MATCH", Box::new(|| Box::new(MatchPlanner::new())));
        registry.register("GO", Box::new(|| Box::new(GoPlanner::new())));
        registry.register("LOOKUP", Box::new(|| Box::new(LookupPlanner::new())));
        registry.register("FETCH", Box::new(|| Box::new(FetchPlanner::new())));
        
        // 初始化缓存
        registry.cache = Some(PlanCache::new(1000));
        
        registry
    }
    
    pub fn register<P>(&mut self, name: &'static str, creator: P)
    where
        P: PlannerCreator + 'static,
    {
        self.planners.insert(name, Box::new(creator));
    }
    
    pub fn unregister(&mut self, name: &'static str) {
        self.planners.remove(name);
    }
    
    pub fn set_config(&mut self, config: PlannerConfig) {
        self.config = config;
    }
    
    pub fn create_plan(
        &self,
        query_context: &mut QueryContext,
        ast: &QueryAstContext,
    ) -> Result<ExecutionPlan, PlannerError> {
        let stmt = ast.base_context().sentence().ok_or_else(|| {
            PlannerError::InvalidAstContext("AstContext 中缺少语句".to_string())
        })?;
        
        // 查找合适的规划器
        let planner_name = self.find_planner_name(&stmt).unwrap_or("Default");
        
        // 尝试从缓存获取
        if self.config.enable_caching {
            let cache_key = self.generate_cache_key(ast);
            if let Some(cached_plan) = self.cache.as_ref().unwrap().get(&cache_key) {
                return Ok(cached_plan.clone());
            }
        }
        
        // 创建规划器并生成计划
        let planner = self.planners.get(planner_name)
            .ok_or_else(|| PlannerError::NoAvailablePlanner(planner_name.to_string()))?;
        
        let plan = planner.create().transform_with_full_context(query_context, ast)?;
        
        // 存入缓存
        if self.config.enable_caching {
            let cache_key = self.generate_cache_key(ast);
            self.cache.as_mut().unwrap().insert(cache_key, plan.clone());
        }
        
        Ok(plan)
    }
}
```

---

### 修改方案 4.5: 计划缓存

**预估工作量**: 1 人天

**修改代码**:

```rust
// src/query/planner/plan_cache.rs

use lru_cache::LruCache;
use std::sync::Mutex;

/// 计划缓存
pub struct PlanCache {
    cache: Mutex<LruCache<String, ExecutionPlan>>,
    max_size: usize,
}

impl PlanCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(max_size)),
            max_size,
        }
    }
    
    pub fn get(&self, key: &str) -> Option<ExecutionPlan> {
        let cache = self.cache.lock().unwrap();
        cache.get(key).cloned()
    }
    
    pub fn insert(&mut self, key: String, plan: ExecutionPlan) {
        let mut cache = self.cache.lock().unwrap();
        cache.insert(key, plan);
    }
    
    pub fn clear(&mut self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }
    
    pub fn size(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }
}
```

---

## 修改优先级

| 序号 | 修改方案 | 优先级 | 预估工作量 | 依赖 |
|------|----------|--------|------------|------|
| 4.1 | 使用完整上下文 | 高 | 3-4 人天 | Validator 重构 |
| 4.2 | 完善 MatchPlanner | 高 | 5-7 人天 | 4.1 |
| 4.3 | 动态规划器配置 | 中 | 2 人天 | 无 |
| 4.5 | 计划缓存 | 低 | 1 人天 | 4.3 |

---

## 测试建议

### 测试用例 1: 完整 MATCH 语句规划

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_match_with_where() {
        let ast = QueryAstContext::new(
            "MATCH (n:Player)-[e:PLAY]->(m:Team) WHERE n.age > 25 RETURN n.name, m.name"
        );
        let mut planner = MatchPlanner::new();
        let plan = planner.transform_with_full_context(&mut QueryContext::new(), &ast);
        
        assert!(plan.is_ok());
        let plan = plan.unwrap();
        
        // 验证计划包含正确的节点
        assert!(plan.contains_node_type("GetVertices"));
        assert!(plan.contains_node_type("Filter"));
        assert!(plan.contains_node_type("Project"));
    }
    
    #[test]
    fn test_match_with_order_by_limit() {
        let query = "MATCH (n) RETURN n.age ORDER BY n.age SKIP 5 LIMIT 10";
        let ast = QueryAstContext::new(query);
        let mut planner = MatchPlanner::new();
        let plan = planner.transform_with_full_context(&mut QueryContext::new(), &ast);
        
        assert!(plan.is_ok());
        // 验证计划包含 Sort 和 Limit 节点
    }
}
```

---

## 风险与注意事项

### 风险 1: MatchPlanner 复杂度

- **风险**: 完整实现 MatchPlanner 需要处理大量边界情况
- **缓解措施**: 分阶段实现，优先处理常用场景
- **实现**: 先实现基本功能，再逐步完善

### 风险 2: 缓存内存使用

- **风险**: 计划缓存可能占用大量内存
- **缓解措施**: 设置合理的缓存大小限制
- **实现**: 使用 LRU 缓存，自动淘汰旧条目
