# 待完成任务清单

## 已完成

- ProcessorBase - 基础处理器模板类
- StorageProcessorExecutor - 存储处理器执行器
- StorageProcessorExecutorImpl - 执行器实现 trait
- GetVerticesExecutor - 顶点获取执行器
- GetNeighborsExecutor - 邻居获取执行器
- ScanVerticesExecutor - 顶点扫描执行器
- GetEdgesExecutor - 边获取执行器
- ScanEdgesExecutor - 边扫描执行器

## 待完成

### T-001: 测试 Fixture 更新

修复测试代码中的 MockStorage/DummyStorage 实现。

涉及文件：
- src/storage/tests/mod.rs
- src/query/executor/tests/mod.rs

需要实现 StorageClient trait 的方法：
- get
- put
- delete
- scan

---

### T-002: IndexScanExecutor 迁移

迁移索引扫描执行器到新架构。

涉及文件：src/query/executor/data_access.rs

要求：
- 实现 StorageProcessorExecutorImpl trait
- 支持索引扫描结果返回

---

### T-003: GetPropExecutor 迁移

迁移属性获取执行器到新架构。

涉及文件：src/query/executor/data_access.rs

要求：
- 实现 StorageProcessorExecutorImpl trait
- 支持属性获取

---

### T-004: ScanEdgesExecutor 验证

验证边扫描执行器迁移是否正确。

涉及文件：src/query/executor/data_access.rs

---

### T-005: InsertVertexExecutor

实现基于新架构的顶点插入执行器。

---

### T-006: InsertEdgeExecutor

实现基于新架构的边插入执行器。

---

### T-007: UpdateVertexExecutor

实现基于新架构的顶点更新执行器。

---

### T-008: DeleteVertexExecutor

实现基于新架构的顶点删除执行器。

---

### T-009: 内存监控优化

优化内存使用阈值配置和回收机制。

---

### T-010: 执行统计完善

添加更多统计指标和导出接口。

---

### T-011: 批量操作优化

优化批量读取性能。

---

### T-012: 并发控制改进

优化锁使用，提高并发性能。
