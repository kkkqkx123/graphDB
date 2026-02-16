# 内置函数实现计划

## 1. 概述

本文档基于对 Nebula-Graph 函数实现的分析，整理出当前项目缺失的内置函数，并提供详细的实现方案。

### 1.1 当前状态
- **已有函数**：约 60+ 个（数学、字符串、正则、类型转换、日期时间、图相关、容器操作、路径、实用函数）
- **缺失函数**：约 35+ 个（扩展数学、字符串、日期时间、图相关、地理空间等）

### 1.2 实现原则
1. **优先核心功能**：图查询相关函数和常用函数优先实现
2. **参考 Nebula 实现**：借鉴成熟的实现逻辑
3. **保持一致性**：与现有函数注册机制保持一致
4. **类型安全**：充分利用 Rust 的类型系统
5. **UDF 说明**：`udf_is_in` 等 UDF 相关函数等待 UDF 模块实现后再引入

---

## 2. 待实现函数清单

### 2.1 数学函数（math.rs 扩展）

| 序号 | 函数名 | 功能描述 | 参数 | 返回值 | 优先级 |
|------|--------|----------|------|--------|--------|
| 1 | `e` | 自然常数 e | 无 | Float | 低 |
| 2 | `pi` | 圆周率 π | 无 | Float | 低 |
| 3 | `exp2` | 2 的幂 | INT/FLOAT | Float | 低 |
| 4 | `log2` | 以 2 为底的对数 | INT/FLOAT | Float | 低 |
| 5 | `radians` | 角度转弧度 | INT/FLOAT | Float | 低 |
| 6 | `sign` | 返回数值符号 (-1, 0, 1) | INT/FLOAT | Int | 中 |
| 7 | `rand` | 0-1 随机浮点数 | 无 | Float | 中 |
| 8 | `rand32` | 32 位随机整数 | 无/INT/INT,INT | Int | 中 |
| 9 | `rand64` | 64 位随机整数 | 无/INT/INT,INT | Int | 中 |

### 2.2 字符串函数（string.rs 扩展）

| 序号 | 函数名 | 功能描述 | 参数 | 返回值 | 优先级 |
|------|--------|----------|------|--------|--------|
| 10 | `strcasecmp` | 不区分大小写比较字符串 | STRING, STRING | Int | 低 |
| 11 | `lpad` | 左侧填充字符串 | STRING, INT, STRING | String | 中 |
| 12 | `rpad` | 右侧填充字符串 | STRING, INT, STRING | String | 中 |
| 13 | `concat_ws` | 带分隔符连接字符串 | STRING, STRING... | String | 中 |

### 2.3 类型转换函数（conversion.rs 扩展）

| 序号 | 函数名 | 功能描述 | 参数 | 返回值 | 优先级 |
|------|--------|----------|------|--------|--------|
| 14 | `toset` | 列表转集合 | LIST | Set | 中 |

### 2.4 日期时间函数（datetime.rs 扩展）

| 序号 | 函数名 | 功能描述 | 参数 | 返回值 | 优先级 |
|------|--------|----------|------|--------|--------|
| 15 | `time` | 创建/获取时间 | 无/STRING/MAP | Time | 高 |
| 16 | `datetime` | 创建/获取日期时间 | 无/STRING/MAP/INT | DateTime | 高 |
| 17 | `timestamp` | 获取时间戳 | 无/STRING/INT/DATETIME | Int | 高 |
| 18 | `duration` | 创建持续时间 | STRING/MAP | Duration | 中 |
| 19 | `extract` | 提取日期时间组件 | STRING, STRING | List | 中 |

### 2.5 图相关函数（graph.rs 扩展）

