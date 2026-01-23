# 模块重构实施计划

## 基于职责分析的实施计划

根据 [module_responsibility_analysis.md](file:///d:/项目/database/graphDB/docs/module_responsibility_analysis.md) 的分析，本计划提供了具体的实施步骤。

## 阶段一：立即修复（第1周）

### 任务1.1：统一类型转换规则

**问题：** `type_inference.rs` 和 `type_system.rs` 的 `can_cast` 实现不一致

**修改文件：** `src/query/validator/strategies/type_inference.rs`

**修改位置：** 第215-222行

**修改前：**
```rust
pub fn can_cast(&self, from: &ValueTypeDef, to: &ValueTypeDef) -> bool {
    match (from, to) {
        (ValueTypeDef::Int, ValueTypeDef::Float) => true,
        (ValueTypeDef::Int, ValueTypeDef::String) => true,
        (ValueTypeDef::Float, ValueTypeDef::String) => true,
        (ValueTypeDef::Bool, ValueTypeDef::String) => true,
        (a, b) => a == b,
    }
}
```

**修改后：**
```rust
pub fn can_cast(&self, from: &ValueTypeDef, to: &ValueTypeDef) -> bool {
    use crate::core::TypeUtils;
    TypeUtils::can_cast(from, to)
}
```

**验证：**
```rust
#[test]
fn test_can_cast_consistency() {
    let type_inference = TypeInference::new();
    
    // 验证与 TypeUtils 的一致性
    assert_eq!(
        type_inference.can_cast(&ValueTypeDef::String, &ValueTypeDef::Int),
        TypeUtils::can_cast(&ValueTypeDef::String, &ValueTypeDef::Int)
    );
}
```

### 任务1.2：增强 type_system.rs

**问题：** `type_system.rs` 缺少基础类型辅助方法

**修改文件：** `src/core/type_system.rs`

**添加方法：**
```rust
impl TypeUtils {
    /// 获取类型的字符串表示
    pub fn type_to_string(type_def: &ValueTypeDef) -> String {
        match type_def {
            ValueTypeDef::Empty => "empty".to_string(),
            ValueTypeDef::Null => "null".to_string(),
            ValueTypeDef::Bool => "bool".to_string(),
            ValueTypeDef::Int | ValueTypeDef::Int8 | ValueTypeDef::Int16 | 
            ValueTypeDef::Int32 | ValueTypeDef::Int64 => "int".to_string(),
            ValueTypeDef::Float | ValueTypeDef::Double => "float".to_string(),
            ValueTypeDef::String => "string".to_string(),
            ValueTypeDef::Date => "date".to_string(),
            ValueTypeDef::Time => "time".to_string(),
            ValueTypeDef::DateTime => "datetime".to_string(),
            ValueTypeDef::Vertex => "vertex".to_string(),
            ValueTypeDef::Edge => "edge".to_string(),
            ValueTypeDef::Path => "path".to_string(),
            ValueTypeDef::List => "list".to_string(),
            ValueTypeDef::Map => "map".to_string(),
            ValueTypeDef::Set => "set".to_string(),
            ValueTypeDef::Geography => "geography".to_string(),
            ValueTypeDef::Duration => "duration".to_string(),
            ValueTypeDef::DataSet => "dataset".to_string(),
        }
    }
    
    /// 检查类型是否可以用于索引
    pub fn is_indexable_type(type_def: &ValueTypeDef) -> bool {
        match type_def {
            ValueTypeDef::Bool => true,
            ValueTypeDef::Int => true,
            ValueTypeDef::Float => true,
            ValueTypeDef::String => true,
            ValueTypeDef::DateTime => true,
            ValueTypeDef::Date => true,
            ValueTypeDef::Time => true,
            ValueTypeDef::Duration => true,
            ValueTypeDef::Geography => true,
            _ => false,
        }
    }
    
    /// 获取类型的默认值
    pub fn get_default_value(type_def: &ValueTypeDef) -> Option<Value> {
        match type_def {
            ValueTypeDef::Bool => Some(Value::Bool(false)),
            ValueTypeDef::Int => Some(Value::Int(0)),
            ValueTypeDef::Float => Some(Value::Float(0.0)),
            ValueTypeDef::String => Some(Value::String(String::new())),
            ValueTypeDef::List => Some(Value::List(Vec::new())),
            ValueTypeDef::Map => Some(Value::Map(Vec::new())),
            _ => None,
        }
    }
}
```

**验证：**
```rust
#[test]
fn test_type_utils_extensions() {
    assert_eq!(TypeUtils::type_to_string(&ValueTypeDef::Int), "int");
    assert!(TypeUtils::is_indexable_type(&ValueTypeDef::Int));
    assert_eq!(TypeUtils::get_default_value(&ValueTypeDef::Int), Some(Value::Int(0)));
}
```

### 任务1.3：创建图结构类型模块

**问题：** 图结构类型定义仅在 `type_inference.rs` 中存在

**创建文件：** `src/core/types/graph_schema.rs`

**完整内容：**
```rust
//! 图结构类型定义
//!
//! 包含图数据库中图结构相关的类型定义

use crate::core::types::DataType;

/// 顶点类型定义
#[derive(Debug, Clone, PartialEq)]
pub struct VertexType {
    pub tag_id: Option<i32>,
    pub tag_name: String,
    pub properties: Vec<PropertyType>,
}

/// 属性类型定义
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyType {
    pub name: String,
    pub type_def: DataType,
    pub is_nullable: bool,
}

/// 边类型定义
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeTypeInfo {
    pub edge_type: i32,
    pub edge_name: String,
    pub src_tag: String,
    pub dst_tag: String,
    pub properties: Vec<PropertyType>,
    pub rank_enabled: bool,
}

/// 路径类型定义
#[derive(Debug, Clone, PartialEq)]
pub enum PathType {
    SimplePath,
    AllPaths,
    ShortestPath,
    NonWeightedShortestPath,
    WeightedShortestPath,
}

/// 路径信息
#[derive(Debug, Clone, PartialEq)]
pub struct PathInfo {
    pub path_type: PathType,
    pub steps: Option<(i32, i32)>,
    pub node_types: Vec<VertexType>,
    pub edge_types: Vec<EdgeTypeInfo>,
}

/// 图结构类型推导器
pub struct GraphTypeInference;

impl GraphTypeInference {
    pub fn new() -> Self {
        Self
    }
    
    /// 推导顶点类型
    pub fn deduce_vertex_type(&self, tag_name: &str, tag_id: Option<i32>) -> VertexType {
        VertexType {
            tag_id,
            tag_name: tag_name.to_string(),
            properties: Vec::new(),
        }
    }
    
    /// 推导边类型
    pub fn deduce_edge_type(&self, edge_name: &str, edge_type: i32) -> EdgeTypeInfo {
        EdgeTypeInfo {
            edge_type,
            edge_name: edge_name.to_string(),
            src_tag: String::new(),
            dst_tag: String::new(),
            properties: Vec::new(),
            rank_enabled: true,
        }
    }
    
    /// 推导路径类型
    pub fn deduce_path_type(&self, path_type: PathType, steps: Option<(i32, i32)>) -> PathInfo {
        PathInfo {
            path_type,
            steps,
            node_types: Vec::new(),
            edge_types: Vec::new(),
        }
    }
    
    /// 推导属性类型
    pub fn deduce_property_type(&self, prop_name: &str, _object_type: &str) -> Option<DataType> {
        match prop_name.to_lowercase().as_str() {
            "id" => Some(DataType::Int),
            "name" | "title" | "desc" | "description" => Some(DataType::String),
            "age" | "count" | "size" | "year" | "month" | "day" | 
            "hour" | "minute" | "second" => Some(DataType::Int),
            "price" | "score" | "rate" | "ratio" | "percent" | 
            "weight" | "height" | "width" | "length" => Some(DataType::Float),
            "created_at" | "updated_at" | "birthday" | "date" | "time" | "datetime" => {
                Some(DataType::DateTime)
            }
            "active" | "enabled" | "visible" | "valid" | "exists" => Some(DataType::Bool),
            "tags" | "labels" | "categories" => Some(DataType::List),
            "properties" | "attrs" | "attributes" => Some(DataType::Map),
            _ => None,
        }
    }
}
```

**更新 `src/core/types/mod.rs`：**
```rust
pub mod graph;
pub mod graph_schema;  // 新增

pub use graph::EdgeDirection;
pub use graph_schema::{GraphTypeInference, VertexType, EdgeTypeInfo, PathInfo};  // 新增
```

## 阶段二：重构 type_inference.rs（第2-3周）

### 任务2.1：移除不符合职责的功能

**修改文件：** `src/query/validator/strategies/type_inference.rs`

**移除的方法：**
```rust
// ❌ 移除这些方法（迁移到 type_system.rs）
pub fn can_cast(&self, from: &ValueTypeDef, to: &ValueTypeDef) -> bool
pub fn type_to_string(&self, type_def: &ValueTypeDef) -> String
pub fn is_indexable_type(&self, type_def: &ValueTypeDef) -> bool
pub fn get_default_value(&self, type_def: &ValueTypeDef) -> Option<Expression>

// ❌ 移除这些方法（迁移到 graph_schema.rs）
pub fn deduce_vertex_type(&self, tag_name: &str, tag_id: Option<i32>) -> VertexType
pub fn deduce_edge_type(&self, edge_name: &str, edge_type: i32) -> EdgeTypeInfo
pub fn deduce_path_type(&self, path_type: PathType, steps: Option<(i32, i32)>) -> PathInfo
pub fn deduce_property_type(&self, prop_name: &str, object_type: &str) -> Option<ValueTypeDef>
pub fn deduce_list_element_type(&self, element_type: &ValueTypeDef) -> ValueTypeDef
pub fn deduce_map_value_type(&self, map_type: &ValueTypeDef) -> ValueTypeDef
pub fn deduce_step_type(&self) -> ValueTypeDef
pub fn deduce_id_type(&self) -> ValueTypeDef
pub fn deduce_labels_type(&self) -> ValueTypeDef
pub fn deduce_properties_type(&self) -> ValueTypeDef
pub fn deduce_edge_type_name_type(&self) -> ValueTypeDef
pub fn deduce_path_result_type(&self) -> ValueTypeDef

// ❌ 移除这些方法（使用 fold_constant_expr_visitor.rs）
pub fn fold_constant_expr_enhanced(&self, expr: &Expression) -> Option<Expression>
fn evaluate_binary_expr_enhanced(...) -> Option<Expression>
fn compute_binary_op_enhanced(...) -> Option<Value>
```

### 任务2.2：统一使用 DeduceTypeVisitor

**修改文件：** `src/query/validator/strategies/type_inference.rs`

**修改方法：**
```rust
impl TypeInference {
    /// 完整的表达式类型推导（使用 DeduceTypeVisitor）
    pub fn deduce_expression_type_full<C: ExpressionValidationContext>(
        &self,
        expr: &Expression,
        context: &C,
    ) -> ValueTypeDef {
        // 使用 DeduceTypeVisitor 而不是自己实现
        let mut visitor = DeduceTypeVisitor::new(
            self.storage,
            self.validate_context,
            self.inputs,
            self.space.clone(),
        );
        visitor.deduce_type(expr).unwrap_or(ValueTypeDef::Empty)
    }
}
```

### 任务2.3：更新调用点

**需要更新的文件：**
1. 所有使用 `type_inference` 的验证器
2. 查询执行器
3. 其他使用相关功能的代码

**更新示例：**
```rust
// 修改前
let type_inference = TypeInference::new();
let vertex_type = type_inference.deduce_vertex_type("person", Some(1));

// 修改后
let graph_inference = GraphTypeInference::new();
let vertex_type = graph_inference.deduce_vertex_type("person", Some(1));

// 修改前
let can_cast = type_inference.can_cast(&from_type, &to_type);

// 修改后
let can_cast = TypeUtils::can_cast(&from_type, &to_type);
```

## 阶段三：清理和优化（第4-5周）

### 任务3.1：清理冗余代码

**移除类型转换函数：**
```rust
// ❌ 移除这些函数（统一使用 DataType）
pub fn value_type_def_to_value_type(type_def: &ValueTypeDef) -> ValueType
pub fn value_type_to_value_type_def(type_: &ValueType) -> ValueTypeDef
```

**统一使用 DataType：**
- 将所有 `ValueType` 替换为 `DataType`
- 更新相关类型定义
- 更新测试用例

### 任务3.2：性能优化

**基准测试：**
```rust
#[bench]
fn bench_type_deduction(b: &mut Bencher) {
    let expr = /* 复杂表达式 */;
    let mut visitor = DeduceTypeVisitor::new(...);
    
    b.iter(|| {
        visitor.deduce_type(&expr).unwrap();
    });
}

#[bench]
fn bench_constant_folding(b: &mut Bencher) {
    let expr = /* 常量表达式 */;
    let mut visitor = FoldConstantExprVisitor::new();
    
    b.iter(|| {
        visitor.fold(&expr).unwrap();
    });
}
```

### 任务3.3：文档更新

**更新文档：**
1. API 文档
2. 架构文档
3. 迁移指南

## 验证检查清单

### 阶段一检查清单
- [ ] `can_cast` 方法统一使用 `TypeUtils::can_cast`
- [ ] `type_system.rs` 添加了 `type_to_string` 方法
- [ ] `type_system.rs` 添加了 `is_indexable_type` 方法
- [ ] `type_system.rs` 添加了 `get_default_value` 方法
- [ ] `graph_schema.rs` 模块创建完成
- [ ] 所有相关测试通过

### 阶段二检查清单
- [ ] `type_inference.rs` 移除了不符合职责的功能
- [ ] `type_inference.rs` 统一使用 `DeduceTypeVisitor`
- [ ] 所有调用点更新完成
- [ ] 集成测试通过
- [ ] 无回归问题

### 阶段三检查清单
- [ ] 冗余代码清理完成
- [ ] 类型系统统一完成
- [ ] 性能基准测试完成
- [ ] 文档更新完成
- [ ] 代码审查通过

## 风险缓解

### 风险1：破坏现有功能
**缓解措施：**
- 每个阶段运行完整测试套件
- 提供兼容性包装器
- 准备回滚计划

### 风险2：性能下降
**缓解措施：**
- 每个阶段进行性能基准测试
- 优化热点代码
- 使用零成本抽象

### 风险3：迁移复杂性
**缓解措施：**
- 详细的实施指南
- 代码审查
- 自动化测试

## 总结

本实施计划基于详细的模块职责分析，提供了清晰的重构路径。通过分阶段实施，确保系统稳定性的同时，逐步实现模块职责的清晰划分。

关键收益：
1. **消除重复**：统一类型推导、常量折叠等功能
2. **职责清晰**：每个模块只负责一个明确的功能
3. **易于维护**：清晰的依赖关系和模块划分
4. **可扩展性**：基于访问者模式，便于扩展新功能