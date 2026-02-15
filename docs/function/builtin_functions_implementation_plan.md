# 内置函数实现计划

## 1. 概述

本文档基于对 Nebula-Graph 函数实现的分析，整理出当前项目缺失的内置函数，并提供详细的实现方案。

### 1.1 当前状态
- **已有函数**：约 40+ 个（数学、字符串、正则、类型转换、日期时间）
- **缺失函数**：约 50+ 个（图相关、容器操作、高级数学等）

### 1.2 实现原则
1. **优先核心功能**：图查询相关函数优先实现
2. **参考 Nebula 实现**：借鉴成熟的实现逻辑
3. **保持一致性**：与现有函数注册机制保持一致
4. **类型安全**：充分利用 Rust 的类型系统

---

## 2. 缺失函数清单与实现方案

### 2.1 图相关函数（优先级：最高）

这些函数是图查询的核心功能，必须实现。

#### 2.1.1 id() - 获取顶点ID

**功能描述**：返回顶点的 ID

**参考实现**（Nebula）：
```cpp
auto &attr = functions_["id"];
attr.body_ = [](const auto &args) -> Value {
  switch (args[0].get().type()) {
    case Value::Type::NULLVALUE: return Value::kNullValue;
    case Value::Type::VERTEX: return args[0].get().getVertex().vid;
    default: return Value::kNullBadType;
  }
};
```

**Rust 实现方案**：
```rust
// 在 registry.rs 中添加
fn register_graph_functions(&mut self) {
    let registry = self;
    
    // id - 获取顶点ID
    registry.register(
        "id",
        FunctionSignature::new(
            "id",
            vec![ValueType::Vertex],
            ValueType::Any,  // VID 可以是任意类型
            1, 1, true, "获取顶点ID",
        ),
        |args| {
            match &args[0] {
                Value::Vertex(v) => Ok(*v.vid.clone()),
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("id函数需要顶点类型")),
            }
        },
    );
}
```

#### 2.1.2 tags() / labels() - 获取顶点标签

**功能描述**：返回顶点所有标签（tag）的名称列表

**参考实现**（Nebula）：
```cpp
auto &attr = functions_["tags"];
attr.body_ = [](const auto &args) -> Value {
  switch (args[0].get().type()) {
    case Value::Type::NULLVALUE: return Value::kNullValue;
    case Value::Type::VERTEX: {
      List tags;
      for (auto &tag : args[0].get().getVertex().tags) {
        tags.emplace_back(tag.name);
      }
      return tags;
    }
    default: return Value::kNullBadType;
  }
};
functions_["labels"] = attr;  // 别名
```

**Rust 实现方案**：
```rust
// tags / labels
for name in ["tags", "labels"] {
    registry.register(
        name,
        FunctionSignature::new(
            name,
            vec![ValueType::Vertex],
            ValueType::List,
            1, 1, true, "获取顶点标签列表",
        ),
        |args| {
            match &args[0] {
                Value::Vertex(v) => {
                    let tags: Vec<Value> = v.tags.iter()
                        .map(|tag| Value::String(tag.name.clone()))
                        .collect();
                    Ok(Value::List(tags))
                }
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("tags函数需要顶点类型")),
            }
        },
    );
}
```

#### 2.1.3 properties() - 获取属性映射

**功能描述**：返回顶点或边的所有属性

**参考实现**（Nebula）：
```cpp
auto &attr = functions_["properties"];
attr.body_ = [](const auto &args) -> Value {
  switch (args[0].get().type()) {
    case Value::Type::NULLVALUE: return Value::kNullValue;
    case Value::Type::VERTEX: {
      Map props;
      for (auto &tag : args[0].get().getVertex().tags) {
        props.kvs.insert(tag.props.cbegin(), tag.props.cend());
      }
      return Value(std::move(props));
    }
    case Value::Type::EDGE: {
      Map props;
      props.kvs = args[0].get().getEdge().props;
      return Value(std::move(props));
    }
    case Value::Type::MAP: return args[0].get();
    default: return Value::kNullBadType;
  }
};
```

