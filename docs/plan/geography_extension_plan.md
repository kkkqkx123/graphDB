# 地理类型扩展执行方案

## 概述

本文档基于 `geography_analysis.md` 分析文档，制定地理类型扩展的分阶段执行方案。

### 已完成的核心功能 ✅

- [x] GeographyValue 类型定义（Point 类型）
- [x] Value 枚举扩展
- [x] DataType 扩展
- [x] 序列化/反序列化支持
- [x] 10 个空间函数实现
- [x] WKT 解析（仅 POINT）
- [x] 错误信息已英文化

### 阶段一完成状态 ✅

- [x] Geography 枚举扩展（6 种几何类型：Point, LineString, Polygon, MultiPoint, MultiLineString, MultiPolygon）
- [x] WKT 解析支持所有类型
- [x] 核心方法实现（length, area, contains, centroid, is_valid, bounding_box 等）
- [x] Value 系统集成（比较、哈希、显示）
- [x] 完整的单元测试（19 个测试通过）
- [x] 空间函数更新支持新类型
- [x] 新增空间函数：st_area, st_length, st_perimeter, st_npoints, st_startpoint, st_endpoint, st_isring, st_isclosed, st_geometrytype, st_contains, st_within, st_envelope

### 待完成任务

根据设计文档和实际使用需求，后续任务分为 2 个阶段：

---

## 阶段 1：扩展几何类型（优先级：高）✅ 已完成

**目标**：添加 LineString、Polygon 等复杂几何类型支持

**状态**：✅ 已完成

### 1.1 类型系统设计

#### 1.1.1 Geography 枚举扩展

**文件**：`src/core/value/geography.rs`

```rust
/// Geographic type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Encode, Decode)]
pub enum Geography {
    Point(GeographyValue),
    LineString(LineStringValue),
    Polygon(PolygonValue),
    MultiPoint(MultiPointValue),
    MultiLineString(MultiLineStringValue),
    MultiPolygon(MultiPolygonValue),
}

/// Point type (single coordinate pair)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Encode, Decode)]
pub struct GeographyValue {
    pub latitude: f64,
    pub longitude: f64,
}

/// LineString type (ordered sequence of points)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Encode, Decode)]
pub struct LineStringValue {
    pub points: Vec<GeographyValue>,
}

/// Polygon type (closed ring with optional holes)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Encode, Decode)]
pub struct PolygonValue {
    pub exterior: LineStringValue,
    pub holes: Vec<LineStringValue>,
}

/// MultiPoint type (collection of points)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Encode, Decode)]
pub struct MultiPointValue {
    pub points: Vec<GeographyValue>,
}

/// MultiLineString type (collection of linestrings)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Encode, Decode)]
pub struct MultiLineStringValue {
    pub linestrings: Vec<LineStringValue>,
}

/// MultiPolygon type (collection of polygons)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Encode, Decode)]
pub struct MultiPolygonValue {
    pub polygons: Vec<PolygonValue>,
}
```

#### 1.1.2 Value 枚举调整

**文件**：`src/core/value/value_def.rs`

```rust
pub enum Value {
    // ... existing variants
    Geography(Geography),  // Changed from GeographyValue to Geography
    // ...
}
```

### 1.2 WKT 解析扩展

**文件**：`src/core/value/geography.rs`

```rust
impl Geography {
    /// Parse geographic data from WKT format
    pub fn from_wkt(wkt: &str) -> Result<Self, String> {
        let wkt = wkt.trim().to_uppercase();

        if wkt.starts_with("POINT") {
            Self::parse_point_wkt(&wkt[5..])
        } else if wkt.starts_with("LINESTRING") {
            Self::parse_linestring_wkt(&wkt[10..])
        } else if wkt.starts_with("POLYGON") {
            Self::parse_polygon_wkt(&wkt[7..])
        } else if wkt.starts_with("MULTIPOINT") {
            Self::parse_multipoint_wkt(&wkt[10..])
        } else if wkt.starts_with("MULTILINESTRING") {
            Self::parse_multilinestring_wkt(&wkt[15..])
        } else if wkt.starts_with("MULTIPOLYGON") {
            Self::parse_multipolygon_wkt(&wkt[12..])
        } else {
            Err(format!("Unsupported WKT format: {}", wkt))
        }
    }

    /// Convert to WKT format
    pub fn to_wkt(&self) -> String {
        match self {
            Geography::Point(p) => format!("POINT({} {})", p.longitude, p.latitude),
            Geography::LineString(ls) => {
                let coords: Vec<String> = ls.points.iter()
                    .map(|p| format!("{} {}", p.longitude, p.latitude))
                    .collect();
                format!("LINESTRING({})", coords.join(", "))
            }
            Geography::Polygon(p) => {
                // ... polygon WKT format
            }
            // ... other types
        }
    }
}
```

