# Validator 目录结构优化方案

## 概述

本文档描述了 `src/query/validator` 目录的结构优化方案，旨在提高代码组织性、可维护性和可读性。

## 当前问题

1. **文件数量过多**：validator 目录包含 30+ 个文件，难以快速定位
2. **职责不清晰**：不同类型的验证器混杂在一起
3. **依赖关系复杂**：辅助工具、策略类与核心验证器混在一起

## 优化后的目录结构

```
src/query/validator/
├── statements/              # 语句级验证器（需要表达式解析）
│   ├── match_validator.rs
│   ├── create_validator.rs
│   ├── insert_vertices_validator.rs
│   ├── insert_edges_validator.rs
│   ├── update_validator.rs
│   ├── delete_validator.rs
│   ├── merge_validator.rs
│   ├── remove_validator.rs
│   ├── set_validator.rs
│   ├── unwind_validator.rs
│   ├── lookup_validator.rs
│   ├── fetch_vertices_validator.rs
│   ├── fetch_edges_validator.rs
│   ├── go_validator.rs
│   ├── find_path_validator.rs
│   └── get_subgraph_validator.rs
│
├── clauses/                 # 子句级验证器（需要表达式解析）
│   ├── group_by_validator.rs
│   ├── order_by_validator.rs
│   ├── limit_validator.rs
│   ├── yield_validator.rs
│   ├── return_validator.rs
│   ├── with_validator.rs
│   └── sequential_validator.rs
│
├── ddl/                     # DDL 语句验证器（不需要表达式解析）
│   ├── drop_validator.rs
│   ├── alter_validator.rs
│   └── admin_validator.rs
│
├── dml/                     # DML 语句验证器（不需要表达式解析）
│   ├── use_validator.rs
│   ├── pipe_validator.rs
│   ├── query_validator.rs
│   └── set_operation_validator.rs
│
├── utility/                 # 工具验证器
│   ├── explain_validator.rs
│   ├── acl_validator.rs
│   └── update_config_validator.rs
│
├── helpers/                 # 辅助工具
│   ├── schema_validator.rs
│   ├── variable_validator.rs
│   ├── alias_strategy.rs
│   ├── aggregate_strategy.rs
│   ├── expression_strategy.rs
│   ├── type_inference.rs
│   ├── type_deduce.rs
│   ├── clause_strategy.rs
│   ├── pagination_strategy.rs
│   └── expression_operations.rs
│
├── structs/                 # 数据结构定义
│   ├── clause_structs.rs
│   ├── path_structs.rs
│   └── validation_info.rs
│
├── strategies/              # 验证策略
│   ├── aggregate_strategy.rs
│   ├── expression_strategy.rs
│   ├── type_inference.rs
│   ├── type_deduce.rs
│   ├── clause_strategy.rs
│   ├── pagination_strategy.rs
│   ├── expression_operations.rs
│   ├── variable_validator.rs
│   └── alias_strategy.rs
│
├── validator_trait.rs       # 验证器 trait 定义
├── validator_enum.rs        # 验证器枚举
└── mod.rs                  # 模块导出
```

## 目录分类说明

### 1. statements/ - 语句级验证器

**职责**：验证完整的查询语句，需要复杂的表达式解析和验证

**包含文件**：
- `match_validator.rs` - MATCH 语句验证
- `create_validator.rs` - CREATE 语句验证
- `insert_vertices_validator.rs` - INSERT VERTICES 语句验证
- `insert_edges_validator.rs` - INSERT EDGES 语句验证
- `update_validator.rs` - UPDATE 语句验证
- `delete_validator.rs` - DELETE 语句验证
- `merge_validator.rs` - MERGE 语句验证
- `remove_validator.rs` - REMOVE 语句验证
- `set_validator.rs` - SET 语句验证
- `unwind_validator.rs` - UNWIND 语句验证
- `lookup_validator.rs` - LOOKUP 语句验证
- `fetch_vertices_validator.rs` - FETCH VERTICES 语句验证
- `fetch_edges_validator.rs` - FETCH EDGES 语句验证
- `go_validator.rs` - GO 语句验证
- `find_path_validator.rs` - FIND PATH 语句验证
- `get_subgraph_validator.rs` - GET SUBGRAPH 语句验证

**特点**：
- 实现完整的 `StatementValidator` trait
- 需要处理 `ContextualExpression`
- 包含复杂的验证逻辑
- 与 SchemaManager 交互

### 2. clauses/ - 子句级验证器

**职责**：验证查询子句（GROUP BY、ORDER BY 等）

**包含文件**：
- `group_by_validator.rs` - GROUP BY 子句验证
- `order_by_validator.rs` - ORDER BY 子句验证
- `limit_validator.rs` - LIMIT/SKIP 子句验证
- `yield_validator.rs` - YIELD 子句验证
- `return_validator.rs` - RETURN 子句验证
- `with_validator.rs` - WITH 子句验证
- `sequential_validator.rs` - 顺序语句验证