**Rust 实现方案**：
```rust
registry.register(
    "properties",
    FunctionSignature::new(
        "properties",
        vec![ValueType::Any],  // 支持 Vertex, Edge, Map
        ValueType::Map,
        1, 1, true, "获取属性映射",
    ),
    |args| {
        match &args[0] {
            Value::Vertex(v) => {
                let mut props = HashMap::new();
                // 合并所有 tag 的属性
                for tag in &v.tags {
                    props.extend(tag.properties.clone());
                }
                // 合并顶点级属性
                props.extend(v.properties.clone());
                Ok(Value::Map(props))
            }
            Value::Edge(e) => Ok(Value::Map(e.props.clone())),
            Value::Map(m) => Ok(Value::Map(m.clone())),
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("properties函数需要顶点、边或映射类型")),
        }
    },
);
```

#### 2.1.4 type() - 获取边类型

**功能描述**：返回边的类型名称

**Rust 实现方案**：
```rust
registry.register(
    "type",
    FunctionSignature::new(
        "type",
        vec![ValueType::Edge],
        ValueType::String,
        1, 1, true, "获取边类型",
    ),
    |args| {
        match &args[0] {
            Value::Edge(e) => Ok(Value::String(e.edge_type.clone())),
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("type函数需要边类型")),
        }
    },
);
```

#### 2.1.5 src() / dst() - 获取边起点/终点

**功能描述**：返回边的源顶点和目标顶点 ID

**Rust 实现方案**：
```rust
// src
registry.register(
    "src",
    FunctionSignature::new(
        "src",
        vec![ValueType::Edge],
        ValueType::Any,
        1, 1, true, "获取边起点",
    ),
    |args| {
        match &args[0] {
            Value::Edge(e) => Ok(*e.src.clone()),
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("src函数需要边类型")),
        }
    },
);

// dst
registry.register(
    "dst",
    FunctionSignature::new(
        "dst",
        vec![ValueType::Edge],
        ValueType::Any,
        1, 1, true, "获取边终点",
    ),
    |args| {
        match &args[0] {
            Value::Edge(e) => Ok(*e.dst.clone()),
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("dst函数需要边类型")),
        }
    },
);
```

#### 2.1.6 rank() - 获取边rank

**功能描述**：返回边的 ranking 值

**Rust 实现方案**：
```rust
registry.register(
    "rank",
    FunctionSignature::new(
        "rank",
        vec![ValueType::Edge],
        ValueType::Int,
        1, 1, true, "获取边rank",
    ),
    |args| {
        match &args[0] {
            Value::Edge(e) => Ok(Value::Int(e.ranking)),
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("rank函数需要边类型")),
        }
    },
);
```

---

### 2.2 容器操作函数（优先级：高）

#### 2.2.1 head() - 获取列表首元素

**参考实现**（Nebula）：
```cpp
auto &attr = functions_["head"];
attr.body_ = [](const auto &args) -> Value {
  switch (args[0].get().type()) {
    case Value::Type::NULLVALUE: return Value::kNullValue;
    case Value::Type::LIST: {
      const auto &items = args[0].get().getList().values;
      return items.empty() ? Value::kNullValue : items.front();
    }
    default: return Value::kNullBadType;
  }
};
```

**Rust 实现方案**：
```rust
registry.register(
    "head",
    FunctionSignature::new(
        "head",
        vec![ValueType::List],
        ValueType::Any,
        1, 1, true, "获取列表首元素",
    ),
    |args| {
        match &args[0] {
            Value::List(list) => {
                Ok(list.first().cloned().unwrap_or(Value::Null(NullType::Null)))
            }
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("head函数需要列表类型")),
        }
    },
);
```

#### 2.2.2 last() - 获取列表末元素

**Rust 实现方案**：
```rust
registry.register(
    "last",
    FunctionSignature::new(
        "last",
        vec![ValueType::List],
        ValueType::Any,
        1, 1, true, "获取列表末元素",
    ),
    |args| {
        match &args[0] {
            Value::List(list) => {
                Ok(list.last().cloned().unwrap_or(Value::Null(NullType::Null)))
            }
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("last函数需要列表类型")),
        }
    },
);
```

#### 2.2.3 tail() - 获取列表尾部

**参考实现**（Nebula）：
```cpp
auto &attr = functions_["tail"];
attr.body_ = [](const auto &args) -> Value {
  switch (args[0].get().type()) {
    case Value::Type::NULLVALUE: return Value::kNullValue;
    case Value::Type::LIST: {
      auto &list = args[0].get().getList();
      if (list.empty()) return List();
      return List(std::vector<Value>(list.values.begin() + 1, list.values.end()));
    }
    default: return Value::kNullBadType;
  }
};
```

