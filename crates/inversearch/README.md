# Inversearch Service

高性能关键词搜索服务，基于 FlexSearch 核心逻辑的 Rust 实现。

## 功能特性

- 高性能倒排索引
- 支持上下文搜索
- 多种分词模式
- 结果高亮
- 查询缓存
- 持久化存储（Redis）

## 快速开始

### 构建

```bash
cargo build --release
```

### 运行

```bash
cargo run --release
```

### 配置

编辑 `configs/config.toml` 文件配置服务参数。

## 开发

### 运行测试

```bash
cargo test
```

### 运行基准测试

```bash
cargo bench
```

## 文档

详细文档请参考 `docs/plan/inversearch/` 目录。

## 许可证

Apache-2.0
