# CoreQueryContext 设计问题分析与改进方案

## 概述

本文档详细分析 GraphDB 中 `CoreQueryContext` 的设计问题，对比 Nebula-Graph 的实现，并提供改进方案。

---

## 一、架构对比

### 1.1 Nebula-Graph QueryContext (C++)

```cpp
class QueryContext {
 private:
  RequestContextPtr rctx_;
  std::unique_ptr<ValidateContext> vctx_;
  std::unique_ptr<ExecutionContext> ectx_;
  std::unique_ptr<ExecutionPlan> ep_;
  meta::SchemaManager* sm_;
  meta::IndexManager* im_;
  storage::StorageClient* storageClient_;
  meta::MetaClient* metaClient_;
  CharsetInfo* charsetInfo_;
  std::unique_ptr<ObjectPool> objPool_;
  std::unique_ptr<IdGenerator> idGen_;
  std::unique_ptr<SymbolTable> symTable_;
  std::atomic<bool> killed_;
};
```

### 1.2 当前 GraphDB CoreQueryContext (Rust)

```rust
pub struct CoreQueryContext {
    vctx: ValidationContext,
    ectx: ExecutionContext,
    plan: Option<ExecutionPlan>,
    sym_table: SymbolTable,
    id_gen: IdGenerator,
    obj_pool: ObjectPool<String>,
}
```

---

## 二、设计问题分析

### 问题1：缺失外部组件引用

| 组件 | Nebula-Graph | GraphDB | 严重性 |
|------|-------------|---------|--------|
| SchemaManager | `sm_` | ❌ 缺失 | 高 |
| IndexManager | `im_` | ❌ 缺失 | 高 |
| StorageClient | `storageClient_` | ❌ 缺失 | 高 |
| MetaClient | `metaClient_` | ❌ 缺失 | 中 |
| CharsetInfo | `charsetInfo_` | ❌ 缺失 | 低 |
| RequestContext | `rctx_` | ❌ 缺失 | 高 |

**影响**：CoreQueryContext 无法访问存储层和元数据层，导致查询无法实际执行。

### 问题2：缺少查询生命周期控制

Nebula-Graph 提供了优雅的查询终止机制：

```cpp
void markKilled() {
    killed_.exchange(true);
}

bool isKilled() const {
    return killed_.load();
}
```

GraphDB 没有等效机制，无法中断长时间运行的查询。

### 问题3：执行计划所有权API设计不直观

```rust
// 当前设计 - 不直观
pub fn plan_mut(&mut self) -> &mut Option<ExecutionPlan> {
    &mut self.plan
}

// 使用时需要额外解包
ctx.plan_mut().as_mut().unwrap()  // 冗长且可能panic
```

### 问题4：ObjectPool 泛型约束不合理

当前设计只能存储字符串对象，而 Nebula-Graph 的 ObjectPool 用于存储：
- 表达式对象
- 计划节点对象
- 执行器对象

```rust
// 当前设计 - 过于局限
obj_pool: ObjectPool<String>,

// 建议：存储任意对象
obj_pool: ObjectPool<Box<dyn Any>>,
```

### 问题5：符号表并发设计过度

使用 `DashMap` 实现线程安全，但在单节点场景下过于复杂：

```rust
pub struct SymbolTable {
    symbols: Arc<DashMap<String, Symbol>>,
}
```

**问题**：
- DashMap 适用于多线程高并发写场景
- 单节点 GraphDB 同一时间通常只有一个查询执行
- 增加不必要的内存开销

---

## 三、问题严重性分级

| 问题 | 等级 | 影响范围 |
|------|------|----------|
| 缺失 SchemaManager/StorageClient 引用 | **严重** | 查询无法执行 |
| 缺少查询终止机制 (killed) | **严重** | 无法中断长时间查询 |
| 计划所有权API设计不直观 | **中等** | 使用体验差 |
| ObjectPool 泛型限制 | **中等** | 无法存储复杂对象 |
| 与 RequestContext 分离 | **中等** | 无法访问请求/响应 |
| 符号表并发设计过度 | **低** | 性能开销 |

---

## 四、改进方案

### 4.1 CoreQueryContext 改进结构

