# 查询优化决策缓存设计方案

## 1. 设计目标

将查询计划缓存从"缓存完整计划树"改为"缓存优化决策"，提高内存效率并增强缓存的适应性。

## 2. 核心概念

### 2.1 优化决策（OptimizationDecision）

优化决策是从 AST 到物理执行计划的"中间表示"，包含所有基于代价的优化选择，但不包含具体的计划树结构。

```rust
/// 完整的优化决策
pub struct OptimizationDecision {
    /// 遍历起点选择决策
    pub traversal_start: TraversalStartDecision,
    /// 索引选择决策
    pub index_selection: IndexSelectionDecision,
    /// 连接顺序决策
    pub join_order: JoinOrderDecision,
    /// 适用的重写规则序列
    pub rewrite_rules: Vec<RewriteRuleId>,
    /// 决策时的统计信息版本
    pub stats_version: u64,
    /// 决策时的索引版本
    pub index_version: u64,
    /// 决策时间戳
    pub created_at: Instant,
}
```

### 2.2 决策缓存键（DecisionCacheKey）

```rust
/// 决策缓存键
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct DecisionCacheKey {
    /// 查询模板哈希
    query_template_hash: u64,
    /// 图空间ID
    space_id: Option<i32>,
    /// 语句类型
    statement_type: SentenceKind,
    /// 模式指纹
    pattern_fingerprint: Option<String>,
}
```

## 3. 架构变化

### 3.1 当前架构

```
Parser → Validator → Planner → [PlanCache] → Rewrite → Executor
                            ↓
                      缓存完整 ExecutionPlan
```

### 3.2 新架构

```
Parser → Validator → [DecisionCache] → Planner → Rewrite → Executor
                          ↓
                    缓存优化决策
                    命中时直接应用决策
                    未命中时执行优化并存储决策
```

## 4. 数据结构详细设计

### 4.1 遍历起点决策

```rust
/// 遍历起点选择决策
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct TraversalStartDecision {
    /// 起始节点变量名
    pub start_variable: String,
    /// 访问路径类型
    pub access_path: AccessPath,
    /// 估计的选择性
    pub estimated_selectivity: f64,
    /// 估计的代价
    pub estimated_cost: f64,
}

/// 访问路径类型
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum AccessPath {
    /// 显式VID指定
    ExplicitVid {
        vid_expression: String, // 序列化后的表达式
    },
    /// 索引扫描
    IndexScan {
        index_name: String,
        property_name: String,
        predicate: String, // 序列化后的谓词
    },
    /// 标签索引
    TagIndex {
        tag_name: String,
    },
    /// 全表扫描
    FullScan {
        entity_type: EntityType,
    },
    /// 变量绑定
    VariableBinding {
        source_variable: String,
    },
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum EntityType {
    Vertex { tag_name: Option<String> },
    Edge { edge_type: Option<String> },
}
```

### 4.2 索引选择决策

```rust
/// 索引选择决策
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct IndexSelectionDecision {
    /// 每个实体类型的索引选择
    pub entity_indexes: Vec<EntityIndexChoice>,
}

/// 实体索引选择
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct EntityIndexChoice {
    /// 实体类型（标签或边类型）
    pub entity_name: String,
    /// 选择的索引
    pub selected_index: IndexChoice,
    /// 估计的选择性
    pub selectivity: f64,
}

/// 索引选择
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum IndexChoice {
    /// 主键索引
    PrimaryKey,
    /// 属性索引
    PropertyIndex {
        property_name: String,
        index_name: String,
    },
    /// 复合索引
    CompositeIndex {
        property_names: Vec<String>,
        index_name: String,
    },
    /// 无可用索引
    None,
}
```

### 4.3 连接顺序决策

```rust
/// 连接顺序决策
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct JoinOrderDecision {
    /// 连接顺序（变量名序列）
    pub join_order: Vec<String>,
    /// 每个连接的算法选择
    pub join_algorithms: Vec<JoinAlgorithm>,
}

/// 连接算法
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum JoinAlgorithm {
    /// 哈希连接
    HashJoin {
        build_side: String,
        probe_side: String,
    },
    /// 嵌套循环连接
    NestedLoopJoin {
        outer: String,
        inner: String,
    },
    /// 索引连接
    IndexJoin {
        indexed_side: String,
    },
}
```

## 5. 决策缓存实现

### 5.1 缓存结构