| 序号 | 函数名 | 功能描述 | 参数 | 返回值 | 优先级 |
|------|--------|----------|------|--------|--------|
| 20 | `typeid` | 获取边类型 ID | EDGE | Int | 低 |
| 21 | `startnode` | 获取边的起始节点 | EDGE/PATH | Vertex | 高 |
| 22 | `endnode` | 获取边的结束节点 | EDGE/PATH | Vertex | 高 |
| 23 | `none_direct_src` | 获取无方向边的源 | EDGE/VERTEX/LIST | Any | 低 |
| 24 | `none_direct_dst` | 获取无方向边的目标 | EDGE/VERTEX/LIST | Any | 低 |

### 2.6 容器/路径函数（container.rs/path.rs 扩展）

| 序号 | 函数名 | 功能描述 | 参数 | 返回值 | 优先级 |
|------|--------|----------|------|--------|--------|
| 25 | `reverse` (列表) | 反转列表 | LIST | List | 中 |
| 26 | `hassameedgeinpath` | 检查路径是否有重复边 | PATH | Bool | 低 |
| 27 | `hassamevertexinpath` | 检查路径是否有重复顶点 | PATH | Bool | 低 |
| 28 | `reversepath` | 反转路径 | PATH | Path | 低 |

### 2.7 JSON 函数（新增 json.rs）

| 序号 | 函数名 | 功能描述 | 参数 | 返回值 | 优先级 |
|------|--------|----------|------|--------|--------|
| 29 | `json_extract` | 提取 JSON 数据 | STRING | Map/Null | 中 |

### 2.8 地理空间函数（新增 geo.rs）

| 序号 | 函数名 | 功能描述 | 参数 | 返回值 | 优先级 |
|------|--------|----------|------|--------|--------|
| 30 | `st_point` | 创建地理点 | FLOAT, FLOAT | Geography | 低 |
| 31 | `st_geogfromtext` | 从 WKT 解析地理数据 | STRING | Geography | 低 |
| 32 | `st_astext` | 地理数据转 WKT | GEOGRAPHY | String | 低 |
| 33 | `st_centroid` | 计算质心 | GEOGRAPHY | Geography | 低 |
| 34 | `st_isvalid` | 检查地理数据有效性 | GEOGRAPHY | Bool | 低 |
| 35 | `st_intersects` | 检查两个地理对象是否相交 | GEOGRAPHY, GEOGRAPHY | Bool | 低 |
| 36 | `st_covers` | 检查覆盖关系 | GEOGRAPHY, GEOGRAPHY | Bool | 低 |
| 37 | `st_coveredby` | 检查被覆盖关系 | GEOGRAPHY, GEOGRAPHY | Bool | 低 |
| 38 | `st_dwithin` | 检查是否在指定距离内 | GEOGRAPHY, GEOGRAPHY, FLOAT/INT | Bool | 低 |
| 39 | `st_distance` | 计算地理距离 | GEOGRAPHY, GEOGRAPHY | Float | 低 |
| 40 | `s2_cellidfrompoint` | S2 单元格 ID | GEOGRAPHY | Int | 低 |
| 41 | `s2_coveringcellids` | S2 覆盖单元格 | GEOGRAPHY | List | 低 |

### 2.9 其他函数

| 序号 | 函数名 | 功能描述 | 参数 | 返回值 | 优先级 |
|------|--------|----------|------|--------|--------|
| 42 | `is_edge` | 检查是否为边类型 | EDGE | Bool | 低 |
| 43 | `cos_similarity` | 计算余弦相似度 | 数值列表... | Float | 低 |

---

## 3. 详细实现方案

### 3.1 数学函数扩展

#### 3.1.1 数学常数 e 和 pi

```rust
// 在 math.rs 中添加
fn register_e(registry: &mut FunctionRegistry) {
    registry.register(
        "e",
        FunctionSignature::new(
            "e",
            vec![],
            ValueType::Float,
            0, 0, true, "自然常数 e",
        ),
        |_args| Ok(Value::Float(std::f64::consts::E)),
    );
}

fn register_pi(registry: &mut FunctionRegistry) {
    registry.register(
        "pi",
        FunctionSignature::new(
            "pi",
            vec![],
            ValueType::Float,
            0, 0, true, "圆周率 π",
        ),
        |_args| Ok(Value::Float(std::f64::consts::PI)),
    );
}
```

