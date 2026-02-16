# GraphDB 数据结构分析

## 概述

GraphDB 是一个基于 Rust 实现的图数据库项目，参考了 Nebula-Graph 的设计，但在实现上进行了简化和优化，以适应单节点部署场景。

## 数据类型分类

### 一、基础数据类型

#### 1. NULL 类型
```rust
pub enum NullType {
    Null,          // 标准 null 值
    NaN,           // 非数字结果
    BadData,       // 坏数据（解析失败）
    BadType,       // 类型不匹配
    ErrOverflow,   // 数值溢出
    UnknownProp,   // 未知属性
    DivByZero,     // 除零错误
    OutOfRange,    // 值超出范围
}
```

**特点：**
- 完全兼容 Nebula-Graph 的 NullType 定义
- 提供了 `is_bad()` 和 `is_computational_error()` 辅助方法
- 实现了 `Display` trait 用于字符串表示

#### 2. 标量类型

| 类型 | 说明 | 存储方式 |
|------|------|----------|
| BOOL | 布尔值 | bool |
| Int | 整数 | i64 |
| Int8 | 8位整数 | i8 |
| Int16 | 16位整数 | i16 |
| Int32 | 32位整数 | i32 |
| Int64 | 64位整数 | i64 |
| Float | 浮点数 | f64 |
| Double | 双精度浮点数 | f64 |
| String | 字符串 | String |
| FixedString(usize) | 定长字符串 | String（固定长度） |
| Blob | 二进制数据 | Vec<u8> |
| VID | 顶点ID类型 | Value |

**特点：**
- 提供了更细粒度的整数类型（Int8, Int16, Int32, Int64）
- 区分了 Float 和 Double（虽然底层都是 f64）
- 新增了 FixedString 类型用于优化存储
- 新增了 Blob 类型用于二进制数据
- 新增了 VID 类型专门用于顶点ID

#### 3. 日期时间类型

| 类型 | 说明 | 存储方式 |
|------|------|----------|
| Date | 日期（年、月、日） | DateValue (year: i32, month: u32, day: u32) |
| Time | 时间（时、分、秒、微秒） | TimeValue (hour, minute, sec, microsec) |
| DateTime | 日期时间 | DateTimeValue (year, month, day, hour, minute, sec, microsec) |
| Timestamp | 时间戳 | i64 |
| Duration | 时间段 | DurationValue (seconds: i64, microseconds: i32, months: i32) |

**特点：**
- 新增了 Timestamp 类型
- DateValue 的 year 使用 i32 而不是 i16（支持更广的年份范围）
- 所有日期时间类型都实现了 `Default` trait

### 二、图数据结构

#### 4. VERTEX（顶点）
```rust
pub struct Vertex {
    pub vid: Box<Value>,                              // 顶点ID（可以是任意Value类型）
    pub id: i64,                                      // 内部整数ID，用于索引和快速查找
    pub tags: Vec<Tag>,                                // 标签列表
    pub properties: HashMap<String, Value>,              // 顶点级属性
}

pub struct Tag {
    pub name: String,                                  // 标签名称
    pub properties: HashMap<String, Value>,             // 标签属性
}
```

**特点：**
- vid 使用 `Box<Value>` 包装，支持任意类型的顶点ID
- 新增了内部整数ID `id`，用于快速索引和查找
- 支持顶点级属性和标签级属性
- 提供了丰富的属性访问方法：
  - `get_property(tag_name, prop_name)` - 获取特定标签的属性
  - `get_property_any(prop_name)` - 从任意标签获取属性
  - `get_all_properties()` - 获取所有属性
  - `vertex_properties()` - 获取顶点级属性
- 实现了 `Ord` 和 `PartialOrd` trait，支持排序
- 提供了 `estimated_size()` 方法估算内存使用

#### 5. EDGE（边）
```rust
pub struct Edge {
    pub src: Box<Value>,                              // 源顶点ID
    pub dst: Box<Value>,                              // 目标顶点ID
    pub edge_type: String,                            // 边类型名称
    pub ranking: i64,                                 // 边的权重/排名
    pub id: i64,                                     // 内部整数ID，用于索引和快速查找
    pub props: HashMap<String, Value>,                 // 边属性
}
```

