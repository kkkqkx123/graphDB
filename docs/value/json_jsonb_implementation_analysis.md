# GraphDB JSON/JSONB 类型实现分析

## 1. 概述

JSON/JSONB 是现代数据库的必备类型，用于存储半结构化数据。本分析文档详细说明如何在 GraphDB 中实现这两种类型。

---

## 2. JSON vs JSONB 对比

| 特性     | JSON                 | JSONB                |
| -------- | -------------------- | -------------------- |
| 存储格式 | 文本格式             | 二进制格式           |
| 存储效率 | 较低（保留原始格式） | 较高（解析后存储）   |
| 查询性能 | 较慢（需解析）       | 较快（已解析）       |
| 写入性能 | 较快（无需解析）     | 较慢（需要解析）     |
| 数据校验 | 写入时可选校验       | 写入时必须校验       |
| 重复键   | 保留最后出现的键     | 保留最后出现的键     |
| 空白字符 | 保留                 | 去除                 |
| 键顺序   | 保留                 | 不保留（排序后存储） |

**参考**: PostgreSQL JSON/JSONB 实现

---

## 3. 实现方案

### 3.1 核心设计

```rust
// src/core/value/json.rs

use oxicode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::hash::{Hash, Hasher};

/// JSON 类型 - 文本存储格式
///
/// 特点：
/// - 保留原始文本格式（包括空白字符、键顺序）
/// - 写入时无需解析，性能更好
/// - 查询时需要解析
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Json {
    /// 原始 JSON 文本
    text: String,
    /// 缓存的解析结果（可选，用于提升查询性能）
    #[serde(skip)]
    cached_value: Option<JsonValue>,
}

/// JSONB 类型 - 二进制存储格式
///
/// 特点：
/// - 存储解析后的二进制格式
/// - 写入时需要解析和校验
/// - 查询时性能更好
/// - 支持创建 GIN 索引
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonB {
    /// 解析后的 JSON 值
    value: JsonValue,
    /// 序列化后的字节缓存（用于存储和传输）
    #[serde(skip)]
    cached_bytes: Option<Vec<u8>>,
}
```

### 3.2 Value 枚举扩展

```rust
// src/core/value/value_def.rs

pub enum Value {
    // ... 现有类型

    /// JSON 类型（文本格式）
    Json(Box<Json>),

    /// JSONB 类型（二进制格式）
    JsonB(Box<JsonB>),
}
```

### 3.3 DataType 枚举扩展

```rust
// src/core/types/mod.rs

pub enum DataType {
    // ... 现有类型

    /// JSON 文本类型
    Json,

    /// JSONB 二进制类型
    JsonB,
}
```

---

## 4. 详细实现

### 4.1 Json 类型实现

```rust
// src/core/value/json.rs

impl Json {
    /// 从字符串创建 JSON
    pub fn from_str(text: &str) -> Result<Self, JsonError> {
        // 可选：验证 JSON 格式
        let value: JsonValue = serde_json::from_str(text)
            .map_err(|e| JsonError::InvalidJson(e.to_string()))?;

        Ok(Self {
            text: text.to_string(),
            cached_value: Some(value),
        })
    }

    /// 从 JsonValue 创建
    pub fn from_value(value: JsonValue) -> Self {
        let text = value.to_string();
        Self {
            text,
            cached_value: Some(value),
        }
    }

    /// 获取原始文本
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// 获取解析后的值（带缓存）
    pub fn to_value(&self) -> Result<JsonValue, JsonError> {
        if let Some(ref value) = self.cached_value {
            return Ok(value.clone());
        }

        serde_json::from_str(&self.text)
            .map_err(|e| JsonError::InvalidJson(e.to_string()))
    }

    /// 获取指定路径的值
    /// 路径格式: "key1.key2[0].key3"
    pub fn get_path(&self, path: &str) -> Result<Option<JsonValue>, JsonError> {
        let value = self.to_value()?;
        Ok(get_json_path(&value, path))
    }

    /// 转换为 JsonB
    pub fn to_jsonb(&self) -> Result<JsonB, JsonError> {
        let value = self.to_value()?;
        Ok(JsonB::from_value(value))
    }

    /// 估算内存使用
    pub fn estimated_size(&self) -> usize {
        std::mem::size_of::<Self>() + self.text.capacity()
    }
}

impl PartialEq for Json {
    fn eq(&self, other: &Self) -> bool {
        // 比较解析后的值，而非文本
        match (self.to_value(), other.to_value()) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Json {}

impl Hash for Json {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // 使用解析后的值进行哈希
        if let Ok(value) = self.to_value() {
            hash_json_value(&value, state);
        }
    }
}
```