**Rust 实现方案**：
```rust
registry.register(
    "tail",
    FunctionSignature::new(
        "tail",
        vec![ValueType::List],
        ValueType::List,
        1, 1, true, "获取列表尾部（除首元素外）",
    ),
    |args| {
        match &args[0] {
            Value::List(list) => {
                if list.is_empty() {
                    Ok(Value::List(vec![]))
                } else {
                    Ok(Value::List(list[1..].to_vec()))
                }
            }
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("tail函数需要列表类型")),
        }
    },
);
```

#### 2.2.4 size() - 获取容器大小

**参考实现**（Nebula）：
```cpp
auto &attr = functions_["size"];
attr.body_ = [](const auto &args) -> Value {
  switch (args[0].get().type()) {
    case Value::Type::NULLVALUE: return Value::kNullValue;
    case Value::Type::__EMPTY__: return Value::kEmpty;
    case Value::Type::STRING: return static_cast<int64_t>(args[0].get().getStr().size());
    case Value::Type::LIST: return static_cast<int64_t>(args[0].get().getList().size());
    case Value::Type::MAP: return static_cast<int64_t>(args[0].get().getMap().size());
    case Value::Type::SET: return static_cast<int64_t>(args[0].get().getSet().size());
    case Value::Type::DATASET: return static_cast<int64_t>(args[0].get().getDataSet().size());
    default: return Value::kNullBadType;
  }
};
```

**Rust 实现方案**：
```rust
registry.register(
    "size",
    FunctionSignature::new(
        "size",
        vec![ValueType::Any],
        ValueType::Int,
        1, 1, true, "获取容器大小",
    ),
    |args| {
        match &args[0] {
            Value::String(s) => Ok(Value::Int(s.len() as i64)),
            Value::List(list) => Ok(Value::Int(list.len() as i64)),
            Value::Map(map) => Ok(Value::Int(map.len() as i64)),
            Value::Set(set) => Ok(Value::Int(set.len() as i64)),
            Value::DataSet(ds) => Ok(Value::Int(ds.rows.len() as i64)),
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("size函数不支持该类型")),
        }
    },
);
```

#### 2.2.5 range() - 生成范围列表

**参考实现**（Nebula）：
```cpp
auto &attr = functions_["range"];
attr.body_ = [](const auto &args) -> Value {
  if (!args[0].get().isInt() || !args[1].get().isInt()) {
    return Value::kNullBadType;
  }
  int64_t start = args[0].get().getInt();
  int64_t end = args[1].get().getInt();
  int64_t step = 1;
  if (args.size() == 3) {
    if (!args[2].get().isInt()) return Value::kNullBadType;
    step = args[2].get().getInt();
  }
  if (step == 0) return Value::kNullBadData;
  
  List res;
  for (auto i = start; step > 0 ? i <= end : i >= end; i = i + step) {
    res.emplace_back(i);
  }
  return Value(res);
};
```

**Rust 实现方案**：
```rust
// range(start, end) 或 range(start, end, step)
registry.register(
    "range",
    FunctionSignature::new(
        "range",
        vec![ValueType::Int, ValueType::Int],  // 基础签名
        ValueType::List,
        2, 3, true, "生成范围列表",
    ),
    |args| {
        let start = match &args[0] {
            Value::Int(i) => *i,
            Value::Null(_) => return Ok(Value::Null(NullType::Null)),
            _ => return Err(ExpressionError::type_error("range函数需要整数参数")),
        };
        let end = match &args[1] {
            Value::Int(i) => *i,
            Value::Null(_) => return Ok(Value::Null(NullType::Null)),
            _ => return Err(ExpressionError::type_error("range函数需要整数参数")),
        };
        let step = if args.len() > 2 {
            match &args[2] {
                Value::Int(i) => *i,
                Value::Null(_) => return Ok(Value::Null(NullType::Null)),
                _ => return Err(ExpressionError::type_error("range函数的step需要整数")),
            }
        } else {
            1
        };
        
        if step == 0 {
            return Err(ExpressionError::new(
                ExpressionErrorType::InvalidOperation,
                "range函数的step不能为0".to_string(),
            ));
        }
        
        let mut result = Vec::new();
        if step > 0 {
            let mut i = start;
            while i <= end {
                result.push(Value::Int(i));
                i += step;
            }
        } else {
            let mut i = start;
            while i >= end {
                result.push(Value::Int(i));
                i += step;
            }
        }
        
        Ok(Value::List(result))
    },
);
```