**特点：**
- src 和 dst 使用 `Box<Value>` 包装，支持任意类型的顶点ID
- 新增了内部整数ID `id`，用于快速索引和查找
- edge_type 使用 String 而不是枚举（更灵活）
- 提供了丰富的属性访问方法
- 实现了 `Ord` 和 `PartialOrd` trait，支持排序
- 提供了 `estimated_size()` 方法估算内存使用

#### 6. PATH（路径）
```rust
pub struct Path {
    pub src: Box<Vertex>,                             // 起始顶点
    pub steps: Vec<Step>,                            // 步骤列表
}

pub struct Step {
    pub dst: Box<Vertex>,                            // 目标顶点
    pub edge: Box<Edge>,                             // 边
}
```

**特点：**
- src 使用 `Box<Vertex>` 包装
- Step 结构与 Nebula-Graph 不同，直接包含完整的 Edge 对象
- 提供了 `has_duplicate_edges()` 方法检测重复边
- 提供了 `reverse()` 方法反转路径
- 提供了 `append_reverse()` 方法用于双向BFS路径拼接
- 实现了 `Ord` 和 `PartialOrd` trait，支持排序
- 提供了 `estimated_size()` 方法估算内存使用

### 三、集合类型

#### 7. LIST（列表）
```rust
pub struct List {
    pub values: Vec<Value>,
}

// 在 Value 枚举中直接使用 Vec<Value>
List(Vec<Value>)
```

**特点：**
- 使用 Rust 的 `Vec<Value>` 实现
- 支持任意类型的元素
- 实现了 `Hash` trait

#### 8. SET（集合）
```rust
// 在 Value 枚举中直接使用 HashSet<Value>
Set(std::collections::HashSet<Value>)
```

**特点：**
- 使用 Rust 的 `HashSet<Value>` 实现
- 元素唯一
- 实现了 `Hash` trait

#### 9. MAP（映射）
```rust
// 在 Value 枚举中直接使用 HashMap<String, Value>
Map(std::collections::HashMap<String, Value>)
```

**特点：**
- 使用 Rust 的 `HashMap<String, Value>` 实现
- 键为字符串，值为任意类型
- 实现了 `Hash` trait

### 四、数据集类型

#### 10. DATASET（数据集）
```rust
pub struct DataSet {
    pub col_names: Vec<String>,                       // 列名
    pub rows: Vec<Vec<Value>>,                        // 行数据
}
```

**特点：**
- 使用 `Vec<Vec<Value>>` 表示行数据（与 Nebula-Graph 的 `Vec<Row>` 不同）
- 提供了 `new()` 构造函数
- 提供了 `estimated_size()` 方法估算内存使用

### 五、地理空间类型

#### 11. GEOGRAPHY（地理空间）
```rust
pub struct GeographyValue {
    pub latitude: f64,                               // 纬度
    pub longitude: f64,                               // 经度
}
```

**特点：**
- 当前版本仅支持基础坐标点
- 不支持 LineString 和 Polygon
- 不支持 WKT/WKB 格式
- 手动实现了 `Hash` trait 以处理 f64 字段
- 提供了 `estimated_size()` 方法

### 六、特殊类型

#### 12. EMPTY（空）
- 表示未初始化或空值状态

## Value 结构设计

```rust
pub enum Value {
    Empty,
    Null(NullType),
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Date(DateValue),
    Time(TimeValue),
    DateTime(DateTimeValue),
    Vertex(Box<crate::core::vertex_edge_path::Vertex>),
    Edge(crate::core::vertex_edge_path::Edge),
    Path(crate::core::vertex_edge_path::Path),
    List(Vec<Value>),
    Map(std::collections::HashMap<String, Value>),
    Set(std::collections::HashSet<Value>),
    Geography(GeographyValue),
    Duration(DurationValue),
    DataSet(DataSet),
}
```

**设计特点：**
1. **使用 Rust enum**：利用 Rust 的 enum 实现类型安全的联合体
2. **Box 包装**：对于大对象（Vertex, Edge）使用 Box 减少栈内存占用
3. **直接使用集合类型**：List、Map、Set 直接使用 Rust 标准库的集合类型
4. **序列化支持**：实现了 `Serialize` 和 `Deserialize` trait（使用 serde）
5. **编码支持**：实现了 `Encode` 和 `Decode` trait（使用 bincode）
6. **类型检查**：提供了 `get_type()` 方法获取值的类型
7. **内存估算**：提供了 `estimated_size()` 方法估算内存使用