### 4.2 JsonB 类型实现

```rust
// src/core/value/json.rs

impl JsonB {
    /// 从字符串创建 JSONB（必须验证）
    pub fn from_str(text: &str) -> Result<Self, JsonError> {
        let value: JsonValue = serde_json::from_str(text)
            .map_err(|e| JsonError::InvalidJson(e.to_string()))?;

        Ok(Self::from_value(value))
    }

    /// 从 JsonValue 创建
    pub fn from_value(value: JsonValue) -> Self {
        // 规范化 JSON（排序键、去除空白）
        let normalized = normalize_json(value);

        Self {
            value: normalized,
            cached_bytes: None,
        }
    }

    /// 获取解析后的值
    pub fn as_value(&self) -> &JsonValue {
        &self.value
    }

    /// 转换为文本格式
    pub fn to_string(&self) -> String {
        self.value.to_string()
    }

    /// 获取指定路径的值
    pub fn get_path(&self, path: &str) -> Option<&JsonValue> {
        get_json_path_ref(&self.value, path)
    }

    /// 转换为 Json
    pub fn to_json(&self) -> Json {
        Json::from_value(self.value.clone())
    }

    /// 检查是否包含指定键
    pub fn contains_key(&self, key: &str) -> bool {
        matches!(self.value, JsonValue::Object(ref map) if map.contains_key(key))
    }

    /// 获取对象键数量
    pub fn key_count(&self) -> usize {
        match self.value {
            JsonValue::Object(ref map) => map.len(),
            _ => 0,
        }
    }

    /// 获取数组长度
    pub fn array_len(&self) -> Option<usize> {
        match self.value {
            JsonValue::Array(ref arr) => Some(arr.len()),
            _ => None,
        }
    }

    /// 估算内存使用
    pub fn estimated_size(&self) -> usize {
        std::mem_of::<Self>() + estimate_json_size(&self.value)
    }
}

impl PartialEq for JsonB {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for JsonB {}

impl Hash for JsonB {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_json_value(&self.value, state);
    }
}

impl Ord for JsonB {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // JSONB 比较规则：按类型优先级，再按值
        compare_json_values(&self.value, &other.value)
    }
}

impl PartialOrd for JsonB {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
```

### 4.3 辅助函数

