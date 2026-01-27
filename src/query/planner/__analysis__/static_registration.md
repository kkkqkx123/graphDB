# Planner 静态注册架构设计

**日期**: 2025-01-27  
**版本**: v1.0  
**目标**: 完全消除动态分发，使用静态注册和编译时多态

---

## 一、问题分析

### 1.1 当前动态分发问题

当前设计使用 `Box<dyn Planner>` 进行动态分发：

```rust
// 当前设计 - 动态分发
pub type PlannerInstantiateFunc = fn() -> Box<dyn Planner>;

pub struct MatchAndInstantiate {
    pub match_func: MatchFunc,                    // fn(&AstContext) -> bool
    pub instantiate_func: PlannerInstantiateFunc, // fn() -> Box<dyn Planner>
    pub priority: i32,
}

pub fn make() -> Box<dyn Planner> {
    Box::new(Self::new())
}
```

**问题**：
1. 每次调用规划器都需要虚函数表查找
2. 无法内联优化
3. 运行时类型检查开销
4. 内存分配（Box）

### 1.2 目标：静态分发

**目标设计**：
```rust
// 静态分发 - 无 Box，无 dyn trait
pub enum PlannerEnum {
    Match(MatchPlanner),
    Go(GoPlanner),
    Lookup(LookupPlanner),
    // ... 其他规划器
}

impl PlannerEnum {
    pub fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        match self {
            PlannerEnum::Match(planner) => planner.transform(ast_ctx),
            PlannerEnum::Go(planner) => planner.transform(ast_ctx),
            // ...
        }
    }
}
```

---

## 二、静态注册架构设计

### 2.1 规划器枚举

```rust
/// 所有规划器的静态枚举
/// 完全消除动态分发
#[derive(Debug)]
pub enum PlannerEnum {
    Match(MatchPlanner),
    Go(GoPlanner),
    Lookup(LookupPlanner),
    Path(PathPlanner),
    Subgraph(SubgraphPlanner),
    FetchVertices(FetchVerticesPlanner),
    FetchEdges(FetchEdgesPlanner),
    Maintain(MaintainPlanner),
}

impl PlannerEnum {
    /// 根据语句类型创建规划器
    pub fn from_sentence_kind(kind: SentenceKind) -> Option<Self> {
        match kind {
            SentenceKind::Match => Some(PlannerEnum::Match(MatchPlanner::new())),
            SentenceKind::Go => Some(PlannerEnum::Go(GoPlanner::new())),
            SentenceKind::Lookup => Some(PlannerEnum::Lookup(LookupPlanner::new())),
            SentenceKind::Path => Some(PlannerEnum::Path(PathPlanner::new())),
            SentenceKind::Subgraph => Some(PlannerEnum::Subgraph(SubgraphPlanner::new())),
            SentenceKind::FetchVertices => Some(PlannerEnum::FetchVertices(FetchVerticesPlanner::new())),
            SentenceKind::FetchEdges => Some(PlannerEnum::FetchEdges(FetchEdgesPlanner::new())),
            SentenceKind::Maintain => Some(PlannerEnum::Maintain(MaintainPlanner::new())),
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
        }
    }
}
```

### 2.2 静态规划器注册表

