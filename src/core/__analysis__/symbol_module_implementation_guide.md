# 符号表模块修改实施步骤

## 概述

本文档提供 `src/core/symbol` 模块修改的详细实施步骤，确保修改过程安全、可控。

## 准备工作

### 1. 代码备份
```bash
# 创建备份分支
git checkout -b backup/symbol-module-analysis

# 提交当前状态
git add .
git commit -m "Backup: current symbol module state"

# 回到主分支继续工作
git checkout main
```

### 2. 环境检查
```bash
# 确保编译环境正常
cargo check

# 运行现有测试
cargo test --lib core::symbol

# 检查是否有未提交的更改
git status
```

## 详细实施步骤

### 步骤 1：创建 PlanNodeRef 文件

**1.1 创建新文件**
```bash
touch src/core/plan_node_ref.rs
```

**1.2 添加文件内容**
```rust
//! 计划节点引用模块
//! 
//! 提供轻量级的计划节点引用，用于查询计划中的节点标识和依赖跟踪
//! 这是一个核心基础类型，供整个查询引擎使用

use std::fmt;

/// 计划节点引用
///
/// 用于在查询处理过程中轻量级地引用计划节点，避免存储完整的节点对象
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlanNodeRef {
    /// 节点标识符
    pub id: String,
    /// 节点ID（来自 PlanNodeEnum.id()）
    pub node_id: i64,
}

impl PlanNodeRef {
    /// 创建新的计划节点引用
    pub fn new(id: String, node_id: i64) -> Self {
        Self { id, node_id }
    }

    /// 从节点ID创建引用
    pub fn from_node_id(id: String, node_id: i64) -> Self {
        Self { id, node_id }
    }

    /// 获取节点标识符
    pub fn id(&self) -> &str {
        &self.id
    }

    /// 获取节点ID
    pub fn node_id(&self) -> i64 {
        self.node_id
    }

    /// 获取节点类型名称
    pub fn type_name(&self) -> &'static str {
        "PlanNode"
    }
}

impl fmt::Display for PlanNodeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PlanNodeRef({}, {})", self.id, self.node_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_node_ref_creation() {
        let node_ref = PlanNodeRef::new("node_1".to_string(), 42);
        assert_eq!(node_ref.id(), "node_1");
        assert_eq!(node_ref.node_id(), 42);
    }

    #[test]
    fn test_plan_node_ref_display() {
        let node_ref = PlanNodeRef::new("node_1".to_string(), 42);
        assert_eq!(format!("{}", node_ref), "PlanNodeRef(node_1, 42)");
    }

    #[test]
    fn test_plan_node_ref_equality() {
        let ref1 = PlanNodeRef::new("node_1".to_string(), 42);
        let ref2 = PlanNodeRef::new("node_1".to_string(), 42);
        let ref3 = PlanNodeRef::new("node_2".to_string(), 42);

        assert_eq!(ref1, ref2);
        assert_ne!(ref1, ref3);
    }
}
```

**1.3 验证编译**
```bash
cargo check
```

### 步骤 2：更新核心模块导出

**2.1 修改 src/core/mod.rs**
```rust
// 在文件开头添加
pub mod plan_node_ref;

// 在导出部分添加
pub use plan_node_ref::*;
```

**2.2 验证编译**
```bash
cargo check
```

### 步骤 3：更新依赖跟踪器

**3.1 修改 src/core/symbol/dependency_tracker.rs**

找到导入部分，替换为：
```rust
use crate::core::PlanNodeRef;
```

**3.2 验证编译**
```bash
cargo check
```

### 步骤 4：更新符号表

**4.1 修改 src/core/symbol/symbol_table.rs**

找到导入部分，替换为：
```rust
use crate::core::PlanNodeRef;
```

**4.2 简化 SymbolTable 结构**

找到 `SymbolTable` 结构定义，修改为：
```rust
#[derive(Debug, Clone)]
pub struct SymbolTable {
    symbols: Arc<RwLock<HashMap<String, Symbol>>>,
    dependency_tracker: Arc<RwLock<DependencyTracker>>,
}
```

**4.3 更新 new() 方法**
```rust
impl SymbolTable {
    /// 创建新的符号表
    pub fn new() -> Self {
        Self {
            symbols: Arc::new(RwLock::new(HashMap::new())),
            dependency_tracker: Arc::new(RwLock::new(DependencyTracker::new())),
        }
    }
    // ... 其余方法保持不变
}
```