#### 2.2.6 keys() - 获取键列表

**参考实现**（Nebula）：
```cpp
auto &attr = functions_["keys"];
attr.body_ = [](const auto &args) -> Value {
  std::set<std::string> tmp;
  switch (args[0].get().type()) {
    case Value::Type::NULLVALUE: return Value::kNullValue;
    case Value::Type::VERTEX:
      for (auto &tag : args[0].get().getVertex().tags) {
        for (auto &prop : tag.props) tmp.emplace(prop.first);
      }
      break;
    case Value::Type::EDGE:
      for (auto &prop : args[0].get().getEdge().props) tmp.emplace(prop.first);
      break;
    case Value::Type::MAP:
      for (auto &kv : args[0].get().getMap().kvs) tmp.emplace(kv.first);
      break;
    default: return Value::kNullBadType;
  }
  List result;
  result.values.assign(tmp.cbegin(), tmp.cend());
  return result;
};
```

**Rust 实现方案**：
```rust
use std::collections::BTreeSet;

registry.register(
    "keys",
    FunctionSignature::new(
        "keys",
        vec![ValueType::Any],
        ValueType::List,
        1, 1, true, "获取键列表",
    ),
    |args| {
        let mut keys: BTreeSet<String> = BTreeSet::new();
        
        match &args[0] {
            Value::Vertex(v) => {
                for tag in &v.tags {
                    for key in tag.properties.keys() {
                        keys.insert(key.clone());
                    }
                }
            }
            Value::Edge(e) => {
                for key in e.props.keys() {
                    keys.insert(key.clone());
                }
            }
            Value::Map(m) => {
                for key in m.keys() {
                    keys.insert(key.clone());
                }
            }
            Value::Null(_) => return Ok(Value::Null(NullType::Null)),
            _ => return Err(ExpressionError::type_error("keys函数需要顶点、边或映射类型")),
        }
        
        let result: Vec<Value> = keys.into_iter().map(Value::String).collect();
        Ok(Value::List(result))
    },
);
```

---

### 2.3 路径相关函数（优先级：高）

#### 2.3.1 nodes() - 获取路径中的节点

**参考实现**（Nebula）：
```cpp
auto &attr = functions_["nodes"];
attr.body_ = [](const auto &args) -> Value {
  switch (args[0].get().type()) {
    case Value::Type::NULLVALUE: return Value::kNullValue;
    case Value::Type::PATH: {
      auto &path = args[0].get().getPath();
      List result;
      result.emplace_back(path.src);
      for (auto &step : path.steps) {
        result.emplace_back(step.dst);
      }
      return result;
    }
    default: return Value::kNullBadType;
  }
};
```

**Rust 实现方案**：
```rust
registry.register(
    "nodes",
    FunctionSignature::new(
        "nodes",
        vec![ValueType::Path],
        ValueType::List,
        1, 1, true, "获取路径中的所有节点",
    ),
    |args| {
        match &args[0] {
            Value::Path(path) => {
                let mut result = vec![Value::Vertex(*path.src.clone())];
                for step in &path.steps {
                    result.push(Value::Vertex(step.dst.clone()));
                }
                Ok(Value::List(result))
            }
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("nodes函数需要路径类型")),
        }
    },
);
```

#### 2.3.2 relationships() - 获取路径中的边

**Rust 实现方案**：
```rust
registry.register(
    "relationships",
    FunctionSignature::new(
        "relationships",
        vec![ValueType::Path],
        ValueType::List,
        1, 1, true, "获取路径中的所有边",
    ),
    |args| {
        match &args[0] {
            Value::Path(path) => {
                let mut result = Vec::new();
                for step in &path.steps {
                    if let Some(ref edge) = step.edge {
                        result.push(Value::Edge(*edge.clone()));
                    }
                }
                Ok(Value::List(result))
            }
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("relationships函数需要路径类型")),
        }
    },
);
```

---

### 2.4 数学函数（优先级：中）

