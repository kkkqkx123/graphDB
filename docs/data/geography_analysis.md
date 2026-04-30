# GraphDB 地理空间功能分析

## 概述

GraphDB 提供了完整的地理空间数据支持,包括多种几何类型、丰富的空间函数以及标准格式支持。本文档详细分析了项目中的地理相关功能。

## 核心数据类型

### 1. 基础几何类型

#### GeographyValue (Point)
- **定义**: 单个坐标点,包含纬度(latitude)和经度(longitude)
- **核心方法**:
  - `distance()`: 计算两点间的 Haversine 距离(单位:公里)
  - `bearing()`: 计算两点间的方位角(单位:度)
  - `in_bbox()`: 检查点是否在指定矩形区域内
  - `is_valid()`: 验证坐标范围(-90~90纬度,-180~180经度)

#### LineStringValue
- **定义**: 有序点序列,表示线串
- **核心方法**:
  - `length()`: 计算总长度(公里)
  - `is_closed()`: 检查是否闭合(首尾点相同)
  - `is_ring()`: 检查是否为环(闭合且至少4个点)
  - `centroid()`: 计算中心点
  - `start_point()`, `end_point()`: 获取起点和终点

#### PolygonValue
- **定义**: 闭合环(外环)和可选的孔洞(内环)
- **核心方法**:
  - `area()`: 计算面积(平方公里,使用球面过剩公式)
  - `perimeter()`: 计算周长(公里)
  - `contains_point()`: 点包含检测(射线投射算法)
  - `bounding_box()`: 获取边界框
  - `is_valid()`: 验证多边形有效性

#### MultiPointValue
- **定义**: 点集合
- **核心方法**:
  - `num_points()`: 获取点数量
  - `centroid()`: 计算中心点
  - `is_valid()`: 验证所有点

#### MultiLineStringValue
- **定义**: 线串集合
- **核心方法**:
  - `num_linestrings()`: 获取线串数量
  - `length()`: 计算总长度
  - `is_valid()`: 验证所有线串

#### MultiPolygonValue
- **定义**: 多边形集合
- **核心方法**:
  - `num_polygons()`: 获取多边形数量
  - `area()`: 计算总面积
  - `contains_point()`: 检查点是否在任一多边形内
  - `is_valid()`: 验证所有多边形

### 2. Geography 枚举类型

统一的地理类型枚举,包含所有几何类型:
```rust
pub enum Geography {
    Point(GeographyValue),
    LineString(LineStringValue),
    Polygon(PolygonValue),
    MultiPoint(MultiPointValue),
    MultiLineString(MultiLineStringValue),
    MultiPolygon(MultiPolygonValue),
}
```

**通用方法**:
- `geometry_type()`: 获取几何类型名称
- `centroid()`: 计算中心点
- `is_valid()`: 验证几何对象
- `bounding_box()`: 获取边界框
- `estimated_size()`: 估算内存使用

## 数据格式支持

### 1. WKT (Well-Known Text)

**解析**: `Geography::from_wkt()`
- 支持格式: POINT, LINESTRING, POLYGON, MULTIPOINT, MULTILINESTRING, MULTIPOLYGON
- 示例: `POINT(116.4074 39.9042)`, `POLYGON((0 0, 0 1, 1 1, 1 0, 0 0))`

**转换**: `Geography::to_wkt()`
- 将地理对象转换为 WKT 字符串

### 2. GeoJSON

**转换**: `Geography::to_geojson()`
- 支持 GeoJSON Geometry 对象格式
- 包含完整的坐标序列和类型信息

**解析**: `Geography::from_geojson()`
- 从 GeoJSON Geometry 对象创建地理对象
- 支持 Feature 和 FeatureCollection 结构

## 地理空间函数

### 1. 构造函数

| 函数名 | 参数 | 说明 |
|--------|------|------|
| `ST_Point(lon, lat)` | 数值,数值 | 创建地理点 |
| `ST_GeogFromText(wkt)` | 字符串 | 从 WKT 创建地理对象 |
| `ST_GeomFromGeoJSON(geojson)` | 字符串 | 从 GeoJSON 创建地理对象 |

### 2. 转换函数

| 函数名 | 参数 | 说明 |
|--------|------|------|
| `ST_AsText(geo)` | 地理对象 | 转换为 WKT 字符串 |
| `ST_AsGeoJSON(geo)` | 地理对象 | 转换为 GeoJSON 字符串 |

### 3. 属性函数

| 函数名 | 参数 | 返回值 | 说明 |
|--------|------|--------|------|
| `ST_GeometryType(geo)` | 地理对象 | 字符串 | 返回几何类型名称 |
| `ST_NPoints(geo)` | 地理对象 | 整数 | 返回点数量 |
| `ST_Centroid(geo)` | 地理对象 | Point | 计算中心点 |
| `ST_IsValid(geo)` | 地理对象 | 布尔 | 检查有效性 |
| `ST_StartPoint(linestring)` | LineString | Point | 返回起点 |
| `ST_EndPoint(linestring)` | LineString | Point | 返回终点 |
| `ST_IsRing(linestring)` | LineString | 布尔 | 检查是否为环 |
| `ST_IsClosed(linestring)` | LineString | 布尔 | 检查是否闭合 |

### 4. 测量函数

