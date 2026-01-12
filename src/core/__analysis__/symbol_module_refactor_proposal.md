# 符号表模块修改方案

## 概述

基于对 `src/core/symbol` 模块的分析，制定具体的修改实施方案。遵循最小改动原则，优先解决关键问题。

## 修改目标

1. **解决模块位置不当问题**：将 `PlanNodeRef` 移至合适位置
2. **简化职责**：移除不相关的对象池功能
3. **保持兼容性**：最小化 API 变更
4. **提升可维护性**：优化代码结构

## 具体修改方案

### 方案一：最小改动（推荐）

#### 步骤 1：移动 PlanNodeRef

**创建新文件**：
```bash
# 创建新的核心基础文件
touch src/core/plan_node_ref.rs
```

**移动内容**：
```rust
// src/core/plan_node_ref.rs
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

**更新模块导出**：
```rust
// src/core/mod.rs
pub mod allocator;
pub mod collect_n_succeeded;
pub mod cord;
pub mod error;
pub mod murmur;
pub mod plan_node_ref;  // 新增
pub mod result;
pub mod schema;
pub mod signal_handler;
pub mod symbol;
pub mod type_utils;
pub mod value;
pub mod vertex_edge_path;
pub mod visitor;
pub mod visitor_state_enum;

// 新增的子模块
pub mod context;
pub mod context_traits;
pub mod types;

// 查询处理模块
pub mod query_pipeline_manager;

// 导出 plan_node_ref
pub use plan_node_ref::*;
```

#### 步骤 2：更新依赖

**更新 dependency_tracker.rs**：
```rust
// src/core/symbol/dependency_tracker.rs
// 修改导入路径
use crate::core::PlanNodeRef;
// 移除：use super::plan_node_ref::PlanNodeRef;
```

**更新 symbol_table.rs**：
```rust
// src/core/symbol/symbol_table.rs
// 修改导入路径
use crate::core::PlanNodeRef;
// 移除：use crate::core::symbol::plan_node_ref::PlanNodeRef;
```

#### 步骤 3：移除对象池功能

**简化 SymbolTable 结构**：
```rust
// src/core/symbol/symbol_table.rs
#[derive(Debug, Clone)]
pub struct SymbolTable {
    symbols: Arc<RwLock<HashMap<String, Symbol>>>,
    dependency_tracker: Arc<RwLock<DependencyTracker>>,
    // 移除 obj_pool 字段
}

impl SymbolTable {
    /// 创建新的符号表
    pub fn new() -> Self {
        Self {
            symbols: Arc::new(RwLock::new(HashMap::new())),
            dependency_tracker: Arc::new(RwLock::new(DependencyTracker::new())),
            // 移除 obj_pool 初始化
        }
    }
    
    // 移除以下方法：
    // - obj_pool()
    // - allocate_from_pool()
    // - deallocate_from_pool()
}
```

**更新相关测试**：
```rust
// 移除 test_object_pool 测试
#[cfg(test)]
mod tests {
    // ... 其他测试
    
    // 移除以下测试
    // #[test]
    // fn test_object_pool() { ... }
}
```

#### 步骤 4：更新文档

**更新 README.md**：
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

### 方案二：全面重构（备选）

如果需要更彻底的重构，可以考虑：

#### 1. 创建计划模块
```
src/core/plan/
├── mod.rs           # 计划模块定义
├── node_ref.rs      # 节点引用
├── node_types.rs    # 节点类型定义
└── plan_utils.rs    # 计划工具函数
```

#### 2. 简化 SymbolTable
```rust
pub struct SymbolTable {
    symbols: Arc<RwLock<HashMap<String, Symbol>>>,
}

impl SymbolTable {
    // 直接返回依赖跟踪器引用
    pub fn dependency_tracker(&self) -> &Arc<RwLock<DependencyTracker>> {
        &self.dependency_tracker
    }
}
```

#### 3. 统一错误处理
```rust
#[derive(Debug, thiserror::Error)]
pub enum SymbolError {
    #[error("Variable '{0}' not found")]
    VariableNotFound(String),
    
    #[error("Variable '{0}' already exists")]
    VariableAlreadyExists(String),
    
    #[error("Lock acquisition failed: {0}")]
    LockAcquisitionFailed(String),
    
    #[error("Dependency error: {0}")]
    DependencyError(String),
}

pub type Result<T> = std::result::Result<T, SymbolError>;
```

## 实施计划

### 第一阶段：立即实施（1-2天）
1. ✅ 创建分析文档（已完成）
2. 实施方案一的步骤1-3
3. 运行测试确保功能正常
4. 提交代码变更

### 第二阶段：验证优化（1天）
1. 检查所有使用 SymbolTable 的地方
2. 验证 PlanNodeRef 的导入路径
3. 性能测试（如果有基准测试）
4. 文档更新

### 第三阶段：长期规划（后续迭代）
1. 评估是否需要更全面的重构
2. 考虑错误处理统一
3. 评估变量管理系统整合

## 风险评估

### 低风险 ✅
- PlanNodeRef 移动：影响范围明确，编译器会捕获所有错误
- 移除 obj_pool：当前功能不完整，移除无副作用

### 中风险 ⚠️
- 导入路径更新：需要检查所有相关文件
- API 变更：需要更新调用代码

### 高风险 ❌
- 全面重构：影响范围广，不建议立即实施
- 变量系统整合：涉及多个模块，需要充分测试

## 回滚策略

1. **保留原始文件**：在修改前备份原始文件
2. **分步提交**：每个步骤单独提交，便于回滚
3. **测试覆盖**：确保所有修改都有测试覆盖

## 预期收益

### 短期收益
- ✅ 模块职责更加清晰
- ✅ 代码结构更加合理
- ✅ 降低模块间耦合

### 长期收益
- 🎯 提升代码可维护性
- 🎯 便于后续功能扩展
- 🎯 符合 Rust 最佳实践

## 结论

建议采用**方案一：最小改动**，能够快速解决当前的主要问题，风险较低，收益明显。全面重构可以在后续迭代中逐步考虑。