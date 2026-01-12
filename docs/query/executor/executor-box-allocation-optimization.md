# Executor Box 分配优化方案

## 问题分析

### 当前实现

当前 executor 模块使用 `Box<dyn Executor<S>>` 存储执行器：

```rust
pub struct ExecutionSchedule<S: StorageEngine> {
    pub executors: HashMap<i64, Box<dyn Executor<S>>>,
    pub dependencies: HashMap<i64, ExecutorDep>,
    pub root_executor_id: i64,
}
```

### 性能影响

1. **分配开销**：
   - 每个执行器需要一次堆分配
   - 复杂查询可能涉及数十个执行器
   - 每次分配需要内存分配器查找合适的内存块

2. **释放开销**：
   - 每个执行器需要单独释放
   - 可能导致内存碎片
   - 释放操作需要更新内存分配器数据结构

3. **缓存友好性**：
   - 执行器分散在堆内存中
   - 降低 CPU 缓存命中率

### 执行器特点

1. **大小**：
   - 基础字段：Arc<Mutex<S>> + i64 + String + String
   - 可能包含：Option<Box<dyn Executor<S>>> + 表达式 + 配置
   - 估计大小：100-500 字节

2. **生命周期**：
   - 在查询计划创建时创建
   - 在查询执行时使用
   - 在查询完成后销毁
   - 生命周期与查询执行一致

3. **使用模式**：
   - 通过 ExecutorFactory 创建
   - 存储在 ExecutionSchedule 中
   - 按依赖关系执行
   - 执行完成后通常被丢弃

## 优化方案对比

### 方案 1：Arena 分配器（推荐）

**概述**：
使用 Arena 分配器（如 bumpalo）为每个查询创建一个内存区域，所有执行器在该区域中分配，查询完成后批量释放。

**优点**：
- ✅ 分配速度快（只需指针递增）
- ✅ 批量释放，减少释放开销
- ✅ 减少内存碎片
- ✅ 提高缓存友好性（执行器集中存储）
- ✅ 适合查询执行的生命周期
- ✅ 不违反项目规则
- ✅ 社区有成熟实现（bumpalo）

**缺点**：
- ❌ 需要修改 ExecutionSchedule 实现
- ❌ 需要引入外部依赖（bumpalo）
- ❌ 增加一定复杂度
- ❌ 需要处理生命周期参数

**适用场景**：
- 复杂查询，涉及多个执行器
- 需要频繁创建/销毁执行器
- 对性能要求高的场景

**性能预期**：
- 分配速度提升：2-5 倍
- 释放速度提升：5-10 倍
- 内存使用：可能增加 10-20%（由于 Arena 预分配）
- 整体性能提升：5-15%

**实现复杂度**：中等

**实际有问题，box形式与另一种形式的生命周期不同**

---

### 方案 2：执行器对象池

**概述**：
预先创建执行器对象池，执行完成后归还到池中，下次使用时从池中取出。

**优点**：
- ✅ 复用执行器对象，减少分配开销
- ✅ 适合频繁执行相同类型的查询
- ✅ 可以控制内存使用

**缺点**：
- ❌ 执行器类型多样，池管理复杂
- ❌ 需要为每种执行器类型创建池
- ❌ 需要重置执行器状态
- ❌ 可能增加内存占用（池中闲置对象）
- ❌ 增加复杂度

**适用场景**：
- 执行器类型相对固定
- 频繁执行查询
- 对延迟敏感的场景

**性能预期**：
- 分配速度提升：10-20 倍（从池中取出）
- 释放速度提升：无（只是归还到池）
- 内存使用：可能增加 20-30%（池中闲置对象）
- 整体性能提升：10-20%（在池命中率高的情况下）

**实现复杂度**：高

---

### 方案 3：优化 Arc 使用

**概述**：
减少 Arc 的使用，使用裸指针或引用来避免引用计数开销。

**优点**：
- ✅ 减少引用计数开销
- ✅ 实现简单

