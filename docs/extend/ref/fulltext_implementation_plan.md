# 全文检索服务集成实现计划

## 概述

基于 gRPC 服务架构，集成 ref/bm25 或 ref/inversearch 服务到 GraphDB。

## 阶段一：基础框架搭建（第 1-2 天）

### 任务 1.1：添加 gRPC 依赖
**时间**: 0.5 天
**优先级**: 高

```toml
# Cargo.toml
[dependencies]
tonic = { version = "0.12", optional = true }
prost = { version = "0.14", optional = true }

[features]
default = ["redb", "embedded", "server", "c-api"]
fulltext = ["dep:tonic", "dep:prost"]

[build-dependencies]
tonic-build = { version = "0.12", optional = true }
```

**验收标准**:
- [ ] `cargo check --features fulltext` 通过
- [ ] 特性开关工作正常

---

### 任务 1.2：创建模块结构
**时间**: 0.5 天
**优先级**: 高

```
src/storage/fulltext/
├── mod.rs              # 模块入口
├── types.rs            # 类型定义
├── client.rs           # gRPC 客户端
├── sync.rs             # 数据同步管理器
├── config.rs           # 配置
└── error.rs            # 错误类型
```

**验收标准**:
- [ ] 目录结构创建完成
- [ ] 基础代码框架编译通过

---

### 任务 1.3：定义核心类型
**时间**: 1 天
**优先级**: 高

**文件**: `src/storage/fulltext/types.rs`

```rust
/// 全文检索配置
#[derive(Debug, Clone, Deserialize)]
pub struct FulltextConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub timeout_ms: u64,
    pub retry_count: u32,
    pub sync: SyncConfig,
}

/// 同步配置
#[derive(Debug, Clone, Deserialize)]
pub struct SyncConfig {
    pub mode: SyncMode,
    pub queue_size: usize,
    pub batch_size: usize,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum SyncMode {
    Sync,
    Async,
    Off,
}

/// 全文搜索结果
#[derive(Debug, Clone)]
pub struct FulltextResult {
    pub doc_id: Value,
    pub score: f32,
    pub highlights: Option<Vec<String>>,
}

/// 索引映射信息
#[derive(Debug, Clone)]
pub struct IndexMapping {
    pub space_id: u64,
    pub tag_name: String,
    pub field_name: String,
    pub index_name: String,
    pub provider: String,
}
```

**验收标准**:
- [ ] 所有类型定义完成
- [ ] 实现 Serialize/Deserialize
- [ ] 单元测试通过

---

## 阶段二：gRPC 客户端实现（第 3-5 天）

### 任务 2.1：引入 proto 文件
**时间**: 0.5 天
**优先级**: 高

**操作**:
1. 复制 `ref/bm25/proto/bm25.proto` 到 `proto/bm25.proto`
2. 创建 `build.rs` 编译 proto

```rust
// build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "fulltext")]
    {
        tonic_build::compile_protos("proto/bm25.proto")?;
    }
    Ok(())
}
```

**验收标准**:
- [ ] proto 文件编译成功
- [ ] 生成的代码能正常引用

---

### 任务 2.2：实现 FulltextClient
**时间**: 2 天
**优先级**: 高

**文件**: `src/storage/fulltext/client.rs`

```rust
pub struct FulltextClient {
    client: Bm25ServiceClient<Channel>,
    config: FulltextConfig,
}

impl FulltextClient {
    pub async fn connect(config: FulltextConfig) -> Result<Self>;
    
    /// 创建索引
    pub async fn create_index(&mut self, index_name: &str) -> Result<()>;
    
    /// 删除索引
    pub async fn drop_index(&mut self, index_name: &str) -> Result<()>;
    
    /// 索引文档
    pub async fn index_document(
        &mut self,
        index_name: &str,
        doc_id: &str,
        fields: HashMap<String, String>,
    ) -> Result<()>;
    
    /// 批量索引
    pub async fn batch_index_documents(
        &mut self,
        index_name: &str,
        documents: Vec<(String, HashMap<String, String>)>,
    ) -> Result<usize>;
    
    /// 删除文档
    pub async fn delete_document(
        &mut self,
        index_name: &str,
        doc_id: &str,
    ) -> Result<()>;
    
    /// 搜索
    pub async fn search(
        &mut self,
        index_name: &str,
        query: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<FulltextResult>>;
    
    /// 获取统计
    pub async fn get_stats(&mut self, index_name: &str) -> Result<IndexStats>;
}
```