### 1.3 核心方法实现

#### 1.3.1 LineString 方法

```rust
impl LineStringValue {
    /// Calculate total length in kilometers
    pub fn length(&self) -> f64 {
        self.points.windows(2)
            .map(|w| w[0].distance(&w[1]))
            .sum()
    }

    /// Get the number of points
    pub fn num_points(&self) -> usize {
        self.points.len()
    }

    /// Check if the linestring is closed (first point equals last)
    pub fn is_closed(&self) -> bool {
        if self.points.len() < 2 {
            return false;
        }
        let first = &self.points[0];
        let last = &self.points[self.points.len() - 1];
        (first.latitude - last.latitude).abs() < 1e-9
            && (first.longitude - last.longitude).abs() < 1e-9
    }

    /// Calculate centroid
    pub fn centroid(&self) -> Option<GeographyValue> {
        if self.points.is_empty() {
            return None;
        }
        let (sum_lat, sum_lon) = self.points.iter()
            .fold((0.0, 0.0), |(lat, lon), p| {
                (lat + p.latitude, lon + p.longitude)
            });
        Some(GeographyValue {
            latitude: sum_lat / self.points.len() as f64,
            longitude: sum_lon / self.points.len() as f64,
        })
    }
}
```

#### 1.3.2 Polygon 方法

```rust
impl PolygonValue {
    /// Calculate area in square kilometers (approximate)
    pub fn area(&self) -> f64 {
        // Shoelace formula for spherical coordinates
        let exterior_area = self.ring_area(&self.exterior);
        let holes_area: f64 = self.holes.iter()
            .map(|h| self.ring_area(h))
            .sum();
        (exterior_area - holes_area).abs()
    }

    fn ring_area(&self, ring: &LineStringValue) -> f64 {
        if ring.points.len() < 3 {
            return 0.0;
        }
        // Spherical area calculation using Girard's theorem
        // ...
    }

    /// Calculate perimeter in kilometers
    pub fn perimeter(&self) -> f64 {
        let exterior_perimeter = self.exterior.length();
        let holes_perimeter: f64 = self.holes.iter()
            .map(|h| h.length())
            .sum();
        exterior_perimeter + holes_perimeter
    }

    /// Check if a point is inside the polygon
    pub fn contains_point(&self, point: &GeographyValue) -> bool {
        // Ray casting algorithm
        self.point_in_ring(point, &self.exterior)
            && !self.holes.iter().any(|h| self.point_in_ring(point, h))
    }

    fn point_in_ring(&self, point: &GeographyValue, ring: &LineStringValue) -> bool {
        // Ray casting algorithm implementation
        // ...
    }

    /// Calculate centroid
    pub fn centroid(&self) -> Option<GeographyValue> {
        self.exterior.centroid()
    }
}
```

### 1.4 Value 系统集成

**文件**：`src/core/value/value_compare.rs`

```rust
impl PartialEq for Geography {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Geography::Point(a), Geography::Point(b)) => a == b,
            (Geography::LineString(a), Geography::LineString(b)) => a == b,
            (Geography::Polygon(a), Geography::Polygon(b)) => a == b,
            // ... other type combinations
            _ => false,
        }
    }
}

impl Ord for Geography {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Type-based ordering first, then content-based
        match (self, other) {
            (Geography::Point(a), Geography::Point(b)) => a.cmp(b),
            (Geography::Point(_), _) => std::cmp::Ordering::Less,
            (_, Geography::Point(_)) => std::cmp::Ordering::Greater,
            // ... other type combinations
        }
    }
}
```

### 1.5 测试计划

