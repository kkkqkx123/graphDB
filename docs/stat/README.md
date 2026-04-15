# GraphDB 统计、监控与遥测文档

本目录包含 GraphDB 项目的统计、监控和遥测体系的完整文档。

---

## 文档导航

### 📚 核心文档

| 文档 | 描述 | 适用对象 |
|------|------|----------|
| [架构文档](architecture.md) | 详细的架构说明，包括模块结构、数据流、设计决策 | 架构师、开发人员 |
| [迁移总结](migration_summary.md) | 从传统计数器向 metrics crate 迁移的完整过程 | 开发人员、技术负责人 |
| [使用指南](usage_guide.md) | 如何使用指标系统的实践指南 | 开发人员、运维人员 |

### 📋 快速参考

**想了解架构？**
- 👉 阅读 [架构文档](architecture.md) 的"整体架构"章节

**想记录指标？**
- 👉 阅读 [使用指南](usage_guide.md) 的"记录指标"章节

**想查询指标？**
- 👉 阅读 [使用指南](usage_guide.md) 的"查询指标"章节

**想集成监控？**
- 👉 阅读 [使用指南](usage_guide.md) 的"集成监控系统"章节

---

## 架构概览

### 三层架构

```
┌─────────────────────────────────────────┐
│          应用层 (Application)           │
├─────────────────────────────────────────┤
│        监控层 (Monitoring)              │
│  GlobalMetrics | Telemetry | CacheStats │
├─────────────────────────────────────────┤
│        统计层 (Statistics)              │
│  StatsManager | QueryProfile | Errors   │
└─────────────────────────────────────────┘
```

### 核心模块

| 模块 | 位置 | 职责 |
|------|------|------|
| **StatsManager** | `core::stats` | 管理全局统计、慢查询日志 |
| **GlobalMetrics** | `core::stats` | Prometheus 风格全局指标 |
| **Telemetry** | `api::telemetry` | 指标收集、HTTP 暴露 |
| **CacheStats** | `core::stats::utils` | 统一缓存统计 |
| **业务指标** | 各业务模块 | Sync、Search、Storage 指标 |

---

## 快速开始

### 1. 记录指标

```rust
use metrics::counter;

// 记录查询数
counter!("graphdb_query_total").increment(1);

// 记录带标签的指标
counter!("graphdb_error_by_type_total", "type" => "timeout").increment(1);
```

### 2. 查询指标

```bash
# 启动 Telemetry 服务器
# 访问 http://localhost:9090/metrics

# 获取 JSON 格式
curl http://localhost:9090/metrics?format=json

# 获取 Prometheus 格式
curl http://localhost:9090/metrics
```

### 3. 集成 Prometheus

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'graphdb'
    static_configs:
      - targets: ['localhost:9090']
```

---

## 主要特性

### ✅ 统一指标收集

- 基于 `metrics` crate
- Prometheus 兼容
- 支持标签、直方图、百分位数

### ✅ 多样化输出

- JSON 格式：程序化处理
- Plain Text 格式：Prometheus 兼容
- 支持按前缀过滤

### ✅ 高性能

- 使用 DashMap 减少锁竞争
- 原子操作优化
- 内存占用低

### ✅ 易于扩展

- 简单的指标注册
- 灵活的标签支持
- 易于集成外部系统

---

## 内置指标

### 查询指标

- `graphdb_query_total`: 总查询数
- `graphdb_query_duration_seconds`: 查询延迟
- `graphdb_query_active`: 活跃查询数
- `graphdb_query_*_total`: 按类型分类

### 存储指标

- `graphdb_storage_scan_total`: 扫描次数
- `graphdb_storage_cache_hits_total`: 缓存命中
- `graphdb_storage_cache_misses_total`: 缓存未命中

### 执行器指标

- `graphdb_executor_rows_processed_total`: 处理行数
- `graphdb_executor_memory_used_bytes`: 内存使用

### 错误指标

- `graphdb_error_total`: 错误总数
- `graphdb_error_by_type_total{type}`: 按类型分类

---

## 迁移成果

### 代码简化

- 📉 减少约 **41%** 的代码
- 🔧 移除 **3** 个冗余结构体/trait
- 📝 减少 **58%** 的方法

### 性能提升

- ⚡ 减少 **50%** 的 Atomic 操作
- 💾 每个实例节省 **30-50 bytes**
- 🎯 消除双重计数

### 维护性提升

- 📖 代码更简洁
- 🧪 测试更容易
- 📚 文档更清晰

详细迁移过程见 [迁移总结](migration_summary.md)。

---

## 最佳实践

### ✅ DO（推荐）

- 使用统一的命名规范
- 使用标签进行维度划分
- 批量记录指标
- 设置合理的告警阈值

### ❌ DON'T（避免）

- 混用不同的命名风格
- 滥用标签（基数爆炸）
- 逐条记录指标
- 忽略告警配置

详见 [使用指南](usage_guide.md) 的"最佳实践"章节。

---

## 故障排查

### 常见问题

| 问题 | 可能原因 | 解决方案 |
|------|----------|----------|
| 指标未记录 | Recorder 未设置 | 检查初始化代码 |
| 内存占用高 | 直方图过多 | 启用清理机制 |
| 性能下降 | 记录过于频繁 | 批量记录 |

详见 [使用指南](usage_guide.md) 的"故障排查"章节。

---

## 更新日志

### v1.0 (2026-04-15)

- ✨ 完成 metrics crate 迁移
- ✨ 统一缓存统计实现
- ✨ 统一时间精度为微秒
- 📝 创建完整文档体系

---

## 相关资源

### 内部文档

- [架构文档](architecture.md)
- [迁移总结](migration_summary.md)
- [使用指南](usage_guide.md)

### 外部资源

- [metrics crate](https://docs.rs/metrics/)
- [Prometheus](https://prometheus.io/)
- [Grafana](https://grafana.com/)

---

## 贡献指南

欢迎贡献！请遵循以下步骤：

1. Fork 项目
2. 创建特性分支
3. 提交更改
4. 推送到分支
5. 创建 Pull Request

---

**文档版本**：1.0  
**最后更新**：2026-04-15  
**维护者**：GraphDB Team
