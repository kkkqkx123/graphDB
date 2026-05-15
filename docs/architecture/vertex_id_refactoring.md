# VertexId 类型统一重构方案

## 背景

原系统中存在多种顶点ID表示方式：
- `u64` 类型别名
- `Value::Int(i64)` / `Value::BigInt(i64)` 
- `Value::String(String)`
- `Box<Value>` 在 Edge 结构中

这导致：
1. 类型不匹配错误频发
2. 存储层与查询层集成困难
3. 代码复杂度高，转换逻辑分散

## 核心设计决策

### VertexId 统一为字节串结构体

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VertexId(Vec<u8>);

impl VertexId {
    pub fn from_int64(id: i64) -> Self {
        VertexId(id.to_be_bytes().to_vec())
    }
    
    pub fn from_string(s: impl Into<String>) -> Self {
        VertexId(s.into().into_bytes())
    }
    
    pub fn as_int64(&self) -> Option<i64> {
        if self.0.len() == 8 {
            let arr: [u8; 8] = self.0[..].try_into().ok()?;
            Some(i64::from_be_bytes(arr))
        } else {
            None
        }
    }
    
    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.0).ok()
    }
}
```

**优势**：
- 统一表示：整数ID和字符串ID使用同一类型
- 存储友好：直接对应RocksDB的字节串键
- 类型安全：编译期保证ID类型正确
- 性能优化：避免运行时类型检查

### Edge 结构修改

```rust
pub struct Edge {
    pub src: VertexId,      // 原 Box<Value>
    pub dst: VertexId,      // 原 Box<Value>
    pub edge_type: String,
    pub ranking: i64,       // 新增字段
    pub id: i64,
    pub props: HashMap<String, Value>,
}
```

### Vertex 结构修改

```rust
pub struct Vertex {
    pub vid: VertexId,      // 原 Box<Value>
    pub id: i64,
    pub tags: Vec<Tag>,
    pub properties: HashMap<String, Value>,
}
```

## 已完成的底层迁移

### 1. 核心类型定义
- ✅ `src/core/types/storage_ids.rs` - VertexId 结构体实现
- ✅ `src/core/vertex_edge_path.rs` - Vertex/Edge 类型更新

### 2. 存储层接口
- ✅ `src/storage/interface/storage_client.rs` - StorageClient trait 签名更新
- ✅ `src/storage/engine/graph_storage/reader.rs` - 读取器实现
- ✅ `src/storage/engine/graph_storage/writer.rs` - 写入器实现
- ✅ `src/storage/engine/graph_storage/mod.rs` - GraphStorage 实现
- ✅ `src/storage/engine/sync_wrapper.rs` - 同步包装器
- ✅ `src/storage/test_mock.rs` - 测试 Mock 实现

### 3. 索引系统
- ✅ `src/storage/index/primary/edge_id_index.rs` - EdgeLocation 结构
- ✅ `src/storage/index/primary/degree_index.rs` - 迭代器适配

### 4. 边存储
- ✅ `src/storage/edge/mod.rs` - Nbr/ImmutableNbr 结构（移除 Copy trait）

## 剩余迁移任务

### 高优先级（影响核心功能）

#### 1. 查询执行器层（约300+错误）
**文件范围**：`src/query/executor/`

**主要修改模式**：
```rust
// 旧代码
let vid = Value::Int(1);
storage.get_vertex(space, &vid)?;

// 新代码
let vid = VertexId::from_int64(1);
storage.get_vertex(space, &vid)?;
```

**涉及模块**：
- `data_modification/` - 插入、更新、删除操作
- `graph_operations/` - 图遍历算法（BFS、DFS、A*等）
- `admin/` - 管理操作（分析、统计）

#### 2. API 层（约50+错误）
**文件范围**：`src/api/`

**主要修改模式**：
```rust
// 旧代码
Vertex::new(Value::Int(id), tags)
Vertex::with_vid(Value::Int(1))

