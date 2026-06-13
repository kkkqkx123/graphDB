# LOOKUP 扫描类型优化：Range 性能问题

## 问题描述

当前 LOOKUP 查询的规划器（`LookupPlanner`）对所有非空的 `scan_limits` 统一使用 `ScanType::Range`，导致即使是最简单的等值查询（如 `WHERE name == 'Alice'`）也会触发**全表扫描 + 内存过滤**，而非高效的索引点查。

### 当前行为

```
lookup_planner.rs:113
    scan_type = if scan_limits.is_empty() {
        ScanType::Full
    } else {
        ScanType::Range    // 所有条件都走 Range
    };
```

### 问题

| 查询模式 | 当前行为 | 预期行为 |
|----------|----------|----------|
| `name == 'Alice'` | Range：扫描全部顶点，内存过滤 | Unique：一次索引点查 |
| `age > 34` | Range：扫描全部顶点，内存过滤 | Range：正确（需全扫描） |
| `name == 'Alice' AND age > 34` | Range：扫描全部顶点，内存过滤 | Range：正确（多条件） |
| `name LIKE 'A%'` | Range：扫描全部顶点，内存过滤 | Prefix：索引前缀扫描 |

对于等值查询，`ScanType::Unique` 能直接调用 `storage.lookup_index()` 完成 O(1) 索引查找，避免扫描全部数据。

## 根因分析

### 为什么一直用 Range？

查看 `IndexScanExecutor` 的 UNIQUE 分支（`search.rs:117-131`）：

```rust
"UNIQUE" => {
    if let Some(first_limit) = self.scan_limits.first() {
        let value = first_limit.begin_value.as_ref()
            .map(|v| Value::String(v.clone()))
            .unwrap_or(Value::Null(NullType::Null));
        storage.lookup_index(&space_name, &self.index_name, &value)
            .map_err(DBError::from)
    } else {
        Ok(Vec::new())
    }
}
```

UNIQUE 分支**只取 `first_scan_limit`**，无法处理多条件查询。例如 `WHERE name == 'Alice' AND age > 34` 有 2 个 `IndexLimit`，但 UNIQUE 只处理第一个。

### 为什么 UNIQUE 不能直接用于多条件？

`storage.lookup_index()` 的语义是：在指定索引中查找**精确匹配**的条目。对于多条件查询：
- 如果所有条件都在同一个索引列上（如 `age > 34 AND age < 50`），可以用 RANGE 扫描索引
- 如果条件跨多个列（如 `name == 'Alice' AND age > 34`），需要扫描后内存过滤

### 关键数据结构

```rust
// IndexLimit 已经内置了 scan_type 字段
pub struct IndexLimit {
    pub column: String,
    pub begin_value: Option<String>,
    pub end_value: Option<String>,
    pub include_begin: bool,
    pub include_end: bool,
    pub scan_type: ScanType,      // 已存在！
}

// 等值构造时已经标记为 Unique
pub fn equal(column: impl Into<String>, value: impl Into<String>) -> Self {
    Self {
        scan_type: ScanType::Unique,  // 正确标记
        ...
    }
}

// 范围构造时标记为 Range
pub fn range(...) -> Self {
    Self {
        scan_type: ScanType::Range,
        ...
    }
}
```

`extract_conditions` 在提取条件时已经为每个 `IndexLimit` 正确设置了 `scan_type`。规划器应该**尊重这个字段**，而非统一覆盖为 `Range`。

## 优化方案

### 核心思路

将 `LookupPlanner` 的 `scan_type` 决策从"统一 Range"改为"基于 `scan_limits` 内容自适应"：

1. **单条件等值** → `ScanType::Unique`（索引点查）
2. **单条件范围** → `ScanType::Range`（索引范围扫描 + 内存过滤）
3. **多条件** → `ScanType::Range`（全扫描 + 内存过滤）
4. **无条件** → `ScanType::Full`（全表扫描）

### 修改文件

