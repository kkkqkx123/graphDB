# 全文检索集成实施计划

## 概述

本文档基于以下架构设计文档制定：
- [全文检索架构设计决策文档](../fulltext_architecture_decision.md)
- [嵌入式全文检索集成设计方案](../fulltext_embedded_design.md)
- [全文检索嵌入式集成分析报告](../fulltext_embedding_analysis.md)

## 实施阶段总览

全文检索功能将分 **5 个阶段** 逐步实现：

| 阶段 | 名称 | 工期 | 主要交付物 | 依赖 |
|------|------|------|------------|------|
| Phase 1 | SearchEngine Trait 和适配器 | 3-5 天 | 引擎抽象层、BM25/Inversearch 适配器 | 无 |
| Phase 2 | FulltextCoordinator 协调器 | 4-6 天 | 索引管理器、协调器 | Phase 1 |
| Phase 3 | 查询引擎集成 | 5-7 天 | SQL 语法扩展、查询执行器 | Phase 2 |
| Phase 4 | 数据同步机制 | 4-6 天 | 同步管理器、批量处理器 | Phase 3 |
| Phase 5 | 测试与优化 | 5-7 天 | 完整测试套件、性能优化 | Phase 4 |

**总预计工期**: 21-31 天

---

## 各阶段详细说明

### Phase 1: SearchEngine Trait 和适配器实现

**目标**: 建立全文检索的基础抽象层

**核心内容**:
- 定义 `SearchEngine` Trait，统一 BM25 和 Inversearch 接口
- 实现 BM25 适配器（包装 bm25-service 库）
- 实现 Inversearch 适配器（包装 inversearch-service 库）
- 错误类型和结果结构定义

**关键文件**:
- `src/search/engine.rs` - Trait 定义
- `src/search/adapters/bm25_adapter.rs` - BM25 适配器
- `src/search/adapters/inversearch_adapter.rs` - Inversearch 适配器

**验收标准**:
- [ ] 两个引擎都实现 `SearchEngine` Trait
- [ ] 单元测试通过
- [ ] 代码通过 clippy 检查

---

### Phase 2: FulltextCoordinator 协调器实现

**目标**: 实现程序层面的索引管理和协调

**核心内容**:
- 实现 `FulltextIndexManager` 管理索引生命周期
- 实现 `FulltextCoordinator` 协调数据变更和索引更新
- 索引元数据管理
- 搜索引擎工厂

**关键文件**:
- `src/search/manager.rs` - 索引管理器
- `src/coordinator/fulltext.rs` - 协调器
- `src/search/factory.rs` - 引擎工厂

**验收标准**:
- [ ] 支持创建、删除、搜索索引
- [ ] 支持 BM25 和 Inversearch 两种引擎
- [ ] 所有单元测试通过

---

### Phase 3: 查询引擎集成

**目标**: 在 nGQL 中支持全文搜索语法

**核心内容**:
- SQL 语法扩展（CREATE FULLTEXT INDEX, MATCH 等）
- AST 节点定义
- 查询计划生成
- 查询执行器实现

**支持语法**:
```sql
-- 创建索引
CREATE FULLTEXT INDEX idx_content ON Post(content) USING bm25;

-- 全文搜索
MATCH (p:Post) WHERE p.content MATCH "图数据库" RETURN p;

-- 带评分排序
MATCH (p:Post) WHERE p.content MATCH "图数据库" 
RETURN p, score(p) as relevance ORDER BY relevance DESC;
```

**验收标准**:
- [ ] 支持所有规划的 SQL 语法
- [ ] 全文搜索查询正确返回结果
- [ ] 集成测试通过

---

### Phase 4: 数据同步机制

**目标**: 实现图数据变更与全文索引的异步同步

**核心内容**:
- 同步任务定义和队列
- 批量提交处理器
- 同步管理器（支持 Sync/Async/Off 模式）
- 后台任务调度

**同步模式**:
| 模式 | 说明 | 适用场景 |
|------|------|----------|
| Sync | 阻塞等待索引完成 | 强一致性要求 |
| Async | 提交到队列立即返回 | 默认推荐 |
| Off | 不更新全文索引 | 维护模式 |

**验收标准**:
- [ ] 三种同步模式正常工作
- [ ] 异步模式不阻塞主事务
- [ ] 批量提交功能正常

---

### Phase 5: 测试与优化

**目标**: 全面测试和性能优化

**核心内容**:
- 单元测试（覆盖率 > 80%）
- 集成测试
- 性能测试和基准测试
- 性能优化（批量处理、缓存等）

**性能目标**:
| 指标 | 目标值 |
|------|--------|
| 单次搜索延迟 | < 5ms (P95) |
| 批量索引速度 | > 5000 doc/s |
| 内存占用 | < 300MB (10万文档) |
| 并发搜索 | > 1000 QPS |

**验收标准**:
- [ ] 测试覆盖率达标
- [ ] 性能测试达到目标值
- [ ] 文档完整

---

## 架构层次

```
┌─────────────────────────────────────────┐
│  查询引擎 (Query Engine)                 │
│  - SQL Parser                           │
│  - AST                                  │
│  - Query Planner                        │
│  - Query Executor                       │
└─────────────────────────────────────────┘
                    │
┌─────────────────────────────────────────┐
│  程序层面 (Application Layer)            │
│  ┌─────────────────────────────────┐    │
│  │  FulltextCoordinator            │    │
│  │  SyncManager                    │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
                    │
┌─────────────────────────────────────────┐
│  服务层 (Service Layer)                  │
│  ┌─────────────────────────────────┐    │
│  │  SearchEngine Trait             │    │
│  │  FulltextIndexManager           │    │
│  │  ├─ Bm25SearchEngine            │    │
│  │  └─ InversearchEngine           │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
                    │
┌─────────────────────────────────────────┐
│  存储层 (Storage Layer)                  │
│  ┌─────────────────────────────────┐    │
│  │  RedbStorage                    │    │
│  │  - 图数据存储 (纯净)             │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
```

---

## 实施建议

### 1. 开发顺序

建议按阶段顺序依次开发，每个阶段完成后进行充分测试再进入下一阶段。

### 2. 并行工作

- Phase 1 和 Phase 2 可以部分并行（先完成 Trait，再开始协调器）
- Phase 5 的测试用例可以在开发过程中同步编写

### 3. 风险缓解

| 风险 | 缓解措施 |
|------|----------|
| 工期延误 | 每个阶段预留 20% 缓冲时间 |
| 性能不达标 | Phase 3 后开始性能测试，预留优化时间 |
| API 不匹配 | Phase 1 尽早验证 BM25/Inversearch API |

### 4. 代码审查

建议每个阶段完成后进行代码审查，确保：
- 符合项目编码规范
- 错误处理完善
- 文档注释完整

---

## 相关文档

- [Phase 1: SearchEngine Trait 和适配器](./phase1_search_engine_trait.md)
- [Phase 2: FulltextCoordinator 协调器](./phase2_fulltext_coordinator.md)
- [Phase 3: 查询引擎集成](./phase3_query_engine_integration.md)
- [Phase 4: 数据同步机制](./phase4_data_sync_mechanism.md)
- [Phase 5: 测试与优化](./phase5_testing_and_optimization.md)

---

## 变更记录

| 日期 | 版本 | 变更内容 |
|------|------|----------|
| 2026-04-02 | 1.0 | 初始版本 |