#### 2.4.1 位运算函数

**Rust 实现方案**：
```rust
fn register_bit_functions(&mut self) {
    let registry = self;
    
    // bit_and
    registry.register(
        "bit_and",
        FunctionSignature::new(
            "bit_and",
            vec![ValueType::Int, ValueType::Int],
            ValueType::Int,
            2, 2, true, "按位与",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a & b)),
                (Value::Null(_), _) | (_, Value::Null(_)) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("bit_and函数需要整数参数")),
            }
        },
    );
    
    // bit_or
    registry.register(
        "bit_or",
        FunctionSignature::new(
            "bit_or",
            vec![ValueType::Int, ValueType::Int],
            ValueType::Int,
            2, 2, true, "按位或",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a | b)),
                (Value::Null(_), _) | (_, Value::Null(_)) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("bit_or函数需要整数参数")),
            }
        },
    );
    
    // bit_xor
    registry.register(
        "bit_xor",
        FunctionSignature::new(
            "bit_xor",
            vec![ValueType::Int, ValueType::Int],
            ValueType::Int,
            2, 2, true, "按位异或",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a ^ b)),
                (Value::Null(_), _) | (_, Value::Null(_)) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("bit_xor函数需要整数参数")),
            }
        },
    );
}
```

#### 2.4.2 反三角函数

**Rust 实现方案**：
```rust
// asin
registry.register(
    "asin",
    FunctionSignature::new(
        "asin",
        vec![ValueType::Float],
        ValueType::Float,
        1, 1, true, "反正弦",
    ),
    |args| {
        match &args[0] {
            Value::Float(f) => Ok(Value::Float(f.asin())),
            Value::Int(i) => Ok(Value::Float((*i as f64).asin())),
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("asin函数需要数值类型")),
        }
    },
);

// acos
registry.register(
    "acos",
    FunctionSignature::new(
        "acos",
        vec![ValueType::Float],
        ValueType::Float,
        1, 1, true, "反余弦",
    ),
    |args| {
        match &args[0] {
            Value::Float(f) => Ok(Value::Float(f.acos())),
            Value::Int(i) => Ok(Value::Float((*i as f64).acos())),
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("acos函数需要数值类型")),
        }
    },
);

// atan
registry.register(
    "atan",
    FunctionSignature::new(
        "atan",
        vec![ValueType::Float],
        ValueType::Float,
        1, 1, true, "反正切",
    ),
    |args| {
        match &args[0] {
            Value::Float(f) => Ok(Value::Float(f.atan())),
            Value::Int(i) => Ok(Value::Float((*i as f64).atan())),
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("atan函数需要数值类型")),
        }
    },
);
```

#### 2.4.3 其他数学函数

```rust
// cbrt - 立方根
registry.register(
    "cbrt",
    FunctionSignature::new(
        "cbrt",
        vec![ValueType::Float],
        ValueType::Float,
        1, 1, true, "立方根",
    ),
    |args| {
        match &args[0] {
            Value::Float(f) => Ok(Value::Float(f.cbrt())),
            Value::Int(i) => Ok(Value::Float((*i as f64).cbrt())),
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("cbrt函数需要数值类型")),
        }
    },
);

// hypot - 欧几里得距离
registry.register(
    "hypot",
    FunctionSignature::new(
        "hypot",
        vec![ValueType::Float, ValueType::Float],
        ValueType::Float,
        2, 2, true, "欧几里得距离",
    ),
    |args| {
        match (&args[0], &args[1]) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.hypot(*b))),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Float((*a as f64).hypot(*b as f64))),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a.hypot(*b as f64))),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64).hypot(*b))),
            (Value::Null(_), _) | (_, Value::Null(_)) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("hypot函数需要数值类型")),
        }
    },
);

// pi - 圆周率常量
registry.register(
    "pi",
    FunctionSignature::new(
        "pi",
        vec![],
        ValueType::Float,
        0, 0, true, "圆周率",
    ),
    |_args| {
        Ok(Value::Float(std::f64::consts::PI))
    },
);

// e - 自然常数
registry.register(
    "e",
    FunctionSignature::new(
        "e",
        vec![],
        ValueType::Float,
        0, 0, true, "自然常数",
    ),
    |_args| {
        Ok(Value::Float(std::f64::consts::E))
    },
);
```

---

### 2.5 字符串函数（优先级：中）

