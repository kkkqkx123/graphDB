# PlanNode 架构改进提案

## 概述

基于对 `plan_node_enum.rs`、`plan_node_operations.rs` 和 `plan_node_traits.rs` 的分析，本提案提出了一系列改进措施，旨在解决当前实现中的不一致性和架构问题，提高代码质量和可维护性。

## 改进目标

1. **统一实现策略**：确保所有节点类型都有一致的实现方式
2. **简化架构**：减少不必要的复杂性和重复代码
3. **提高可维护性**：使添加新节点类型更加容易
4. **改善性能**：减少不必要的内存开销和计算成本

## 具体改进方案

### 1. 统一节点类型定义

#### 1.1 修改 PlanNodeEnum 定义
```rust
// 移除 Arc<T> 包装，统一使用直接类型
#[derive(Debug, Clone)]
pub enum PlanNodeEnum {
    // 基础节点类型
    Start(StartNode),
    Project(ProjectNode),
    // ... 其他基础节点
    
    // 管理节点类型 - 统一使用直接类型
    CreateUser(CreateUser),
    DropUser(DropUser),
    CreateRole(CreateRole),
    DropRole(DropRole),
    // ... 其他管理节点
}
```

#### 1.2 统一管理节点结构
确保所有管理节点都有相同的基础字段：
```rust
// 为所有管理节点添加基础字段
#[derive(Debug, Clone)]
pub struct CreateRole {
    pub id: i64,
    pub deps: Vec<PlanNodeEnum>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    // 特定字段
    pub role_name: String,
    pub if_not_exists: bool,
}
```

### 2. 重构 Trait 设计

#### 2.1 分层 Trait 设计
```rust
// 基础 trait，所有节点都必须实现
pub trait PlanNodeBase {
    fn id(&self) -> i64;
    fn name(&self) -> &'static str;
    fn cost(&self) -> f64;
    fn into_enum(self) -> PlanNodeEnum;
}

// 可选 trait，用于有输出变量的节点
pub trait PlanNodeWithOutput: PlanNodeBase {
    fn output_var(&self) -> Option<&Variable>;
    fn set_output_var(&mut self, var: Variable);
}

// 可选 trait，用于有列名的节点
pub trait PlanNodeWithColumns: PlanNodeBase {
    fn col_names(&self) -> &[String];
    fn set_col_names(&mut self, names: Vec<String>);
}

// 可选 trait，用于有依赖关系的节点
pub trait PlanNodeWithDependencies: PlanNodeBase {
    fn dependencies(&self) -> Vec<Box<PlanNodeEnum>>;
}

// 完整 trait，组合所有功能
pub trait PlanNode: PlanNodeBase + PlanNodeWithOutput + PlanNodeWithColumns + PlanNodeWithDependencies {}
```

#### 2.2 自动 Trait 实现
使用宏自动为节点类型实现 trait：
```rust
macro_rules! impl_plan_node_base {
    ($node_type:ty, $name:expr) => {
        impl PlanNodeBase for $node_type {
            fn id(&self) -> i64 { self.id }
            fn name(&self) -> &'static str { $name }
            fn cost(&self) -> f64 { self.cost }
            fn into_enum(self) -> PlanNodeEnum { PlanNodeEnum::$node_type(self) }
        }
    };
}

// 使用宏
impl_plan_node_base!(CreateRole, "CreateRole");
impl_plan_node_base!(CreateUser, "CreateUser");
```

### 3. 简化操作实现

#### 3.1 使用泛型减少重复代码
```rust
impl PlanNodeEnum {
    // 使用泛型方法减少重复
    pub fn get_base_info<T>(&self) -> Option<(i64, &'static str, f64)>
    where
        T: PlanNodeBase,
    {
        match self {
            // 使用 trait 对象
            node if node.as_any().downcast_ref::<T>().is_some() => {
                let node = node.as_any().downcast_ref::<T>().unwrap();
                Some((node.id(), node.name(), node.cost()))
            }
            _ => None,
        }
    }
}
```

#### 3.2 使用宏生成操作方法
```rust
macro_rules! impl_plan_node_enum_methods {
    () => {
        pub fn id(&self) -> i64 {
            match self {
                // 基础节点
                PlanNodeEnum::Start(node) => node.id(),
                PlanNodeEnum::Project(node) => node.id(),
                // ... 其他基础节点
                
                // 管理节点
                PlanNodeEnum::CreateUser(node) => node.id(),
                PlanNodeEnum::CreateRole(node) => node.id(),
                // ... 其他管理节点
            }
        }
        
        // 类似地实现其他方法...
    };
}

impl PlanNodeEnum {
    impl_plan_node_enum_methods!();
}
```

### 4. 完善访问者模式

