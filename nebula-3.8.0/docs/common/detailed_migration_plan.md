# NebulaGraph Common 模块到 Rust 架构的详细迁移方案

## 概述

本文档详细说明了如何将 NebulaGraph 3.8.0 的 common 模块迁移到新的 Rust 架构中，包括每个子模块的对应关系和具体迁移步骤。

## 1. Base 模块迁移

### 1.1 现状分析
- **C++ 实现**: `nebula-3.8.0/src/common/base/`
  - `Status.h/Status.cpp`: 状态和错误码管理
  - `StatusOr.h`: Result 类型的等价实现
  - `Base.h`: 基础类型和宏定义

### 1.2 Rust 对应关系
- **Status** → **Result<T, E>** 和自定义错误类型
- **StatusOr** → **Result<T, CustomError>**
- **基础类型/宏** → **Rust 原生类型和特质**

### 1.3 迁移方案
```rust
// 定义错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum GraphDBError {
    Ok,
    Inserted,
    Error(String),
    NoSuchFile,
    NotSupported,
    SyntaxError(String),
    SemanticError(String),
    GraphMemoryExceeded,
    StatementEmpty,
    KeyNotFound,
    PartialSuccess,
    StorageMemoryExceeded,
    SpaceNotFound,
    HostNotFound,
    TagNotFound,
    EdgeNotFound,
    UserNotFound,
    IndexNotFound,
    GroupNotFound,
    ZoneNotFound,
    LeaderChanged,
    Balanced,
    PartNotFound,
    ListenerNotFound,
    SessionNotFound,
    PermissionError,
}

// 实现显示trait
impl std::fmt::Display for GraphDBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphDBError::Ok => write!(f, "OK"),
            GraphDBError::Inserted => write!(f, "Inserted"),
            GraphDBError::Error(msg) => write!(f, "Error: {}", msg),
            // ... 其他错误类型
        }
    }
}

impl std::error::Error for GraphDBError {}

// 通用返回类型
pub type GraphDBResult<T> = Result<T, GraphDBError>;
```

## 2. Datatypes 模块迁移

### 2.1 现状分析
- **C++ 实现**: `nebula-3.8.0/src/common/datatypes/`
  - `Value.h/Value.cpp`: 核心值类型，支持多种数据类型
  - `Vertex.h/Edge.h/Path.h`: 图数据结构
  - `Date.h/Time.h/DateTime.h`: 时间类型
  - `List.h/Map.h/Set.h`: 容器类型

### 2.2 Rust 对应关系
- **Value** → **Rust 枚举类型**
- **图数据结构** → **Rust 结构体**
- **容器类型** → **Rust 标准库容器类型**

### 2.3 迁移方案
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Empty,
    Null(NullType),
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Date(Date),
    Time(Time),
    DateTime(DateTime),
    Vertex(Vertex),
    Edge(Edge),
    Path(Path),
    List(Vec<Value>),
    Map(std::collections::HashMap<String, Value>),
    Set(std::collections::HashSet<Value>),
    DataSet(DataSet),
    Geography(Geography),
    Duration(Duration),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NullType {
    Null,
    NaN,
    BadData,
    BadType,
    Overflow,
    UnknownProp,
    DivByZero,
    OutOfRange,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
    pub vid: Value,
    pub tags: std::collections::HashMap<String, Tag>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub src: Value,
    pub dst: Value,
    pub rank: i64,
    pub type_name: String,
    pub props: std::collections::HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub src: Vertex,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    pub dst: Vertex,
    pub edge: Edge,
}
```

## 3. Expression 模块迁移

### 3.1 现状分析
- **C++ 实现**: `nebula-3.8.0/src/common/expression/`
  - `Expression.h`: 表达式基类
  - 各种子表达式类型（算术、逻辑、属性访问等）
  - `ExprVisitor.h`: 访问者模式

### 3.2 Rust 对应关系
- **表达式系统** → **Rust 枚举 + 特质（trait）**
- **访问者模式** → **Rust 特质方法**

### 3.3 迁移方案
```rust
use std::collections::HashMap;

// 表达式特质
trait Expression: Send + Sync {
    fn eval(&self, context: &mut ExpressionContext) -> GraphDBResult<Value>;
    fn accept<V: ExpressionVisitor>(&self, visitor: &mut V) -> GraphDBResult<()>;
    fn to_string(&self) -> String;
}

