# RwLock<HashMap> 与 DashMap 性能分析文档

## 分析日期
2026-02-24

## 分析背景
对项目中所有使用 `RwLock<HashMap>` 或 `RwLock` 包装集合类型的场景进行分析，评估是否应替换为 `Arc<DashMap>` 以获得更好的并发性能。

## 核心结论

**只有 `TransactionManager` 是明确值得替换的** - 它是全局共享的、高并发、读写混合的核心组件。

其他模块要么不适合，要么收益有限。

---

## 深度对比分析

### 1. 核心机制差异

#### `RwLock<HashMap>` (标准库组合)
*   **原理**：使用一个全局的读写锁保护整个 `HashMap`。
    *   **读**：多个线程可以同时获取"读锁"进行读取。
    *   **写**：写入时需要获取"写锁"，此时会阻塞所有其他的读和写操作。
*   **粒度**：**粗粒度（Coarse-grained）**。锁的是整个 Map，而不是单个 Key。
*   **瓶颈**：只要有一个线程在写入，整个 Map 对其他人都是不可见的。即使两个线程操作的是完全不同的 Key，它们也会互相阻塞。

#### `Arc<DashMap>` (第三方库 `dashmap`)
*   **原理**：基于分片（Sharding）技术。内部将数据分散到多个独立的段（Segment）中，每个段有自己的 `RwLock`。
*   **粒度**：**细粒度（Fine-grained）**。通过哈希计算定位到具体的段，只锁定该段。
*   **优势**：不同 Key 的操作很可能落在不同的段上，从而实现真正的并行读写。即使发生哈希冲突，也只会阻塞同一段内的操作，不会影响其他段。

---

### 2. 性能场景详细对比

| 场景特征 | `RwLock<HashMap>` 表现 | `Arc<DashMap>` 表现 | 获胜者 |
| :--- | :--- | :--- | :--- |
| **高并发写入** | **极差**。写锁互斥，所有写操作串行化，成为严重瓶颈。 | **优秀**。写入分散到不同分片，并行度高。 | **DashMap** |
| **高并发读取 (无写入)** | **良好**。读锁共享，但存在锁竞争开销。 | **优秀**。读操作分散，几乎无锁竞争。 | **DashMap** (略优) |
| **读写混合 (高竞争)** | **差**。写操作会阻塞所有读操作，导致读延迟抖动大。 | **好**。读写干扰小，延迟更稳定。 | **DashMap** |
| **低并发 / 单线程** | **极佳**。无分片开销，内存布局紧凑，CPU 缓存命中率高。 | **一般**。分片机制带来额外的内存占用和哈希计算开销。 | **RwLock** |
| **数据量很小 (<100 项)** | **极佳**。锁竞争概率低，结构简单。 | **较差**。分片带来的 overhead 占比过大。 | **RwLock** |
| **需要迭代整个 Map** | **简单高效**。持有一个读锁即可安全遍历。 | **复杂/慢**。需要依次锁定所有分片，或者使用特殊的迭代器，性能较差且可能阻塞写入。 | **RwLock** |

---

### 3. 选择建议

**选择 `Arc<DashMap>` 如果：**
1.  **高并发场景**：线程数较多（例如 > 4 个活跃线程）。
2.  **写操作频繁**：或者有大量的读写混合操作。
3.  **Key 分布均匀**：哈希碰撞较少，能充分利用分片优势。
4.  **不需要频繁全量遍历**：主要是点查（Get/Set/Delete）。
5.  **追求高吞吐量**：愿意牺牲少量内存换取性能。

**选择 `RwLock<HashMap>` 如果：**
1.  **读多写极少**：且写操作非常稀疏，几乎感觉不到锁竞争。
2.  **低并发**：线程数很少，或者大部分时间是单线程运行。
3.  **数据量很小**：Map 中元素很少，锁竞争本身就不是问题。
4.  **需要频繁全量遍历**：例如定期导出所有数据。
5.  **极度敏感于内存占用**：无法接受 `DashMap` 的分片内存开销。
6.  **不想引入第三方依赖**：希望只用标准库。

---

## 各模块详细分析

### 1. TransactionManager (`active_transactions`)