```rust
pub struct CoreQueryContext {
    vctx: ValidationContext,
    ectx: ExecutionContext,
    plan: Option<ExecutionPlan>,
    sym_table: SymbolTable,
    id_gen: IdGenerator,
    obj_pool: ObjectPool<Box<dyn Any>>,

    // 新增：组件访问器
    components: Option<QueryComponents>,

    // 新增：请求上下文引用
    rctx: Option<Arc<RequestContext>>,

    // 新增：查询终止标志
    killed: Arc<AtomicBool>,
}
```

### 4.2 关键方法改进

```rust
impl CoreQueryContext {
    // 改进：直接获取可变计划引用
    pub fn plan_mut(&mut self) -> Option<&mut ExecutionPlan> {
        self.plan.as_mut()
    }

    // 新增：查询终止控制
    pub fn kill(&self) {
        self.killed.store(true, Ordering::SeqCst);
    }

    pub fn is_killed(&self) -> bool {
        self.killed.load(Ordering::SeqCst)
    }

    // 新增：组件访问
    pub fn components(&self) -> Option<&QueryComponents> {
        self.components.as_ref()
    }

    pub fn components_mut(&mut self) -> Option<&mut QueryComponents> {
        self.components.as_mut()
    }
}
```

### 4.3 ObjectPool 泛型改进

```rust
pub struct ObjectPool<T: Any> {
    pool: Vec<Box<T>>,
    capacity: usize,
}

impl<T: Any> ObjectPool<Box<dyn Any>> {
    pub fn new(capacity: usize) -> Self {
        Self {
            pool: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn allocate<F, R>(&mut self, constructor: F) -> &mut Box<dyn Any>
    where
        F: FnOnce() -> R,
        R: 'static,
    {
        let obj = constructor();
        let boxed: Box<dyn Any> = Box::new(obj);
        self.pool.push(boxed);
        self.pool.last_mut().unwrap()
    }
}
```

---

## 五、实施计划

### Phase 1: 基础修复（高优先级）

1. ✅ 修复 `plan_mut()` 返回类型
2. ✅ 添加 `killed` 标志
3. ✅ 改进 ObjectPool 泛型

### Phase 2: 组件集成（中优先级）

1. 集成 `QueryComponents` 到 CoreQueryContext
2. 集成 `RequestContext` 引用
3. 更新相关测试

### Phase 3: 优化（低优先级）

1. 简化符号表并发设计
2. 优化性能敏感代码路径
3. 添加基准测试

---

## 六、向后兼容性

### 6.1 破坏性变更

以下变更可能影响现有代码：

1. `ObjectPool<String>` -> `ObjectPool<Box<dyn Any>>`
2. 新增 `killed` 字段（添加 Default 实现）
3. `plan_mut()` 返回类型变化

### 6.2 迁移策略

对于 API 变更，提供兼容性包装：

```rust
#[deprecated(since = "0.1.0", note = "使用 plan_mut() 替代")]
pub fn plan_option_mut(&mut self) -> &mut Option<ExecutionPlan> {
    &mut self.plan
}
```

---

## 七、测试验证

### 7.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_mut_returns_direct_reference() {
        let mut ctx = CoreQueryContext::new();
        let plan = ExecutionPlan::new(1);
        ctx.set_plan(plan);

        let plan_ref = ctx.plan_mut();
        assert!(plan_ref.is_some());
        assert_eq!(plan_ref.unwrap().id(), 1);
    }

    #[test]
    fn test_killed_flag() {
        let ctx = CoreQueryContext::new();
        assert!(!ctx.is_killed());

        ctx.kill();
        assert!(ctx.is_killed());
    }
}
```

### 7.2 集成测试

需要验证以下场景：

1. 正常查询执行流程
2. 查询终止功能
3. 组件访问功能
4. 并发安全性

---

## 八、相关文档

- [design_philosophy.md](design_philosophy.md) - 设计哲学分析
- [context_comparison.md](context_comparison.md) - 上下文类型对比
- [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md) - 实施计划
- [README.md](../README.md) - 模块说明文档

---

## 版本历史

| 版本 | 日期 | 作者 | 变更说明 |
|------|------|------|----------|
| 1.0 | 2024-01-30 | GraphDB Team | 初始版本 |