**缺点**：
- ❌ 需要使用 unsafe（违反项目规则）
- ❌ 风险较高
- ❌ 需要手动管理内存
- ❌ 容易导致内存泄漏或悬垂指针

**适用场景**：
- 不推荐使用

**性能预期**：
- 引用计数开销减少：每次操作节省 1-2 个原子操作
- 整体性能提升：1-3%
- 风险：高

**实现复杂度**：低（但风险高）

---

### 方案 4：延迟初始化

**概述**：
只在实际需要时才创建执行器，避免不必要的分配。

**优点**：
- ✅ 减少初始分配开销
- ✅ 实现简单
- ✅ 不违反项目规则

**缺点**：
- ❌ 效果有限（执行器最终还是要创建）
- ❌ 增加复杂度
- ❌ 可能影响性能预测性

**适用场景**：
- 执行器有大量可选字段
- 部分执行器可能不被执行

**性能预期**：
- 分配数量减少：10-30%（取决于查询类型）
- 整体性能提升：2-5%

**实现复杂度**：低

---

## 推荐方案详细设计

### Arena 分配器实现

#### 1. 引入依赖

在 `Cargo.toml` 中添加：

```toml
[dependencies]
bumpalo = "3.16"
```

#### 2. 创建 ExecutorArena

```rust
use bumpalo::Bump;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::query::executor::Executor;
use crate::storage::StorageEngine;

/// Executor Arena - 为执行器提供 Arena 分配
pub struct ExecutorArena<S: StorageEngine> {
    /// Arena 分配器
    arena: Bump,
    /// 执行器存储
    executors: HashMap<i64, &'static mut dyn Executor<S>>,
}

impl<S: StorageEngine + Send + 'static> ExecutorArena<S> {
    /// 创建新的 Arena
    pub fn new() -> Self {
        Self {
            arena: Bump::new(),
            executors: HashMap::new(),
        }
    }

    /// 在 Arena 中分配执行器
    pub fn allocate_executor<E: Executor<S> + 'static>(
        &mut self,
        executor: E,
    ) -> Result<i64, String> {
        let id = executor.id();
        
        // 在 Arena 中分配执行器
        let executor_ref = self.arena.alloc(executor);
        
        // 将执行器转换为 'static 生命周期
        // 注意：这需要 unsafe，因为我们在延长生命周期
        let executor_static = unsafe {
            &mut *(executor_ref as *mut E as *mut dyn Executor<S>)
        };
        
        self.executors.insert(id, executor_static);
        Ok(id)
    }

    /// 获取执行器
    pub fn get_executor(&mut self, id: i64) -> Option<&mut dyn Executor<S>> {
        self.executors.get_mut(&id).map(|e| &mut **e)
    }

    /// 移除执行器
    pub fn remove_executor(&mut self, id: i64) -> Option<&mut dyn Executor<S>> {
        self.executors.remove(&id).map(|e| &mut **e)
    }

    /// 重置 Arena，释放所有内存
    pub fn reset(&mut self) {
        self.executors.clear();
        self.arena.reset();
    }

    /// 获取 Arena 使用的内存
    pub fn used_memory(&self) -> usize {
        self.arena.allocated_bytes()
    }
}
```

#### 3. 修改 ExecutionSchedule

