# E2E 测试遗留问题汇总

## 问题概述

在 E2E 测试验证过程中，发现以下遗留问题需要后续修复。这些问题主要涉及 GQL 查询语法与服务器解析器的兼容性。

## 问题列表

### 1. CREATE SPACE 语法解析错误

**问题描述：**
- 测试用例：`test_002_create_and_use_space`
- 错误信息：`Parse error at line 1, column 48: Expected identifier, found String`
- 失败查询：`CREATE SPACE social_network (vid_type=STRING)`

**预期行为：**
应该支持 `CREATE SPACE <name> (vid_type=<type>)` 语法

**实际行为：**
解析器无法识别括号内的 `vid_type` 参数

**建议修复：**
检查 `src/query/parser` 中的 CREATE SPACE 语法定义，确保支持属性参数列表

---

### 2. USE SPACE 后上下文未保持

**问题描述：**
- 测试用例：`test_003_create_tags_and_edges`
- 错误信息：`Semantic error: No graph space selected, please execute USE <space> first`
- 失败查询：`CREATE TAG person(...)`

**预期行为：**
执行 `USE <space>` 后，后续查询应该在指定 space 的上下文中执行

**实际行为：**
每个查询都是独立的，没有保持 session 级别的 space 上下文

**建议修复：**
在 `src/api/server/session` 中维护当前 space 上下文，并在查询执行时自动应用

---

### 3. CREATE TAG 语法差异

**问题描述：**
- 测试用例：`test_003_create_tags_and_edges`
- 错误信息：语法解析错误或语义错误
- 测试使用语法：
  ```sql
  CREATE TAG person(
      name: STRING NOT NULL,
      age: INT,
      email: STRING,
      city: STRING
  )
  ```

**预期行为：**
支持使用冒号 `:` 定义属性类型

**实际行为：**
服务器可能期望不同的语法（如使用空格而不是冒号）

**建议修复：**
确认服务器支持的 CREATE TAG 语法，更新测试用例或解析器

---

### 4. SHOW TAGS / SHOW EDGES 返回空结果

**问题描述：**
- 测试用例：`test_004_show_tags`, `test_005_show_edges`
- 错误信息：返回空结果集，预期包含已创建的 tag/edge

**预期行为：**
`SHOW TAGS` 应该返回当前 space 中所有已创建的 tag

**实际行为：**
返回空列表 `{'columns': [], 'rows': [], 'row_count': 0}`

**可能原因：**
1. 前面的 CREATE TAG 失败，没有真正创建 tag
2. SHOW TAGS 实现未正确查询元数据

**建议修复：**
检查 `src/query/executor/admin/schema` 中的 SHOW 语句实现

---

### 5. INSERT VERTEX 语法问题

**问题描述：**
- 测试用例：`test_006_insert_vertex`
- 错误信息：`Semantic error: No image space selected, please execute first USE <space>`

**预期行为：**
支持 `INSERT VERTEX <tag>(props) VALUES <vid>:(values)` 语法

**实际行为：**
与问题 #2 相同，缺少 space 上下文

**建议修复：**
同问题 #2

---

### 6. MATCH 查询不支持

**问题描述：**
- 测试用例：`test_011_match_basic`, `test_012_match_with_filter`, `test_013_match_path`
- 错误信息：查询执行失败

**预期行为：**
支持 `MATCH (v:person) RETURN v` 等图遍历查询

**实际行为：**
MATCH 查询可能未完全实现或返回错误

**建议修复：**
检查 `src/query/executor/dql` 中的 MATCH 查询实现

---

### 7. GO 遍历查询不支持

**问题描述：**
- 测试用例：`test_014_go_traversal`, `test_015_go_multiple_steps`
- 错误信息：查询执行失败

**预期行为：**
支持 `GO FROM <vid> OVER <edge>` 语法

**实际行为：**
GO 查询可能未完全实现

**建议修复：**
检查 `src/query/executor/dql` 中的 GO 查询实现

---

### 8. LOOKUP 索引查询不支持

**问题描述：**
- 测试用例：`test_016_lookup_index`
- 错误信息：查询执行失败

**预期行为：**
支持 `LOOKUP ON <tag> WHERE <condition>` 语法

**实际行为：**
LOOKUP 查询可能未完全实现

**建议修复：**
检查 `src/query/executor/dql` 中的 LOOKUP 查询实现

---

### 9. EXPLAIN / PROFILE 不支持

**问题描述：**
- 测试用例：`test_017_explain_basic`, `test_018_explain_with_index`, `test_019_profile_query`
- 错误信息：查询执行失败

**预期行为：**
支持 `EXPLAIN <query>` 和 `PROFILE <query>` 语法

**实际行为：**
EXPLAIN/PROFILE 功能可能未完全实现

**建议修复：**
检查 `src/query/executor/explain` 和 `src/query/executor/profile` 实现

---

### 10. Transaction 支持不完整

**问题描述：**
- 测试用例：`test_020_transaction_commit`, `test_021_transaction_rollback`
- 部分通过，部分失败

**预期行为：**
支持 `BEGIN`, `COMMIT`, `ROLLBACK` 事务控制语句

**实际行为：**
事务功能部分工作，但可能与预期有差异

**建议修复：**
检查 `src/query/executor/transaction` 实现，确保事务状态正确管理

---

## 优先级建议

### 高优先级（阻塞基本功能）
1. 问题 #2 - USE SPACE 上下文保持
2. 问题 #1 - CREATE SPACE 语法
3. 问题 #5 - INSERT VERTEX 上下文问题

### 中优先级（影响查询功能）
4. 问题 #3 - CREATE TAG 语法
5. 问题 #4 - SHOW TAGS/EDGES
6. 问题 #6 - MATCH 查询
7. 问题 #10 - Transaction 支持

### 低优先级（增强功能）
8. 问题 #7 - GO 查询
9. 问题 #8 - LOOKUP 查询
10. 问题 #9 - EXPLAIN/PROFILE

## 相关代码位置

- 查询解析器：`src/query/parser/`
- 查询验证器：`src/query/validator/`
- 查询执行器：`src/query/executor/`
- 会话管理：`src/api/server/session/`
- Schema 管理：`src/query/executor/admin/schema/`

## 测试文件

- E2E 测试：`tests/e2e/test_social_network.py`
- 测试客户端：`tests/e2e/graphdb_client.py`