// 表达式枚举
#[derive(Debug, Clone)]
pub enum ExpressionEnum {
    Constant(ConstantExpression),
    Add(AddExpression),
    Subtract(SubtractExpression),
    Multiply(MultiplyExpression),
    Division(DivisionExpression),
    Modulo(ModuloExpression),
    LogicalAnd(LogicalAndExpression),
    LogicalOr(LogicalOrExpression),
    Equal(EqualExpression),
    NotEqual(NotEqualExpression),
    LessThan(LessThanExpression),
    LessEqual(LessEqualExpression),
    GreaterThan(GreaterThanExpression),
    GreaterEqual(GreaterEqualExpression),
    FunctionCall(FunctionCallExpression),
    // ... 其他表达式类型
}

impl Expression for ExpressionEnum {
    fn eval(&self, context: &mut ExpressionContext) -> GraphDBResult<Value> {
        match self {
            ExpressionEnum::Constant(e) => e.eval(context),
            ExpressionEnum::Add(e) => e.eval(context),
            ExpressionEnum::Subtract(e) => e.eval(context),
            // ... 其他表达式类型
        }
    }

    fn accept<V: ExpressionVisitor>(&self, visitor: &mut V) -> GraphDBResult<()> {
        match self {
            ExpressionEnum::Constant(e) => e.accept(visitor),
            // ... 其他表达式类型
        }
    }

    fn to_string(&self) -> String {
        match self {
            ExpressionEnum::Constant(e) => e.to_string(),
            // ... 其他表达式类型
        }
    }
}

// 访问者特质
trait ExpressionVisitor {
    fn visit_constant(&mut self, expr: &ConstantExpression) -> GraphDBResult<()>;
    fn visit_add(&mut self, expr: &AddExpression) -> GraphDBResult<()>;
    // ... 其他表达式类型
}

// 具体表达式实现示例
#[derive(Debug, Clone)]
pub struct ConstantExpression {
    value: Value,
}

impl Expression for ConstantExpression {
    fn eval(&self, _context: &mut ExpressionContext) -> GraphDBResult<Value> {
        Ok(self.value.clone())
    }

    fn accept<V: ExpressionVisitor>(&self, visitor: &mut V) -> GraphDBResult<()> {
        visitor.visit_constant(self)
    }

    fn to_string(&self) -> String {
        format!("{:?}", self.value)
    }
}

#[derive(Debug, Clone)]
pub struct AddExpression {
    left: Box<ExpressionEnum>,
    right: Box<ExpressionEnum>,
}

impl Expression for AddExpression {
    fn eval(&self, context: &mut ExpressionContext) -> GraphDBResult<Value> {
        let left_val = self.left.eval(context)?;
        let right_val = self.right.eval(context)?;
        
        match (left_val, right_val) {
            (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l + r)),
            (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l + r)),
            (Value::String(l), Value::String(r)) => Ok(Value::String(format!("{}{}", l, r))),
            _ => Err(GraphDBError::Error("Type mismatch in add operation".to_string())),
        }
    }

    fn accept<V: ExpressionVisitor>(&self, visitor: &mut V) -> GraphDBResult<()> {
        visitor.visit_add(self)
    }

    fn to_string(&self) -> String {
        format!("({} + {})", self.left.to_string(), self.right.to_string())
    }
}
```

## 4. Utils 模块迁移

### 4.1 现状分析
- **C++ 实现**: `nebula-3.8.0/src/common/utils/`
  - `NebulaKeyUtils.h/cpp`: 存储键值的构建工具
  - `IndexKeyUtils.h/cpp`: 索引键构建工具
  - `Types.h`: 类型定义

### 4.2 Rust 对应关系
- **工具函数** → **Rust 模块和函数**
- **类型定义** → **Rust 类型别名**

### 4.3 迁移方案
```rust
use std::collections::HashMap;

// 类型别名
pub type PartitionID = u32;
pub type TagID = i32;
pub type EdgeType = i32;
pub type EdgeRanking = i64;
pub type EdgeVerPlaceHolder = u8;
pub type VertexID = String;
pub type GraphSpaceID = i32;
pub type JobID = i32;
pub type TaskID = i32;

// 键值构建工具
pub struct NebulaKeyUtils;

impl NebulaKeyUtils {
    pub fn tag_key(v_id_len: usize, part_id: PartitionID, v_id: &str, tag_id: TagID) -> String {
        let mut key = Vec::new();
        key.push(0x01);  // type tag
        key.extend_from_slice(&part_id.to_be_bytes()[1..4]);  // 3-byte partition ID
        key.extend_from_slice(v_id.as_bytes());
        key.extend_from_slice(&tag_id.to_be_bytes());
        String::from_utf8(key).unwrap()
    }