```rust
/// 决策缓存
pub struct DecisionCache {
    /// LRU 缓存
    cache: Mutex<LruCache<DecisionCacheKey, CachedDecision>>,
    /// 统计信息
    stats: Mutex<DecisionCacheStats>,
    /// 配置
    config: DecisionCacheConfig,
}

/// 缓存的决策项
#[derive(Debug, Clone)]
pub struct CachedDecision {
    /// 优化决策
    pub decision: OptimizationDecision,
    /// 最后访问时间
    pub last_accessed: Instant,
    /// 访问次数
    pub access_count: u64,
}

/// 决策缓存配置
#[derive(Debug, Clone)]
pub struct DecisionCacheConfig {
    /// 最大缓存条目数
    pub max_entries: usize,
    /// 决策过期时间
    pub ttl_seconds: u64,
    /// 启用统计
    pub enable_stats: bool,
}
```

### 5.2 缓存接口

```rust
impl DecisionCache {
    /// 获取或计算决策
    pub fn get_or_compute<F>(
        &self,
        key: DecisionCacheKey,
        stats_version: u64,
        index_version: u64,
        compute: F,
    ) -> Result<OptimizationDecision, CacheError>
    where
        F: FnOnce() -> Result<OptimizationDecision, CacheError>;
    
    /// 检查决策是否仍然有效
    fn is_decision_valid(&self, decision: &OptimizationDecision, stats_version: u64, index_version: u64) -> bool;
    
    /// 使过期决策失效
    pub fn invalidate_outdated(&self, current_stats_version: u64, current_index_version: u64);
}
```

## 6. 修改计划

### 阶段 1：定义数据结构

1. 创建 `src/query/optimizer/decision/` 目录
2. 定义 `OptimizationDecision` 及相关结构
3. 定义 `DecisionCacheKey`
4. 添加序列化/反序列化支持

### 阶段 2：实现决策缓存

1. 实现 `DecisionCache` 结构
2. 实现缓存接口方法
3. 添加单元测试

### 阶段 3：重构 Planner

1. 修改 `Planner` trait，支持决策注入
2. 修改各 statement planner 使用决策
3. 在 `MatchStatementPlanner` 中应用遍历起点决策
4. 在索引选择中应用索引决策

### 阶段 4：修改 Pipeline

1. 修改 `query_pipeline_manager.rs`
2. 在优化阶段使用决策缓存
3. 移除旧的 `PlanCache` 使用

### 阶段 5：清理

1. 移除 `PlanCache` 及相关代码
2. 更新文档
3. 运行完整测试

## 7. 关键修改点

### 7.1 query_pipeline_manager.rs

```rust
// 修改前
fn generate_execution_plan(&mut self, ...) -> DBResult<ExecutionPlan> {
    // 尝试从 PlanCache 获取
    if let Some(cached) = self.plan_cache.get(&key)? {
        return Ok(cached);
    }
    // 生成计划并缓存
    let plan = planner.transform(...)?;
    self.plan_cache.insert(key, plan.clone())?;
    Ok(plan)
}

// 修改后
fn generate_execution_plan(&mut self, ...) -> DBResult<ExecutionPlan> {
    // 获取或计算优化决策
    let decision = self.decision_cache.get_or_compute(
        decision_key,
        stats.version(),
        index.version(),
        || self.compute_decision(stmt, qctx),
    )?;
    
    // 使用决策生成计划
    let plan = planner.transform_with_decision(stmt, qctx, &decision)?;
    Ok(plan)
}
```

### 7.2 planner trait

```rust
// 修改前
trait Planner {
    fn transform(&mut self, stmt: &Stmt, qctx: Arc<QueryContext>) -> Result<SubPlan, PlannerError>;
}

// 修改后
trait Planner {
    fn transform(&mut self, stmt: &Stmt, qctx: Arc<QueryContext>) -> Result<SubPlan, PlannerError>;
    
    /// 使用预计算的优化决策生成计划
    fn transform_with_decision(
        &mut self,
        stmt: &Stmt,
        qctx: Arc<QueryContext>,
        decision: &OptimizationDecision,
    ) -> Result<SubPlan, PlannerError>;
    
    /// 计算优化决策（用于缓存未命中时）
    fn compute_decision(
        &mut self,
        stmt: &Stmt,
        qctx: Arc<QueryContext>,
    ) -> Result<OptimizationDecision, PlannerError>;
}
```

## 8. 预期收益

1. **内存效率**：决策对象大小约为完整计划树的 10-20%
2. **缓存命中率**：版本感知使缓存更智能，减少不必要的失效
3. **适应性**：统计信息小幅变化时，可以基于旧决策快速调整
4. **可测试性**：决策是纯粹的数据结构，易于单元测试

## 9. 风险评估

1. **复杂度增加**：需要维护决策到计划的转换逻辑
2. **序列化成本**：决策需要序列化以支持持久化（如果需要）
3. **一致性风险**：决策与应用时必须确保统计信息版本匹配

## 10. 回滚策略

由于不需要向后兼容，如果出现问题：
1. 可以暂时禁用缓存（设置 max_entries = 0）
2. 或者回滚到旧版本（git revert）