**当前实现**：
```rust
pub struct TransactionManager {
    active_transactions: Arc<RwLock<HashMap<TransactionId, Arc<TransactionContext>>>>,
}
```

**使用模式**：
- 写入：事务开始（insert）、提交/回滚（remove）
- 读取：获取上下文、检查活跃状态、清理任务
- **读写比例**：约 1:1 到 1:2（每个事务至少一次读+写）
- **并发度**：**极高** - 所有数据库操作都要经过这里

**分析**：
- ✅ **强烈推荐替换为 DashMap**
- 高并发 + 读写混合 = DashMap 的优势场景
- 事务ID分布均匀（自增），哈希冲突少
- 预估性能提升：**5x-15x**

**建议修改**：
```rust
pub struct TransactionManager {
    active_transactions: Arc<DashMap<TransactionId, Arc<TransactionContext>>>,
}
```

---

### 2. PasswordAuthenticator (`login_attempts`)

**当前实现**：
```rust
pub struct PasswordAuthenticator {
    login_attempts: Arc<RwLock<HashMap<String, LoginAttempt>>>,
}
```

**使用模式**：
- 写入：登录失败时更新计数、登录成功时清除
- 读取：验证时检查剩余次数
- **读写比例**：约 1:1
- **并发度**：中等 - 登录高峰期并发高
- **数据量**：小（仅记录失败的用户）

**分析**：
- ⚠️ **可选替换，收益有限**
- 虽然读写混合，但数据量很小
- 登录是低频操作（相比数据库查询）
- DashMap 的分片开销可能抵消收益

**建议**：保持现状，除非有明确性能瓶颈

---

### 3. TwoPhaseCoordinator

**当前实现**：
```rust
pub struct TwoPhaseCoordinator {
    transactions: RwLock<HashMap<TwoPhaseId, Arc<RwLock<TwoPhaseTransaction>>>>,
    txn_to_2pc: RwLock<HashMap<TransactionId, TwoPhaseId>>,
}
```

**使用模式**：
- `transactions`: 2PC事务管理
- `txn_to_2pc`: 事务ID映射
- **读写比例**：写入（begin/remove）多于读取
- **并发度**：取决于是否使用2PC

**分析**：
- ⚠️ **视情况而定**
- 如果项目**实际使用2PC**：建议替换
- 如果2PC只是预留功能（当前单节点架构）：保持现状
- 当前单节点部署可能用不到2PC

---

### 4. PermissionManager

**当前实现**：
```rust
pub struct PermissionManager {
    user_roles: Arc<RwLock<HashMap<String, HashMap<i64, RoleType>>>>,
    space_permissions: Arc<RwLock<HashMap<i64, HashMap<String, Vec<Permission>>>>>,
}
```

**使用模式**：
- `user_roles`: 用户角色查询（读多写少）
- `space_permissions`: 空间权限（读写都有）
- **读写比例**：约 10:1 到 100:1（权限在会话建立后很少变更）
- **并发度**：中等 - 每个查询都要检查权限

**分析**：
- ❌ **不建议替换**
- **读多写少**场景下，`RwLock` 的读锁共享已经足够高效
- 权限数据量小（用户数量有限）
- DashMap 的分片开销在此场景下是负收益
- 遍历需求：`is_god()`、`is_admin()` 需要遍历所有角色，DashMap 遍历性能差

---

### 5. SavepointManager

**当前实现**：
```rust
pub struct SavepointManager {
    sequence_generator: RwLock<HashMap<TransactionId, AtomicU64>>,
    savepoints: RwLock<HashMap<SavepointId, Arc<RwLock<Savepoint>>>>,
    txn_savepoints: RwLock<HashMap<TransactionId, Vec<SavepointId>>>,
}
```

**使用模式**：
- `savepoints`: 保存点存储
- `txn_savepoints`: 事务到保存点的映射
- **读写比例**：创建（写）和回滚（读写）混合
- **并发度**：中等 - 依赖事务并发度
- **特殊需求**：`rollback_to_savepoint` 需要遍历事务的所有保存点