    pub fn edge_key(
        v_id_len: usize,
        part_id: PartitionID,
        src_id: &str,
        edge_type: EdgeType,
        rank: EdgeRanking,
        dst_id: &str,
        ev: EdgeVerPlaceHolder,
    ) -> String {
        let mut key = Vec::new();
        key.push(0x02);  // type tag for edge
        key.extend_from_slice(&part_id.to_be_bytes()[1..4]);  // 3-byte partition ID
        key.extend_from_slice(src_id.as_bytes());
        key.extend_from_slice(&edge_type.to_be_bytes());
        key.extend_from_slice(&rank.to_be_bytes());
        key.extend_from_slice(dst_id.as_bytes());
        key.push(ev);
        String::from_utf8(key).unwrap()
    }

    pub fn vertex_prefix(part_id: PartitionID) -> String {
        String::from_utf8(vec![0x03])  // type tag for vertex
            .unwrap()
            .chars()
            .take(1)
            .collect::<String>() + &String::from_utf8(part_id.to_be_bytes()[1..4].to_vec()).unwrap()
    }

    pub fn is_edge(v_id_len: usize, raw_key: &str) -> bool {
        if raw_key.len() != 1 + 3 + v_id_len * 2 + 4 + 8 + 1 {  // type(1) + partId(3) + srcId(vIdLen) + edgeType(4) + rank(8) + dstId(vIdLen) + placeholder(1)
            return false;
        }
        
        let bytes = raw_key.as_bytes();
        bytes[0] == 0x02  // edge type tag
    }

    // ... 其他工具方法
}
```

## 5. Graph 模块迁移

### 5.1 现状分析
- **C++ 实现**: `nebula-3.8.0/src/common/graph/`
  - `Response.h`: 查询响应结构定义，包括错误码、执行结果等

### 5.2 Rust 对应关系
- **响应结构** → **Rust 结构体和枚举**
- **错误处理** → **Rust Result 类型**

### 5.3 迁移方案
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCode {
    Succeeded = 0,
    Disconnected = -1,
    FailToConnect = -2,
    RpcFailure = -3,
    LeaderChanged = -4,
    SpaceNotFound = -5,
    TagNotFound = -6,
    EdgeNotFound = -7,
    IndexNotFound = -8,
    // ... 其他错误码
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuthResponse {
    pub error_code: ErrorCode,
    pub session_id: Option<i64>,
    pub error_msg: Option<String>,
    pub time_zone_offset_seconds: Option<i32>,
    pub time_zone_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProfilingStats {
    pub rows: i64,
    pub exec_duration_in_us: i64,
    pub total_duration_in_us: i64,
    pub other_stats: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionResponse {
    pub error_code: ErrorCode,
    pub latency_in_us: i64,
    pub data: Option<DataSet>,
    pub space_name: Option<String>,
    pub error_msg: Option<String>,
    pub plan_desc: Option<PlanDescription>,
    pub comment: Option<String>,
}
```

## 6. 其他模块简要说明

### 6.1 Configuration 模块
- **C++ 实现**: `nebula-3.8.0/src/common/conf/`
- **Rust 对应**: 使用 `config` crate 或 `serde` 进行配置管理

### 6.2 Context 模块
- **C++ 实现**: `nebula-3.8.0/src/common/context/`
- **Rust 对应**: 使用 `Arc<Mutex<T>>` 或 `thread_local` 存储请求上下文

### 6.3 Memory 模块
- **C++ 实现**: `nebula-3.8.0/src/common/memory/`
- **Rust 对应**: 利用 Rust 的所有权和借用系统，通常无需手动内存管理

### 6.4 Network 模块
- **C++ 实现**: `nebula-3.8.0/src/common/network/`
- **Rust 对应**: 使用 `tokio`、`async-std`、`hyper` 等异步网络库

## 7. 迁移实施建议

### 7.1 迁移顺序
1. **基础数据类型** (`datatypes`): 实现 Value、Vertex、Edge 等核心数据结构
2. **错误处理系统** (`base`): 实现 Result 和错误码系统
3. **表达式系统** (`expression`): 实现查询表达式的解析和计算
4. **工具模块** (`utils`): 实现键值构建等工具函数
5. **响应结构** (`graph`): 实现查询结果的数据结构
6. **其他辅助模块**: 配置、上下文、网络等

### 7.2 注意事项
1. **内存安全**: 利用 Rust 的所有权系统替代 C++ 的手动内存管理
2. **线程安全**: 使用 Rust 的并发原语替代 C++ 的锁机制
3. **性能优化**: 使用 Rust 的零成本抽象保持性能优势
4. **兼容性**: 保证新架构与原有数据格式的兼容性

## 8. 测试策略

### 8.1 单元测试
- 针对每个数据类型和方法编写单元测试
- 确保边界条件和错误处理正确

### 8.2 集成测试
- 迁移完核心模块后，进行集成测试
- 对比新旧系统的功能和性能

### 8.3 兼容性测试
- 确保新系统能正确处理现有的数据格式
- 验证查询结果的一致性