#### 4.1 统一访问者接口
```rust
pub trait PlanNodeVisitor {
    type Result;
    
    // 基础节点访问方法
    fn visit_start(&mut self, node: &StartNode) -> Self::Result;
    fn visit_project(&mut self, node: &ProjectNode) -> Self::Result;
    // ... 其他基础节点
    
    // 管理节点访问方法
    fn visit_create_user(&mut self, node: &CreateUser) -> Self::Result;
    fn visit_create_role(&mut self, node: &CreateRole) -> Self::Result;
    // ... 其他管理节点
    
    // 默认实现
    fn default_result(&mut self) -> Self::Result {
        // 提供默认实现
    }
}
```

#### 4.2 自动生成访问者实现
```rust
impl PlanNodeEnum {
    pub fn accept<V>(&self, visitor: &mut V) -> V::Result
    where
        V: PlanNodeVisitor,
    {
        match self {
            // 基础节点
            PlanNodeEnum::Start(node) => visitor.visit_start(node),
            PlanNodeEnum::Project(node) => visitor.visit_project(node),
            // ... 其他基础节点
            
            // 管理节点
            PlanNodeEnum::CreateUser(node) => visitor.visit_create_user(node),
            PlanNodeEnum::CreateRole(node) => visitor.visit_create_role(node),
            // ... 其他管理节点
        }
    }
}
```

### 5. 解决循环依赖

#### 5.1 使用前向声明和 trait 对象
```rust
// 在 plan_node_traits.rs 中
pub trait PlanNodeBase {
    fn id(&self) -> i64;
    fn name(&self) -> &'static str;
    fn cost(&self) -> f64;
    fn into_enum(self) -> PlanNodeEnum;
    
    // 使用 trait 对象避免循环依赖
    fn dependencies(&self) -> Vec<&dyn PlanNodeBase>;
}

// 在 plan_node_enum.rs 中
impl PlanNodeBase for PlanNodeEnum {
    fn dependencies(&self) -> Vec<&dyn PlanNodeBase> {
        match self {
            PlanNodeEnum::Start(_) => vec![],
            PlanNodeEnum::Project(node) => vec![&node.input],
            // ... 其他实现
        }
    }
}
```

### 6. 改进工厂模式

#### 6.1 统一节点创建接口
```rust
pub trait PlanNodeFactory {
    fn create_node<T>(&self, node_type: T) -> PlanNodeEnum
    where
        T: Into<PlanNodeEnum>;
    
    fn create_with_id<T>(&self, node_type: T, id: i64) -> PlanNodeEnum
    where
        T: PlanNodeBase;
}

impl PlanNodeFactory for DefaultPlanNodeFactory {
    fn create_node<T>(&self, node_type: T) -> PlanNodeEnum
    where
        T: Into<PlanNodeEnum>,
    {
        node_type.into()
    }
    
    fn create_with_id<T>(&self, mut node_type: T, id: i64) -> PlanNodeEnum
    where
        T: PlanNodeBase,
    {
        // 设置 ID 并转换
        node_type.set_id(id);
        node_type.into_enum()
    }
}
```

## 实施计划

### 阶段 1：基础重构（1-2 周）
1. 统一所有管理节点的结构定义
2. 为所有管理节点实现 PlanNode trait
3. 修改 PlanNodeEnum 定义，移除 Arc<T> 包装

### 阶段 2：Trait 重构（2-3 周）
1. 设计并实现分层 trait 系统
2. 创建宏自动生成 trait 实现
3. 重构现有节点以使用新的 trait 系统

### 阶段 3：操作简化（1-2 周）
1. 使用宏简化 PlanNodeEnum 操作方法
2. 减少重复代码
3. 优化性能

### 阶段 4：访问者模式完善（1 周）
1. 完善访问者模式实现
2. 为所有节点类型添加访问方法
3. 测试访问者模式功能

### 阶段 5：测试和优化（1 周）
1. 编写全面的测试用例
2. 性能测试和优化
3. 文档更新

## 风险评估

### 高风险
1. **大规模重构**：可能影响现有代码的稳定性
2. **API 变更**：可能需要修改使用这些接口的代码

### 中风险
1. **性能影响**：新的 trait 系统可能影响性能
2. **学习成本**：开发人员需要学习新的架构

### 低风险
1. **向后兼容性**：可以通过适配器模式保持兼容
2. **渐进式迁移**：可以逐步迁移现有代码

## 成功指标

1. **代码重复率**：减少 50% 以上的重复代码
2. **新节点添加成本**：从修改 3-4 个文件减少到 1-2 个文件
3. **编译时间**：减少 20% 以上的编译时间
4. **运行时性能**：减少 10% 以上的内存使用

## 结论

通过实施这些改进措施，我们可以显著提高 PlanNode 系统的代码质量和可维护性。虽然重构需要一定的时间和精力，但长期来看，这将大大降低维护成本，提高开发效率，并为未来的功能扩展奠定良好的基础。

建议采用渐进式重构的方式，先解决最关键的不一致性问题，然后逐步完善架构设计。同时，应该保持充分的测试覆盖，确保重构过程中不会引入新的 bug。