```rust
/// 静态规划器注册表
/// 编译时确定所有规划器
#[derive(Debug)]
pub struct StaticPlannerRegistry {
    planners: Vec<PlannerEnum>,
}

impl StaticPlannerRegistry {
    /// 创建注册表并注册所有规划器
    pub fn new() -> Self {
        Self {
            planners: vec![
                PlannerEnum::Match(MatchPlanner::new()),
                PlannerEnum::Go(GoPlanner::new()),
                PlannerEnum::Lookup(LookupPlanner::new()),
                PlannerEnum::Path(PathPlanner::new()),
                PlannerEnum::Subgraph(SubgraphPlanner::new()),
                PlannerEnum::FetchVertices(FetchVerticesPlanner::new()),
                PlannerEnum::FetchEdges(FetchEdgesPlanner::new()),
                PlannerEnum::Maintain(MaintainPlanner::new()),
            ],
        }
    }

    /// 根据语句类型获取规划器
    pub fn get_planner(&self, kind: SentenceKind) -> Option<&PlannerEnum> {
        Self::get_planner_static(kind)
    }

    /// 获取可变的规划器
    pub fn get_planner_mut(&mut self, kind: SentenceKind) -> Option<&mut PlannerEnum> {
        Self::get_planner_mut_static(kind)
    }

    /// 静态方法 - 根据语句类型获取规划器
    fn get_planner_static(kind: SentenceKind) -> Option<&'static PlannerEnum> {
        match kind {
            SentenceKind::Match => Some(&PLANNERS[0]),
            SentenceKind::Go => Some(&PLANNERS[1]),
            SentenceKind::Lookup => Some(&PLANNERS[2]),
            SentenceKind::Path => Some(&PLANNERS[3]),
            SentenceKind::Subgraph => Some(&PLANNERS[4]),
            SentenceKind::FetchVertices => Some(&PLANNERS[5]),
            SentenceKind::FetchEdges => Some(&PLANNERS[6]),
            SentenceKind::Maintain => Some(&PLANNERS[7]),
        }
    }

    /// 静态方法 - 获取可变规划器
    fn get_planner_mut_static(kind: SentenceKind) -> Option<&'static mut PlannerEnum> {
        match kind {
            SentenceKind::Match => PLANNERS[0].as_mut(),
            SentenceKind::Go => PLANNERS[1].as_mut(),
            SentenceKind::Lookup => PLANNERS[2].as_mut(),
            SentenceKind::Path => PLANNERS[3].as_mut(),
            SentenceKind::Subgraph => PLANNERS[4].as_mut(),
            SentenceKind::FetchVertices => PLANNERS[5].as_mut(),
            SentenceKind::FetchEdges => PLANNERS[6].as_mut(),
            SentenceKind::Maintain => PLANNERS[7].as_mut(),
        }
    }

    /// 创建执行计划
    pub fn create_plan(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let kind = self.extract_sentence_kind(ast_ctx)?;
        
        if let Some(planner) = self.get_planner_mut(kind) {
            if planner.matches(ast_ctx) {
                return planner.transform(ast_ctx);
            }
        }
        
        Err(PlannerError::NoSuitablePlanner(
            "No suitable planner found for the given AST context".to_string(),
        ))
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
}

/// 全局静态规划器数组
static PLANNERS: [PlannerEnum; 8] = [
    PlannerEnum::Match(MatchPlanner::new()),
    PlannerEnum::Go(GoPlanner::new()),
    PlannerEnum::Lookup(LookupPlanner::new()),
    PlannerEnum::Path(PathPlanner::new()),
    PlannerEnum::Subgraph(SubgraphPlanner::new()),
    PlannerEnum::FetchVertices(FetchVerticesPlanner::new()),
    PlannerEnum::FetchEdges(FetchEdgesPlanner::new()),
    PlannerEnum::Maintain(MaintainPlanner::new()),
];
```

### 2.3 规划器 trait（可选，用于类型约束）

```rust
/// 规划器 trait
/// 仅用于泛型约束，不用于动态分发
pub trait IntoPlanner {
    fn into_planner(self) -> PlannerEnum;
}

impl IntoPlanner for MatchPlanner {
    fn into_planner(self) -> PlannerEnum {
        PlannerEnum::Match(self)
    }
}

impl IntoPlanner for GoPlanner {
    fn into_planner(self) -> PlannerEnum {
        PlannerEnum::Go(self)
    }
}

/// 规划器通用接口
pub trait PlannerInterface {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError>;
    fn match_planner(&self, ast_ctx: &AstContext) -> bool;
    fn name(&self) -> &'static str;
}

impl PlannerInterface for MatchPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        self.transform(ast_ctx)
    }
    
    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
    
    fn name(&self) -> &'static str {
        "MatchPlanner"
    }
}

impl PlannerInterface for GoPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        self.transform(ast_ctx)
    }
    
    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
    
    fn name(&self) -> &'static str {
        "GoPlanner"
    }
}
```

