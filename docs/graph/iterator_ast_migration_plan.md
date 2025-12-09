# Iterator和AST Context迁移方案设计文档

## 概述

本文档详细说明了将 NebulaGraph 3.8.0 的 Iterator 和 AST Context 模块迁移到 Rust 实现的 GraphDB 中的方案设计。

## 现状分析

### NebulaGraph 3.8.0 中的 Iterator 模块

NebulaGraph 中的 Iterator 模块位于 `nebula-3.8.0/src/graph/context/iterator/` 目录下，主要包含以下组件：

1. **Iterator.h/cpp** - 基础迭代器接口定义
2. **SequentialIter.h/cpp** - 顺序迭代器实现
3. **GetNeighborsIter.h/cpp** - 邻居查询迭代器实现
4. **PropIter.h/cpp** - 属性迭代器实现
5. **GetNbrsRespDataSetIter.h/cpp** - 响应数据集迭代器实现
6. **GetNeighborsIter.h/cpp** - 邻居迭代器实现

主要功能包括：
- 统一的迭代器接口
- 不同类型的迭代器（默认、邻居、顺序、属性）
- 高效的数据访问和过滤
- 内存管理

### NebulaGraph 3.8.0 中的 AST Context 模块

NebulaGraph 中的 AST Context 模块位于 `nebula-3.8.0/src/graph/context/ast/` 目录下，主要包含以下组件：

1. **AstContext.h** - 基础AST上下文定义
2. **CypherAstContext.h** - Cypher查询的AST上下文定义
3. **QueryAstContext.h** - 查询语句的AST上下文定义

主要功能包括：
- 查询解析后的上下文信息
- Cypher子句上下文定义
- 不同查询语句的上下文定义（GO、MATCH、LOOKUP等）

### 当前 GraphDB 中的实现

当前 GraphDB 中已实现了部分功能：

1. **AST Context** (`src/core/ast_context.rs`) - 简单的AST上下文定义
2. **Result和Iterator** (`src/core/result.rs`) - 简单的结果和迭代器实现

## 迁移需求

### Iterator 模块迁移需求

需要实现以下迭代器类型：

1. **基础迭代器接口**
   - 支持各种数据类型（默认、邻居、顺序、属性）
   - 支持数据访问、过滤和操作方法
   - 支持内存管理

2. **SequentialIterator**
   - 顺序数据访问
   - 支持数据选择和采样

3. **GetNeighborsIterator**
   - 邻居查询数据访问
   - 特定于图遍历场景

4. **PropIterator**
   - 属性数据访问
   - 支持标签和边属性

### AST Context 模块迁移需求

需要实现以下AST上下文类型：

1. **基础AstContext**
   - 查询上下文
   - 空间信息

2. **CypherContext**
   - Cypher查询的完整上下文
   - 包括MATCH、WHERE、WITH、RETURN等子句

3. **查询特定上下文**
   - GO语句上下文
   - LOOKUP语句上下文
   - PATH语句上下文
   - 子图查询上下文等

## 迁移方案设计

### 1. Iterator 模块设计

#### 1.1 基础迭代器接口 (`src/query/iterator/mod.rs`)

```rust
/// 迭代器类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum IteratorKind {
    Default,
    GetNeighbors,
    Sequential,
    Prop,
}

/// 迭代器接口
pub trait BaseIterator: Send + Sync {
    /// 获取值引用
    fn value_ptr(&self) -> Arc<Value>;
    
    /// 检查是否有效
    fn is_valid(&self) -> bool;
    
    /// 移动到下一个元素
    fn next(&mut self);
    
    /// 擦除当前元素
    fn erase(&mut self);
    
    /// 获取当前行
    fn row(&self) -> Option<&Row>;
    
    /// 移动当前行
    fn move_row(&mut self) -> Option<Row>;
    
    /// 获取大小
    fn size(&self) -> usize;
    
    /// 是否为空
    fn is_empty(&self) -> bool { self.size() == 0 }
    
    /// 重置迭代器位置
    fn reset(&mut self);
    
    /// 清空迭代器
    fn clear(&mut self);
    
    /// 获取列值
    fn get_column(&self, col_name: &str) -> &Value;
    
    /// 通过索引获取列值
    fn get_column_by_index(&self, index: usize) -> &Value;
    
    /// 获取迭代器类型
    fn kind(&self) -> IteratorKind;
    
    /// 检查是否为默认迭代器
    fn is_default_iter(&self) -> bool { self.kind() == IteratorKind::Default }
    
    /// 检查是否为邻居迭代器
    fn is_get_neighbors_iter(&self) -> bool { self.kind() == IteratorKind::GetNeighbors }
    
    /// 检查是否为顺序迭代器
    fn is_sequential_iter(&self) -> bool { self.kind() == IteratorKind::Sequential }
    
    /// 检查是否为属性迭代器
    fn is_prop_iter(&self) -> bool { self.kind() == IteratorKind::Prop }
}
```