#### 3.1.2 exp2 和 log2

```rust
fn register_exp2(registry: &mut FunctionRegistry) {
    for (type_in, converter) in [(ValueType::Int, |i: i64| i as f64), (ValueType::Float, |f: f64| f)] {
        registry.register(
            "exp2",
            FunctionSignature::new("exp2", vec![type_in], ValueType::Float, 1, 1, true, "2的幂"),
            move |args| {
                match &args[0] {
                    Value::Int(i) => Ok(Value::Float(2.0_f64.powf(*i as f64))),
                    Value::Float(f) => Ok(Value::Float(2.0_f64.powf(*f))),
                    Value::Null(_) => Ok(Value::Null(NullType::Null)),
                    _ => Err(ExpressionError::type_error("exp2函数需要数值类型")),
                }
            },
        );
    }
}

fn register_log2(registry: &mut FunctionRegistry) {
    registry.register(
        "log2",
        FunctionSignature::new("log2", vec![ValueType::Int], ValueType::Float, 1, 1, true, "以2为底的对数"),
        |args| {
            match &args[0] {
                Value::Int(i) if *i > 0 => Ok(Value::Float((*i as f64).log2())),
                Value::Int(_) => Err(ExpressionError::invalid_operation("log2 of non-positive number")),
                Value::Float(f) if *f > 0.0 => Ok(Value::Float(f.log2())),
                Value::Float(_) => Err(ExpressionError::invalid_operation("log2 of non-positive number")),
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("log2函数需要数值类型")),
            }
        },
    );
}
```

#### 3.1.3 sign 函数

```rust
fn register_sign(registry: &mut FunctionRegistry) {
    registry.register(
        "sign",
        FunctionSignature::new("sign", vec![ValueType::Int], ValueType::Int, 1, 1, true, "返回数值符号"),
        |args| {
            match &args[0] {
                Value::Int(i) => Ok(Value::Int(if *i > 0 { 1 } else if *i < 0 { -1 } else { 0 })),
                Value::Float(f) => Ok(Value::Int(if *f > 0.0 { 1 } else if *f < 0.0 { -1 } else { 0 })),
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("sign函数需要数值类型")),
            }
        },
    );
}
```

#### 3.1.4 随机数函数

```rust
use rand::{thread_rng, Rng};

fn register_rand(registry: &mut FunctionRegistry) {
    // rand() - 0-1 随机浮点数
    registry.register(
        "rand",
        FunctionSignature::new("rand", vec![], ValueType::Float, 0, 0, false, "0-1随机浮点数"),
        |_args| {
            let mut rng = thread_rng();
            Ok(Value::Float(rng.gen::<f64>()))
        },
    );
}

fn register_rand32(registry: &mut FunctionRegistry) {
    // rand32(), rand32(max), rand32(min, max)
    registry.register(
        "rand32",
        FunctionSignature::new("rand32", vec![], ValueType::Int, 0, 2, false, "32位随机整数"),
        |args| {
            let mut rng = thread_rng();
            match args.len() {
                0 => Ok(Value::Int(rng.gen::<i32>() as i64)),
                1 => match &args[0] {
                    Value::Int(max) if *max > 0 => Ok(Value::Int(rng.gen_range(0..*max) as i64)),
                    Value::Null(_) => Ok(Value::Null(NullType::Null)),
                    _ => Err(ExpressionError::type_error("rand32参数必须是正整数")),
                },
                2 => match (&args[0], &args[1]) {
                    (Value::Int(min), Value::Int(max)) if *min < *max => {
                        Ok(Value::Int(rng.gen_range(*min..*max)))
                    }
                    (Value::Null(_), _) | (_, Value::Null(_)) => Ok(Value::Null(NullType::Null)),
                    _ => Err(ExpressionError::type_error("rand32参数范围无效")),
                },
                _ => unreachable!(),
            }
        },
    );
}

fn register_rand64(registry: &mut FunctionRegistry) {
    // rand64(), rand64(max), rand64(min, max)
    registry.register(
        "rand64",
        FunctionSignature::new("rand64", vec![], ValueType::Int, 0, 2, false, "64位随机整数"),
        |args| {
            let mut rng = thread_rng();
            match args.len() {
                0 => Ok(Value::Int(rng.gen::<i64>())),
                1 => match &args[0] {
                    Value::Int(max) if *max > 0 => Ok(Value::Int(rng.gen_range(0..*max))),
                    Value::Null(_) => Ok(Value::Null(NullType::Null)),
                    _ => Err(ExpressionError::type_error("rand64参数必须是正整数")),
                },
                2 => match (&args[0], &args[1]) {
                    (Value::Int(min), Value::Int(max)) if *min < *max => {
                        Ok(Value::Int(rng.gen_range(*min..*max)))
                    }
                    (Value::Null(_), _) | (_, Value::Null(_)) => Ok(Value::Null(NullType::Null)),
                    _ => Err(ExpressionError::type_error("rand64参数范围无效")),
                },
                _ => unreachable!(),
            }
        },
    );
}
```