---

## 三、规划器实现示例

### 3.1 MatchPlanner 静态化

```rust
/// MATCH 规划器
/// 静态分发版本
#[derive(Debug)]
pub struct MatchPlanner {
    config: MatchPlannerConfig,
}

impl MatchPlanner {
    /// 创建新的 MATCH 规划器
    pub fn new() -> Self {
        Self {
            config: MatchPlannerConfig::default(),
        }
    }

    /// 检查是否匹配 MATCH 语句
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "MATCH"
    }

    /// 转换 MATCH 语句为执行计划
    pub fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // 实现转换逻辑...
        let space_id = ast_ctx.space().space_id.unwrap_or(1) as i32;
        
        // 创建起始节点
        let start_node = StartNode::new();
        let mut plan = SubPlan::from_root(start_node.into_enum());
        
        // 处理 WHERE 子句
        if let Some(where_condition) = self.extract_where_condition(ast_ctx)? {
            plan = self.plan_filter(plan, where_condition, space_id)?;
        }
        
        // 处理 RETURN 子句
        if let Some(return_columns) = self.extract_return_columns(ast_ctx)? {
            plan = self.plan_project(plan, return_columns, space_id)?;
        }
        
        Ok(plan)
    }

    fn extract_where_condition(&self, ast_ctx: &AstContext) -> Result<Option<Expression>, PlannerError> {
        // 实现...
        Ok(None)
    }

    fn extract_return_columns(&self, ast_ctx: &AstContext) -> Result<Option<Vec<YieldColumn>>, PlannerError> {
        // 实现...
        Ok(None)
    }

    fn plan_filter(&self, input_plan: SubPlan, condition: Expression, _space_id: i32) -> Result<SubPlan, PlannerError> {
        let input_node = input_plan.root().as_ref().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string())
        })?;

        let filter_node = FilterNode::new(input_node.clone(), condition)?;
        Ok(SubPlan::new(Some(filter_node.into_enum()), input_plan.tail))
    }

    fn plan_project(&self, input_plan: SubPlan, columns: Vec<YieldColumn>, _space_id: i32) -> Result<SubPlan, PlannerError> {
        let input_node = input_plan.root().as_ref().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string())
        })?;

        let project_node = ProjectNode::new(input_node.clone(), columns)?;
        Ok(SubPlan::new(Some(project_node.into_enum()), input_plan.tail))
    }
}

impl Default for MatchPlanner {
    fn default() -> Self {
        Self::new()
    }
}
```

### 3.2 GoPlanner 静态化

```rust
/// GO 语句规划器
/// 静态分发版本
#[derive(Debug)]
pub struct GoPlanner {}

impl GoPlanner {
    /// 创建新的 GO 规划器
    pub fn new() -> Self {
        Self {}
    }

    /// 检查是否匹配 GO 语句
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "GO"
    }

    /// 转换 GO 语句为执行计划
    pub fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // 实现转换逻辑...
        let arg_node = ArgumentNode::new(0, "");
        let arg_enum = PlanNodeEnum::Argument(arg_node);

        let expand_all = ExpandAllNode::new(1, vec![], "both");
        let expand_enum = PlanNodeEnum::ExpandAll(expand_all);

        // 构建执行计划...
        
        Ok(SubPlan {
            root: Some(expand_enum),
            tail: Some(arg_enum),
        })
    }
}

impl Default for GoPlanner {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## 四、使用方式

### 4.1 基本使用

```rust
use crate::query::planner::planner::{
    SentenceKind, StaticPlannerRegistry, PlannerEnum
};

/// 执行查询规划
pub fn plan_query(ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
    let mut registry = StaticPlannerRegistry::new();
    registry.create_plan(ast_ctx)
}