#### 1. `crates/graphdb-query/src/query/executor/data_access/search.rs`

扩展 UNIQUE 分支支持多条件等值查询：

```rust
"UNIQUE" => {
    // 收集所有限制条件中的等值列
    let mut limit_columns: HashMap<String, String> = HashMap::new();
    for limit in &self.scan_limits {
        if let Some(ref val) = limit.begin_value {
            if limit.include_begin && limit.end_value.as_ref().map_or(false, |e| e == val) {
                limit_columns.insert(limit.column.clone(), val.clone());
            }
        }
    }

    if limit_columns.is_empty() {
        return Ok(Vec::new());
    }

    // 选择第一个等值列进行索引查找
    let first_limit = self.scan_limits.first().unwrap();
    let value = first_limit.begin_value.as_ref()
        .map(|v| Value::String(v.clone()))
        .unwrap_or(Value::Null(NullType::Null));

    let results = storage.lookup_index(&space_name, &self.index_name, &value)
        .map_err(DBError::from)?;

    // 如果有多余的等值条件，在内存中过滤
    // 后续的 Filter 节点会处理这些条件，这里直接返回即可
    Ok(results)
}
```

> **说明**：UNIQUE 路径返回候选结果后，后续的 `FilterNode`（由 `LookupPlanner` 在 `where_clause` 非空时自动添加）会在内存中应用剩余条件。因此 UNIQUE 路径只需对第一个等值列做索引查找即可。

#### 2. `crates/graphdb-query/src/query/planning/statements/dql/lookup_planner.rs`

修改 `scan_type` 决策逻辑：

```rust
// 替换原来的统一 Range 逻辑
scan_type = if scan_limits.is_empty() {
    ScanType::Full
} else if scan_limits.len() == 1 && scan_limits[0].scan_type == ScanType::Unique {
    // 单条件等值：使用索引点查
    ScanType::Unique
} else {
    // 多条件或范围查询：使用 Range
    ScanType::Range
};
```

### 性能影响

| 场景 | 优化前 | 优化后 |
|------|--------|--------|
| `LOOKUP ON person WHERE name == 'Alice'` | O(N) 全表扫描 | O(1) 索引点查 |
| `LOOKUP ON person WHERE age > 34` | O(N) 全表扫描 | O(N) 全表扫描（不变） |
| `LOOKUP ON person WHERE name == 'Alice' AND age > 34` | O(N) 全表扫描 | O(1) 索引点查 + 内存过滤 |

### 兼容性

- **不影响现有 Range 行为**：范围查询和多条件查询仍走 Range 路径
- **不影响 FULL 行为**：无条件查询仍走 FULL 路径
- **FilterNode 保持不变**：后续过滤节点会处理所有条件，UNIQUE 路径只负责缩小候选集
- **存储层无需修改**：`storage.lookup_index()` 已经支持正确的 VID 类型转换

### 实施步骤

1. 修改 `lookup_planner.rs` 的 `scan_type` 决策逻辑
2. 扩展 `search.rs` 的 UNIQUE 分支支持多条件
3. 添加单元测试验证等值查询走 Unique 路径
4. 运行全量 e2e 测试确认无回归

## 相关文件

| 文件 | 作用 |
|------|------|
| `crates/graphdb-query/src/query/planning/statements/dql/lookup_planner.rs` | LOOKUP 规划器，决定 scan_type |
| `crates/graphdb-query/src/query/executor/data_access/search.rs` | 索引扫描执行器，UNIQUE/RANGE/FULL 分支 |
| `crates/graphdb-query/src/query/planning/plan/core/nodes/access/index_scan.rs` | IndexLimit 和 ScanType 定义 |
| `crates/graphdb-storage/src/storage/engine/graph_storage/index_manager.rs` | 底层索引查找实现 |
| `crates/graphdb-storage/src/storage/index/vertex_index_manager.rs` | `lookup_tag_index_mvcc` 实现 |