**4.4 移除对象池相关方法**

删除以下方法：
- `obj_pool()`
- `allocate_from_pool()`
- `deallocate_from_pool()`

**4.5 验证编译**
```bash
cargo check
```

### 步骤 5：移除旧的 plan_node_ref.rs

**5.1 删除文件**
```bash
rm src/core/symbol/plan_node_ref.rs
```

**5.2 更新 symbol/mod.rs**
```rust
//! 符号表模块 - 管理查询中的变量和别名
//! 对应原C++中的context/Symbols.h

pub mod dependency_tracker;
pub mod symbol_table;
// 移除：pub mod plan_node_ref;

pub use dependency_tracker::*;
pub use symbol_table::*;
// 移除：pub use plan_node_ref::*;
```

**5.3 验证编译**
```bash
cargo check
```

### 步骤 6：更新测试

**6.1 移除对象池测试**

在 `src/core/symbol/symbol_table.rs` 中，找到并删除 `test_object_pool` 测试函数。

**6.2 验证测试**
```bash
cargo test --lib core::symbol
```

### 步骤 7：更新文档

**7.1 更新 README.md**
```markdown
# 符号表模块 (Symbol Table Module)

## 概述

符号表模块是图数据库查询处理系统中的核心组件，用于管理查询中的变量、别名和符号，并跟踪它们之间的依赖关系。

## 文件结构

```
src/core/symbol/
├── mod.rs              # 模块定义和导出
├── symbol_table.rs     # 符号表主实现
├── dependency_tracker.rs # 依赖关系跟踪器
└── README.md          # 本说明文档

src/core/
└── plan_node_ref.rs   # 计划节点引用（被依赖跟踪器使用）
```

## 主要变更
- 移除了对象池功能（职责分离）
- PlanNodeRef 移至 src/core/plan_node_ref.rs（位置优化）
```

**7.2 添加变更记录**
在 README.md 末尾添加：
```markdown
## 版本历史

### v2.0 (当前)
- 优化：PlanNodeRef 移至核心模块，提升复用性
- 移除：对象池功能（职责分离，简化设计）

### v1.0
- 初始实现，包含符号表、依赖跟踪、对象池功能
```

## 验证清单

### 编译检查
- [ ] `cargo check` 通过
- [ ] `cargo build` 通过
- [ ] `cargo build --release` 通过

### 测试检查
- [ ] `cargo test --lib core::symbol` 通过
- [ ] `cargo test --lib core::plan_node_ref` 通过
- [ ] 所有相关测试通过

### 代码质量
- [ ] 无警告（`cargo check`）
- [ ] 格式化正确（`cargo fmt`）
- [ ] 文档完整（`cargo doc`）

### 功能验证
- [ ] PlanNodeRef 可以正常导入和使用
- [ ] SymbolTable 功能完整
- [ ] DependencyTracker 正常工作
- [ ] 相关模块能正常使用新位置

## 回滚步骤

如果发现问题需要回滚：

```bash
# 1. 回滚到备份分支
git checkout backup/symbol-module-analysis

# 2. 强制回滚主分支
git branch -D main
git checkout -b main

# 3. 重新推送（如果需要）
git push origin main --force
```

## 后续工作

### 立即检查
1. **验证所有导入**：检查项目中所有使用 `PlanNodeRef` 的地方
2. **运行完整测试**：确保没有破坏其他功能
3. **性能检查**：如果有基准测试，验证性能没有下降

### 中期优化
1. **错误处理统一**：考虑统一 SymbolTable 的错误处理
2. **API 简化**：评估是否可以进一步简化委托方法
3. **文档完善**：更新相关文档和注释

### 长期规划
1. **变量系统整合**：评估 BasicValidationContext 和 SymbolTable 的整合
2. **模块职责优化**：持续优化模块职责划分
3. **架构演进**：根据实际需求调整架构设计

## 注意事项

⚠️ **重要提醒**:
1. 每个步骤都要验证编译通过
2. 保持小步提交，便于回滚
3. 及时运行测试，确保功能正确
4. 注意更新所有相关的导入路径
5. 保持代码格式一致

## 总结

通过以上步骤，可以安全、有序地完成符号表模块的优化工作。整个过程遵循最小改动原则，确保风险可控，同时解决当前的主要设计问题。