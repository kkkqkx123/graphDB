# 符号表模块 (Symbol Table Module)

## 概述

符号表模块是图数据库查询处理系统中的核心组件，用于管理查询中的变量、别名和符号，并跟踪它们之间的依赖关系。

## 文件结构

```
src/core/symbol/
├── mod.rs          # 模块定义和导出
├── symbol_table.rs # 符号表主实现
└── README.md       # 本说明文档
```

## 文件说明

### `symbol_table.rs` - 符号表主实现

**功能：**
- 管理查询中的变量、别名和符号
- 提供变量的创建、删除、重命名等基本操作
- 维护变量与计划节点之间的依赖关系
- 跟踪变量的使用频率

**主要结构：**
- `SymbolTable`: 符号表主结构，包含变量存储
- `Symbol`: 符号定义，包含名称、类型、依赖关系和使用计数
- `SymbolType`: 符号类型枚举（变量、别名、参数、函数、数据集、顶点、边、路径）

**使用位置：**
- `src/query/context/validate/context.rs` - 验证上下文
- `src/query/context/execution/query_execution.rs` - 执行上下文
- `src/utils/anon_var_generator.rs` - 匿名变量生成器

## 模块关系

```
    +-------------------+
    |   SymbolTable     |
    |                   |
    | + symbols         |
    +---------+---------+
              |
              | manages
              v
    +-------------------+
    |      Symbol       |
    |                   |
    | + name            |
    | + symbol_type     |
    | + col_names       |
    | + readers         |
    | + writers         |
    | + user_count      |
    +---------+---------+
              |
              | references
              v
    +-------------------+
    |   PlanNodeRef     |
    |                   |
    | + id, node_type   |
    +-------------------+
```

## Symbol 结构字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | String | 符号名称 |
| `symbol_type` | SymbolType | 符号类型 |
| `col_names` | Vec<String> | 列名列表（Dataset 类型使用） |
| `readers` | HashSet<PlanNodeRef> | 读取该变量的计划节点集合 |
| `writers` | HashSet<PlanNodeRef> | 写入该变量的计划节点集合 |
| `user_count` | Arc<AtomicU64> | 变量使用计数 |
| `created_at` | SystemTime | 创建时间 |

## SymbolType 枚举

| 变体 | 说明 |
|------|------|
| `Variable` | 普通变量 |
| `Alias` | 别名 |
| `Parameter` | 参数 |
| `Function` | 函数 |
| `Dataset` | 数据集类型 |
| `Vertex` | 顶点类型 |
| `Edge` | 边类型 |
| `Path` | 路径类型 |

## 使用场景

### 1. 查询验证阶段
- 在查询解析后，验证变量的使用是否合法
- 确保变量在使用前已定义
- 检测变量作用域问题

### 2. 查询执行阶段
- 管理执行过程中的变量生命周期
- 跟踪变量的读写操作
- 检测并处理数据竞争（通过 detect_write_conflicts）

### 3. 优化器
- 提供依赖信息以进行查询优化
- 帮助识别可并行执行的操作

## 依赖关系

- 标准库依赖：`std::collections`, `std::sync`, `std::sync::atomic`, `std::time`
- 项目内部依赖：`crate::core::PlanNodeRef`

## 注意事项

1. 由于使用了 `Arc` 和 `RwLock`，该模块是线程安全的，适用于并发环境
2. 依赖关系直接嵌入到 Symbol 结构中，简化了 API 调用路径
3. user_count 可用于优化：清理未使用的变量、决定变量保留策略
4. detect_write_conflicts 可用于检测数据竞争