```rust
// src/core/value/json.rs

/// JSON 错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonError {
    InvalidJson(String),
    InvalidPath(String),
    TypeMismatch { expected: String, actual: String },
}

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonError::InvalidJson(msg) => write!(f, "Invalid JSON: {}", msg),
            JsonError::InvalidPath(path) => write!(f, "Invalid JSON path: {}", path),
            JsonError::TypeMismatch { expected, actual } => {
                write!(f, "Type mismatch: expected {}, got {}", expected, actual)
            }
        }
    }
}

impl std::error::Error for JsonError {}

/// 规范化 JSON 值（用于 JSONB）
fn normalize_json(value: JsonValue) -> JsonValue {
    match value {
        JsonValue::Object(map) => {
            // 排序键并递归规范化
            let mut entries: Vec<_> = map.into_iter().collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            let normalized: serde_json::Map<String, JsonValue> = entries
                .into_iter()
                .map(|(k, v)| (k, normalize_json(v)))
                .collect();
            JsonValue::Object(normalized)
        }
        JsonValue::Array(arr) => {
            JsonValue::Array(arr.into_iter().map(normalize_json).collect())
        }
        // 其他类型保持不变
        other => other,
    }
}

/// 获取 JSON 路径值
fn get_json_path(value: &JsonValue, path: &str) -> Option<JsonValue> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for part in parts {
        // 处理数组索引，如 "items[0]"
        if let Some(idx_start) = part.find('[') {
            let key = &part[..idx_start];
            let idx_end = part.find(']').unwrap_or(part.len());
            let idx: usize = part[idx_start + 1..idx_end].parse().ok()?;

            if !key.is_empty() {
                current = current.get(key)?;
            }
            current = current.get(idx)?;
        } else {
            current = current.get(part)?;
        }
    }

    Some(current.clone())
}

/// 获取 JSON 路径值（引用版本）
fn get_json_path_ref<'a>(value: &'a JsonValue, path: &str) -> Option<&'a JsonValue> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for part in parts {
        if let Some(idx_start) = part.find('[') {
            let key = &part[..idx_start];
            let idx_end = part.find(']').unwrap_or(part.len());
            let idx: usize = part[idx_start + 1..idx_end].parse().ok()?;

            if !key.is_empty() {
                current = current.get(key)?;
            }
            current = current.get(idx)?;
        } else {
            current = current.get(part)?;
        }
    }

    Some(current)
}

/// 哈希 JSON 值
fn hash_json_value<H: Hasher>(value: &JsonValue, state: &mut H) {
    match value {
        JsonValue::Null => 0u8.hash(state),
        JsonValue::Bool(b) => {
            1u8.hash(state);
            b.hash(state);
        }
        JsonValue::Number(n) => {
            2u8.hash(state);
            n.to_string().hash(state);
        }
        JsonValue::String(s) => {
            3u8.hash(state);
            s.hash(state);
        }
        JsonValue::Array(arr) => {
            4u8.hash(state);
            for item in arr {
                hash_json_value(item, state);
            }
        }
        JsonValue::Object(map) => {
            5u8.hash(state);
            for (k, v) in map {
                k.hash(state);
                hash_json_value(v, state);
            }
        }
    }
}

/// 比较两个 JSON 值
fn compare_json_values(a: &JsonValue, b: &JsonValue) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    // 类型优先级：Null < Bool < Number < String < Array < Object
    let type_priority = |v: &JsonValue| match v {
        JsonValue::Null => 0,
        JsonValue::Bool(_) => 1,
        JsonValue::Number(_) => 2,
        JsonValue::String(_) => 3,
        JsonValue::Array(_) => 4,
        JsonValue::Object(_) => 5,
    };

    let priority_a = type_priority(a);
    let priority_b = type_priority(b);

    match priority_a.cmp(&priority_b) {
        Ordering::Equal => match (a, b) {
            (JsonValue::Null, JsonValue::Null) => Ordering::Equal,
            (JsonValue::Bool(a), JsonValue::Bool(b)) => a.cmp(b),
            (JsonValue::Number(a), JsonValue::Number(b)) => {
                // 尝试作为整数比较，否则作为浮点数
                if let (Some(a_i), Some(b_i)) = (a.as_i64(), b.as_i64()) {
                    a_i.cmp(&b_i)
                } else if let (Some(a_f), Some(b_f)) = (a.as_f64(), b.as_f64()) {
                    a_f.partial_cmp(&b_f).unwrap_or(Ordering::Equal)
                } else {
                    a.to_string().cmp(&b.to_string())
                }
            }
            (JsonValue::String(a), JsonValue::String(b)) => a.cmp(b),
            (JsonValue::Array(a), JsonValue::Array(b)) => {
                a.iter().zip(b.iter())
                    .map(|(x, y)| compare_json_values(x, y))
                    .find(|&ord| ord != Ordering::Equal)
                    .unwrap_or_else(|| a.len().cmp(&b.len()))
            }
            (JsonValue::Object(a), JsonValue::Object(b)) => {
                let a_keys: Vec<_> = a.keys().collect();
                let b_keys: Vec<_> = b.keys().collect();

                match a_keys.cmp(&b_keys) {
                    Ordering::Equal => {
                        a_keys.iter()
                            .map(|k| compare_json_values(&a[*k], &b[*k]))
                            .find(|&ord| ord != Ordering::Equal)
                            .unwrap_or(Ordering::Equal)
                    }
                    other => other,
                }
            }
            _ => Ordering::Equal, // 不同类型已被优先级处理
        },
        other => other,
    }
}

/// 估算 JSON 值内存使用
fn estimate_json_size(value: &JsonValue) -> usize {
    match value {
        JsonValue::Null => 0,
        JsonValue::Bool(_) => 1,
        JsonValue::Number(n) => n.to_string().len(),
        JsonValue::String(s) => s.len(),
        JsonValue::Array(arr) => {
            arr.iter().map(estimate_json_size).sum::<usize>()
                + arr.len() * std::mem::size_of::<JsonValue>()
        }
        JsonValue::Object(map) => {
            map.iter()
                .map(|(k, v)| k.len() + estimate_json_size(v))
                .sum::<usize>()
                + map.len() * std::mem::size_of::<(String, JsonValue)>()
        }
    }
}
```