### 3.2 字符串函数扩展

#### 3.2.1 strcasecmp

```rust
fn register_strcasecmp(registry: &mut FunctionRegistry) {
    registry.register(
        "strcasecmp",
        FunctionSignature::new(
            "strcasecmp",
            vec![ValueType::String, ValueType::String],
            ValueType::Int,
            2, 2, true, "不区分大小写比较字符串",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::String(s1), Value::String(s2)) => {
                    let cmp = s1.to_lowercase().cmp(&s2.to_lowercase());
                    Ok(Value::Int(match cmp {
                        std::cmp::Ordering::Less => -1,
                        std::cmp::Ordering::Equal => 0,
                        std::cmp::Ordering::Greater => 1,
                    }))
                }
                (Value::Null(_), _) | (_, Value::Null(_)) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("strcasecmp函数需要字符串类型")),
            }
        },
    );
}
```

#### 3.2.2 lpad 和 rpad

```rust
fn register_lpad(registry: &mut FunctionRegistry) {
    registry.register(
        "lpad",
        FunctionSignature::new(
            "lpad",
            vec![ValueType::String, ValueType::Int, ValueType::String],
            ValueType::String,
            3, 3, true, "左侧填充字符串",
        ),
        |args| {
            match (&args[0], &args[1], &args[2]) {
                (Value::String(s), Value::Int(len), Value::String(pad)) => {
                    let target_len = *len as usize;
                    if target_len <= s.len() {
                        Ok(Value::String(s[..target_len].to_string()))
                    } else {
                        let pad_len = target_len - s.len();
                        let mut result = String::new();
                        while result.len() < pad_len {
                            result.push_str(pad);
                        }
                        result.truncate(pad_len);
                        result.push_str(s);
                        Ok(Value::String(result))
                    }
                }
                (Value::Null(_), _, _) | (_, Value::Null(_), _) | (_, _, Value::Null(_)) => {
                    Ok(Value::Null(NullType::Null))
                }
                _ => Err(ExpressionError::type_error("lpad函数参数类型错误")),
            }
        },
    );
}

fn register_rpad(registry: &mut FunctionRegistry) {
    registry.register(
        "rpad",
        FunctionSignature::new(
            "rpad",
            vec![ValueType::String, ValueType::Int, ValueType::String],
            ValueType::String,
            3, 3, true, "右侧填充字符串",
        ),
        |args| {
            match (&args[0], &args[1], &args[2]) {
                (Value::String(s), Value::Int(len), Value::String(pad)) => {
                    let target_len = *len as usize;
                    if target_len <= s.len() {
                        Ok(Value::String(s[..target_len].to_string()))
                    } else {
                        let pad_len = target_len - s.len();
                        let mut result = s.clone();
                        while result.len() < target_len {
                            result.push_str(pad);
                        }
                        result.truncate(target_len);
                        Ok(Value::String(result))
                    }
                }
                (Value::Null(_), _, _) | (_, Value::Null(_), _) | (_, _, Value::Null(_)) => {
                    Ok(Value::Null(NullType::Null))
                }
                _ => Err(ExpressionError::type_error("rpad函数参数类型错误")),
            }
        },
    );
}
```