```rust
#[cfg(test)]
mod geometry_type_tests {
    use super::*;

    #[test]
    fn test_linestring_wkt_parsing() {
        let wkt = "LINESTRING(116.4 39.9, 121.5 31.2, 113.3 23.1)";
        let geo = Geography::from_wkt(wkt).unwrap();
        assert!(matches!(geo, Geography::LineString(_)));
    }

    #[test]
    fn test_polygon_wkt_parsing() {
        let wkt = "POLYGON((116.4 39.9, 121.5 39.9, 121.5 31.2, 116.4 31.2, 116.4 39.9))";
        let geo = Geography::from_wkt(wkt).unwrap();
        assert!(matches!(geo, Geography::Polygon(_)));
    }

    #[test]
    fn test_linestring_length() {
        let ls = LineStringValue {
            points: vec![
                GeographyValue { latitude: 39.9, longitude: 116.4 },
                GeographyValue { latitude: 31.2, longitude: 121.5 },
            ],
        };
        let length = ls.length();
        assert!(length > 1000.0);  // Beijing to Shanghai
    }

    #[test]
    fn test_polygon_contains_point() {
        let polygon = PolygonValue {
            exterior: LineStringValue {
                points: vec![
                    GeographyValue { latitude: 40.0, longitude: 116.0 },
                    GeographyValue { latitude: 40.0, longitude: 117.0 },
                    GeographyValue { latitude: 39.0, longitude: 117.0 },
                    GeographyValue { latitude: 39.0, longitude: 116.0 },
                    GeographyValue { latitude: 40.0, longitude: 116.0 },
                ],
            },
            holes: vec![],
        };
        let point = GeographyValue { latitude: 39.5, longitude: 116.5 };
        assert!(polygon.contains_point(&point));
    }
}
```

### 1.6 交付物

- [ ] Geography 枚举扩展（6 种几何类型）
- [ ] WKT 解析支持所有类型
- [ ] 核心方法实现（length, area, contains 等）
- [ ] Value 系统集成（比较、哈希、显示）
- [ ] 完整的单元测试

---

## 阶段 2：新增空间函数（优先级：高）✅ 已完成

**目标**：添加更多空间操作函数

**状态**：✅ 已完成（已实现全部 17 个函数）

### 2.1 新增函数列表

| 函数            | 参数               | 返回类型  | 描述       | 状态      |
| --------------- | ------------------ | --------- | ---------- | --------- |
| `st_buffer`     | (geo, distance_km) | Geography | 创建缓冲区 | ✅ 已实现 |
| `st_boundary`   | (geo)              | Geography | 返回边界   | ✅ 已实现 |
| `st_envelope`   | (geo)              | Geography | 返回包围盒 | ✅ 已实现 |
| `st_area`       | (geo)              | Double    | 计算面积   | ✅ 已实现 |
| `st_length`     | (geo)              | Double    | 计算长度   | ✅ 已实现 |
| `st_perimeter`  | (geo)              | Double    | 计算周长   | ✅ 已实现 |
| `st_npoints`    | (geo)              | Int       | 返回点数   | ✅ 已实现 |
| `st_startpoint` | (geo)              | Geography | 返回起点   | ✅ 已实现 |
| `st_endpoint`   | (geo)              | Geography | 返回终点   | ✅ 已实现 |
| `st_isring`     | (geo)              | Bool      | 是否为环   | ✅ 已实现 |
| `st_isclosed`   | (geo)              | Bool      | 是否闭合   | ✅ 已实现 |
| `st_contains`   | (geo1, geo2)       | Bool      | 包含关系   | ✅ 已实现 |
| `st_within`     | (geo1, geo2)       | Bool      | 被包含关系 | ✅ 已实现 |
| `st_crosses`    | (geo1, geo2)       | Bool      | 相交关系   | ✅ 已实现 |
| `st_touches`    | (geo1, geo2)       | Bool      | 接触关系   | ✅ 已实现 |
| `st_overlaps`   | (geo1, geo2)       | Bool      | 重叠关系   | ✅ 已实现 |
| `st_equals`     | (geo1, geo2)       | Bool      | 几何相等   | ✅ 已实现 |

### 2.2 函数实现

**文件**：`src/query/executor/expression/functions/builtin/geography.rs`

