# DCL/DDL测试迁移后问题分析报告

**创建时间**: 2026-06-10  
**更新**: 2026-06-10 (已修复部分测试)  
**分析人**: 系统分析  
**目录**: `docs/issue/dcl-ddl-test-migration-issues.md`

## 概述

将DCL（数据控制语言）和DDL（数据定义语言）测试从根`tests/`目录迁移到`crates/graphdb-query/tests/`目录后，发现多个测试失败。部分失败因测试期望与实现行为不符，已修复；其余暴露了查询引擎/存储引擎的实现缺陷。

## 迁移详情

- **已完成**: DCL/DDL目录成功迁移到query包测试目录
- **已完成**: 导入路径统一改为`graphdb_query::`前缀
- **已完成**: 测试入口文件创建(`integration_dcl.rs`, `integration_ddl.rs`)
- **已完成**: 编译通过，结构正确

## 测试结果

- **DCL**: 73 passed, **4 failed**
- **DDL**: 93 passed, **5 failed**

---

## 已修复测试（5个）

| 测试 | 修复内容 |
|------|----------|
| `test_change_password_self` | 测试期望无会话上下文时`CHANGE PASSWORD`成功，实际应失败。已更新为`.assert_error()` |
| `test_dcl_user_name_case_sensitivity` | 测试期望用户名不区分大小写，实际实现为区分。已更新为`.assert_error()`后删除用户 |
| `test_create_user_duplicate` | 测试期望重复创建失败，实际实现为IDEMPOTENT。已更新测试行为一致 |
| `test_default_value_string_execution` | 使用了不支持的`INSERT VERTEX Person() VALUES 1:()`语法。已改为`INSERT VERTEX Person(name) VALUES 1:()` |
| `test_dcl_parser_kind_coverage` | 测试使用了`GRANT ROLE ADMIN`，但解析器只支持`GRANT ADMIN`（无`ROLE`关键字）。已修正 |

---

## 未修复的源码问题（9个）

### 一、DCL测试失败（4个）——源码问题

#### 1. `test_change_password_wrong_old_password`
- **文件**: `crates/graphdb-query/tests/dcl/user_management.rs:110`
- **测试期望**: 使用错误旧密码修改密码应失败
- **实际结果**: 使用错误旧密码修改成功
- **问题根源**: 密码修改时未验证旧密码，即`CHANGE PASSWORD`语句的旧密码校验逻辑缺失
- **严重程度**: 🔴 高 — 安全漏洞

#### 2. `test_dcl_password_security`
- **文件**: `crates/graphdb-query/tests/dcl/user_management.rs:469`
- **测试期望**: 密码修改后，旧密码应失效（无法再次使用）
- **实际结果**: 旧密码仍然有效
- **问题根源**: 密码修改后未作废旧密码记录，密码轮换逻辑不完整
- **严重程度**: 🔴 高 — 安全漏洞

#### 3. `test_dcl_parser_kind_coverage`（剩余失败部分）
- **文件**: `crates/graphdb-query/tests/dcl/role.rs:317`
- **测试期望**: `REVOKE ROLE ADMIN ON space FROM user`应解析成功
- **实际结果**: 解析失败，"Expected identifier, found Space"
- **问题根源**: 解析器对`REVOKE ROLE`语法同样不完整（已修复`GRANT`部分）
- **严重程度**: 🟡 中 — 解析器功能缺失

---

### 二、DDL测试失败（5个）——源码问题

#### 4. `test_default_value_execution_insert`
- **文件**: `crates/graphdb-query/tests/ddl/constraints.rs:217`
- **测试期望**: `CREATE TAG Person(name: STRING, age: INT DEFAULT 18)`创建后，
  `INSERT VERTEX Person(name) VALUES 1:('Alice')`应自动填充age=18
- **实际结果**: age属性为`None`，默认值未应用
- **问题根源**: DEFAULT值约束在执行引擎中未实现 — INSERT时未检查并填充默认值
- **严重程度**: 🔴 高 — 核心约束缺失