#### 3.2.3 concat_ws

```rust
fn register_concat_ws(registry: &mut FunctionRegistry) {
    registry.register(
        "concat_ws",
        FunctionSignature::new(
            "concat_ws",
            vec![ValueType::String],
            ValueType::String,
            2, usize::MAX, true, "带分隔符连接字符串",
        ),
        |args| {
            match &args[0] {
                Value::String(sep) => {
                    let parts: Vec<String> = args[1..]
                        .iter()
                        .filter_map(|arg| match arg {
                            Value::String(s) => Some(s.clone()),
                            Value::Int(i) => Some(i.to_string()),
                            Value::Float(f) => Some(f.to_string()),
                            Value::Bool(b) => Some(b.to_string()),
                            _ => None,
                        })
                        .collect();
                    Ok(Value::String(parts.join(sep)))
                }
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("concat_ws第一个参数必须是字符串")),
            }
        },
    );
}
```

### 3.3 类型转换函数扩展

#### 3.3.1 toset

```rust
use std::collections::HashSet;

fn register_toset(registry: &mut FunctionRegistry) {
    registry.register(
        "toset",
        FunctionSignature::new(
            "toset",
            vec![ValueType::List],
            ValueType::Set,
            1, 1, true, "列表转集合",
        ),
        |args| {
            match &args[0] {
                Value::List(list) => {
                    let set: HashSet<Value> = list.values.iter().cloned().collect();
                    Ok(Value::Set(set))
                }
                Value::Set(set) => Ok(Value::Set(set.clone())),
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("toset函数需要列表类型")),
            }
        },
    );
}
```

### 3.4 日期时间函数扩展

#### 3.4.1 time 函数

```rust
use chrono::{NaiveTime, Timelike};

fn register_time(registry: &mut FunctionRegistry) {
    // time() - 当前时间
    registry.register(
        "time",
        FunctionSignature::new("time", vec![], ValueType::Time, 0, 0, false, "获取当前时间"),
        |_args| {
            let now = chrono::Local::now();
            Ok(Value::Time(TimeValue {
                hour: now.hour() as u32,
                minute: now.minute() as u32,
                sec: now.second() as u32,
                microsec: 0,
            }))
        },
    );

    // time(string) - 从字符串解析
    registry.register(
        "time",
        FunctionSignature::new("time", vec![ValueType::String], ValueType::Time, 1, 1, true, "从字符串创建时间"),
        |args| {
            match &args[0] {
                Value::String(s) => {
                    let time = NaiveTime::parse_from_str(s, "%H:%M:%S")
                        .map_err(|_| ExpressionError::type_error("无法解析时间字符串"))?;
                    Ok(Value::Time(TimeValue {
                        hour: time.hour(),
                        minute: time.minute(),
                        sec: time.second(),
                        microsec: 0,
                    }))
                }
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("time函数需要字符串类型")),
            }
        },
    );
}
```

#### 3.4.2 datetime 函数