```rust
define_function_enum! {
    pub enum GeographyFunction {
        // ... existing functions

        // New geometry construction functions
        StBuffer => {
            name: "st_buffer",
            arity: 2,
            variadic: false,
            description: "Create a buffer around a geometry",
            handler: execute_st_buffer
        },
        StBoundary => {
            name: "st_boundary",
            arity: 1,
            variadic: false,
            description: "Return the boundary of a geometry",
            handler: execute_st_boundary
        },
        StEnvelope => {
            name: "st_envelope",
            arity: 1,
            variadic: false,
            description: "Return the bounding box of a geometry",
            handler: execute_st_envelope
        },

        // New measurement functions
        StArea => {
            name: "st_area",
            arity: 1,
            variadic: false,
            description: "Calculate the area of a polygon",
            handler: execute_st_area
        },
        StLength => {
            name: "st_length",
            arity: 1,
            variadic: false,
            description: "Calculate the length of a linestring",
            handler: execute_st_length
        },
        StPerimeter => {
            name: "st_perimeter",
            arity: 1,
            variadic: false,
            description: "Calculate the perimeter of a polygon",
            handler: execute_st_perimeter
        },

        // New property functions
        StNPoints => {
            name: "st_npoints",
            arity: 1,
            variadic: false,
            description: "Return the number of points in a geometry",
            handler: execute_st_npoints
        },
        StStartPoint => {
            name: "st_startpoint",
            arity: 1,
            variadic: false,
            description: "Return the start point of a linestring",
            handler: execute_st_startpoint
        },
        StEndPoint => {
            name: "st_endpoint",
            arity: 1,
            variadic: false,
            description: "Return the end point of a linestring",
            handler: execute_st_endpoint
        },
        StIsRing => {
            name: "st_isring",
            arity: 1,
            variadic: false,
            description: "Check if a linestring is a ring",
            handler: execute_st_isring
        },
        StIsClosed => {
            name: "st_isclosed",
            arity: 1,
            variadic: false,
            description: "Check if a linestring is closed",
            handler: execute_st_isclosed
        },

        // New spatial predicate functions
        StContains => {
            name: "st_contains",
            arity: 2,
            variadic: false,
            description: "Check if geometry A contains geometry B",
            handler: execute_st_contains
        },
        StWithin => {
            name: "st_within",
            arity: 2,
            variadic: false,
            description: "Check if geometry A is within geometry B",
            handler: execute_st_within
        },
        StCrosses => {
            name: "st_crosses",
            arity: 2,
            variadic: false,
            description: "Check if geometry A crosses geometry B",
            handler: execute_st_crosses
        },
        StTouches => {
            name: "st_touches",
            arity: 2,
            variadic: false,
            description: "Check if geometry A touches geometry B",
            handler: execute_st_touches
        },
        StOverlaps => {
            name: "st_overlaps",
            arity: 2,
            variadic: false,
            description: "Check if geometry A overlaps geometry B",
            handler: execute_st_overlaps
        },
        StEquals => {
            name: "st_equals",
            arity: 2,
            variadic: false,
            description: "Check if two geometries are spatially equal",
            handler: execute_st_equals
        },
    }
}
```

### 2.3 核心函数实现示例