#### 1.2 SequentialIterator 实现 (`src/query/iterator/sequential.rs`)

```rust
use std::sync::Arc;
use crate::core::{Value, Row, BaseIterator, IteratorKind};

pub struct SequentialIterator {
    /// 存储行数据
    rows: Vec<Row>,
    /// 当前位置
    current_pos: usize,
    /// 值引用
    value: Arc<Value>,
    /// 列索引映射
    col_indices: HashMap<String, usize>,
}

impl SequentialIterator {
    pub fn new(rows: Vec<Row>, value: Arc<Value>) -> Self {
        let mut col_indices = HashMap::new();
        
        // 如果值是数据集，则构建列索引映射
        if let Value::DataSet(ref dataset) = *value {
            for (index, col_name) in dataset.col_names.iter().enumerate() {
                col_indices.insert(col_name.clone(), index);
            }
        }
        
        Self {
            rows,
            current_pos: 0,
            value,
            col_indices,
        }
    }
}

impl BaseIterator for SequentialIterator {
    // 实现所有必要的方法...
}
```

#### 1.3 GetNeighborsIterator 实现 (`src/query/iterator/get_neighbors.rs`)

```rust
use std::sync::Arc;
use crate::core::{Value, Row, BaseIterator, IteratorKind};

pub struct GetNeighborsIterator {
    /// 数据集索引
    ds_indices: Vec<DataSetIndex>,
    /// 当前数据集
    current_ds: usize,
    /// 当前行
    current_row: usize,
    /// 当前列
    col_idx: i64,
    /// 当前列数据
    current_col: Option<Arc<List>>,
    /// 当前边
    current_edge: Option<Row>,
    /// 有效性状态
    valid: bool,
    /// 值引用
    value: Arc<Value>,
}

impl GetNeighborsIterator {
    pub fn new(value: Arc<Value>) -> Self {
        // 解析数据集并建立索引
        // 实现邻居遍历逻辑
        unimplemented!()
    }
}

impl BaseIterator for GetNeighborsIterator {
    // 实现邻居迭代器的特定方法...
}
```

### 2. AST Context 模块设计

#### 2.1 基础 AST Context (`src/query/context/mod.rs`)

```rust
use crate::core::SpaceInfo;
use crate::query::QueryContext;

/// AST上下文基础结构
#[derive(Debug, Clone)]
pub struct AstContext {
    pub query_context: Arc<QueryContext>,
    pub space_info: SpaceInfo,
}

impl AstContext {
    pub fn new(query_context: Arc<QueryContext>, space_info: SpaceInfo) -> Self {
        Self {
            query_context,
            space_info,
        }
    }
}
```

#### 2.2 Cypher AST Context (`src/query/context/cypher.rs`)

```rust
use std::collections::HashMap;
use crate::core::{Value, Expression, YieldColumns};
use crate::query::context::{AstContext, AliasType};

/// Cypher子句类型
#[derive(Debug, Clone, PartialEq)]
pub enum CypherClauseKind {
    Match,
    Unwind,
    With,
    Where,
    Return,
    OrderBy,
    Pagination,
    Yield,
    ShortestPath,
    AllShortestPaths,
}

/// 别名类型
#[derive(Debug, Clone, PartialEq)]
pub enum AliasType {
    Node,
    Edge,
    Path,
    NodeList,
    EdgeList,
    Runtime,
}

impl AliasType {
    pub fn to_str(&self) -> &'static str {
        match self {
            AliasType::Node => "Node",
            AliasType::Edge => "Edge",
            AliasType::Path => "Path",
            AliasType::NodeList => "NodeList",
            AliasType::EdgeList => "EdgeList",
            AliasType::Runtime => "Runtime",
        }
    }
}

/// 路径定义
#[derive(Debug, Clone)]
pub struct Path {
    pub anonymous: bool,
    pub alias: String,
    pub node_infos: Vec<NodeInfo>,
    pub edge_infos: Vec<EdgeInfo>,
    // 其他相关字段...
}

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub anonymous: bool,
    pub labels: Vec<String>,
    pub alias: String,
    pub props: Option<MapExpression>,
    pub filter: Option<Expression>,
    // 其他相关字段...
}

#[derive(Debug, Clone)]
pub struct EdgeInfo {
    pub anonymous: bool,
    pub direction: Direction,
    pub types: Vec<String>,
    pub alias: String,
    pub props: Option<MapExpression>,
    pub filter: Option<Expression>,
    // 其他相关字段...
}

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    OutEdge,
    InEdge,
    Bidirectional,
}

/// Cypher子句上下文基础
#[derive(Debug, Clone)]
pub struct CypherClauseContextBase {
    pub kind: CypherClauseKind,
    pub input_col_names: Vec<String>,
    pub aliases_available: HashMap<String, AliasType>,
}

impl CypherClauseContextBase {
    pub fn new(kind: CypherClauseKind) -> Self {
        Self {
            kind,
            input_col_names: Vec::new(),
            aliases_available: HashMap::new(),
        }
    }
}

/// WHERE子句上下文
#[derive(Debug, Clone)]
pub struct WhereClauseContext {
    base: CypherClauseContextBase,
    pub paths: Vec<Path>,
    pub filter: Option<Expression>,
}

impl WhereClauseContext {
    pub fn new() -> Self {
        Self {
            base: CypherClauseContextBase::new(CypherClauseKind::Where),
            paths: Vec::new(),
            filter: None,
        }
    }
}

// ... 其他Cypher子句上下文定义
```