---

## 5. Value 枚举集成

### 5.1 扩展 Value 枚举

```rust
// src/core/value/value_def.rs

pub enum Value {
    // ... 现有类型

    /// JSON 类型（文本格式）
    Json(Box<Json>),

    /// JSONB 类型（二进制格式）
    JsonB(Box<JsonB>),
}

impl Value {
    pub fn get_type(&self) -> DataType {
        match self {
            // ... 现有匹配
            Value::Json(_) => DataType::Json,
            Value::JsonB(_) => DataType::JsonB,
        }
    }

    /// 创建 JSON 值
    pub fn json(text: &str) -> Result<Self, JsonError> {
        Ok(Value::Json(Box::new(Json::from_str(text)?)))
    }

    /// 创建 JSONB 值
    pub fn jsonb(text: &str) -> Result<Self, JsonError> {
        Ok(Value::JsonB(Box::new(JsonB::from_str(text)?)))
    }

    /// 从 serde_json::Value 创建
    pub fn from_json_value(value: JsonValue) -> Self {
        Value::JsonB(Box::new(JsonB::from_value(value)))
    }
}
```

### 5.2 比较实现

```rust
// src/core/value/value_compare.rs

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // ... 现有匹配
            (Value::Json(a), Value::Json(b)) => a == b,
            (Value::JsonB(a), Value::JsonB(b)) => a == b,
            // JSON 和 JSONB 可以比较
            (Value::Json(a), Value::JsonB(b)) => {
                a.to_value().ok() == Some(b.as_value().clone())
            }
            (Value::JsonB(a), Value::Json(b)) => {
                Some(a.as_value().clone()) == b.to_value().ok()
            }
            _ => false,
        }
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            // ... 现有匹配
            (Value::Json(a), Value::Json(b)) => {
                // 比较解析后的值
                match (a.to_value(), b.to_value()) {
                    (Ok(a_val), Ok(b_val)) => compare_json_values(&a_val, &b_val),
                    _ => std::cmp::Ordering::Equal,
                }
            }
            (Value::JsonB(a), Value::JsonB(b)) => a.cmp(b),
            // 跨类型比较
            (Value::Json(a), Value::JsonB(b)) => {
                match a.to_value() {
                    Ok(a_val) => compare_json_values(&a_val, b.as_value()),
                    _ => std::cmp::Ordering::Equal,
                }
            }
            (Value::JsonB(a), Value::Json(b)) => {
                match b.to_value() {
                    Ok(b_val) => compare_json_values(a.as_value(), &b_val),
                    _ => std::cmp::Ordering::Equal,
                }
            }
            // ...
        }
    }
}
```

---

## 6. 查询操作支持

### 6.1 JSON 路径查询

