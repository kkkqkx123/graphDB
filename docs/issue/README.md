# GraphDB 问题追踪文档

本文档目录用于记录 GraphDB 项目中的已知问题、修复记录和待办事项。

## 文档列表

### 1. [server_startup_fixes.md](./server_startup_fixes.md)
**服务器启动问题修复记录**

记录了服务器启动过程中发现并修复的 5 个关键问题：
- Tokio Runtime 未运行
- 嵌套 Runtime 错误
- VectorManager 连接失败导致 Panic
- shutdown_signal 同步/异步不匹配
- 认证不工作（Login 未创建 Session）

### 2. [e2e_test_failures.md](./e2e_test_failures.md)
**E2E 测试遗留问题汇总**

记录了 E2E 测试中发现但未修复的 10 个问题：
- CREATE SPACE 语法解析错误
- USE SPACE 后上下文未保持
- CREATE TAG 语法差异
- SHOW TAGS / SHOW EDGES 返回空结果
- INSERT VERTEX 语法问题
- MATCH 查询不支持
- GO 遍历查询不支持
- LOOKUP 索引查询不支持
- EXPLAIN / PROFILE 不支持
- Transaction 支持不完整

## 问题优先级

### 高优先级（阻塞基本功能）
1. USE SPACE 上下文保持
2. CREATE SPACE 语法
3. INSERT VERTEX 上下文问题

### 中优先级（影响查询功能）
4. CREATE TAG 语法
5. SHOW TAGS/EDGES
6. MATCH 查询
7. Transaction 支持

### 低优先级（增强功能）
8. GO 查询
9. LOOKUP 查询
10. EXPLAIN/PROFILE

## 如何贡献修复

1. 选择要修复的问题
2. 在对应文档中标记为"修复中"
3. 提交代码修复
4. 更新文档，记录修复方案
5. 运行测试验证修复

## 相关资源

- [测试文档](../tests/e2e/README.md)
- [API 规范](../api/server/http_api_specification.md)
- [发布文档](../release/)