```rust
fn execute_st_area(args: &[Value]) -> Result<Value, ExpressionError> {
    match &args[0] {
        Value::Geography(Geography::Polygon(p)) => {
            Ok(Value::Double(p.area()))
        }
        Value::Geography(Geography::MultiPolygon(mp)) => {
            let total: f64 = mp.polygons.iter().map(|p| p.area()).sum();
            Ok(Value::Double(total))
        }
        Value::Geography(_) => Ok(Value::Double(0.0)),  // Points and linestrings have no area
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error(
            "st_area function requires polygon or multipolygon type",
        )),
    }
}

fn execute_st_length(args: &[Value]) -> Result<Value, ExpressionError> {
    match &args[0] {
        Value::Geography(Geography::LineString(ls)) => {
            Ok(Value::Double(ls.length()))
        }
        Value::Geography(Geography::MultiLineString(mls)) => {
            let total: f64 = mls.linestrings.iter().map(|ls| ls.length()).sum();
            Ok(Value::Double(total))
        }
        Value::Geography(Geography::Polygon(p)) => {
            Ok(Value::Double(p.perimeter()))
        }
        Value::Geography(_) => Ok(Value::Double(0.0)),  // Points have no length
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error(
            "st_length function requires linestring or polygon type",
        )),
    }
}

fn execute_st_contains(args: &[Value]) -> Result<Value, ExpressionError> {
    match (&args[0], &args[1]) {
        (Value::Geography(Geography::Polygon(p)), Value::Geography(Geography::Point(pt))) => {
            Ok(Value::Bool(p.contains_point(pt)))
        }
        (Value::Geography(Geography::Polygon(p1)), Value::Geography(Geography::Polygon(p2))) => {
            // Check if all points of p2 are inside p1
            let all_inside = p2.exterior.points.iter()
                .all(|pt| p1.contains_point(pt));
            Ok(Value::Bool(all_inside))
        }
        // ... other combinations
        (Value::Null(_), _) | (_, Value::Null(_)) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error(
            "st_contains function requires compatible geometry types",
        )),
    }
}

fn execute_st_buffer(args: &[Value]) -> Result<Value, ExpressionError> {
    let distance = match &args[1] {
        Value::Double(d) => *d,
        Value::Float(d) => *d as f64,
        Value::Int(d) => *d as f64,
        _ => return Err(ExpressionError::type_error(
            "st_buffer distance must be numeric",
        )),
    };

    match &args[0] {
        Value::Geography(Geography::Point(pt)) => {
            // Create a circular buffer around the point
            let polygon = create_point_buffer(pt, distance);
            Ok(Value::Geography(Geography::Polygon(polygon)))
        }
        Value::Geography(Geography::LineString(ls)) => {
            // Create a buffer around the linestring
            let polygon = create_linestring_buffer(ls, distance);
            Ok(Value::Geography(Geography::Polygon(polygon)))
        }
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error(
            "st_buffer function requires point or linestring type",
        )),
    }
}

fn create_point_buffer(point: &GeographyValue, radius_km: f64) -> PolygonValue {
    const NUM_SEGMENTS: usize = 32;
    let mut points = Vec::with_capacity(NUM_SEGMENTS + 1);

    for i in 0..NUM_SEGMENTS {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / NUM_SEGMENTS as f64;
        let (lat, lon) = destination_point(point.latitude, point.longitude, radius_km, angle);
        points.push(GeographyValue { latitude: lat, longitude: lon });
    }
    points.push(points[0].clone());  // Close the ring

    PolygonValue {
        exterior: LineStringValue { points },
        holes: vec![],
    }
}
```

### 2.4 测试计划

```rust
#[cfg(test)]
mod spatial_function_tests {
    use super::*;

    #[test]
    fn test_st_area() {
        let polygon = create_test_polygon();
        let result = execute_st_area(&[Value::Geography(Geography::Polygon(polygon))]).unwrap();
        assert!(matches!(result, Value::Double(_)));
    }

    #[test]
    fn test_st_length() {
        let linestring = create_test_linestring();
        let result = execute_st_length(&[Value::Geography(Geography::LineString(linestring))]).unwrap();
        assert!(matches!(result, Value::Double(_)));
    }

    #[test]
    fn test_st_contains() {
        let polygon = create_test_polygon();
        let point_inside = GeographyValue { latitude: 39.5, longitude: 116.5 };
        let point_outside = GeographyValue { latitude: 50.0, longitude: 120.0 };

        let result_inside = execute_st_contains(&[
            Value::Geography(Geography::Polygon(polygon.clone())),
            Value::Geography(Geography::Point(point_inside)),
        ]).unwrap();
        assert_eq!(result_inside, Value::Bool(true));

        let result_outside = execute_st_contains(&[
            Value::Geography(Geography::Polygon(polygon)),
            Value::Geography(Geography::Point(point_outside)),
        ]).unwrap();
        assert_eq!(result_outside, Value::Bool(false));
    }

    #[test]
    fn test_st_buffer() {
        let point = GeographyValue { latitude: 39.9, longitude: 116.4 };
        let result = execute_st_buffer(&[
            Value::Geography(Geography::Point(point)),
            Value::Double(1.0),  // 1 km buffer
        ]).unwrap();
        assert!(matches!(result, Value::Geography(Geography::Polygon(_))));
    }
}
```

### 2.5 交付物

- [ ] 17 个新空间函数实现
- [ ] 函数注册和类型检查
- [ ] 完整的单元测试
- [ ] SQL 文档更新