**验收标准**:
- [ ] 所有方法实现完成
- [ ] 错误处理完善
- [ ] 单元测试覆盖率 > 80%

---

### 任务 2.3：连接池管理
**时间**: 0.5 天
**优先级**: 中

**功能**:
- 连接复用
- 自动重连
- 健康检查

**验收标准**:
- [ ] 连接池正常工作
- [ ] 断线自动重连
- [ ] 并发安全

---

## 阶段三：数据同步实现（第 6-8 天）

### 任务 3.1：实现 FulltextSyncManager
**时间**: 2 天
**优先级**: 高

**文件**: `src/storage/fulltext/sync.rs`

```rust
pub struct FulltextSyncManager {
    client: FulltextClient,
    index_mappings: Arc<RwLock<HashMap<IndexKey, String>>>,
    config: SyncConfig,
    // 异步队列
    async_queue: Option<mpsc::Sender<SyncTask>>,
}

#[derive(Debug)]
pub enum SyncTask {
    IndexDocument {
        index_name: String,
        doc_id: String,
        fields: HashMap<String, String>,
    },
    DeleteDocument {
        index_name: String,
        doc_id: String,
    },
}

impl FulltextSyncManager {
    /// 顶点插入时同步
    pub async fn on_vertex_inserted(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
        properties: &HashMap<String, Value>,
    ) -> Result<()>;
    
    /// 顶点更新时同步
    pub async fn on_vertex_updated(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
        properties: &HashMap<String, Value>,
    ) -> Result<()>;
    
    /// 顶点删除时同步
    pub async fn on_vertex_deleted(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
    ) -> Result<()>;
    
    /// 批量同步
    pub async fn batch_sync(&self, tasks: Vec<SyncTask>) -> Result<()>;
}
```

**验收标准**:
- [ ] 同步逻辑正确
- [ ] 支持同步/异步模式
- [ ] 错误处理完善

---

### 任务 3.2：集成到存储层
**时间**: 1.5 天
**优先级**: 高

**修改文件**:
- `src/storage/redb_storage.rs` - 添加同步钩子
- `src/storage/vertex_storage.rs` - 插入/更新/删除时触发同步

```rust
// 在 VertexStorage::insert_vertex 中添加
if let Some(ref sync_manager) = self.fulltext_sync {
    sync_manager.on_vertex_inserted(
        space_id, tag_name, vertex_id, &properties
    ).await?;
}
```

**验收标准**:
- [ ] 数据变更自动同步
- [ ] 不影响原有存储性能
- [ ] 集成测试通过

---

### 任务 3.3：异步队列实现
**时间**: 0.5 天
**优先级**: 中

**功能**:
- 批量处理
- 失败重试
- 队列满处理策略

**验收标准**:
- [ ] 异步队列工作正常
- [ ] 批量处理提升性能
- [ ] 内存使用可控

---

## 阶段四：查询层集成（第 9-11 天）

### 任务 4.1：实现 FulltextScanExecutor
**时间**: 2 天
**优先级**: 高

**文件**: `src/query/executor/data_access/fulltext_scan.rs`

```rust
pub struct FulltextScanExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    index_name: String,
    query: String,
    limit: Option<usize>,
}

#[async_trait::async_trait]
impl<S: StorageClient + Send + 'static> Executor for FulltextScanExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 1. 调用全文搜索
        let results = self.storage
            .fulltext_search(&self.index_name, &self.query, self.limit.unwrap_or(100))
            .await?;
        
        // 2. 根据 doc_ids 查询完整数据
        let mut rows = Vec::new();
        for result in results {
            if let Some(vertex) = self.storage.get_vertex(&result.doc_id).await? {
                rows.push(self.build_row(vertex, result.score));
            }
        }
        
        Ok(ExecutionResult::new(rows))
    }
}
```

**验收标准**:
- [ ] 执行器能正确执行
- [ ] 结果格式正确
- [ ] 单元测试通过

---

### 任务 4.2：扩展查询解析器
**时间**: 1.5 天
**优先级**: 中

**支持的语法**:
```sql
-- MATCH 表达式
WHERE field MATCH "query"

-- CONTAINS 表达式
WHERE field CONTAINS "query"

-- 评分函数
score(vertex) as relevance
```

**修改文件**:
- `src/query/parser/ast/expr.rs`
- `src/query/parser/parser/expr_parser.rs`

**验收标准**:
- [ ] 新语法能正确解析
- [ ] 解析器测试通过

---

### 任务 4.3：查询计划生成
**时间**: 0.5 天
**优先级**: 中