```rust
// src/core/value/json.rs

impl Json {
    /// 提取 JSON 路径值，返回新的 Value
    pub fn extract(&self, path: &str) -> Result<Value, JsonError> {
        let json_value = self.get_path(path)?;
        Ok(json_value_to_value(json_value.unwrap_or(JsonValue::Null)))
    }
}

impl JsonB {
    /// 提取 JSON 路径值，返回新的 Value
    pub fn extract(&self, path: &str) -> Value {
        let json_value = self.get_path(path).cloned().unwrap_or(JsonValue::Null);
        json_value_to_value(json_value)
    }
}

/// 将 serde_json::Value 转换为 GraphDB Value
fn json_value_to_value(value: JsonValue) -> Value {
    match value {
        JsonValue::Null => Value::Null(NullType::Null),
        JsonValue::Bool(b) => Value::Bool(b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Null(NullType::BadData)
            }
        }
        JsonValue::String(s) => Value::String(s),
        JsonValue::Array(arr) => {
            let values: Vec<Value> = arr.into_iter().map(json_value_to_value).collect();
            Value::list(List { values })
        }
        JsonValue::Object(map) => {
            let hashmap: HashMap<String, Value> = map
                .into_iter()
                .map(|(k, v)| (k, json_value_to_value(v)))
                .collect();
            Value::map(hashmap)
        }
    }
}
```

### 6.2 JSON 操作符

```rust
// src/core/value/value_arithmetic.rs

impl Value {
    /// JSON 字段访问操作符 ->
    pub fn json_get(&self, key: &str) -> Result<Value, String> {
        match self {
            Value::Json(json) => {
                json.get_path(key)
                    .map_err(|e| e.to_string())?
                    .map(|v| json_value_to_value(v))
                    .ok_or_else(|| format!("Key '{}' not found", key))
            }
            Value::JsonB(jsonb) => {
                Ok(json_value_to_value(
                    jsonb.get_path(key).cloned().unwrap_or(JsonValue::Null)
                ))
            }
            _ => Err("JSON field access only supported for JSON/JSONB types".to_string()),
        }
    }

    /// JSON 文本字段访问操作符 ->>
    pub fn json_get_text(&self, key: &str) -> Result<Value, String> {
        match self {
            Value::Json(json) => {
                let value = json.get_path(key)
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| format!("Key '{}' not found", key))?;
                Ok(Value::String(value.to_string()))
            }
            Value::JsonB(jsonb) => {
                let value = jsonb.get_path(key).cloned().unwrap_or(JsonValue::Null);
                Ok(Value::String(value.to_string()))
            }
            _ => Err("JSON text access only supported for JSON/JSONB types".to_string()),
        }
    }
}
```

---

## 7. 存储层集成

### 7.1 oxicode 序列化

```rust
// src/core/value/json.rs

// Json 已实现 Encode/Decode（通过 derive）
// JsonB 需要手动实现，因为 serde_json::Value 不支持 oxicode

impl Encode for JsonB {
    fn encode<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        // 序列化为 JSON 文本后编码
        let text = self.to_string();
        text.encode(writer)
    }
}

impl Decode for JsonB {
    fn decode<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let text = String::decode(reader)?;
        Self::from_str(&text)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }
}
```

### 7.2 内存估算

```rust
// src/core/value/memory.rs

impl MemoryEstimatable for Value {
    fn estimate_memory(&self) -> usize {
        match self {
            // ... 现有匹配
            Value::Json(json) => std::mem::size_of::<Value>() + json.estimated_size(),
            Value::JsonB(jsonb) => std::mem::size_of::<Value>() + jsonb.estimated_size(),
        }
    }
}
```

---

## 8. 索引支持（未来扩展）

### 8.1 GIN 索引设计

