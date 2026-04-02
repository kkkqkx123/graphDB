# BM25 Service

BM25 全文搜索服务，基于 Tantivy 搜索引擎实现。

## 功能特性

- 基于 BM25 算法的全文搜索
- 文档索引、更新、删除
- 字段加权
- 查询缓存
- 结果高亮
- gRPC 接口

## 快速开始

### 构建项目

```bash
cargo build --release
```

### 运行服务

```bash
cargo run --release
```

### 配置

服务支持通过环境变量或配置文件进行配置：

#### 环境变量

- `SERVER_ADDRESS`: 服务监听地址 (默认: 0.0.0.0:50051)
- `REDIS_URL`: Redis 连接 URL (默认: redis://localhost:6379)
- `DATA_DIR`: 数据目录 (默认: ./data)
- `INDEX_PATH`: 索引目录 (默认: ./index)

#### 配置文件

编辑 `configs/config.toml` 文件进行配置。

## 开发

### 运行测试

```bash
cargo test
```

### 运行 Clippy

```bash
cargo clippy
```

### 格式化代码

```bash
cargo fmt
```

## gRPC 接口

服务提供以下 gRPC 接口：

- `IndexDocument`: 索引单个文档
- `BatchIndexDocuments`: 批量索引文档
- `Search`: 搜索文档
- `DeleteDocument`: 删除文档
- `GetStats`: 获取统计信息

## 技术栈

- Rust 2021
- Tantivy: 搜索引擎
- Tokio: 异步运行时
- Tonic: gRPC 框架
- Redis: 缓存和存储