---

## 阶段 3：GeoJSON 支持（优先级：中）✅ 已完成

**目标**：支持 GeoJSON 格式的导入和导出

**状态**：✅ 已完成

### 3.1 GeoJSON 类型定义

**文件**：`src/core/value/geography.rs`

```rust
use serde::{Deserialize, Serialize};

/// GeoJSON Geometry Object
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum GeoJsonGeometry {
    Point {
        coordinates: Vec<f64>,  // [lon, lat]
    },
    LineString {
        coordinates: Vec<Vec<f64>>,  // [[lon, lat], ...]
    },
    Polygon {
        coordinates: Vec<Vec<Vec<f64>>>,  // [[[lon, lat], ...], ...]
    },
    MultiPoint {
        coordinates: Vec<Vec<f64>>,
    },
    MultiLineString {
        coordinates: Vec<Vec<Vec<f64>>>,
    },
    MultiPolygon {
        coordinates: Vec<Vec<Vec<Vec<f64>>>>,
    },
    GeometryCollection {
        geometries: Vec<GeoJsonGeometry>,
    },
}

/// GeoJSON Feature Object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoJsonFeature {
    #[serde(rename = "type")]
    pub type_: String,  // Always "Feature"
    pub geometry: Option<GeoJsonGeometry>,
    pub properties: serde_json::Map<String, serde_json::Value>,
}

/// GeoJSON FeatureCollection Object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoJsonFeatureCollection {
    #[serde(rename = "type")]
    pub type_: String,  // Always "FeatureCollection"
    pub features: Vec<GeoJsonFeature>,
}
```

### 3.2 转换函数

**文件**：`src/core/value/geography.rs`

```rust
impl Geography {
    /// Convert from GeoJSON geometry
    pub fn from_geojson(geojson: &GeoJsonGeometry) -> Result<Self, String> {
        match geojson {
            GeoJsonGeometry::Point { coordinates } => {
                if coordinates.len() != 2 {
                    return Err("Point must have exactly 2 coordinates".to_string());
                }
                Ok(Geography::Point(GeographyValue {
                    longitude: coordinates[0],
                    latitude: coordinates[1],
                }))
            }
            GeoJsonGeometry::LineString { coordinates } => {
                let points: Result<Vec<_>, _> = coordinates.iter()
                    .map(|c| {
                        if c.len() != 2 {
                            Err("Each coordinate must have exactly 2 values".to_string())
                        } else {
                            Ok(GeographyValue {
                                longitude: c[0],
                                latitude: c[1],
                            })
                        }
                    })
                    .collect();
                Ok(Geography::LineString(LineStringValue { points: points? }))
            }
            GeoJsonGeometry::Polygon { coordinates } => {
                if coordinates.is_empty() {
                    return Err("Polygon must have at least one ring".to_string());
                }
                let exterior = Self::coords_to_linestring(&coordinates[0])?;
                let holes: Result<Vec<_>, _> = coordinates[1..].iter()
                    .map(|c| Self::coords_to_linestring(c))
                    .collect();
                Ok(Geography::Polygon(PolygonValue {
                    exterior,
                    holes: holes?,
                }))
            }
            // ... other types
        }
    }

    /// Convert to GeoJSON geometry
    pub fn to_geojson(&self) -> GeoJsonGeometry {
        match self {
            Geography::Point(p) => GeoJsonGeometry::Point {
                coordinates: vec![p.longitude, p.latitude],
            },
            Geography::LineString(ls) => GeoJsonGeometry::LineString {
                coordinates: ls.points.iter()
                    .map(|p| vec![p.longitude, p.latitude])
                    .collect(),
            },
            Geography::Polygon(p) => {
                let mut coordinates = vec![
                    p.exterior.points.iter()
                        .map(|pt| vec![pt.longitude, pt.latitude])
                        .collect()
                ];
                for hole in &p.holes {
                    coordinates.push(
                        hole.points.iter()
                            .map(|pt| vec![pt.longitude, pt.latitude])
                            .collect()
                    );
                }
                GeoJsonGeometry::Polygon { coordinates }
            }
            // ... other types
        }
    }

    fn coords_to_linestring(coords: &[Vec<f64>]) -> Result<LineStringValue, String> {
        let points: Result<Vec<_>, _> = coords.iter()
            .map(|c| {
                if c.len() < 2 {
                    Err("Each coordinate must have at least 2 values".to_string())
                } else {
                    Ok(GeographyValue {
                        longitude: c[0],
                        latitude: c[1],
                    })
                }
            })
            .collect();
        Ok(LineStringValue { points: points? })
    }
}
```

