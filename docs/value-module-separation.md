# Value 模块职责分离方案

## 问题分析

当前 `src/core/value.rs` 文件包含过多职责，违反单一职责原则：
- **文件规模**：约800行代码
- **职责混杂**：类型定义、比较逻辑、算术运算、类型转换等
- **维护困难**：修改比较逻辑可能影响核心类型定义

## 分离方案设计

### 目标原则
1. **零运行时开销** - 使用Rust模块系统，避免动态分发
2. **编译时优化** - 保持内联优化能力
3. **职责清晰** - 每个模块单一职责
4. **接口稳定** - 保持现有API不变

### 模块结构设计

```
src/core/
├── value/
│   ├── mod.rs              # 主模块，重新导出所有子模块
│   ├── types.rs            # 核心类型定义 (Value, ValueTypeDef, 辅助类型)
│   ├── comparison.rs       # 比较逻辑 (Ord, PartialEq, Hash trait实现)
│   ├── operations.rs       # 算术运算 (add, sub等)
│   └── conversion.rs       # 类型转换 (From trait实现)
└── vertex_edge_path.rs     # 保持不变
```

### 详细分离方案

#### 1. `types.rs` - 核心类型定义
**职责**：定义核心数据类型，不包含任何逻辑
```rust
// Value 枚举定义
pub enum Value { /* ... */ }

// 辅助类型定义
pub struct DateValue { /* ... */ }
pub struct TimeValue { /* ... */ }
// ... 其他辅助类型

// 类型枚举定义
pub enum ValueTypeDef { /* ... */ }
```

#### 2. `comparison.rs` - 比较逻辑
**职责**：实现所有比较相关的trait
```rust
impl PartialEq for Value { /* ... */ }
impl Ord for Value { /* ... */ }
impl Hash for Value { /* ... */ }

// 所有比较辅助函数
fn cmp_date(a: &DateValue, b: &DateValue) -> CmpOrdering { /* ... */ }
// ... 其他比较函数
```

#### 3. `operations.rs` - 算术运算
**职责**：实现数学运算功能
```rust
impl Value {
    pub fn add(&self, other: &Value) -> Result<Value, String> { /* ... */ }
    pub fn sub(&self, other: &Value) -> Result<Value, String> { /* ... */ }
    // ... 其他运算方法
}
```

#### 4. `conversion.rs` - 类型转换
**职责**：实现From trait转换
```rust
impl From<bool> for Value { /* ... */ }
impl From<i64> for Value { /* ... */ }
// ... 其他转换实现
```

#### 5. `mod.rs` - 主模块
**职责**：重新导出所有功能，保持API兼容
```rust
pub mod types;
pub mod comparison;
pub mod operations;
pub mod conversion;

// 重新导出所有公共类型和功能
pub use types::*;
pub use comparison::*;
pub use operations::*;
pub use conversion::*;
```

## 技术优势

### 1. 编译时优化
- **内联优化**：所有函数保持内联能力
- **零抽象开销**：模块分离在编译时完成
- **增量编译**：修改单个模块不影响其他部分

### 2. 维护性提升
- **职责清晰**：每个模块单一职责
- **测试友好**：可以单独测试比较逻辑
- **扩展性**：新增功能在对应模块添加

### 3. 性能保证
- **无运行时开销**：纯静态模块分离
- **类型安全**：Rust编译器保证类型安全
- **内存布局不变**：Value枚举内存布局保持不变

## 实施步骤

1. **创建目录结构**：建立 `src/core/value/` 目录
2. **分离代码**：按职责将代码拆分到对应文件
3. **更新mod.rs**：创建新的模块结构
4. **更新引用**：修改其他文件的导入语句
5. **验证测试**：确保所有功能正常工作

## 预期效果

- **编译时间**：减少增量编译时间
- **代码可读性**：每个文件职责明确
- **维护成本**：降低修改风险
- **团队协作**：不同开发者可并行工作

此方案在保持零运行时开销的前提下，实现了职责的清晰分离，符合Rust的最佳实践。