#### 2.5.1 left() / right() - 左右截取

**Rust 实现方案**：
```rust
// left(string, length)
registry.register(
    "left",
    FunctionSignature::new(
        "left",
        vec![ValueType::String, ValueType::Int],
        ValueType::String,
        2, 2, true, "左侧截取字符串",
    ),
    |args| {
        match (&args[0], &args[1]) {
            (Value::String(s), Value::Int(n)) => {
                if *n <= 0 {
                    Ok(Value::String(String::new()))
                } else {
                    let end = (*n as usize).min(s.len());
                    Ok(Value::String(s[..end].to_string()))
                }
            }
            (Value::Null(_), _) | (_, Value::Null(_)) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("left函数需要字符串和整数参数")),
        }
    },
);

// right(string, length)
registry.register(
    "right",
    FunctionSignature::new(
        "right",
        vec![ValueType::String, ValueType::Int],
        ValueType::String,
        2, 2, true, "右侧截取字符串",
    ),
    |args| {
        match (&args[0], &args[1]) {
            (Value::String(s), Value::Int(n)) => {
                if *n <= 0 {
                    Ok(Value::String(String::new()))
                } else {
                    let start = s.len().saturating_sub(*n as usize);
                    Ok(Value::String(s[start..].to_string()))
                }
            }
            (Value::Null(_), _) | (_, Value::Null(_)) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("right函数需要字符串和整数参数")),
        }
    },
);
```

#### 2.5.2 split() - 字符串分割

**Rust 实现方案**：
```rust
registry.register(
    "split",
    FunctionSignature::new(
        "split",
        vec![ValueType::String, ValueType::String],
        ValueType::List,
        2, 2, true, "分割字符串",
    ),
    |args| {
        match (&args[0], &args[1]) {
            (Value::String(s), Value::String(delimiter)) => {
                let parts: Vec<Value> = s.split(delimiter)
                    .map(|part| Value::String(part.to_string()))
                    .collect();
                Ok(Value::List(parts))
            }
            (Value::Null(_), _) | (_, Value::Null(_)) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("split函数需要两个字符串参数")),
        }
    },
);
```

#### 2.5.3 reverse() - 字符串反转

**Rust 实现方案**：
```rust
registry.register(
    "reverse",
    FunctionSignature::new(
        "reverse",
        vec![ValueType::String],
        ValueType::String,
        1, 1, true, "反转字符串",
    ),
    |args| {
        match &args[0] {
            Value::String(s) => {
                Ok(Value::String(s.chars().rev().collect()))
            }
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("reverse函数需要字符串类型")),
        }
    },
);
```

#### 2.5.4 substr() / substring() - 子字符串

**Rust 实现方案**：
```rust
// substr/substring(string, start, [length])
for name in ["substr", "substring"] {
    registry.register(
        name,
        FunctionSignature::new(
            name,
            vec![ValueType::String, ValueType::Int],
            ValueType::String,
            2, 3, true, "获取子字符串",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::String(s), Value::Int(start)) => {
                    let len = if args.len() > 2 {
                        match &args[2] {
                            Value::Int(l) => *l as usize,
                            Value::Null(_) => return Ok(Value::Null(NullType::Null)),
                            _ => return Err(ExpressionError::type_error("substr函数的length需要整数")),
                        }
                    } else {
                        s.len()
                    };
                    
                    if *start < 0 || len == 0 {
                        return Ok(Value::String(String::new()));
                    }
                    
                    let start = *start as usize;
                    if start >= s.len() {
                        return Ok(Value::String(String::new()));
                    }
                    
                    let end = (start + len).min(s.len());
                    Ok(Value::String(s[start..end].to_string()))
                }
                (Value::Null(_), _) | (_, Value::Null(_)) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("substr函数需要字符串和整数参数")),
            }
        },
    );
}
```

---

### 2.6 实用函数（优先级：中）

#### 2.6.1 coalesce() - 返回第一个非NULL值

**Rust 实现方案**：
```rust
registry.register(
    "coalesce",
    FunctionSignature::new(
        "coalesce",
        vec![ValueType::Any],
        ValueType::Any,
        1, usize::MAX, true, "返回第一个非NULL值",
    ),
    |args| {
        for arg in args {
            if !matches!(arg, Value::Null(_)) {
                return Ok(arg.clone());
            }
        }
        Ok(Value::Null(NullType::Null))
    },
);
```