**功能**:
- 识别全文检索条件
- 生成 FulltextScan 计划节点
- 成本估算

**验收标准**:
- [ ] 计划生成正确
- [ ] 集成测试通过

---

## 阶段五：SQL 语法实现（第 12-13 天）

### 任务 5.1：CREATE FULLTEXT INDEX
**时间**: 1 天
**优先级**: 高

**语法**:
```sql
CREATE FULLTEXT INDEX index_name ON tag_name(field_name)
[USING 'bm25' | 'inversearch']
[WITH TOKENIZER = 'standard' | 'cjk']
```

**实现**:
- 解析器扩展
- 执行器实现
- 元数据存储

**验收标准**:
- [ ] SQL 语法正确执行
- [ ] 索引元数据持久化
- [ ] 服务通知成功

---

### 任务 5.2：DROP FULLTEXT INDEX
**时间**: 0.5 天
**优先级**: 高

**语法**:
```sql
DROP FULLTEXT INDEX index_name
```

**验收标准**:
- [ ] 索引删除成功
- [ ] 服务通知成功
- [ ] 元数据清理

---

### 任务 5.3：REBUILD FULLTEXT INDEX
**时间**: 0.5 天
**优先级**: 低

**语法**:
```sql
REBUILD FULLTEXT INDEX index_name
```

**功能**:
- 全量重新索引
- 用于数据修复

**验收标准**:
- [ ] 重建功能正常
- [ ] 进度反馈

---

## 阶段六：测试与优化（第 14-17 天）

### 任务 6.1：单元测试
**时间**: 2 天
**优先级**: 高

**测试范围**:
- [ ] FulltextClient 测试
- [ ] FulltextSyncManager 测试
- [ ] 执行器测试
- [ ] 解析器测试

**验收标准**:
- [ ] 测试覆盖率 > 80%
- [ ] 所有测试通过

---

### 任务 6.2：集成测试
**时间**: 2 天
**优先级**: 高

**测试场景**:
1. 启动 BM25 服务
2. 创建全文索引
3. 插入顶点数据
4. 执行全文搜索
5. 验证结果正确性
6. 删除顶点数据
7. 验证索引更新

**验收标准**:
- [ ] 完整流程测试通过
- [ ] 性能指标达标

---

### 任务 6.3：性能测试
**时间**: 1 天
**优先级**: 中

**测试指标**:
| 指标 | 目标值 |
|------|--------|
| 单次搜索延迟 | < 20ms |
| 批量索引速度 | > 3000 doc/s |
| 并发查询 | 支持 100+ QPS |

**验收标准**:
- [ ] 性能指标达标
- [ ] 性能报告生成

---

## 阶段七：文档与部署（第 18-19 天）

### 任务 7.1：使用文档
**时间**: 1 天
**优先级**: 中

**文档内容**:
- [ ] 快速开始指南
- [ ] SQL 语法参考
- [ ] 配置说明
- [ ] 故障排查

---

### 任务 7.2：部署脚本
**时间**: 0.5 天
**优先级**: 中

**脚本**:
- `scripts/start-fulltext.ps1` - 启动全文服务
- `scripts/stop-fulltext.ps1` - 停止全文服务
- `scripts/start-all.ps1` - 一键启动所有服务

---

### 任务 7.3：示例代码
**时间**: 0.5 天
**优先级**: 低

**示例**:
- 基本搜索示例
- 批量索引示例
- 配置示例

---

## 里程碑

| 里程碑 | 日期 | 交付物 |
|--------|------|--------|
| M1 | 第 2 天 | 基础框架完成 |
| M2 | 第 5 天 | gRPC 客户端完成 |
| M3 | 第 8 天 | 数据同步完成 |
| M4 | 第 11 天 | 查询层集成完成 |
| M5 | 第 13 天 | SQL 语法完成 |
| M6 | 第 17 天 | 测试完成 |
| M7 | 第 19 天 | 文档与部署完成 |

---

## 依赖项

### 外部服务
- ref/bm25 服务已编译可用
- ref/inversearch 服务已编译可用（可选）

### 内部依赖
- gRPC 客户端库
- 异步运行时 (tokio)
- 序列化库 (prost)

---

## 风险评估

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| 服务连接不稳定 | 中 | 高 | 实现重试和熔断机制 |
| 数据同步延迟 | 中 | 中 | 提供同步模式选项 |
| 性能不达标 | 低 | 高 | 提前进行原型验证 |
| proto 版本不兼容 | 低 | 高 | 锁定 proto 版本 |
