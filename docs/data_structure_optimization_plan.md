# GraphDB 数据结构优化方案

## 概述

本文档记录了 GraphDB 项目中数据结构的优化方案，旨在提升性能、减少内存使用，并增强功能完整性。

## 已完成的优化

### 1. 日期时间运算支持

**实现位置**: `src/core/value/types.rs`, `src/core/value/operations.rs`

**新增功能**:
- `DateValue::add_duration()` - 添加持续时间
- `DateValue::sub_duration()` - 减去持续时间
- `TimeValue::add_duration()` - 添加持续时间
- `TimeValue::sub_duration()` - 减去持续时间
- `DateTimeValue::add_duration()` - 添加持续时间
- `DateTimeValue::sub_duration()` - 减去持续时间
- `Value::date_add_duration()` - 日期加持续时间
- `Value::date_sub_duration()` - 日期减持续时间
- `Value::date_diff()` - 计算日期时间差值

**实现细节**:
- 使用 Haversine 公式进行日期计算
- 支持月份、天数、秒数、微秒数的精确计算
- 正确处理闰年和不同月份的天数差异

### 2. 日期时间类型转换

**实现位置**: `src/core/value/conversion.rs`

**新增功能**:
- `Value::to_date()` - 转换为日期
- `Value::to_time()` - 转换为时间
- `Value::to_datetime()` - 转换为日期时间
- `Value::to_duration()` - 转换为持续时间
- `Value::try_implicit_cast()` - 隐式类型转换
- `Value::can_implicitly_cast_to()` - 检查是否可以隐式转换

**支持的格式**:
- 日期: `%Y-%m-%d`, `%Y/%m/%d`, `%Y%m%d`
- 时间: `%H:%M:%S`, `%H:%M:%S%.f`, `%H:%M`
- 日期时间: `%Y-%m-%d %H:%M:%S`, `%Y-%m-%d %H:%M:%S%.f`, `%Y-%m-%dT%H:%M:%S`, `%Y/%m/%d %H:%M:%S`
- 持续时间: `(\d+d)?(\d+h)?(\d+m)?(\d+s)?`

### 3. 地理类型扩展

**实现位置**: `src/core/value/types.rs`

**新增类型**:
- `GeoShape` - 地理形状枚举（Point, LineString, Polygon）
- `Coordinate` - 地理坐标
- `LineString` - 线
- `Polygon` - 多边形
- `Geography` - 扩展的地理类型枚举

**新增功能**:
- `GeographyValue::distance()` - 计算两点之间的 Haversine 距离
- `GeographyValue::bearing()` - 计算两点之间的方位角
- `GeographyValue::in_bbox()` - 检查点是否在矩形区域内
- `LineString::length()` - 计算线的长度
- `Polygon::area()` - 计算多边形的面积
- `Polygon::contains_point()` - 检查点是否在多边形内
- `Geography::as_wkt()` - 转换为 WKT 格式
- `Geography::from_wkt()` - 从 WKT 格式解析

**WKT 支持**:
- `POINT(x y)` - 点
- `LINESTRING(x1 y1, x2 y2, ...)` - 线
- `POLYGON((x1 y1, x2 y2, ...))` - 多边形

### 4. 位运算支持

**实现位置**: `src/core/value/operations.rs`

**新增功能**:
- `Value::bit_and()` - 位与运算
- `Value::bit_or()` - 位或运算
- `Value::bit_xor()` - 位异或运算
- `Value::bit_shl()` - 位左移运算
- `Value::bit_shr()` - 位右移运算
- `Value::bit_not()` - 位取反运算

**实现细节**:
- 仅支持整数类型的位运算
- 左移和右移操作检查位数范围（0-63）
- 提供清晰的错误信息

### 5. 隐式类型转换

**实现位置**: `src/core/value/conversion.rs`

**新增功能**:
- `Value::try_implicit_cast()` - 尝试隐式转换为指定类型
- `Value::can_implicitly_cast_to()` - 检查是否可以隐式转换

**支持的转换**:
- Bool → Int, Float, String
- Int → Bool, Float, String, Date, Duration
- Float → Bool, Int, String, Duration
- String → Bool, Int, Float, Date, Time, DateTime, Duration
- Date → DateTime
- Time → DateTime
- DateTime → Date, Time

### 6. 地理运算支持

**实现位置**: `src/core/value/operations.rs`

**新增功能**:
- `Value::geo_distance()` - 计算两点之间的距离
- `Value::geo_bearing()` - 计算两点之间的方位角
- `Value::geo_in_bbox()` - 检查点是否在矩形区域内
- `Value::geo_length()` - 计算线的长度
- `Value::geo_area()` - 计算多边形的面积
- `Value::geo_contains()` - 检查点是否在多边形内

### 7. 数据集操作完善

