# CSR Metrics 整合设计决策

## 问题陈述

项目中存在两个独立的 metrics 体系：
1. **集中式**：`core::stats::StatsManager` —— 统一管理查询、搜索、存储、事务等全局指标
2. **孤立式**：`CsrMetrics` —— 边存储层的独立 metrics，未被实际使用

### CsrMetrics 的问题

| 问题 | 影响 |
|------|------|
| **未被集成** | CSR 实现中完全没有调用 CsrMetrics（仅在单元测试中出现） |
| **指标碎片化** | CSR 性能数据无法通过 StatsManager 访问 |
| **模式不一致** | 项目使用**装饰者模式**（MetricsStorage、MetricsSearchEngine），但 CSR 采用**嵌入式设计** |
| **代码重复** | 与 StatsManager 中的存储指标重复实现计数逻辑 |

## 解决方案：方案 C — 扩展 StatsManager

直接在 `StatsManager` 中添加 CSR 特定的 `MetricType` 条目和记录方法。

### 实现细节

#### 1. MetricType 扩展

```rust
pub enum MetricType {
    // 存在的指标...
    
    // CSR 新增指标
    CsrInsertions,              // 边插入次数
    CsrDeletions,               // 边删除次数
    CsrOverflowExpansions,      // 顶点容量扩展次数
    CsrCompactions,             // 压缩操作次数
    CsrEdgesCompacted,          // 压缩移除的总边数
    CsrBytesAllocated,          // 当前分配的字节数
}
```

#### 2. StatsManager 新增方法

```rust
impl StatsManager {
    /// 记录 CSR 边插入
    pub fn record_csr_insertion(&self)
    
    /// 记录 CSR 边删除
    pub fn record_csr_deletion(&self)
    
    /// 记录溢出扩展（顶点容量增长）
    pub fn record_csr_overflow_expansion(&self)
    
    /// 记录压缩操作
    pub fn record_csr_compaction(&self, edges_removed: u64)
    
    /// 记录内存分配
    pub fn record_csr_allocation(&self, bytes: u64)
    
    /// 设置当前分配字节数快照
    pub fn set_csr_bytes_allocated(&self, bytes: u64)
}
```

#### 3. 文件变更

- **删除**：`crates/graphdb-storage/src/storage/edge/performance_metrics.rs`
- **修改**：移除 `mod.rs` 中的 `performance_metrics` 模块和 `CsrMetrics` 导出

## 优势

✅ **统一可观测性**  
所有存储层指标（包括 CSR）都通过 StatsManager 访问

✅ **架构一致性**  
遵循项目已有的装饰者模式和指标组织方式

✅ **代码简洁**  
消除重复实现，CSR 实现只需调用 StatsManager 方法

✅ **易于查询**  
通过统一接口查询 CSR 性能、生成报告

✅ **低侵入**  
无需修改 CSR 实现本身，可渐进式集成

## 迁移路径

### 立即可用

```rust
// 在 CSR 实现中
let stats = stats_manager.clone();

// MutableCsr::insert_edge
stats.record_csr_insertion();

// MutableCsr::delete_edge
stats.record_csr_deletion();

// 扩展顶点容量时
stats.record_csr_overflow_expansion();

// 压缩后
stats.record_csr_compaction(edges_removed);
stats.set_csr_bytes_allocated(current_bytes);
```

### 查询指标

```rust
let insertions = stats.get_value(MetricType::CsrInsertions);
let compactions = stats.get_value(MetricType::CsrCompactions);
let current_bytes = stats.get_value(MetricType::CsrBytesAllocated);

// 与其他存储指标一起查询
let all_metrics = stats.get_all_metrics();
```

## 与现有指标的关系

| 现有指标 | CSR 新增指标 | 关系 |
|---------|-------------|------|
| StorageReadOps | CsrDeletions | 读取操作（遍历） |
| StorageWriteOps | CsrInsertions | 写入操作（插入） |
| StorageErrors | — | 共享错误计数 |
| StorageCacheHitCount | — | 共享缓存统计 |

CSR 指标补充了存储层的细粒度操作计数，而通用存储指标保持高层次抽象。

## 版本兼容性

**无兼容性破坏**  
- CsrMetrics 未被任何生产代码使用（仅测试）
- 删除是干净的，无影响

## 未来扩展

如需追踪更多 CSR 特定指标，只需：
1. 在 `MetricType` 中添加新条目
2. 在 `StatsManager` 中添加对应方法
3. 在 CSR 实现中调用该方法

示例：
```rust
pub enum MetricType {
    CsrCompactionDurationUs,    // 压缩耗时
    CsrFragmentationRatio,      // 碎片率
}
```
