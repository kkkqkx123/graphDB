# Nebula-Graph 数据结构分析

## 概述

Nebula-Graph 是一个分布式图数据库，支持丰富的数据类型。所有数据类型都封装在统一的 `Value` 结构中，使用联合体（union）存储以节省内存。Value 结构体大小固定为 16 字节。

## 数据类型分类

### 一、基础数据类型

#### 1. NULL 类型
```cpp
enum class NullType {
    __NULL__ = 0,      // 普通空值
    NaN = 1,           // 非数字
    BAD_DATA = 2,      // 错误数据
    BAD_TYPE = 3,      // 错误类型
    ERR_OVERFLOW = 4,  // 溢出错误
    UNKNOWN_PROP = 5,  // 未知属性
    DIV_BY_ZERO = 6,   // 除零错误
    OUT_OF_RANGE = 7,  // 超出范围
};
```

#### 2. 标量类型

| 类型 | 说明 | 存储方式 |
|------|------|----------|
| BOOL | 布尔值 | bool |
| INT | 整数 | int64_t |
| FLOAT | 浮点数 | double |
| STRING | 字符串 | std::unique_ptr<std::string> |

#### 3. 日期时间类型

| 类型 | 说明 | 存储方式 |
|------|------|----------|
| DATE | 日期（年、月、日） | Date (year, month, day) |
| TIME | 时间（时、分、秒、微秒） | Time (hour, minute, sec, microsec) |
| DATETIME | 日期时间 | DateTime (year, month, day, hour, minute, sec, microsec) |
| DURATION | 时间段 | Duration (months, seconds, microseconds) |

### 二、图数据结构

#### 4. VERTEX（顶点）
```cpp
struct Vertex {
    Value vid;                              // 顶点ID
    std::vector<Tag> tags;                  // 标签列表
    std::atomic<size_t> refcnt{1};          // 引用计数
};

struct Tag {
    std::string name;                       // 标签名称
    std::unordered_map<std::string, Value> props;  // 属性映射
};
```

**特点：**
- 每个顶点包含一个 ID 和多个标签
- 每个标签包含名称和属性映射
- 使用引用计数管理内存
- 支持属性查找和标签属性访问

#### 5. EDGE（边）
```cpp
struct Edge {
    Value src;                              // 源顶点ID
    Value dst;                              // 目标顶点ID
    EdgeType type;                          // 边类型
    std::string name;                       // 边名称
    EdgeRanking ranking;                    // 边的权重/排名
    std::unordered_map<std::string, Value> props;  // 属性
    std::atomic<size_t> refcnt{1};          // 引用计数
};
```

**特点：**
- 支持有向边（通过 type 正负区分方向）
- 支持边权重（ranking）
- 支持属性存储
- 支持边反转操作

#### 6. PATH（路径）
```cpp
struct Path {
    Vertex src;                     // 起始顶点
    std::vector<Step> steps;       // 步骤列表
};

struct Step {
    Vertex dst;                                    // 目标顶点
    EdgeType type;                                 // 边类型
    std::string name;                              // 边名称
    EdgeRanking ranking;                           // 排名
    std::unordered_map<std::string, Value> props; // 属性
};
```

**特点：**
- 路径由起始顶点和一系列步骤组成
- 每个步骤包含目标顶点和边信息
- 支持路径拼接（append）
- 支持路径反转（reverse）
- 支持检测重复顶点和边

### 三、集合类型

#### 7. LIST（列表）
```cpp
struct List {
    std::vector<Value> values;  // 有序的值列表
};
```

**特点：**
- 有序集合，支持重复元素
- 支持索引访问
- 支持追加、合并操作
- 支持包含检查

#### 8. SET（集合）
```cpp
struct Set {
    std::unordered_set<Value> values;  // 无序的值集合
};
```

**特点：**
- 无序集合，元素唯一
- 支持集合交集操作
- 支持包含检查

#### 9. MAP（映射）
```cpp
struct Map {
    std::unordered_map<std::string, Value> kvs;  // 键值对映射
};
```

**特点：**
- 键为字符串，值为任意类型
- 支持键查找
- 支持包含检查

### 四、数据集类型

#### 10. DATASET（数据集）
```cpp
struct DataSet {
    std::vector<std::string> colNames;  // 列名
    std::vector<Row> rows;              // 行数据（Row = List）
};
```

**特点：**
- 表格形式的数据结构
- 支持行追加（append）
- 支持列合并（merge）
- 支持水平合并
- 支持按列名或索引访问数据

### 五、地理空间类型

#### 11. GEOGRAPHY（地理空间）
```cpp
struct Geography {
    std::variant<Point, LineString, Polygon> geo_;
};
```

支持三种地理形状：

**Point（点）**
```cpp
struct Point {
    Coordinate coord;  // (x, y) 坐标
};
```

**LineString（线）**
```cpp
struct LineString {
    std::vector<Coordinate> coordList;  // 坐标点列表
};
```

**Polygon（多边形）**
```cpp
struct Polygon {
    std::vector<std::vector<Coordinate>> coordListList;  // 多个坐标环
};
```