#### 2.3 查询特定 AST Context (`src/query/context/query.rs`)

```rust
use crate::query::context::AstContext;
use crate::core::{Expression, YieldColumns};

/// GO语句上下文
#[derive(Debug, Clone)]
pub struct GoContext {
    base: AstContext,
    pub from: Starts,
    pub steps: StepClause,
    pub over: Over,
    pub filter: Option<Expression>,
    pub yield_expr: Option<YieldColumns>,
    pub distinct: bool,
    pub random: bool,
    pub limits: Vec<i64>,
    pub col_names: Vec<String>,
    // 其他相关字段...
}

#[derive(Debug, Clone)]
pub struct Starts {
    pub from_type: FromType,
    pub src: Option<Expression>,
    pub original_src: Option<Expression>,
    pub user_defined_var_name: String,
    pub runtime_vid_name: String,
    pub vids: Vec<Value>,
}

#[derive(Debug, Clone)]
pub enum FromType {
    InstantExpr,
    Variable,
    Pipe,
}

#[derive(Debug, Clone)]
pub struct Over {
    pub is_over_all: bool,
    pub edge_types: Vec<i32>,
    pub direction: EdgeDirection,
    pub all_edges: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum EdgeDirection {
    OutEdge,
    InEdge,
    Bidirectional,
}

// 类似地定义其他查询上下文结构...
```

### 3. 集成和扩展

#### 3.1 模块组织

在 `src/query/` 目录下进行以下模块组织：

```
src/query/
├── context/
│   ├── mod.rs
│   ├── ast.rs (AST Context定义)
│   ├── cypher.rs (Cypher相关Context)
│   └── query.rs (查询特定Context)
├── iterator/
│   ├── mod.rs
│   ├── base.rs (基础迭代器接口)
│   ├── sequential.rs (顺序迭代器)
│   └── get_neighbors.rs (邻居迭代器)
└── mod.rs
```

#### 3.2 与查询执行器的集成

1. **查询解析阶段**：使用 AST Context 存储解析后的查询信息
2. **查询优化阶段**：使用 AST Context 进行优化决策
3. **查询执行阶段**：使用 Iterator 处理查询结果

## 迁移计划

### 阶段一：基础实现 (2周)
1. 实现基础迭代器接口 (`BaseIterator`)
2. 实现 `SequentialIterator`
3. 实现基础 AST Context

### 阶段二：高级迭代器 (2周)
1. 实现 `GetNeighborsIterator`
2. 实现 `PropIterator`
3. 添加迭代器测试

### 阶段三：AST Context 完善 (2周)
1. 实现 Cypher AST Context
2. 实现查询特定上下文 (GO, MATCH, LOOKUP)
3. 添加 AST Context 测试

### 阶段四：集成和优化 (1周)
1. 将新实现集成到查询引擎
2. 性能优化和基准测试
3. 文档完善

## 结论

本方案提出了一个完整的 Iterator 和 AST Context 模块迁移方案，旨在将 NebulaGraph 3.8.0 中的相关功能迁移到 Rust 实现的 GraphDB 中。通过模块化设计，逐步实现各个组件，确保功能完整性和性能。