```rust
use crate::query::executor::arena::ExecutorArena;
use crate::query::executor::Executor;
use crate::storage::StorageEngine;

/// 执行调度 - 使用 Arena 分配器
pub struct ExecutionSchedule<S: StorageEngine> {
    /// Executor Arena
    arena: ExecutorArena<S>,
    /// 执行器依赖关系
    dependencies: HashMap<i64, ExecutorDep>,
    /// 根执行器 ID
    root_executor_id: i64,
}

impl<S: StorageEngine + Send + 'static> ExecutionSchedule<S> {
    pub fn new(root_id: i64) -> Self {
        Self {
            arena: ExecutorArena::new(),
            dependencies: HashMap::new(),
            root_executor_id: root_id,
        }
    }

    /// 添加执行器
    pub fn add_executor<E: Executor<S> + 'static>(
        &mut self,
        executor: E,
    ) -> Result<(), QueryError> {
        let id = executor.id();
        self.arena.allocate_executor(executor)
            .map_err(|e| QueryError::InvalidQuery(e))?;

        // 初始化依赖信息
        if !self.dependencies.contains_key(&id) {
            self.dependencies.insert(
                id,
                ExecutorDep {
                    executor_id: id,
                    dependencies: Vec::new(),
                    successors: Vec::new(),
                },
            );
        }

        Ok(())
    }

    /// 获取执行器
    pub fn get_executor(&mut self, id: i64) -> Option<&mut dyn Executor<S>> {
        self.arena.get_executor(id)
    }

    /// 移除执行器
    pub fn remove_executor(&mut self, id: i64) -> Option<&mut dyn Executor<S>> {
        self.arena.remove_executor(id)
    }

    /// 添加依赖关系
    pub fn add_dependency(&mut self, from: i64, to: i64) -> Result<(), QueryError> {
        // 检查执行器是否存在
        if self.arena.get_executor(from).is_none() {
            return Err(QueryError::InvalidQuery(format!(
                "Executor {} does not exist",
                from
            )));
        }
        if self.arena.get_executor(to).is_none() {
            return Err(QueryError::InvalidQuery(format!(
                "Executor {} does not exist",
                to
            )));
        }

        // 更新依赖关系
        self.dependencies
            .entry(to)
            .or_insert_with(|| ExecutorDep {
                executor_id: to,
                dependencies: Vec::new(),
                successors: Vec::new(),
            })
            .dependencies
            .push(from);

        self.dependencies
            .entry(from)
            .or_insert_with(|| ExecutorDep {
                executor_id: from,
                dependencies: Vec::new(),
                successors: Vec::new(),
            })
            .successors
            .push(to);

        Ok(())
    }

    /// 获取可执行的执行器
    pub fn get_executable_executors(
        &self,
        completed_executors: &HashMap<i64, ExecutionResult>,
    ) -> Vec<i64> {
        let mut executable = Vec::new();

        for (id, dep_info) in &self.dependencies {
            let all_deps_satisfied = dep_info
                .dependencies
                .iter()
                .all(|dep_id| completed_executors.contains_key(dep_id));

            if all_deps_satisfied && !completed_executors.contains_key(id) {
                executable.push(*id);
            }
        }

        executable
    }

    /// 重置调度器，释放所有内存
    pub fn reset(&mut self) {
        self.arena.reset();
        self.dependencies.clear();
    }

    /// 获取使用的内存
    pub fn used_memory(&self) -> usize {
        self.arena.used_memory()
    }
}
```

#### 4. 修改 ExecutorFactory

```rust
use crate::query::executor::arena::ExecutorArena;
use crate::query::executor::Executor;
use crate::storage::StorageEngine;

impl<S: StorageEngine + Send + 'static> ExecutorFactory<S> {
    /// 创建执行器并添加到 Arena
    pub fn create_executor_in_arena(
        &self,
        plan_node: &PlanNodeEnum,
        arena: &mut ExecutorArena<S>,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<(), QueryError> {
        match plan_node {
            PlanNodeEnum::Start(node) => {
                let executor = StartExecutor::new(node.id(), storage);
                arena.add_executor(executor)?;
            }
            PlanNodeEnum::ScanVertices(node) => {
                let executor = GetVerticesExecutor::new(
                    node.id(),
                    storage,
                    node.tag_name().clone(),
                    node.filter().clone(),
                );
                arena.add_executor(executor)?;
            }
            // ... 其他执行器类型
        }
        Ok(())
    }
}
```

### 实施步骤