## 支持的操作

### 算术运算
- `negate()` - 取反（一元）
- `abs()` - 绝对值

### 比较运算
- 实现了 `PartialEq` 和 `Eq` trait
- 实现了 `PartialOrd` 和 `Ord` trait

### 逻辑运算
- 通过 `bool_value()` 方法获取布尔值

### 其他操作
- `length()` - 计算长度（适用于 String、List、Map、Set、Path）
- `hash_value()` - 计算哈希值

## 序列化支持

所有类型都支持以下序列化方式：
- **serde**：`Serialize` 和 `Deserialize` trait（用于 JSON、MessagePack 等）
- **bincode**：`Encode` 和 `Decode` trait（用于二进制序列化）

## 哈希支持

所有类型都实现了 `Hash` trait，可用于哈希表和集合：
- 对于复杂类型（如 Tag、Vertex、Edge、List、Map、Set），手动实现了 `Hash` trait
- 对于包含 f64 的类型（如 GeographyValue），使用 `to_bits()` 转换为整数后再哈希

## 内存管理

### 内存估算
所有主要类型都提供了 `estimated_size()` 方法，用于估算内存使用：
- 考虑了栈内存和堆内存
- 考虑了容量的预分配
- 递归计算嵌套类型的大小

### Box 使用
- Vertex 的 vid 使用 `Box<Value>` 包装
- Edge 的 src 和 dst 使用 `Box<Value>` 包装
- Path 的 src 使用 `Box<Vertex>` 包装
- Step 的 dst 和 edge 使用 `Box` 包装

**目的：**
- 减少栈内存占用
- 避免递归类型导致的无限大小

## 类型系统工具

### TypeUtils
提供了丰富的类型系统工具：
- `are_types_compatible()` - 检查类型兼容性
- `is_superior_type()` - 检查是否为优越类型
- `get_type_priority()` - 获取类型优先级
- `get_common_type()` - 获取公共类型
- `can_cast()` - 检查类型转换是否可行
- `get_cast_targets()` - 获取可转换的目标类型
- `is_indexable_type()` - 检查类型是否可用于索引
- `get_default_value()` - 获取类型的默认值

## 文件位置

所有数据类型的定义位于 `src/core/` 目录：

- [src/core/types/mod.rs](../src/core/types/mod.rs) - DataType 枚举定义
- [src/core/value/types.rs](../src/core/value/types.rs) - Value 枚举和基础类型定义
- [src/core/vertex_edge_path.rs](../src/core/vertex_edge_path.rs) - 图数据结构定义
- [src/core/type_system.rs](../src/core/type_system.rs) - 类型系统工具

## 与 Nebula-Graph 的主要区别

### 新增类型
1. **Int8, Int16, Int32, Int64** - 细粒度的整数类型
2. **Double** - 双精度浮点数
3. **FixedString(usize)** - 定长字符串
4. **Blob** - 二进制数据
5. **VID** - 顶点ID专用类型
6. **Timestamp** - 时间戳类型

### 设计差异
1. **使用 Rust enum** 而不是 C++ union
2. **Box 包装** 大对象以优化内存
3. **内部整数ID** 用于快速索引
4. **顶点级属性** 支持顶点和标签两级属性
5. **简化的地理空间类型** 仅支持坐标点
6. **更丰富的类型系统工具**

### 功能增强
1. **内存估算** - 所有类型都支持内存使用估算
2. **序列化支持** - 同时支持 serde 和 bincode
3. **类型优先级** - 支持类型提升和转换
4. **索引类型检查** - 支持检查类型是否可用于索引

## 总结

GraphDB 提供了完整的图数据库数据类型系统，包括：
- 18 种主要数据类型（比 Nebula-Graph 多 6 种）
- 8 种 NULL 状态（与 Nebula-Graph 相同）
- 完整的类型系统工具
- 内存使用估算
- 多种序列化支持
- 简化的地理空间支持
- 优化的内存管理

这种设计使得 GraphDB 在保持与 Nebula-Graph 兼容性的同时，充分利用了 Rust 的类型系统和内存安全特性。