/// 直接使用规划器枚举
pub fn plan_with_enum(ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
    let kind = SentenceKind::from_str(ast_ctx.statement_type().as_str())?;
    
    let mut planner = PlannerEnum::from_sentence_kind(kind)
        .ok_or_else(|| PlannerError::NoSuitablePlanner(kind.as_str().to_string()))?;
    
    planner.transform(ast_ctx)
}
```

### 4.2 批量规划

```rust
/// 批量规划查询
pub fn plan_batch<'a, I>(ast_contexts: I) -> Vec<Result<SubPlan, PlannerError>>
where
    I: IntoIterator<Item = &'a AstContext>,
{
    ast_contexts
        .into_iter()
        .map(|ctx| plan_query(ctx))
        .collect()
}

/// 并行规划（静态分发，无运行时开销）
pub async fn plan_parallel(ast_contexts: Vec<AstContext>) -> Vec<Result<SubPlan, PlannerError>> {
    use rayon::prelude::*;
    
    ast_contexts
        .into_par_iter()
        .map(|ctx| plan_query(&ctx))
        .collect()
}
```

---

## 五、优势分析

### 5.1 性能优势

| 方面 | 动态分发 (Box<dyn Planner>) | 静态分发 (PlannerEnum) |
|------|---------------------------|------------------------|
| 虚函数查找 | ❌ 每次调用 | ✅ 无 |
| 内联优化 | ❌ 不支持 | ✅ 完全支持 |
| 内存分配 | ❌ 需要 Box | ✅ 无需分配 |
| 线程安全 | ❌ 需要 Arc | ✅ 栈分配 |

### 5.2 代码质量

| 方面 | 动态分发 | 静态分发 |
|------|---------|---------|
| 类型安全 | ✅ 编译时 | ✅ 编译时 |
| 可维护性 | ⚠️ 复杂 | ✅ 清晰 |
| 可测试性 | ⚠️ 需要 mock | ✅ 直接测试 |
| 可扩展性 | ✅ 容易 | ⚠️ 需要修改枚举 |

---

## 六、实施步骤

### 阶段 1：提取公共接口（1 天）

1. 定义 `PlannerInterface` trait
2. 为每个规划器实现该 trait
3. 保持 `Box<dyn Planner>` 向后兼容

### 阶段 2：创建规划器枚举（2 天）

1. 定义 `PlannerEnum` 枚举
2. 实现 `from_sentence_kind()` 方法
3. 实现 `transform()` 方法

### 阶段 3：创建静态注册表（1 天）

1. 定义 `StaticPlannerRegistry` 结构体
2. 实现 `get_planner()` 和 `create_plan()` 方法
3. 创建全局 `PLANNERS` 数组

### 阶段 4：迁移规划器（3 天）

1. 迁移 MatchPlanner
2. 迁移 GoPlanner
3. 迁移其他规划器
4. 移除 `Box<dyn Planner>` 返回类型

### 阶段 5：清理和优化（1 天）

1. 移除旧的动态分发代码
2. 更新使用点
3. 运行性能测试

---

## 七、完整代码示例

### 7.1 模块结构

```
src/query/planner/
├── mod.rs                      # 模块入口
├── planner.rs                  # 规划器核心（静态分发版）
├── statements/
│   ├── mod.rs
│   ├── match_planner.rs        # 静态化
│   ├── go_planner.rs           # 静态化
│   ├── lookup_planner.rs       # 静态化
│   └── ...
└── plan/
    └── ...
```

### 7.2 planner.rs 完整代码

```rust
//! 规划器模块 - 静态分发版本
//! 完全消除动态分发，使用编译时多态

use crate::query::context::ast::AstContext;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;

/// 语句类型枚举
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

/// 规划器通用接口
pub trait PlannerInterface {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError>;
    fn match_planner(&self, ast_ctx: &AstContext) -> bool;
    fn name(&self) -> &'static str;
}