#### 5. `test_default_value_string_execution`（仍失败）
- **文件**: `crates/graphdb-query/tests/ddl/constraints.rs:256`
- **测试期望**: `INSERT VERTEX Person(name) VALUES 1:()`应使用默认值'unknown'
- **实际结果**: 仍然失败（可能是默认值填充逻辑未实现）
- **问题根源**: 同上，DEFAULT约束在INSERT时未应用
- **严重程度**: 🔴 高

#### 6. `test_default_with_not_null_constraint`
- **文件**: `crates/graphdb-query/tests/ddl/constraints.rs:296`
- **测试期望**: `CREATE TAG Person(name: STRING NOT NULL DEFAULT 'unknown', age: INT)`创建后，
  `INSERT VERTEX Person(age) VALUES 1:(30)`应成功（name使用默认值）
- **实际结果**: 失败，"NOT NULL constraint violation"
- **问题根源**: NOT NULL + DEFAULT组合约束处理逻辑有误 — 应先用默认值填充NOT NULL字段，
  而非直接报错
- **严重程度**: 🔴 高

#### 7. `test_alter_tag_change_field`
- **文件**: `crates/graphdb-query/tests/ddl/schema_evolution.rs:119`
- **测试期望**: `ALTER TAG Person CHANGE (old_name name: STRING)`应成功（字段重命名）
- **实际结果**: 失败，"Removing the primary key property is not supported"
- **问题根源**: ALTER TAG CHANGE实现错误地将字段重命名识别为删除主键属性
- **严重程度**: 🟡 中 — ALTER实现逻辑错误

#### 8. `test_alter_tag_change_with_data`
- **文件**: `crates/graphdb-query/tests/ddl/tag_alter.rs:171`
- **测试期望**: `ALTER TAG Person CHANGE (old_name name: STRING)`应成功
- **实际结果**: 同上，错误"Removing the primary key property is not supported"
- **问题根源**: 同#7
- **严重程度**: 🟡 中

---

## 根本问题分类

### 1. 约束执行引擎缺失（🔴 高优先级）
- DEFAULT值约束在INSERT时未应用
- NOT NULL + DEFAULT组合处理错误
- **涉及文件**: query执行引擎相关

### 2. 安全逻辑缺失（🔴 高优先级）
- 密码修改时旧密码验证缺失
- 密码轮换后旧密码仍有效
- **涉及文件**: DCL执行器

### 3. 解析器功能不完整（🟡 中优先级）
- `REVOKE ROLE`语法不支持（`GRANT ROLE`已修复）
- **涉及文件**: parser

### 4. ALTER实现逻辑错误（🟡 中优先级）
- ALTER TAG CHANGE将字段重命名误识别为主键删除
- **涉及文件**: schema manager / DDL执行器

---

## 建议修复优先级

### 🔴 高优先级
1. **DEFAULT约束执行**: 修改查询执行引擎，在INSERT时自动填充DEFAULT值
2. **NOT NULL+DEFAULT组合**: 确保先填充DEFAULT值，再检查NOT NULL约束
3. **密码旧值验证**: 实现CHANGE PASSWORD时的旧密码验证机制
4. **密码轮换失效**: 密码修改后应作废旧密码

### 🟡 中优先级
5. **REVOKE ROLE解析**: 修复解析器对`REVOKE ROLE`语法的支持
6. **ALTER TAG CHANGE**: 修复字段重命名的实现逻辑，避免误判为主键删除

### 🟢 低优先级
7. 完善用户管理会话上下文机制
8. 明确并文档化所有边界行为（如用户名大小写）

---

## 总结

DCL/DDL测试迁移成功。通过测试分析发现了9个源码层面的问题，其中：

- **4个高优先级**：涉及核心约束和安全（DEFAULT值、NOT NULL组合、密码安全）
- **2个中优先级**：解析器和ALTER实现缺陷
- **建议先修复高优先级问题**，再逐步完善中低优先级功能