```rust
use chrono::{NaiveDateTime, Datelike};

fn register_datetime(registry: &mut FunctionRegistry) {
    // datetime() - 当前日期时间
    registry.register(
        "datetime",
        FunctionSignature::new("datetime", vec![], ValueType::DateTime, 0, 0, false, "获取当前日期时间"),
        |_args| {
            let now = chrono::Local::now();
            Ok(Value::DateTime(DateTimeValue {
                year: now.year(),
                month: now.month(),
                day: now.day(),
                hour: now.hour(),
                minute: now.minute(),
                sec: now.second(),
                microsec: 0,
            }))
        },
    );

    // datetime(string) - 从字符串解析
    registry.register(
        "datetime",
        FunctionSignature::new("datetime", vec![ValueType::String], ValueType::DateTime, 1, 1, true, "从字符串创建日期时间"),
        |args| {
            match &args[0] {
                Value::String(s) => {
                    let dt = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                        .map_err(|_| ExpressionError::type_error("无法解析日期时间字符串"))?;
                    Ok(Value::DateTime(DateTimeValue {
                        year: dt.year(),
                        month: dt.month(),
                        day: dt.day(),
                        hour: dt.hour(),
                        minute: dt.minute(),
                        sec: dt.second(),
                        microsec: 0,
                    }))
                }
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("datetime函数需要字符串类型")),
            }
        },
    );

    // datetime(timestamp) - 从时间戳转换
    registry.register(
        "datetime",
        FunctionSignature::new("datetime", vec![ValueType::Int], ValueType::DateTime, 1, 1, true, "从时间戳创建日期时间"),
        |args| {
            match &args[0] {
                Value::Int(ts) => {
                    let dt = chrono::DateTime::from_timestamp(*ts, 0)
                        .ok_or_else(|| ExpressionError::type_error("无效的时间戳"))?;
                    Ok(Value::DateTime(DateTimeValue {
                        year: dt.year(),
                        month: dt.month(),
                        day: dt.day(),
                        hour: dt.hour(),
                        minute: dt.minute(),
                        sec: dt.second(),
                        microsec: 0,
                    }))
                }
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("datetime函数需要整数类型")),
            }
        },
    );
}
```

#### 3.4.3 timestamp 函数

```rust
fn register_timestamp(registry: &mut FunctionRegistry) {
    // timestamp() - 当前时间戳
    registry.register(
        "timestamp",
        FunctionSignature::new("timestamp", vec![], ValueType::Int, 0, 0, false, "获取当前时间戳"),
        |_args| {
            let now = std::time::SystemTime::now();
            let since_epoch = now.duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards");
            Ok(Value::Int(since_epoch.as_secs() as i64))
        },
    );

    // timestamp(string) - 从字符串解析
    registry.register(
        "timestamp",
        FunctionSignature::new("timestamp", vec![ValueType::String], ValueType::Int, 1, 1, true, "从字符串获取时间戳"),
        |args| {
            match &args[0] {
                Value::String(s) => {
                    let dt = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                        .map_err(|_| ExpressionError::type_error("无法解析日期时间字符串"))?;
                    Ok(Value::Int(dt.timestamp()))
                }
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("timestamp函数需要字符串类型")),
            }
        },
    );
}
```

### 3.5 图相关函数扩展

#### 3.5.1 startnode 和 endnode

```rust
fn register_startnode(registry: &mut FunctionRegistry) {
    registry.register(
        "startnode",
        FunctionSignature::new("startnode", vec![ValueType::Edge], ValueType::Vertex, 1, 1, true, "获取边的起始节点"),
        |args| {
            match &args[0] {
                Value::Edge(e) => {
                    let vertex = Vertex::new((*e.src).clone(), vec![]);
                    Ok(Value::Vertex(Box::new(vertex)))
                }
                Value::Path(p) => Ok(Value::Vertex(Box::new((*p.src).clone()))),
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("startnode函数需要边或路径类型")),
            }
        },
    );
}

fn register_endnode(registry: &mut FunctionRegistry) {
    registry.register(
        "endnode",
        FunctionSignature::new("endnode", vec![ValueType::Edge], ValueType::Vertex, 1, 1, true, "获取边的结束节点"),
        |args| {
            match &args[0] {
                Value::Edge(e) => {
                    let vertex = Vertex::new((*e.dst).clone(), vec![]);
                    Ok(Value::Vertex(Box::new(vertex)))
                }
                Value::Path(p) => {
                    if let Some(last_step) = p.steps.last() {
                        Ok(Value::Vertex(Box::new((*last_step.dst).clone())))
                    } else {
                        Ok(Value::Vertex(Box::new((*p.src).clone())))
                    }
                }
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("endnode函数需要边或路径类型")),
            }
        },
    );
}
```