```rust
/// JSONB GIN 索引支持
pub struct JsonbGinIndex {
    /// 存储键路径到文档 ID 的映射
    entries: HashMap<String, Vec<u64>>,
}

impl JsonbGinIndex {
    /// 索引 JSONB 文档
    pub fn index(&mut self, doc_id: u64, jsonb: &JsonB) {
        self.extract_keys(&jsonb.as_value(), "", doc_id);
    }

    fn extract_keys(&mut self, value: &JsonValue, path: &str, doc_id: u64) {
        match value {
            JsonValue::Object(map) => {
                for (key, val) in map {
                    let new_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    self.entries
                        .entry(new_path.clone())
                        .or_default()
                        .push(doc_id);
                    self.extract_keys(val, &new_path, doc_id);
                }
            }
            JsonValue::Array(arr) => {
                for (idx, val) in arr.iter().enumerate() {
                    let new_path = format!("{}[{}]", path, idx);
                    self.extract_keys(val, &new_path, doc_id);
                }
            }
            _ => {
                // 叶子节点，索引值
                let key = format!("{}={}", path, value);
                self.entries.entry(key).or_default().push(doc_id);
            }
        }
    }
}
```

---

## 9. 实现文件清单

| 文件                              | 说明                | 优先级 |
| --------------------------------- | ------------------- | ------ |
| `src/core/value/json.rs`          | JSON/JSONB 类型定义 | P0     |
| `src/core/value/value_def.rs`     | 扩展 Value 枚举     | P0     |
| `src/core/value/value_compare.rs` | 比较逻辑            | P0     |
| `src/core/types/mod.rs`           | 扩展 DataType 枚举  | P0     |
| `src/core/value/memory.rs`        | 内存估算            | P1     |
| `src/query/parser/`               | JSON 路径表达式解析 | P2     |
| `src/index/jsonb_gin.rs`          | GIN 索引支持        | P3     |

---

## 10. 依赖检查

当前 `Cargo.toml` 已包含必要的依赖：

```toml
serde_json = "1.0.145"  # ✅ 已存在
serde = { version = "1.0.228", features = ["derive"] }  # ✅ 已存在
```

无需添加新依赖。

---

## 11. 测试建议

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_creation() {
        let json = Json::from_str(r#"{"name": "test", "value": 123}"#).unwrap();
        assert_eq!(json.get_path("name").unwrap(), Some(JsonValue::String("test".to_string())));
        assert_eq!(json.get_path("value").unwrap(), Some(JsonValue::Number(123.into())));
    }

    #[test]
    fn test_jsonb_normalization() {
        let jsonb1 = JsonB::from_str(r#"{"b": 1, "a": 2}"#).unwrap();
        let jsonb2 = JsonB::from_str(r#"{"a": 2, "b": 1}"#).unwrap();
        assert_eq!(jsonb1, jsonb2); // 键顺序不影响相等性
    }

    #[test]
    fn test_json_to_jsonb_conversion() {
        let json = Json::from_str(r#"[1, 2, 3]"#).unwrap();
        let jsonb = json.to_jsonb().unwrap();
        assert_eq!(jsonb.array_len(), Some(3));
    }

    #[test]
    fn test_nested_path_access() {
        let json = Json::from_str(r#"{"level1": {"level2": {"value": 42}}}"#).unwrap();
        assert_eq!(
            json.get_path("level1.level2.value").unwrap(),
            Some(JsonValue::Number(42.into()))
        );
    }
}
```

---

## 12. 总结

JSON/JSONB 类型的实现方案：

1. **Json**: 文本存储，保留原始格式，适合存储和传输
2. **JsonB**: 二进制存储，解析后存储，适合查询和索引
3. **依赖**: 复用已有的 `serde_json`，无需新依赖
4. **关键特性**:
   - 路径查询支持（`->`, `->>` 操作符）
   - 规范化存储（JSONB 排序键、去除空白）
   - 完整的比较和哈希支持
   - 与现有 Value 类型系统的无缝集成

**建议实施顺序**:

1. 创建 `json.rs` 模块
2. 扩展 Value 和 DataType 枚举
3. 实现比较和转换逻辑
4. 添加查询操作符支持
5. 实现 GIN 索引（可选）