#### 2.6.2 hash() - 计算哈希值

**Rust 实现方案**：
```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

registry.register(
    "hash",
    FunctionSignature::new(
        "hash",
        vec![ValueType::Any],
        ValueType::Int,
        1, 1, true, "计算哈希值",
    ),
    |args| {
        match &args[0] {
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            value => {
                // 使用 Value 的 Hash 实现
                let mut hasher = DefaultHasher::new();
                value.hash(&mut hasher);
                Ok(Value::Int(hasher.finish() as i64))
            }
        }
    },
);
```

---

## 3. 实现步骤

### 3.1 文件结构

```
src/expression/functions/
├── mod.rs           # 模块导出
├── registry.rs      # 函数注册表（添加新函数）
├── signature.rs     # 函数签名
└── README.md        # 函数文档
```

### 3.2 修改 registry.rs

在 `registry.rs` 中添加新的注册方法：

```rust
impl FunctionRegistry {
    /// 注册所有内置函数
    fn register_all_builtin_functions(&mut self) {
        self.register_math_functions();
        self.register_string_functions();
        self.register_regex_functions();
        self.register_conversion_functions();
        self.register_datetime_functions();
        // 新增：
        self.register_graph_functions();
        self.register_collection_functions();
        self.register_bit_functions();
    }
    
    // 图相关函数
    fn register_graph_functions(&mut self) { /* ... */ }
    
    // 容器操作函数
    fn register_collection_functions(&mut self) { /* ... */ }
    
    // 位运算函数
    fn register_bit_functions(&mut self) { /* ... */ }
}
```

### 3.3 实现顺序建议

| 阶段 | 函数类别 | 预计工作量 |
|------|----------|-----------|
| Phase 1 | 图相关函数（id, tags, properties, type, src, dst, rank） | 2-3 天 |
| Phase 2 | 容器操作函数（head, last, tail, size, range, keys） | 2 天 |
| Phase 3 | 路径函数（nodes, relationships） | 1 天 |
| Phase 4 | 数学函数（bit_and/or/xor, asin, acos, atan, cbrt, hypot） | 2 天 |
| Phase 5 | 字符串函数（left, right, split, reverse, substr） | 2 天 |
| Phase 6 | 实用函数（coalesce, hash） | 1 天 |

**总计：约 10-12 天**

---

## 4. 测试建议

### 4.1 单元测试模板

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_id_function() {
        let registry = FunctionRegistry::new();
        
        // 测试顶点ID
        let vertex = Value::Vertex(Vertex::new(
            Value::Int(100),
            vec![Tag::new("Person", HashMap::new())]
        ));
        
        let result = registry.execute("id", &[vertex]).unwrap();
        assert_eq!(result, Value::Int(100));
        
        // 测试NULL
        let result = registry.execute("id", &[Value::Null(NullType::Null)]).unwrap();
        assert!(matches!(result, Value::Null(_)));
    }
    
    #[test]
    fn test_tags_function() {
        let registry = FunctionRegistry::new();
        
        let mut props = HashMap::new();
        props.insert("name".to_string(), Value::String("Alice".to_string()));
        
        let vertex = Value::Vertex(Vertex::new(
            Value::Int(100),
            vec![Tag::new("Person", props)]
        ));
        
        let result = registry.execute("tags", &[vertex]).unwrap();
        assert_eq!(result, Value::List(vec![Value::String("Person".to_string())]));
    }
}
```

---

## 5. 注意事项

### 5.1 类型匹配
- 使用 `ValueType::Any` 接受多种类型时，在函数体内进行运行时类型检查
- 对于图相关函数，明确指定 `ValueType::Vertex`、`ValueType::Edge` 等

### 5.2 NULL 处理
- 遵循 Nebula 的惯例：参数为 NULL 时返回 NULL
- 使用 `Value::Null(NullType::Null)` 表示 NULL 值

### 5.3 错误处理
- 类型错误使用 `ExpressionError::type_error()`
- 无效操作使用 `ExpressionError::new(ExpressionErrorType::InvalidOperation, ...)`

### 5.4 性能考虑
- 对于纯函数，确保 `is_pure` 标记为 `true`
- 避免不必要的内存分配（如字符串克隆）