### 3.6 容器函数扩展

#### 3.6.1 reverse 列表版本

```rust
fn register_reverse_list(registry: &mut FunctionRegistry) {
    registry.register(
        "reverse",
        FunctionSignature::new("reverse", vec![ValueType::List], ValueType::List, 1, 1, true, "反转列表"),
        |args| {
            match &args[0] {
                Value::List(list) => {
                    let mut reversed = list.values.clone();
                    reversed.reverse();
                    Ok(Value::List(List { values: reversed }))
                }
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("reverse(列表)函数需要列表类型")),
            }
        },
    );
}
```

### 3.7 JSON 函数（新增模块）

#### 3.7.1 json_extract

```rust
// 在 expression/functions/builtin/ 下新建 json.rs
use serde_json::Value as JsonValue;

fn register_json_extract(registry: &mut FunctionRegistry) {
    registry.register(
        "json_extract",
        FunctionSignature::new("json_extract", vec![ValueType::String], ValueType::Map, 1, 1, true, "提取JSON数据"),
        |args| {
            match &args[0] {
                Value::String(s) => {
                    let json: JsonValue = serde_json::from_str(s)
                        .map_err(|_| ExpressionError::type_error("无效的JSON字符串"))?;
                    
                    fn json_to_value(json: JsonValue) -> Value {
                        match json {
                            JsonValue::Null => Value::Null(NullType::Null),
                            JsonValue::Bool(b) => Value::Bool(b),
                            JsonValue::Number(n) => {
                                if let Some(i) = n.as_i64() {
                                    Value::Int(i)
                                } else {
                                    Value::Float(n.as_f64().unwrap_or(0.0))
                                }
                            }
                            JsonValue::String(s) => Value::String(s),
                            JsonValue::Array(arr) => {
                                Value::List(List {
                                    values: arr.into_iter().map(json_to_value).collect(),
                                })
                            }
                            JsonValue::Object(obj) => {
                                let map: HashMap<String, Value> = obj
                                    .into_iter()
                                    .map(|(k, v)| (k, json_to_value(v)))
                                    .collect();
                                Value::Map(map)
                            }
                        }
                    }
                    
                    Ok(json_to_value(json))
                }
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("json_extract函数需要字符串类型")),
            }
        },
    );
}
```

---

## 4. 实现步骤

### 4.1 文件修改清单

| 文件 | 修改内容 |
|------|----------|
| `src/expression/functions/builtin/math.rs` | 添加 e, pi, exp2, log2, sign, rand, rand32, rand64 |
| `src/expression/functions/builtin/string.rs` | 添加 strcasecmp, lpad, rpad, concat_ws |
| `src/expression/functions/builtin/conversion.rs` | 添加 toset |
| `src/expression/functions/builtin/datetime.rs` | 添加 time, datetime, timestamp, duration, extract |
| `src/expression/functions/builtin/graph.rs` | 添加 typeid, startnode, endnode, none_direct_src, none_direct_dst |
| `src/expression/functions/builtin/container.rs` | 添加 reverse(列表) |
| `src/expression/functions/builtin/path.rs` | 添加 hassameedgeinpath, hassamevertexinpath, reversepath |
| `src/expression/functions/builtin/json.rs` | 新建文件，添加 json_extract |
| `src/expression/functions/builtin/geo.rs` | 新建文件，添加地理空间函数 |
| `src/expression/functions/builtin/mod.rs` | 添加 json 和 geo 模块导出 |

### 4.2 实现顺序建议