**特点：**
- 支持 WKT（Well-Known Text）格式
- 支持 WKB（Well-Known Binary）格式
- 支持坐标归一化
- 支持有效性验证
- 支持转换为 S2 库的地理对象

### 六、特殊类型

#### 12. EMPTY（空）
- 表示未初始化或空值状态
- 在比较中是最小的值

## Value 结构设计

```cpp
struct Value {
    enum class Type : uint64_t {
        __EMPTY__ = 1UL,
        BOOL = 1UL << 1,
        INT = 1UL << 2,
        FLOAT = 1UL << 3,
        STRING = 1UL << 4,
        DATE = 1UL << 5,
        TIME = 1UL << 6,
        DATETIME = 1UL << 7,
        VERTEX = 1UL << 8,
        EDGE = 1UL << 9,
        PATH = 1UL << 10,
        LIST = 1UL << 11,
        MAP = 1UL << 12,
        SET = 1UL << 13,
        DATASET = 1UL << 14,
        GEOGRAPHY = 1UL << 15,
        DURATION = 1UL << 16,
        NULLVALUE = 1UL << 63,
    };

    Type type_;
    union Storage {
        NullType nVal;
        bool bVal;
        int64_t iVal;
        double fVal;
        std::unique_ptr<std::string> sVal;
        Date dVal;
        Time tVal;
        DateTime dtVal;
        Vertex* vVal;
        Edge* eVal;
        std::unique_ptr<Path> pVal;
        std::unique_ptr<List> lVal;
        std::unique_ptr<Map> mVal;
        std::unique_ptr<Set> uVal;
        std::unique_ptr<DataSet> gVal;
        std::unique_ptr<Geography> ggVal;
        std::unique_ptr<Duration> duVal;
    } value_;
};
```

**设计特点：**
1. **内存高效**：使用 union 存储不同类型的值，结构体大小固定为 16 字节
2. **类型安全**：每个类型都有对应的类型检查方法
3. **智能指针**：复杂类型使用 `std::unique_ptr` 管理堆内存
4. **引用计数**：Vertex 和 Edge 使用引用计数管理内存

## 支持的操作

### 算术运算
- `+` 加法
- `-` 减法
- `*` 乘法
- `/` 除法
- `%` 取模
- `-` 取负（一元）

### 比较运算
- `<` 小于
- `>` 大于
- `<=` 小于等于
- `>=` 大于等于
- `==` 等于
- `!=` 不等于

### 逻辑运算
- `&&` 逻辑与
- `||` 逻辑或
- `!` 逻辑非

### 位运算
- `&` 按位与
- `|` 按位或
- `^` 按位异或

## 序列化支持

所有类型都支持以下方法：
- `toString()` - 转换为字符串
- `toJson()` - 转换为 JSON
- `getMetaData()` - 获取元数据（用于 JSON 格式的查询结果）

## 哈希支持

所有类型都实现了 `std::hash` 特化，可用于哈希表和集合。

## 日期时间计算

支持日期时间与 Duration 的加减运算：
```cpp
Date + Duration
Date - Duration
Time + Duration
Time - Duration
DateTime + Duration
DateTime - Duration
```

## 地理空间操作

- `fromWKT()` - 从 WKT 格式解析
- `fromWKB()` - 从 WKB 格式解析
- `asWKT()` - 转换为 WKT 格式
- `asWKB()` - 转换为 WKB 格式
- `asWKBHex()` - 转换为 WKB 十六进制格式
- `asS2()` - 转换为 S2 库对象
- `centroid()` - 计算几何中心
- `normalize()` - 坐标归一化
- `isValid()` - 有效性验证

## 文件位置

所有数据类型的定义位于 `nebula-3.8.0/src/common/datatypes/` 目录：

- [Value.h](../src/common/datatypes/Value.h) - 核心值类型定义
- [Vertex.h](../src/common/datatypes/Vertex.h) - 顶点类型定义
- [Edge.h](../src/common/datatypes/Edge.h) - 边类型定义
- [Path.h](../src/common/datatypes/Path.h) - 路径类型定义
- [List.h](../src/common/datatypes/List.h) - 列表类型定义
- [Set.h](../src/common/datatypes/Set.h) - 集合类型定义
- [Map.h](../src/common/datatypes/Map.h) - 映射类型定义
- [DataSet.h](../src/common/datatypes/DataSet.h) - 数据集类型定义
- [Date.h](../src/common/datatypes/Date.h) - 日期时间类型定义
- [Duration.h](../src/common/datatypes/Duration.h) - 时间段类型定义
- [Geography.h](../src/common/datatypes/Geography.h) - 地理空间类型定义

## 总结

Nebula-Graph 提供了完整的图数据库数据类型系统，包括：
- 12 种主要数据类型
- 8 种 NULL 状态
- 完整的运算支持
- 丰富的序列化选项
- 地理空间支持
- 高效的内存管理

这种设计使得 Nebula-Graph 能够支持复杂的图查询和数据分析任务。