/// 规划器枚举 - 静态分发核心
#[derive(Debug)]
pub enum PlannerEnum {
    Match(crate::query::planner::statements::MatchPlanner),
    Go(crate::query::planner::statements::GoPlanner),
    Lookup(crate::query::planner::statements::LookupPlanner),
    Path(crate::query::planner::statements::PathPlanner),
    Subgraph(crate::query::planner::statements::SubgraphPlanner),
    FetchVertices(crate::query::planner::statements::FetchVerticesPlanner),
    FetchEdges(crate::query::planner::statements::FetchEdgesPlanner),
    Maintain(crate::query::planner::statements::MaintainPlanner),
}

impl PlannerEnum {
    pub fn from_sentence_kind(kind: SentenceKind) -> Option<Self> {
        match kind {
            SentenceKind::Match => Some(PlannerEnum::Match(MatchPlanner::new())),
            SentenceKind::Go => Some(PlannerEnum::Go(GoPlanner::new())),
            SentenceKind::Lookup => Some(PlannerEnum::Lookup(LookupPlanner::new())),
            SentenceKind::Path => Some(PlannerEnum::Path(PathPlanner::new())),
            SentenceKind::Subgraph => Some(PlannerEnum::Subgraph(SubgraphPlanner::new())),
            SentenceKind::FetchVertices => Some(PlannerEnum::FetchVertices(FetchVerticesPlanner::new())),
            SentenceKind::FetchEdges => Some(PlannerEnum::FetchEdges(FetchEdgesPlanner::new())),
            SentenceKind::Maintain => Some(PlannerEnum::Maintain(MaintainPlanner::new())),
        }
    }

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
        }
    }

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
        }
    }

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
        }
    }
}

/// 静态规划器注册表
#[derive(Debug, Default)]
pub struct StaticPlannerRegistry {
    planners: Vec<PlannerEnum>,
}

impl StaticPlannerRegistry {
    pub fn new() -> Self {
        Self {
            planners: vec![
                PlannerEnum::Match(MatchPlanner::new()),
                PlannerEnum::Go(GoPlanner::new()),
                PlannerEnum::Lookup(LookupPlanner::new()),
                PlannerEnum::Path(PathPlanner::new()),
                PlannerEnum::Subgraph(SubgraphPlanner::new()),
                PlannerEnum::FetchVertices(FetchVerticesPlanner::new()),
                PlannerEnum::FetchEdges(FetchEdgesPlanner::new()),
                PlannerEnum::Maintain(MaintainPlanner::new()),
            ],
        }
    }

    pub fn create_plan(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let kind = SentenceKind::from_str(ast_ctx.statement_type().as_str())
            .map_err(|_| PlannerError::NoSuitablePlanner("Unknown statement type".to_string()))?;

        if let Some(planner) = self.planners.iter_mut().find(|p| {
            p.name() == kind.as_str() && p.matches(ast_ctx)
        }) {
            return planner.transform(ast_ctx);
        }

        Err(PlannerError::NoSuitablePlanner(
            "No suitable planner found".to_string(),
        ))
    }
}

/// 便捷函数 - 创建规划器
pub fn create_planner(kind: SentenceKind) -> Option<PlannerEnum> {
    PlannerEnum::from_sentence_kind(kind)
}

/// 便捷函数 - 执行规划
pub fn plan(ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
    let mut registry = StaticPlannerRegistry::new();
    registry.create_plan(ast_ctx)
}
```

---

## 八、总结

### 核心改进

| 改进项 | 原来 | 现在 |
|--------|------|------|
| 分发方式 | 动态 (Box<dyn Planner>) | 静态 (PlannerEnum) |
| 内存分配 | 堆分配 (Box) | 栈分配 |
| 内联优化 | 不支持 | 完全支持 |
| 类型安全 | ✅ 编译时 | ✅ 编译时 |

### 预期效果

1. **性能提升**：消除虚函数调用和内存分配开销
2. **代码清晰**：使用枚举表达类型变体
3. **易于维护**：编译时确定所有类型
4. **测试友好**：无需 mock 即可测试

---

**版本**: v1.0  
**创建时间**: 2025-01-27  
**更新历史**: 初始版本