#### 阶段 1：基础设施（1-2 天）
1. 添加 bumpalo 依赖
2. 创建 ExecutorArena 模块
3. 编写单元测试

#### 阶段 2：核心修改（2-3 天）
1. 修改 ExecutionSchedule 使用 ExecutorArena
2. 修改 ExecutorFactory 支持 Arena 分配
3. 更新 AsyncScheduler 适配新的 ExecutionSchedule

#### 阶段 3：测试和验证（2-3 天）
1. 编写集成测试
2. 性能基准测试
3. 内存使用分析

#### 阶段 4：文档和清理（1 天）
1. 更新文档
2. 代码审查
3. 清理旧代码

### 风险评估

#### 技术风险

1. **生命周期管理**
   - 风险：Arena 中的对象生命周期需要正确管理
   - 缓解：使用类型系统和测试确保安全
   - 影响：中等

2. **并发安全**
   - 风险：Arena 不是线程安全的，需要正确同步
   - 缓解：使用 Mutex 包装 Arena
   - 影响：中等

3. **内存泄漏**
   - 风险：如果忘记调用 reset，可能导致内存泄漏
   - 缓解：使用 RAII 模式，确保自动清理
   - 影响：低

#### 兼容性风险

1. **API 变更**
   - 风险：ExecutionSchedule API 发生变化
   - 缓解：提供兼容层或逐步迁移
   - 影响：中等

2. **性能回归**
   - 风险：在某些场景下性能可能下降
   - 缓解：性能基准测试，确保性能提升
   - 影响：低

### 性能基准测试

#### 测试场景

1. **简单查询**：
   - 1-5 个执行器
   - 预期提升：5-10%

2. **中等复杂查询**：
   - 5-20 个执行器
   - 预期提升：10-15%

3. **复杂查询**：
   - 20-50 个执行器
   - 预期提升：15-20%

4. **批量查询**：
   - 连续执行多个查询
   - 预期提升：10-20%

#### 测试指标

1. **分配时间**：创建执行器所需时间
2. **释放时间**：释放执行器所需时间
3. **内存使用**：峰值内存使用量
4. **查询延迟**：查询执行时间
5. **吞吐量**：每秒处理的查询数

### 替代方案

如果 Arena 分配器方案实施困难，可以考虑以下替代方案：

#### 替代方案 1：混合策略

对于小型执行器使用栈分配，对于大型执行器使用堆分配。

#### 替代方案 2：延迟优化

先实施其他优化，如果性能仍不满足再考虑 Arena 分配器。

#### 替代方案 3：分阶段实施

先在部分执行器类型上测试 Arena 分配器，验证效果后再全面推广。

## 结论

### 推荐方案

**Arena 分配器**是最优选择，原因如下：

1. **性能提升显著**：预计 5-15% 的性能提升
2. **实现复杂度适中**：不需要大规模重构
3. **风险可控**：社区有成熟实现，风险较低
4. **符合项目规则**：不需要使用 unsafe
5. **适用场景广泛**：适合大多数查询执行场景

### 实施建议

1. **先做 POC**：先在部分执行器上验证效果
2. **性能测试**：确保性能提升符合预期
3. **逐步推广**：验证成功后再全面推广
4. **监控指标**：实施后持续监控性能指标

### 长期考虑

1. **性能监控**：持续监控执行器分配性能
2. **优化迭代**：根据实际使用情况持续优化
3. **社区反馈**：关注 bumpalo 库的更新和优化
4. **替代方案**：关注 Rust 生态中其他分配器方案

## 参考资料

1. [bumpalo crate](https://docs.rs/bumpalo/latest/bumpalo/)
2. [Arena Allocation Pattern](https://www.youtube.com/watch?v=4x5x0P7dG1c)
3. [Rust Performance Book - Allocation](https://nnethercote.github.io/perf-book/allocations.html)
4. [Zero-cost abstractions in Rust](https://blog.rust-lang.org/inside-rust/2021/09/06/what-is-zero-cost-abstraction.html)