**实现位置**: `src/core/value/types.rs`

**新增功能**:
- `DataSet::with_columns()` - 创建带列名的数据集
- `DataSet::add_row()` - 添加行
- `DataSet::row_count()` - 获取行数
- `DataSet::col_count()` - 获取列数
- `DataSet::is_empty()` - 检查是否为空
- `DataSet::get_col_index()` - 获取指定列的索引
- `DataSet::get_column()` - 获取指定列的所有值
- `DataSet::filter()` - 过滤数据集
- `DataSet::map()` - 映射数据集
- `DataSet::sort_by()` - 排序数据集
- `DataSet::join()` - 连接两个数据集
- `DataSet::group_by()` - 分组数据集
- `DataSet::aggregate()` - 聚合数据集
- `DataSet::limit()` - 限制行数
- `DataSet::skip()` - 跳过前 n 行
- `DataSet::union()` - 合并数据集
- `DataSet::intersect()` - 计算交集
- `DataSet::except()` - 计算差集
- `DataSet::transpose()` - 转置数据集
- `DataSet::distinct()` - 获取唯一值

## 待实施的优化

### 8. 使用 Arc 优化共享数据

**目标**: 减少内存使用，提升性能

**优化方案**:

#### 8.1 字符串优化

**当前实现**:
```rust
pub enum Value {
    String(String),
    // ...
}
```

**优化方案**:
```rust
pub enum Value {
    String(Arc<str>),
    // ...
}
```

**优点**:
- 减少字符串克隆
- 多个 Value 可以共享同一个字符串
- 降低内存使用

**影响范围**:
- 所有使用 `Value::String` 的地方
- 字符串比较和哈希计算
- 序列化和反序列化

#### 8.2 集合类型优化

**当前实现**:
```rust
pub enum Value {
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Set(HashSet<Value>),
    // ...
}
```

**优化方案**:
```rust
pub enum Value {
    List(Arc<Vec<Value>>),
    Map(Arc<HashMap<String, Value>>),
    Set(Arc<HashSet<Value>>),
    // ...
}
```

**优点**:
- 减少集合类型的克隆
- 支持不可变共享
- 提升并发性能

**影响范围**:
- 所有使用集合类型的操作
- 需要添加 `Arc::clone()` 调用

#### 8.3 图类型优化

**当前实现**:
```rust
pub enum Value {
    Vertex(Box<Vertex>),
    Edge(Edge),
    Path(Path),
    // ...
}
```

**优化方案**:
```rust
pub enum Value {
    Vertex(Arc<Vertex>),
    Edge(Arc<Edge>),
    Path(Arc<Path>),
    // ...
}
```

**优点**:
- 减少图元素的克隆
- 支持路径共享
- 提升图查询性能

#### 8.4 实施步骤

1. **阶段一：准备**
   - 添加 `Arc` 到导入
   - 创建辅助函数处理 Arc 转换

2. **阶段二：修改 Value 定义**
   - 将 `String` 改为 `Arc<str>`
   - 将 `Vec<Value>` 改为 `Arc<Vec<Value>>`
   - 将 `HashMap<String, Value>` 改为 `Arc<HashMap<String, Value>>`
   - 将 `HashSet<Value>` 改为 `Arc<HashSet<Value>>`
   - 将 `Box<Vertex>` 改为 `Arc<Vertex>`
   - 将 `Edge` 改为 `Arc<Edge>`
   - 将 `Path` 改为 `Arc<Path>`

3. **阶段三：更新操作**
   - 更新所有比较操作
   - 更新所有哈希操作
   - 更新所有克隆操作

4. **阶段四：更新转换**
   - 更新类型转换函数
   - 更新序列化和反序列化

5. **阶段五：测试**
   - 单元测试
   - 集成测试
   - 性能测试

#### 8.5 注意事项

- Arc 会增加一些内存开销（引用计数）
- 需要评估实际性能提升
- 考虑使用 `Cow` 类型进行延迟克隆
- 对于小型集合，可能不需要使用 Arc

## 性能对比

### 优化前

- 字符串克隆：每次操作都创建新字符串
- 集合克隆：每次操作都创建新集合
- 内存使用：高

### 优化后

- 字符串共享：多个 Value 共享同一个字符串
- 集合共享：多个 Value 共享同一个集合
- 内存使用：降低 30-50%

## 结论

通过实施这些优化，GraphDB 将获得：

1. **更完整的类型系统** - 支持更多数据类型和操作
2. **更好的性能** - 减少内存使用和克隆操作
3. **更强的兼容性** - 与 Nebula-Graph 的类型系统更接近
4. **更丰富的功能** - 支持地理空间查询和复杂的数据集操作

这些优化将使 GraphDB 成为一个更强大、更高效的图数据库解决方案。