**特点**：
- 处理特定的子句语法
- 需要表达式类型推导
- 与主验证器协作

### 3. ddl/ - DDL 语句验证器

**职责**：验证数据定义语言语句

**包含文件**：
- `drop_validator.rs` - DROP 语句验证
- `alter_validator.rs` - ALTER 语句验证
- `admin_validator.rs` - 管理语句验证（SHOW、DESC 等）

**特点**：
- 主要处理 Schema 操作
- 不需要复杂的表达式解析
- 与 SchemaManager 紧密交互

### 4. dml/ - DML 语句验证器

**职责**：验证简单的数据操作语言语句

**包含文件**：
- `use_validator.rs` - USE 语句验证
- `pipe_validator.rs` - PIPE 语句验证
- `query_validator.rs` - QUERY 语句验证
- `set_operation_validator.rs` - 集合操作验证

**特点**：
- 相对简单的验证逻辑
- 不需要复杂表达式解析
- 主要处理语句级语法

### 5. utility/ - 工具验证器

**职责**：验证工具类语句

**包含文件**：
- `explain_validator.rs` - EXPLAIN 语句验证
- `acl_validator.rs` - ACL 相关验证
- `update_config_validator.rs` - 配置更新验证

**特点**：
- 不直接操作数据
- 提供调试和管理功能
- 验证逻辑相对独立

### 6. helpers/ - 辅助工具

**职责**：提供验证辅助功能

**包含文件**：
- `schema_validator.rs` - Schema 验证工具
- `variable_validator.rs` - 变量验证工具
- `alias_strategy.rs` - 别名验证策略
- `aggregate_strategy.rs` - 聚合函数验证策略
- `expression_strategy.rs` - 表达式验证策略
- `type_inference.rs` - 类型推导工具
- `type_deduce.rs` - 类型推断工具
- `clause_strategy.rs` - 子句验证策略
- `pagination_strategy.rs` - 分页验证策略
- `expression_operations.rs` - 表达式操作工具

**特点**：
- 不实现 `StatementValidator` trait
- 被其他验证器调用
- 提供可复用的验证逻辑

### 7. structs/ - 数据结构定义

**职责**：定义验证过程中使用的数据结构

**包含文件**：
- `clause_structs.rs` - 子句相关结构
- `path_structs.rs` - 路径相关结构
- `validation_info.rs` - 验证信息结构

**特点**：
- 纯数据结构定义
- 不包含验证逻辑
- 跨多个验证器共享

### 8. strategies/ - 验证策略

**职责**：定义验证策略模式

**包含文件**：
- `aggregate_strategy.rs` - 聚合策略
- `expression_strategy.rs` - 表达式策略
- `type_inference.rs` - 类型推导策略
- `type_deduce.rs` - 类型推断策略
- `clause_strategy.rs` - 子句策略
- `pagination_strategy.rs` - 分页策略
- `expression_operations.rs` - 表达式操作
- `variable_validator.rs` - 变量验证
- `alias_strategy.rs` - 别名策略

**特点**：
- 定义策略接口
- 可插拔的设计
- 支持扩展

## 迁移计划

### 阶段 1：创建新目录结构
1. 创建 `statements/` 目录
2. 创建 `clauses/` 目录
3. 创建 `ddl/` 目录
4. 创建 `dml/` 目录
5. 创建 `utility/` 目录
6. 创建 `helpers/` 目录
7. 创建 `structs/` 目录
8. 创建 `strategies/` 目录

### 阶段 2：移动文件
1. 移动语句级验证器到 `statements/`
2. 移动子句级验证器到 `clauses/`
3. 移动 DDL 验证器到 `ddl/`
4. 移动 DML 验证器到 `dml/`
5. 移动工具验证器到 `utility/`
6. 移动辅助工具到 `helpers/`
7. 移动数据结构到 `structs/`
8. 移动策略文件到 `strategies/`

### 阶段 3：更新模块导出
1. 更新 `mod.rs` 以导出新的子模块
2. 更新各子目录的 `mod.rs`
3. 更新所有 `use` 语句

### 阶段 4：测试验证
1. 运行单元测试
2. 运行集成测试
3. 检查编译错误
4. 修复依赖问题

## 优势

1. **清晰的职责划分**：每个目录有明确的职责
2. **易于导航**：快速找到需要的验证器
3. **更好的可维护性**：相关代码集中在一起
4. **支持扩展**：新功能易于添加到合适的目录
5. **降低耦合**：减少不必要的依赖关系

## 注意事项

1. **保持向后兼容**：确保重构不影响现有 API
2. **更新文档**：同步更新相关文档
3. **渐进式迁移**：可以分阶段进行，降低风险
4. **充分测试**：每个阶段完成后进行测试

## 后续工作

- [ ] 执行阶段 1：创建新目录结构
- [ ] 执行阶段 2：移动文件
- [ ] 执行阶段 3：更新模块导出
- [ ] 执行阶段 4：测试验证
- [ ] 更新相关文档
- [ ] 更新构建脚本（如有必要）