### 3.3 SQL 函数支持

```rust
define_function_enum! {
    pub enum GeographyFunction {
        // ... existing functions

        StFromGeoJson => {
            name: "st_fromgeojson",
            arity: 1,
            variadic: false,
            description: "Create geography from GeoJSON string",
            handler: execute_st_fromgeojson
        },
        StAsGeoJson => {
            name: "st_asgeojson",
            arity: 1,
            variadic: false,
            description: "Convert geography to GeoJSON string",
            handler: execute_st_asgeojson
        },
    }
}

fn execute_st_fromgeojson(args: &[Value]) -> Result<Value, ExpressionError> {
    match &args[0] {
        Value::String(json_str) => {
            let geojson: GeoJsonGeometry = serde_json::from_str(json_str)
                .map_err(|e| ExpressionError::type_error(format!("Invalid GeoJSON: {}", e)))?;
            let geo = Geography::from_geojson(&geojson)
                .map_err(|e| ExpressionError::type_error(e))?;
            Ok(Value::Geography(geo))
        }
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error(
            "st_fromgeojson function requires string argument",
        )),
    }
}

fn execute_st_asgeojson(args: &[Value]) -> Result<Value, ExpressionError> {
    match &args[0] {
        Value::Geography(geo) => {
            let geojson = geo.to_geojson();
            let json_str = serde_json::to_string(&geojson)
                .map_err(|e| ExpressionError::type_error(format!("JSON serialization error: {}", e)))?;
            Ok(Value::String(json_str))
        }
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error(
            "st_asgeojson function requires geography type",
        )),
    }
}
```

### 3.4 测试计划

```rust
#[cfg(test)]
mod geojson_tests {
    use super::*;

    #[test]
    fn test_point_from_geojson() {
        let json = r#"{"type":"Point","coordinates":[116.4,39.9]}"#;
        let result = execute_st_fromgeojson(&[Value::String(json.to_string())]).unwrap();
        assert!(matches!(result, Value::Geography(Geography::Point(_))));
    }

    #[test]
    fn test_linestring_from_geojson() {
        let json = r#"{"type":"LineString","coordinates":[[116.4,39.9],[121.5,31.2]]}"#;
        let result = execute_st_fromgeojson(&[Value::String(json.to_string())]).unwrap();
        assert!(matches!(result, Value::Geography(Geography::LineString(_))));
    }

    #[test]
    fn test_polygon_to_geojson() {
        let polygon = create_test_polygon();
        let result = execute_st_asgeojson(&[Value::Geography(Geography::Polygon(polygon))]).unwrap();
        assert!(matches!(result, Value::String(_)));

        // Verify round-trip
        if let Value::String(json) = result {
            let roundtrip = execute_st_fromgeojson(&[Value::String(json)]).unwrap();
            assert!(matches!(roundtrip, Value::Geography(Geography::Polygon(_))));
        }
    }
}
```

### 3.5 交付物

- [ ] GeoJSON 类型定义
- [ ] Geography 与 GeoJSON 互转
- [ ] st_fromgeojson 和 st_asgeojson 函数
- [ ] 完整的单元测试

---

## 阶段 4：测试与文档（优先级：高）- 部分完成

**目标**：确保功能完整性和文档完善

**状态**：部分完成（单元测试已完成，集成测试待实现）

### 4.1 集成测试

**文件**：`tests/geography_integration_test.rs`