| 阶段 | 函数类别 | 预计工作量 | 依赖 |
|------|----------|-----------|------|
| Phase 1 | 日期时间函数（time, datetime, timestamp） | 2-3 天 | chrono crate |
| Phase 2 | 图相关函数（startnode, endnode） | 1-2 天 | 无 |
| Phase 3 | 数学函数（sign, rand系列） | 2 天 | rand crate |
| Phase 4 | 字符串函数（lpad, rpad, concat_ws） | 1-2 天 | 无 |
| Phase 5 | 类型转换（toset）、容器（reverse列表） | 1 天 | 无 |
| Phase 6 | JSON 函数 | 1-2 天 | serde_json crate |
| Phase 7 | 其他（数学常数、strcasecmp等） | 1-2 天 | 无 |
| Phase 8 | 地理空间函数 | 3-5 天 | 需地理类型支持 |

**总计：约 12-18 天**

---

## 5. 依赖添加

在 `Cargo.toml` 中添加以下依赖（如尚未添加）：

```toml
[dependencies]
# 已有依赖
chrono = "0.4"
rand = "0.8"
serde_json = "1.0"
# 地理空间函数需要
# geo = "0.28"  # 等待地理类型实现后添加
```

---

## 6. UDF 相关函数说明

以下函数与 UDF（用户自定义函数）机制相关，**等待 UDF 模块实现后再引入**：

| 函数名 | 说明 |
|--------|------|
| `udf_is_in` | 检查值是否在列表中，用于 UDF 内部 |
| `cos_similarity` | 可能依赖 UDF 框架进行向量计算 |

---

## 7. 测试建议

### 7.1 单元测试模板

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_function() {
        let registry = create_test_registry();
        
        assert_eq!(registry.execute("sign", &[Value::Int(10)]).unwrap(), Value::Int(1));
        assert_eq!(registry.execute("sign", &[Value::Int(-5)]).unwrap(), Value::Int(-1));
        assert_eq!(registry.execute("sign", &[Value::Int(0)]).unwrap(), Value::Int(0));
        assert_eq!(registry.execute("sign", &[Value::Float(3.14)]).unwrap(), Value::Int(1));
        assert_eq!(registry.execute("sign", &[Value::Float(-2.5)]).unwrap(), Value::Int(-1));
    }

    #[test]
    fn test_rand_functions() {
        let registry = create_test_registry();
        
        // rand() 返回 0-1 之间的值
        let result = registry.execute("rand", &[]).unwrap();
        if let Value::Float(f) = result {
            assert!(f >= 0.0 && f <= 1.0);
        } else {
            panic!("rand should return float");
        }
        
        // rand32 范围测试
        let result = registry.execute("rand32", &[Value::Int(100)]).unwrap();
        if let Value::Int(i) = result {
            assert!(i >= 0 && i < 100);
        }
    }

    #[test]
    fn test_datetime_functions() {
        let registry = create_test_registry();
        
        // 测试 time 函数
        let result = registry.execute("time", &[Value::String("14:30:00".to_string())]).unwrap();
        if let Value::Time(t) = result {
            assert_eq!(t.hour, 14);
            assert_eq!(t.minute, 30);
            assert_eq!(t.sec, 0);
        }
        
        // 测试 timestamp 函数
        let result = registry.execute("timestamp", &[]).unwrap();
        assert!(matches!(result, Value::Int(_)));
    }
}
```

---

## 8. 注意事项

### 8.1 NULL 处理
- 所有函数遵循 Nebula 惯例：参数为 NULL 时返回 NULL
- 使用 `Value::Null(NullType::Null)` 表示 NULL 值

### 8.2 类型错误
- 类型错误使用 `ExpressionError::type_error()`
- 无效操作使用 `ExpressionError::invalid_operation()`

### 8.3 纯函数标记
- 对于纯函数（无副作用、相同输入相同输出），设置 `is_pure = true`
- 对于非纯函数（如 rand, now），设置 `is_pure = false`

### 8.4 性能考虑
- 避免不必要的内存分配
- 使用适当的数据结构（如 BTreeSet 用于有序键）
