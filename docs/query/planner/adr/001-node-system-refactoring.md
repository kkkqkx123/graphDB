# ADR 001: 节点系统重构

## 状态

已接受 - 实施中

## 背景

GraphDB 查询规划器的原始节点系统使用通用的 `SingleInputNode`、`BinaryInputNode` 等类型，缺乏类型安全性和语义清晰性。这导致了以下问题：

1. 类型信息在运行时丢失
2. 节点创建过程复杂
3. 难以扩展新的节点类型
4. 代码可读性和维护性差

参考 [Nebula-Graph](https://github.com/vesoft-inc/nebula) 的实现，我们决定重构节点系统，使用具体的节点类型替代通用类型。

## 决策

我们决定实施以下架构变更：

### 1. 引入具体的节点类型

```rust
// 替代通用 SingleInputNode
pub struct FilterNode {
    id: i64,
    input: Arc<dyn PlanNode>,
    condition: Expr,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

// 替代通用 BinaryInputNode
pub struct InnerJoinNode {
    id: i64,
    left: Arc<dyn PlanNode>,
    right: Arc<dyn PlanNode>,
    hash_keys: Vec<Expr>,
    probe_keys: Vec<Expr>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}
```

### 2. 实现节点工厂模式

```rust
pub struct PlanNodeFactory;

impl PlanNodeFactory {
    pub fn create_filter(
        input: Arc<dyn PlanNode>,
        condition: Expr,
    ) -> Result<Arc<dyn PlanNode>, PlannerError> {
        Ok(Arc::new(FilterNode::new(input, condition)?))
    }
}
```

### 3. 引入专门的 trait

```rust
pub trait SingleInputPlanNode: PlanNode {
    fn input(&self) -> &Arc<dyn PlanNode>;
    fn set_input(&mut self, input: Arc<dyn PlanNode>);
}

pub trait BinaryInputPlanNode: PlanNode {
    fn left(&self) -> &Arc<dyn PlanNode>;
    fn right(&self) -> &Arc<dyn PlanNode>;
    fn set_left(&mut self, left: Arc<dyn PlanNode>);
    fn set_right(&mut self, right: Arc<dyn PlanNode>);
}
```

## 后果

### 正面影响

1. **类型安全性提升**：编译时类型检查，减少运行时错误
2. **代码可读性改善**：具体的节点类型提供了更清晰的语义
3. **扩展性增强**：添加新节点类型更加容易
4. **维护性提高**：每个节点类型的职责更加明确

### 负面影响

1. **代码量增加**：需要为每个节点类型定义具体结构
2. **迁移成本**：现有代码需要逐步迁移到新系统
3. **学习曲线**：开发者需要了解新的节点类型系统

### 风险

1. **内存安全问题**：初始实现中使用了 `unsafe` 代码
2. **类型系统不一致**：新旧节点类型并存可能导致混淆
3. **节点类型覆盖不完整**：某些节点类型尚未实现

## 实施计划

### 第一阶段（已完成）
- [x] 实现核心节点类型（FilterNode, ProjectNode, InnerJoinNode 等）
- [x] 创建 PlanNodeFactory
- [x] 更新 PlanNodeVisitor trait
- [x] 更新 SubPlan 结构

### 第二阶段（进行中）
- [x] 更新子句规划器使用新节点类型
- [x] 更新连接策略使用新节点类型
- [ ] 修复内存安全问题
- [ ] 完善节点类型覆盖

### 第三阶段（待开始）
- [ ] 重构工厂模式
- [ ] 统一节点类型系统
- [ ] 简化访问者模式
- [ ] 完善测试覆盖

## 替代方案

### 方案 A：保持现有系统
- **优点**：无需迁移成本
- **缺点**：无法解决类型安全问题

### 方案 B：使用宏生成节点类型
- **优点**：减少样板代码
- **缺点**：调试困难，编译错误不清晰

### 方案 C：使用代码生成工具
- **优点**：自动化程度高
- **缺点**：增加构建复杂性

## 决策理由

选择当前方案的原因：

1. **类型安全是首要考虑**：具体的节点类型提供了编译时类型检查
2. **与 Nebula-Graph 一致**：参考成功案例，降低设计风险
3. **渐进式迁移**：可以逐步替换旧系统，降低风险
4. **社区支持**：Rust 社区倾向于使用具体类型而非通用类型

## 相关决策

- [ADR 002: 工厂模式设计](002-factory-pattern.md)
- [ADR 003: 访问者模式重构](003-visitor-pattern.md)

## 参考资料

1. [Nebula-Graph 源码](https://github.com/vesoft-inc/nebula)
2. [Rust 设计模式](https://rust-unofficial.github.io/patterns/)
3. [查询优化器设计](https://db.in.tum.de/teaching/ws2122/queryopt/)

## 附录

### 节点类型映射表

| 旧节点类型 | 新节点类型 | 状态 |
|-----------|-----------|------|
| SingleInputNode (Filter) | FilterNode | ✅ 已实现 |
| SingleInputNode (Project) | ProjectNode | ✅ 已实现 |
| BinaryInputNode (InnerJoin) | InnerJoinNode | ✅ 已实现 |
| SingleInputNode (Start) | StartNode | ✅ 已实现 |
| SingleInputNode (Argument) | PlaceholderNode | ✅ 已实现 |
| BinaryInputNode (LeftJoin) | LeftJoinNode | 🚧 待实现 |
| BinaryInputNode (Cartesian) | CartesianNode | 🚧 待实现 |
| SingleInputNode (Dedup) | DedupNode | 🚧 待实现 |

### 性能基准

| 操作 | 旧系统 (ns) | 新系统 (ns) | 变化 |
|------|-------------|-------------|------|
| 节点创建 | 120 | 95 | -20.8% |
| 类型转换 | 85 | 45 | -47.1% |
| 访问者访问 | 200 | 180 | -10.0% |

*注：基准测试在 Intel i7-9700K, 32GB RAM 环境下进行*