| 函数名 | 参数 | 返回值 | 说明 |
|--------|------|--------|------|
| `ST_Distance(geo1, geo2)` | 地理对象,地理对象 | Double | 计算距离(公里) |
| `ST_Area(geo)` | Polygon/MultiPolygon | Double | 计算面积(平方公里) |
| `ST_Length(geo)` | LineString/Polygon | Double | 计算长度/周长(公里) |
| `ST_Perimeter(geo)` | Polygon/MultiPolygon | Double | 计算周长(公里) |

### 5. 空间关系函数

| 函数名 | 参数 | 返回值 | 说明 |
|--------|------|--------|------|
| `ST_Intersects(geo1, geo2)` | 地理对象,地理对象 | 布尔 | 检查是否相交 |
| `ST_Contains(geo1, geo2)` | 地理对象,地理对象 | 布尔 | 检查包含关系 |
| `ST_Within(geo1, geo2)` | 地理对象,地理对象 | 布尔 | 检查被包含关系 |
| `ST_Covers(geo1, geo2)` | 地理对象,地理对象 | 布尔 | 检查覆盖关系 |
| `ST_CoveredBy(geo1, geo2)` | 地理对象,地理对象 | 布尔 | 检查被覆盖关系 |
| `ST_DWithin(geo1, geo2, distance)` | 地理对象,地理对象,数值 | 布尔 | 检查是否在指定距离内 |
| `ST_Crosses(geo1, geo2)` | 地理对象,地理对象 | 布尔 | 检查交叉关系 |
| `ST_Touches(geo1, geo2)` | 地理对象,地理对象 | 布尔 | 检查接触关系 |
| `ST_Overlaps(geo1, geo2)` | 地理对象,地理对象 | 布尔 | 检查重叠关系 |
| `ST_Equals(geo1, geo2)` | 地理对象,地理对象 | 布尔 | 检查相等关系 |

### 6. 操作函数

| 函数名 | 参数 | 返回值 | 说明 |
|--------|------|--------|------|
| `ST_Envelope(geo)` | 地理对象 | Polygon | 返回边界框多边形 |
| `ST_Buffer(geo, distance)` | 地理对象,数值 | Polygon | 创建缓冲区多边形 |
| `ST_Boundary(geo)` | 地理对象 | Geography | 返回边界 |

## 核心算法实现

### 1. 距离计算

#### 点对点距离 (Haversine 公式)
```rust
pub fn distance(&self, other: &GeographyValue) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;
    // 使用 Haversine 公式计算球面距离
}
```

#### 点到线串距离
- 计算点到线段的最短距离
- 支持投影点计算

#### 点到多边形距离
- 内部点距离为 0
- 外部点计算到边界的最短距离

### 2. 面积计算

使用球面过剩公式计算多边形面积:
```rust
fn ring_area(&self, ring: &LineStringValue) -> f64 {
    // 球面过剩公式实现
}
```

### 3. 点包含检测

使用射线投射算法判断点是否在多边形内:
```rust
fn point_in_ring(&self, point: &GeographyValue, ring: &LineStringValue) -> bool {
    // 射线投射算法实现
}
```

### 4. 缓冲区生成

为点和线串创建圆形/带状缓冲区:
```rust
fn create_buffer(geo: &Geography, radius_km: f64) -> Option<PolygonValue> {
    // 使用目标点算法生成缓冲区
}
```

## 测试数据

项目包含完整的地理测试数据集 (`tests/e2e/data/geography_data.gql`):
- 200 个地理位置点(中国主要城市)
- 包含名称、坐标、地址、类别等属性
- 生成邻近关系边(距离和步行时间)

## 应用场景

### 1. 位置服务
- 地点查询和距离计算
- 附近地点搜索
- 路径规划基础

### 2. 地理分析
- 区域包含关系判断
- 空间范围查询
- 地理围栏

### 3. 地图可视化
- GeoJSON 格式输出
- 与前端地图库集成
- 地理数据展示

## 技术特点

### 1. 高精度计算
- 使用 Haversine 公式计算球面距离
- 球面过剩公式计算面积
- 考虑地球曲率

### 2. 标准格式支持
- WKT 格式兼容主流 GIS 系统
- GeoJSON 支持 Web 应用
- 完整的解析和序列化

### 3. 类型安全
- Rust 类型系统保证内存安全
- 编译时类型检查
- 无运行时错误

### 4. 性能优化
- 边界框快速过滤
- 空间索引支持(未来)
- 内存使用估算

## 文件位置

- **核心类型定义**: `src/core/value/geography.rs`
- **地理函数实现**: `src/query/executor/expression/functions/builtin/geography.rs`
- **测试数据**: `tests/e2e/data/geography_data.gql`
- **扩展计划**: `docs/plan/geography_extension_plan.md`

## 未来扩展

根据 `geography_extension_plan.md`,计划扩展:
- 空间索引(R-Tree, Quad-Tree)
- 更多空间操作(并集、交集、差集)
- 坐标系转换
- 空间聚合函数
- 性能优化

## 总结

GraphDB 的地理空间功能提供了完整的 GIS 支持,包括:
- 6 种核心几何类型
- 30+ 地理空间函数
- WKT 和 GeoJSON 格式支持
- 高精度球面计算
- 丰富的测试数据

这些功能为图数据库提供了强大的地理空间分析能力,适用于位置服务、地理分析、地图可视化等应用场景。
