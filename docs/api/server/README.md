# Server API 文档

本目录包含 GraphDB Server API 的设计文档和实现计划。

## 文档列表

| 文档 | 描述 |
|------|------|
| [api_implementation_plan.md](./api_implementation_plan.md) | Server API 分阶段实现方案 |

## 快速导航

### 实现阶段

1. **[第一阶段：核心功能完善](./api_implementation_plan.md#第一阶段核心功能完善-p0)**
   - 批量操作 API
   - 结构化结果返回
   - 预编译语句 API

2. **[第二阶段：监控与配置](./api_implementation_plan.md#第二阶段监控与配置-p1)**
   - 统计信息 API
   - 配置管理 API

3. **[第三阶段：高级功能](./api_implementation_plan.md#第三阶段高级功能-p2)**
   - 自定义函数 API
   - 流式结果处理

## 相关文档

- [Embedded API 文档](../embedded/README.md) - 嵌入式 API 参考
- [C API 文档](../embedded/c_api.md) - C 语言接口文档
- [API 架构重新设计](../api_architecture_redesign.md) - 整体架构设计

## 当前状态

Server API 目前实现了基础功能：
- ✅ 认证与授权
- ✅ 会话管理
- ✅ 基本查询执行
- ✅ 事务管理
- ✅ Schema 管理

待实现功能：
- ⏳ 批量操作
- ⏳ 预编译语句
- ⏳ 结构化结果
- ⏳ 统计信息
- ⏳ 配置管理
- ⏳ 自定义函数