**分析**：
- ⚠️ **部分替换，需重构**
- `savepoints`: 可以替换为 `DashMap<SavepointId, Arc<RwLock<Savepoint>>>`
- `txn_savepoints`: ❌ **不能简单替换** - 存储的是 `Vec<SavepointId>`，需要按顺序遍历
- 建议：保持现状，或仅替换 `savepoints` 字段

---

### 6. ClientSession

**当前实现**：
```rust
pub struct ClientSession {
    roles: Arc<RwLock<HashMap<i64, RoleType>>>,
    contexts: Arc<RwLock<HashMap<u32, String>>>,
    // ... 其他字段
}
```

**使用模式**：
- `roles`: 用户角色（读多写少，通常会话建立后不变）
- `contexts`: 查询上下文（频繁插入/删除）
- **读写比例**：
  - `roles`: 约 100:1（会话建立时写入，之后只读）
  - `contexts`: 约 1:1（每个查询开始和结束都要更新）
- **并发度**：每个会话独立，无跨会话竞争

**分析**：
- ❌ **不建议替换**
- **关键点**：`ClientSession` 是**每个会话一个实例**，不是全局共享的
- 会话之间的数据不共享，不存在锁竞争
- 即使是 `Arc<ClientSession>`，也是单个会话内部的操作
- DashMap 的优势在于**跨线程并发**，单会话内部无收益

---

### 7. QueryContext

**当前实现**：
```rust
pub struct QueryContext {
    plan: RwLock<Option<Box<ExecutionPlan>>>,
    space_info: RwLock<Option<SpaceInfo>>,
}
```

**使用模式**：
- `plan`: 执行计划（一次写入，多次读取）
- `space_info`: 空间信息（一次写入，多次读取）

**分析**：
- ❌ **不适合替换**
- 存储的是 `Option<T>`，不是集合类型
- 写入极少，RwLock 读锁共享已足够

---

### 8. ThreadSafeObjectPool

**当前实现**：
```rust
pub struct ThreadSafeObjectPool<T: Clone + Send + 'static> {
    pool: Arc<RwLock<Vec<T>>>,
}
```

**使用模式**：
- 对象池，栈语义（LIFO）

**分析**：
- ❌ **不适合替换**
- 存储的是 `Vec<T>`，不是键值对
- 需要栈/队列语义，不是 Map 语义

---

## 最终建议汇总

| 模块 | 当前实现 | 建议 | 理由 |
|------|----------|------|------|
| **TransactionManager** | `RwLock<HashMap>` | ✅ **替换为 DashMap** | 高并发+读写混合，核心性能路径 |
| **PasswordAuthenticator** | `RwLock<HashMap>` | ❌ 保持现状 | 数据量小，低频操作 |
| **TwoPhaseCoordinator** | `RwLock<HashMap>` | ⚠️ 视使用情况 | 单节点可能用不到2PC |
| **PermissionManager** | `RwLock<HashMap>` | ❌ 保持现状 | 读多写少，遍历需求 |
| **SavepointManager** | `RwLock<HashMap>` | ⚠️ 可选部分替换 | 有遍历需求，结构复杂 |
| **ClientSession** | `RwLock<HashMap>` | ❌ 保持现状 | 会话隔离，无跨线程竞争 |
| **QueryContext** | `RwLock<Option>` | ❌ 保持现状 | 非集合类型 |
| **ThreadSafeObjectPool** | `RwLock<Vec>` | ❌ 保持现状 | 非Map语义 |

---

## 实施优先级

1. **🔴 高优先级**：`TransactionManager` - 这是唯一一个明确能从 DashMap 获得显著收益的模块
2. **🟡 中优先级**：`TwoPhaseCoordinator` - 确认2PC使用情况后决定
3. **🟢 低优先级**：其他模块 - 当前实现已足够，替换收益不明显

---

## 注意事项

1. **避免过度优化**：DashMap 虽然有优势，但也有内存开销和API差异，不应该无脑替换所有 RwLock<HashMap>
2. **测试覆盖**：替换后需要充分测试并发场景
3. **迭代器语义**：DashMap 的迭代器会持有分片锁，需要注意
4. **内存开销**：DashMap 有一定内存开销（分片数组）