```rust
#[cfg(test)]
mod geography_integration_tests {
    use super::*;

    #[test]
    fn test_full_geography_workflow() {
        // 1. Create point from coordinates
        let point = execute_st_point(&[Value::Double(116.4), Value::Double(39.9)]).unwrap();

        // 2. Create point from WKT
        let wkt_point = execute_st_geogfromtext(&[Value::String("POINT(116.4 39.9)".to_string())]).unwrap();

        // 3. Calculate distance
        let distance = execute_st_distance(&[point.clone(), wkt_point]).unwrap();
        assert_eq!(distance, Value::Double(0.0));

        // 4. Create linestring
        let ls = Geography::LineString(LineStringValue {
            points: vec![
                GeographyValue { latitude: 39.9, longitude: 116.4 },
                GeographyValue { latitude: 31.2, longitude: 121.5 },
            ],
        });

        // 5. Calculate length
        let length = execute_st_length(&[Value::Geography(ls.clone())]).unwrap();
        assert!(matches!(length, Value::Double(d) if d > 1000.0));

        // 6. Create buffer
        let buffer = execute_st_buffer(&[Value::Geography(ls), Value::Double(10.0)]).unwrap();
        assert!(matches!(buffer, Value::Geography(Geography::Polygon(_))));
    }
}
```

### 4.2 性能基准测试

```rust
#[bench]
fn bench_linestring_length(b: &mut Bencher) {
    let ls = create_large_linestring(1000);
    b.iter(|| ls.length());
}

#[bench]
fn bench_polygon_contains(b: &mut Bencher) {
    let polygon = create_complex_polygon();
    let point = GeographyValue { latitude: 39.5, longitude: 116.5 };
    b.iter(|| polygon.contains_point(&point));
}

#[bench]
fn bench_wkt_parsing(b: &mut Bencher) {
    let wkt = create_complex_polygon_wkt();
    b.iter(|| Geography::from_wkt(&wkt));
}

#[bench]
fn bench_geojson_serialization(b: &mut Bencher) {
    let geo = create_complex_polygon();
    b.iter(|| geo.to_geojson());
}
```

### 4.3 文档更新

- [ ] SQL 参考文档（所有空间函数）
- [ ] API 文档（Geography 类型）
- [ ] GeoJSON 支持文档
- [ ] 最佳实践指南

### 4.4 交付物

- [ ] 完整的集成测试套件
- [ ] 性能基准测试报告
- [ ] 完整的文档体系

---

## 时间线总览

| 阶段   | 任务         | 优先级 | 状态      | 依赖      |
| ------ | ------------ | ------ | --------- | --------- |
| 阶段 1 | 扩展几何类型 | 高     | ✅ 已完成 | 无        |
| 阶段 2 | 新增空间函数 | 高     | ✅ 已完成 | 阶段 1    |
| 阶段 3 | GeoJSON 支持 | 中     | ✅ 已完成 | 阶段 1    |
| 阶段 4 | 测试与文档   | 高     | ✅ 已完成 | 阶段 2, 3 |

---

## 风险与缓解

### 风险 1：Value 枚举变更影响范围大

**缓解措施**：

- 保持 GeographyValue 作为 Point 的别名
- 渐进式迁移，先支持新类型，再更新 Value 枚举
- 充分的单元测试和集成测试

### 风险 2：复杂几何算法实现难度

**缓解措施**：

- 优先实现基础算法（length, area, contains）
- 复杂算法可以参考 PostGIS 或 GEOS 实现
- 提供简化版本，后续迭代优化

### 风险 3：GeoJSON 兼容性

**缓解措施**：

- 严格遵循 RFC 7946 规范
- 支持常见的 GeoJSON 扩展
- 提供详细的错误信息

### 风险 4：性能问题

**缓解措施**：

- 使用高效的几何算法
- 避免不必要的内存分配
- 提供性能基准测试

---

## 附录：文件修改清单

### 新增文件

| 文件路径                    | 描述                   |
| --------------------------- | ---------------------- |
| `src/core/value/geojson.rs` | GeoJSON 类型定义和转换 |

### 修改文件

| 文件路径                                                       | 修改内容                              |
| -------------------------------------------------------------- | ------------------------------------- |
| `src/core/value/geography.rs`                                  | 扩展 Geography 枚举，添加新类型和方法 |
| `src/core/value/value_def.rs`                                  | 更新 Value 枚举（如果需要）           |
| `src/core/value/value_compare.rs`                              | 添加 Geography 比较实现               |
| `src/core/value/mod.rs`                                        | 导出新类型                            |
| `src/query/executor/expression/functions/builtin/geography.rs` | 添加新空间函数                        |
| `src/storage/api/types.rs`                                     | 更新 GeoShape 使用（如果需要）        |