// 新代码
Vertex::new(VertexId::from_int64(id), tags)
Vertex::with_vid(VertexId::from_int64(1))
```

**涉及模块**：
- `api/core/batch.rs` - 批量操作
- `api/embedded/c_api/` - C API 绑定
- `api/server/` - 服务器 API

### 中优先级（影响测试和工具）

#### 3. 测试代码（约100+错误）
**文件范围**：各模块的 `#[cfg(test)]` 块

**修改模式**：将所有测试中的 `Value::Int(n)` 替换为 `VertexId::from_int64(n)`

#### 4. 工具函数
**文件范围**：`src/core/npath.rs` 等

**修改内容**：
- `contains_vertex()` 参数类型
- `contains_edge()` 参数类型
- `collect_vertex_ids()` 返回类型

### 低优先级（不影响核心功能）

#### 5. 文档和示例
- 更新 API 文档中的示例代码
- 更新 README 中的使用示例

## 迁移策略建议

### 阶段一：核心路径（当前）
1. 保持底层存储层正确迁移 ✅
2. 确保核心类型定义完整 ✅
3. 验证存储接口一致性 ✅

### 阶段二：查询层迁移
1. 创建辅助函数简化迁移：
```rust
fn value_to_vertex_id(value: &Value) -> Option<VertexId> {
    match value {
        Value::Int(i) => Some(VertexId::from_int64(*i as i64)),
        Value::BigInt(i) => Some(VertexId::from_int64(*i)),
        Value::String(s) => Some(VertexId::from_string(s)),
        _ => None,
    }
}
```

2. 批量替换模式：
   - `Value::Int(n)` → `VertexId::from_int64(n)`
   - `Value::String(s)` → `VertexId::from_string(s)`
   - `*vid` → `vid.clone()`
   - `vid.as_ref()` → `&vid`

### 阶段三：API 层迁移
1. 在 API 边界添加类型转换
2. 保持外部接口兼容性（可选）
3. 更新 API 文档

### 阶段四：测试和验证
1. 修复所有测试用例
2. 运行完整测试套件
3. 性能基准测试

## 兼容性考虑

### 不提供向后兼容
根据项目要求，当前处于开发阶段，不考虑向后兼容：
- 未迁移的代码将产生编译错误
- 这有助于发现所有需要修改的地方
- 避免维护多套类型系统

### 迁移辅助工具
建议创建临时辅助模块：
```rust
// src/core/types/vertex_id_compat.rs (临时)
impl VertexId {
    pub fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Int(i) => Some(Self::from_int64(*i as i64)),
            Value::BigInt(i) => Some(Self::from_int64(*i)),
            Value::String(s) => Some(Self::from_string(s)),
            _ => None,
        }
    }
    
    pub fn to_value(&self) -> Value {
        if let Some(i) = self.as_int64() {
            Value::BigInt(i)
        } else if let Some(s) = self.as_str() {
            Value::String(s.to_string())
        } else {
            Value::Null(NullType::Null)
        }
    }
}
```

## 性能影响分析

### 正面影响
1. **减少内存分配**：VertexId 是小对象（Vec<u8>），比 Box<Value> 更轻量
2. **减少类型检查**：编译期确定类型，无需运行时 match
3. **存储效率**：直接序列化为字节串，无需额外转换

### 潜在开销
1. **字符串ID场景**：每次访问需要 UTF-8 验证（as_str()）
2. **整数ID场景**：需要 8 字节转换（as_int64()）

### 优化建议
1. 对于纯整数ID场景，可考虑特化实现
2. 缓存常用ID的字符串表示
3. 在热路径上避免重复转换

## 总结

本次重构将 VertexId 统一为字节串结构体，从根本上解决了类型不一致问题。底层存储层已完成正确迁移，剩余工作主要在上层应用代码的类型适配。

**关键成果**：
- ✅ 核心类型系统统一
- ✅ 存储层接口一致
- ✅ Edge ranking 字段支持
- ✅ 底层实现正确迁移

**后续工作**：
- 查询执行器层迁移（约300+错误）
- API 层迁移（约50+错误）
- 测试代码修复（约100+错误）
