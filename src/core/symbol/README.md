# 符号表模块 (Symbol Table Module)

## 概述

符号表模块是图数据库查询处理系统中的核心组件，用于管理查询中的变量、别名和符号，并跟踪它们之间的依赖关系。该模块对应于原 C++ 代码中的 `context/Symbols.h`。

## 文件结构

```
src/core/symbol/
├── mod.rs          # 模块定义和导出
├── symbol_table.rs # 符号表主实现
├── dependency_tracker.rs # 依赖关系跟踪器
├── plan_node.rs    # 计划节点定义
└── README.md       # 本说明文档
```

## 文件说明

### 1. `symbol_table.rs` - 符号表主实现

**功能：**
- 管理查询中的变量、别名和符号
- 提供变量的创建、删除、重命名等基本操作
- 维护变量与计划节点之间的依赖关系
- 提供对象池功能以优化内存使用

**主要结构：**
- `SymbolTable`: 符号表主结构，包含变量存储、依赖跟踪器和对象池
- `Symbol`: 符号定义，包含名称、类型和创建时间
- `SymbolType`: 符号类型枚举（变量、别名、参数、函数）

**使用位置：**
- `src/query/context/validate/context.rs` - 验证上下文
- `src/query/context/execution/query_execution.rs` - 执行上下文
- `src/utils/anon_var_generator.rs` - 匿名变量生成器

### 2. `dependency_tracker.rs` - 依赖关系跟踪器

**功能：**
- 跟踪变量与计划节点之间的读写依赖关系
- 检测数据竞争（多个节点写入同一变量）
- 提供依赖关系的统计信息
- 管理变量的读取者和写入者列表

**主要结构：**
- `DependencyTracker`: 依赖关系跟踪器主结构
- `VariableDependencies`: 单个变量的依赖信息
- `Dependency`: 依赖关系定义
- `DependencyType`: 依赖类型枚举（读、写、读写）

**使用位置：**
- `src/core/symbol/symbol_table.rs` - 被符号表使用

### 3. `plan_node.rs` - 计划节点定义

**功能：**
- 定义查询计划节点的引用和类型
- 提供计划节点的标识和类型管理

**主要结构：**
- `PlanNodeRef`: 计划节点引用，包含 ID 和类型
- `PlanNodeType`: 计划节点类型枚举（扫描、过滤、投影等）

**使用位置：**
- `src/core/symbol/dependency_tracker.rs` - 被依赖跟踪器使用
- `src/core/symbol/symbol_table.rs` - 被符号表使用

## 模块关系

```
    +-------------------+
    |   SymbolTable     |
    |                   |
    | + symbols         |
    | + dependency_     |
    |   tracker         |
    | + obj_pool        |
    +---------+---------+
              |
              | uses
              v
    +-------------------+
    | DependencyTracker |
    |                   |
    | + dependencies    |
    +---------+---------+
              |
              | tracks
              v
    +-------------------+
    |   PlanNodeRef     |
    |                   |
    | + id, node_type   |
    +-------------------+
```

## 使用场景

### 1. 查询验证阶段
- 在查询解析后，验证变量的使用是否合法
- 确保变量在使用前已定义
- 检测变量作用域问题

### 2. 查询执行阶段
- 管理执行过程中的变量生命周期
- 跟踪变量的读写操作
- 检测并处理数据竞争

### 3. 优化器
- 提供依赖信息以进行查询优化
- 帮助识别可并行执行的操作

## 依赖关系

- 标准库依赖：`std::collections`, `std::sync`, `std::time`, `std::sync::atomic`
- 项目内部依赖：无外部依赖

## 注意事项

1. 由于使用了 `Arc` 和 `RwLock`，该模块是线程安全的，适用于并发环境
2. 依赖跟踪器提供了详细的依赖信息，有助于查询优化和错误检测
3. 对象池机制有助于减少内存分配和提升性能