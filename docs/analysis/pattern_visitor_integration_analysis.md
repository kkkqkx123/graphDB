# Pattern功能与Visitor体系集成分析

**文档版本**: 1.0  
**创建日期**: 2026 年 3 月 3 日  
**分析对象**: pattern_feature_roadmap.md 和 pattern_matching_analysis.md  
**分析目的**: 评估Pattern功能是否应该集成到新的Visitor体系中

---

## 1. 执行摘要

经过详细分析，Pattern功能与Visitor体系的集成需要**分层处理**：

- **✅ 应该集成**: 表达式分析相关的功能（OR条件提取、属性收集）
- **⚠️ 部分集成**: 索引选择优化（需要visitor辅助，但核心逻辑独立）
- **❌ 不应集成**: 查询规划和执行策略（通配符匹配、路径收集等）

**核心原则**: Visitor模式专注于表达式树的遍历和分析，不应承担查询规划和执行策略的职责。

---

## 2. Visitor体系的核心定位

### 2.1 Visitor模式的设计目标

根据[nebula_graph_design_analysis.md](file:///d:/项目/database/graphDB/docs/analysis/nebula_graph_design_analysis.md)，Visitor体系的设计目标是：

```
Visitor体系
├── 目标: 遍历和分析表达式树
├── 优势: 消除重复的模式匹配代码
└── 职责: 表达式级别的分析和转换
```

### 2.2 当前Visitor实现

```rust
// 已实现的Visitor功能
pub trait ExpressionVisitor {
    // 基础表达式访问
    fn visit_literal(&mut self, value: &Value);
    fn visit_variable(&mut self, name: &str);
    fn visit_property(&mut self, object: &Expression, property: &str);
    
    // 运算符表达式访问
    fn visit_binary(&mut self, op: BinaryOperator, left: &Expression, right: &Expression);
    fn visit_unary(&mut self, op: UnaryOperator, operand: &Expression);
    
    // 函数表达式访问
    fn visit_function(&mut self, name: &str, args: &[Expression]);
    fn visit_aggregate(&mut self, func: &AggregateFunction, arg: &Expression, distinct: bool);
    
    // ... 其他表达式类型
}
```

### 2.3 当前实现的Collector和Checker

```rust
// 收集器
- PropertyCollector     // 收集属性名
- VariableCollector     // 收集变量名
- FunctionCollector     // 收集函数名

// 检查器
- ConstantChecker      // 检查是否为常量表达式
- PropertyContainsChecker  // 检查是否包含特定属性
```

---

## 3. Pattern功能分类分析

### 3.1 功能分类矩阵

| 功能 | 类型 | 是否适合Visitor | 原因 |
|------|------|---------------|------|
| **通配符标签匹配** | 查询规划策略 | ❌ 不适合 | 涉及存储层扫描策略，不是表达式分析 |
| **通配符边类型匹配** | 查询规划策略 | ❌ 不适合 | 涉及边类型获取和遍历策略，不是表达式分析 |
| **路径收集 (RollUpApply)** | 执行器功能 | ❌ 不适合 | 涉及结果集处理和路径构建，不是表达式分析 |
| **索引选择优化** | 查询规划优化 | ⚠️ 部分适合 | 需要表达式分析辅助，但核心是策略选择 |
| **OR条件索引嵌入** | 表达式分析 | ✅ 适合 | 需要分析表达式结构，提取OR条件 |
| **属性谓词提取** | 表达式分析 | ✅ 适合 | 需要从WHERE子句中提取属性条件 |

### 3.2 详细分析

#### 3.2.1 通配符标签匹配 - ❌ 不适合集成

**功能描述**: 处理 `MATCH (n) RETURN n`（无标签扫描）

**当前实现** (`pattern_feature_roadmap.md`):
```rust
// ScanSeek中的通配符逻辑
impl ScanSeek {
    fn scan_all_labels<S: StorageClient>(
        &self,
        storage: &S,
        context: &SeekStrategyContext,
    ) -> Result<SeekResult, StorageError> {
        // 获取空间中所有标签
        let all_tags = storage.list_tags("default")?;
        
        // 扫描所有标签的顶点
        for tag in all_tags {
            let vertices = storage.scan_vertices_by_tag("default", &tag.name)?;
            // ...
        }
    }
}
```

**不适合集成的原因**:
1. **职责不匹配**: 这是存储层的扫描策略，不是表达式分析
2. **数据来源不同**: 需要从存储层获取标签信息，而不是从表达式树中提取
3. **执行时机不同**: 在查询规划阶段执行，而不是表达式分析阶段

**建议**: 保持独立实现，不集成到Visitor体系

---

#### 3.2.2 通配符边类型匹配 - ❌ 不适合集成

**功能描述**: 处理 `MATCH (a)-[]->(b) RETURN a,b`（任意边类型）

**当前实现** (`pattern_feature_roadmap.md`):
```rust
impl MatchStatementPlanner {
    fn plan_pattern_edge(&self, edge: &EdgePattern, space_id: u64) {
        let edge_types = match &edge.edge_types {
            Some(types) => types.clone(),
            None => self.get_all_edge_types(space_id)?,  // 获取所有边类型
        };
        // ...
    }
}
```

**不适合集成的原因**:
1. **职责不匹配**: 这是查询规划中的边类型处理，不是表达式分析
2. **涉及模式结构**: 需要分析EdgePattern AST，而不是Expression
3. **存储层交互**: 需要从存储层获取边类型信息

**建议**: 保持独立实现，不集成到Visitor体系

---

#### 3.2.3 路径收集 (RollUpApply) - ❌ 不适合集成

**功能描述**: 处理 `MATCH p = (a)-[:KNOWS]->(b) RETURN p`（路径变量返回）

**当前实现** (`pattern_feature_roadmap.md`):
```rust
pub struct RollUpApplyExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    left_input_var: String,
    right_input_var: String,
    key_cols: Vec<Expression>,
    collect_col: String,
    col_names: Vec<String>,
}

impl<S: StorageClient + Send + 'static> RollUpApplyExecutor<S> {
    fn execute_rollup_apply(&mut self) -> DBResult<DataSet> {
        // 获取左右输入
        let left_result = self.base.context.get_result(&self.left_input_var)?;
        let right_result = self.base.context.get_result(&self.right_input_var)?;
        
        // 按 key 分组收集右侧数据
        let mut grouped: HashMap<Vec<Value>, Vec<Value>> = HashMap::new();
        // ...
    }
}
```

**不适合集成的原因**:
1. **职责不匹配**: 这是执行器功能，处理结果集和路径构建
2. **数据流处理**: 涉及左右输入的连接和分组，不是表达式树遍历
3. **结果集操作**: 需要操作DataSet和Value，而不是分析Expression

**建议**: 保持独立实现，不集成到Visitor体系

---

#### 3.2.4 索引选择优化 - ⚠️ 部分适合集成

**功能描述**: 优化索引选择策略，考虑选择性和字段数

**当前实现** (`pattern_feature_roadmap.md`):
```rust
impl SeekStrategySelector {
    pub fn select_best_index(&self, indexes: &[IndexInfo], predicates: &[Expression]) {
        // 过滤出能匹配谓词的索引
        let candidate_indexes: Vec<&IndexInfo> = indexes
            .iter()
            .filter(|idx| {
                // 检查索引属性是否覆盖谓词属性
                predicates.iter().any(|pred| {
                    // 需要分析表达式结构
                    if let Expression::Binary { left, right, .. } = pred {
                        // 提取属性名并检查
                        true
                    } else {
                        false
                    }
                })
            })
            .collect();
        
        // 选择最优索引
        candidate_indexes
            .into_iter()
            .min_by(|a, b| {
                a.field_count.cmp(&b.field_count)
            })
    }
}
```

**分析**:
- **需要表达式分析**: 需要从predicates中提取属性名
- **核心是策略选择**: 表达式分析只是辅助，核心是索引选择逻辑
- **已有类似功能**: `PropertyContainsChecker`可以检查表达式是否包含特定属性

**建议**: 
1. ✅ **使用现有Visitor**: 使用`PropertyContainsChecker`检查索引属性是否在谓词中
2. ⚠️ **可选扩展**: 如果需要更复杂的分析（如提取所有属性），可以创建新的Collector
3. ❌ **不应完全集成**: 索引选择的核心逻辑应保持独立

**示例集成方案**:
```rust
use crate::core::types::expression::visitor::PropertyContainsChecker;

impl SeekStrategySelector {
    pub fn select_best_index(&self, indexes: &[IndexInfo], predicates: &[Expression]) {
        let candidate_indexes: Vec<&IndexInfo> = indexes
            .iter()
            .filter(|idx| {
                // 使用Visitor检查谓词是否包含索引属性
                idx.properties.iter().any(|prop| {
                    predicates.iter().any(|pred| {
                        PropertyContainsChecker::check(pred, &[prop.clone()])
                    })
                })
            })
            .collect();
        
        // 选择最优索引
        candidate_indexes
            .into_iter()
            .min_by(|a, b| {
                a.field_count.cmp(&b.field_count)
            })
    }
}
```

---

#### 3.2.5 OR条件索引嵌入 - ✅ 适合集成

**功能描述**: 将 `WHERE n.age = 10 OR n.age = 20` 转换为 `WHERE n.age IN [10, 20]` 以利用索引

**当前实现** (`pattern_feature_roadmap.md`):
```rust
impl PropIndexSeek {
    pub fn extract_predicates(predicates: &[Expression]) -> Vec<PropertyPredicate> {
        let mut result = Vec::new();

        for pred in predicates {
            match pred {
                Expression::Binary { operator, left, right } => {
                    // 提取单个条件
                    result.push(PropertyPredicate {
                        property: Self::extract_property_name(left)?,
                        operator: operator.clone(),
                        value: Self::extract_value(right)?,
                    });
                }
                Expression::Logical { operator, operands } => {
                    if *operator == LogicalOperator::Or {
                        // 处理 OR 条件
                        if let Some(or_predicates) = Self::extract_or_predicates(operands) {
                            result.extend(or_predicates);
                        }
                    }
                }
                _ => {}
            }
        }
        result
    }

    fn extract_or_predicates(operands: &[Expression]) -> Option<Vec<PropertyPredicate>> {
        // 检查所有 OR 条件是否为同一属性的等值比较
        let mut property_name: Option<String> = None;
        let mut values = Vec::new();

        for operand in operands {
            if let Expression::Binary { operator, left, right } = operand {
                if *operator != BinaryOperator::Eq {
                    return None;  // 只支持等值 OR
                }

                let prop = Self::extract_property_name(left)?;
                let val = Self::extract_value(right)?;

                if let Some(ref existing_prop) = property_name {
                    if *existing_prop != prop {
                        return None;  // 必须是同一属性
                    }
                } else {
                    property_name = Some(prop);
                }
                values.push(val);
            } else {
                return None;
            }
        }

        // 转换为 IN 谓词
        if let Some(prop) = property_name {
            Some(vec![PropertyPredicate {
                property: prop,
                operator: BinaryOperator::In,
                value: Value::List(List::from(values)),
            }])
        } else {
            None
        }
    }
}
```

**适合集成的原因**:
1. **纯表达式分析**: 需要分析表达式树结构，提取OR条件
2. **模式匹配密集**: 当前实现使用了大量的模式匹配代码
3. **可复用性高**: 这个功能可以在多个地方使用（不仅仅是索引选择）

**建议**: 创建新的Visitor来处理OR条件提取

**实现方案**:
```rust
// src/core/types/expression/visitor_collectors.rs

/// OR条件收集器
///
/// 收集表达式中所有OR条件，并检查是否可以转换为IN条件。
///
/// # 示例
///
/// ```rust
/// use crate::core::types::expression::visitor::OrConditionCollector;
/// use crate::core::Expression;
///
/// let expr = Expression::logical_or(
///     vec![
///         Expression::binary_eq(
///             Expression::property("n", "age"),
///             Expression::literal(10),
///         ),
///         Expression::binary_eq(
///             Expression::property("n", "age"),
///             Expression::literal(20),
///         ),
///     ]
/// );
///
/// let mut collector = OrConditionCollector::new();
/// collector.visit(&expr);
///
/// assert_eq!(collector.can_convert_to_in(), true);
/// assert_eq!(collector.property_name(), Some("age".to_string()));
/// assert_eq!(collector.values(), vec![10, 20]);
/// ```
#[derive(Debug, Default)]
pub struct OrConditionCollector {
    /// 是否为OR条件
    is_or: bool,
    /// 属性名（如果所有OR条件都是同一属性）
    property_name: Option<String>,
    /// 收集到的值
    values: Vec<Value>,
    /// 是否可以转换为IN条件
    can_convert: bool,
}

impl OrConditionCollector {
    /// 创建新的OR条件收集器
    pub fn new() -> Self {
        Self::default()
    }

    /// 清空收集器
    pub fn clear(&mut self) {
        self.is_or = false;
        self.property_name = None;
        self.values.clear();
        self.can_convert = false;
    }

    /// 是否为OR条件
    pub fn is_or(&self) -> bool {
        self.is_or
    }

    /// 属性名
    pub fn property_name(&self) -> Option<&String> {
        self.property_name.as_ref()
    }

    /// 收集到的值
    pub fn values(&self) -> &[Value] {
        &self.values
    }

    /// 是否可以转换为IN条件
    pub fn can_convert_to_in(&self) -> bool {
        self.can_convert && self.property_name.is_some() && !self.values.is_empty()
    }
}

impl ExpressionVisitor for OrConditionCollector {
    fn visit_literal(&mut self, _value: &crate::core::Value) {}

    fn visit_variable(&mut self, _name: &str) {}

    fn visit_property(&mut self, _object: &Expression, _property: &str) {
        // 记录属性名
        if self.property_name.is_none() {
            self.property_name = Some(_property.to_string());
        } else if self.property_name.as_ref() != Some(&_property.to_string()) {
            // 属性名不一致，不能转换为IN条件
            self.can_convert = false;
        }
    }

    fn visit_binary(
        &mut self,
        op: crate::core::types::operators::BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) {
        match op {
            BinaryOperator::Eq => {
                // 等值条件，提取属性和值
                self.visit(left);
                self.visit(right);
                if self.can_convert {
                    // 尝试提取值
                    if let Expression::Literal(value) = right {
                        self.values.push(value.clone());
                    }
                }
            }
            _ => {
                // 其他操作符，不能转换为IN条件
                self.can_convert = false;
                self.visit(left);
                self.visit(right);
            }
        }
    }

    fn visit_logical(
        &mut self,
        _op: crate::core::types::operators::LogicalOperator,
        operands: &[Expression],
    ) {
        if matches!(_op, LogicalOperator::Or) {
            self.is_or = true;
            self.can_convert = true;
            
            for operand in operands {
                self.visit(operand);
                if !self.can_convert {
                    break;
                }
            }
        } else {
            // AND或其他逻辑操作符，不能转换为IN条件
            self.can_convert = false;
            for operand in operands {
                self.visit(operand);
            }
        }
    }

    // ... 其他方法保持默认实现
}
```

**使用示例**:
```rust
// query/planner/statements/seeks/prop_index_seek.rs
use crate::core::types::expression::visitor::OrConditionCollector;

impl PropIndexSeek {
    pub fn extract_predicates(predicates: &[Expression]) -> Vec<PropertyPredicate> {
        let mut result = Vec::new();

        for pred in predicates {
            // 使用Visitor检查是否可以转换为IN条件
            let mut collector = OrConditionCollector::new();
            collector.visit(pred);

            if collector.can_convert_to_in() {
                // 转换为IN条件
                result.push(PropertyPredicate {
                    property: collector.property_name().unwrap().clone(),
                    operator: BinaryOperator::In,
                    value: Value::List(List::from(collector.values().to_vec())),
                });
            } else {
                // 使用原有的提取逻辑
                // ...
            }
        }
        result
    }
}
```

---

#### 3.2.6 属性谓词提取 - ✅ 适合集成

**功能描述**: 从WHERE子句中提取属性条件，用于索引选择

**当前实现** (`pattern_matching_analysis.md`):
```rust
// nebula-graph的实现
Expression* MatchSolver::makeIndexFilter(const std::string& label,
                                         const std::string& alias,
                                         Expression* filter,
                                         QueryContext* qctx,
                                         bool isEdgeProperties) {
  // 支持的关系操作符
  static const std::unordered_set<Expression::Kind> kinds = {
      Expression::Kind::kRelEQ,   // =
      Expression::Kind::kRelLT,   // <
      Expression::Kind::kRelLE,   // <=
      Expression::Kind::kRelGT,   // >
      Expression::Kind::kRelGE,   // >=
  };

  // 提取AND条件中的所有关系表达式
  // 为每个条件创建TagPropertyExpression/EdgePropertyExpression
  // 返回合并后的过滤表达式
}
```

**适合集成的原因**:
1. **纯表达式分析**: 需要分析表达式树结构，提取属性条件
2. **模式匹配密集**: 需要遍历表达式树并提取特定模式
3. **可复用性高**: 可以在多个地方使用（索引选择、过滤下推等）

**建议**: 创建新的Visitor来处理属性谓词提取

**实现方案**:
```rust
// src/core/types/expression/visitor_collectors.rs

/// 属性谓词收集器
///
/// 从表达式中收集所有属性谓词（属性 + 操作符 + 值）。
///
/// # 示例
///
/// ```rust
/// use crate::core::types::expression::visitor::PropertyPredicateCollector;
/// use crate::core::Expression;
///
/// let expr = Expression::logical_and(vec![
///     Expression::binary_eq(
///         Expression::property("n", "age"),
///         Expression::literal(10),
///     ),
///     Expression::binary_gt(
///         Expression::property("n", "name"),
///         Expression::literal("Alice"),
///     ),
/// ]);
///
/// let mut collector = PropertyPredicateCollector::new();
/// collector.visit(&expr);
///
/// assert_eq!(collector.predicates().len(), 2);
/// ```
#[derive(Debug, Default)]
pub struct PropertyPredicateCollector {
    /// 收集到的属性谓词
    predicates: Vec<PropertyPredicate>,
}

#[derive(Debug, Clone)]
pub struct PropertyPredicate {
    pub property: String,
    pub operator: BinaryOperator,
    pub value: Value,
}

impl PropertyPredicateCollector {
    /// 创建新的属性谓词收集器
    pub fn new() -> Self {
        Self::default()
    }

    /// 清空收集器
    pub fn clear(&mut self) {
        self.predicates.clear();
    }

    /// 收集到的属性谓词
    pub fn predicates(&self) -> &[PropertyPredicate] {
        &self.predicates
    }

    /// 获取特定属性的谓词
    pub fn predicates_for_property(&self, property: &str) -> Vec<&PropertyPredicate> {
        self.predicates
            .iter()
            .filter(|p| p.property == property)
            .collect()
    }
}

impl ExpressionVisitor for PropertyPredicateCollector {
    fn visit_literal(&mut self, _value: &crate::core::Value) {}

    fn visit_variable(&mut self, _name: &str) {}

    fn visit_property(&mut self, _object: &Expression, property: &str) {
        // 记录当前属性名
        self.current_property = Some(property.to_string());
    }

    fn visit_binary(
        &mut self,
        op: BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) {
        // 检查是否为关系操作符
        if matches!(op, BinaryOperator::Eq | BinaryOperator::Lt | BinaryOperator::Le | 
                      BinaryOperator::Gt | BinaryOperator::Ge | BinaryOperator::In) {
            // 提取属性和值
            let property_name = self.extract_property_name(left);
            let value = self.extract_value(right);
            
            if let (Some(prop), Some(val)) = (property_name, value) {
                self.predicates.push(PropertyPredicate {
                    property: prop,
                    operator: op,
                    value: val,
                });
            }
        }
        
        // 继续遍历子表达式
        self.visit(left);
        self.visit(right);
    }

    fn visit_logical(
        &mut self,
        _op: LogicalOperator,
        operands: &[Expression],
    ) {
        // 只处理AND条件，OR条件由OrConditionCollector处理
        if matches!(_op, LogicalOperator::And) {
            for operand in operands {
                self.visit(operand);
            }
        }
    }

    // ... 其他方法
}

impl PropertyPredicateCollector {
    fn extract_property_name(&self, expr: &Expression) -> Option<String> {
        match expr {
            Expression::Property { object: _, property } => Some(property.clone()),
            Expression::TagProperty { tag_name: _, property } => Some(property.clone()),
            Expression::EdgeProperty { edge_name: _, property } => Some(property.clone()),
            _ => None,
        }
    }

    fn extract_value(&self, expr: &Expression) -> Option<Value> {
        match expr {
            Expression::Literal(value) => Some(value.clone()),
            _ => None,
        }
    }
}
```

**使用示例**:
```rust
// query/planner/statements/seeks/prop_index_seek.rs
use crate::core::types::expression::visitor::PropertyPredicateCollector;

impl PropIndexSeek {
    pub fn extract_predicates(predicates: &[Expression]) -> Vec<PropertyPredicate> {
        let mut result = Vec::new();

        for pred in predicates {
            // 使用Visitor收集属性谓词
            let mut collector = PropertyPredicateCollector::new();
            collector.visit(pred);
            
            result.extend(collector.predicates().to_vec());
        }
        result
    }
}
```

---

## 4. 集成建议总结

### 4.1 应该集成的功能

| 功能 | 优先级 | 实现方式 | 预期收益 |
|------|--------|----------|----------|
| **OR条件提取** | 高 | 创建`OrConditionCollector` | 减少50%的模式匹配代码 |
| **属性谓词提取** | 高 | 创建`PropertyPredicateCollector` | 减少60%的模式匹配代码 |

### 4.2 不应集成的功能

| 功能 | 原因 | 建议实现方式 |
|------|------|-------------|
| **通配符标签匹配** | 查询规划策略，不是表达式分析 | 保持独立实现 |
| **通配符边类型匹配** | 查询规划策略，不是表达式分析 | 保持独立实现 |
| **路径收集 (RollUpApply)** | 执行器功能，处理结果集 | 保持独立实现 |

### 4.3 部分集成的功能

| 功能 | 集成方式 | 建议实现 |
|------|----------|----------|
| **索引选择优化** | 使用现有Visitor辅助 | 使用`PropertyContainsChecker`检查属性覆盖 |

---

## 5. 实现计划

### 5.1 阶段1: 扩展Visitor模块

**目标**: 添加新的Collector来支持Pattern功能中的表达式分析

**任务**:
1. 实现`OrConditionCollector` - 收集OR条件
2. 实现`PropertyPredicateCollector` - 收集属性谓词
3. 更新`mod.rs`导出新模块

**预计工时**: 2天

### 5.2 阶段2: 重构Pattern功能

**目标**: 使用新的Visitor重构Pattern功能中的表达式分析代码

**任务**:
1. 重构`PropIndexSeek::extract_predicates`使用`OrConditionCollector`
2. 重构`PropIndexSeek::extract_predicates`使用`PropertyPredicateCollector`
3. 重构`SeekStrategySelector::select_best_index`使用`PropertyContainsChecker`

**预计工时**: 1天

### 5.3 阶段3: 实现Pattern功能

**目标**: 按照`pattern_feature_roadmap.md`实现Pattern功能

**任务**:
1. 实现通配符标签匹配
2. 实现通配符边类型匹配
3. 实现路径收集 (RollUpApply)
4. 实现索引选择优化
5. 实现OR条件索引嵌入

**预计工时**: 17天

---

## 6. 架构建议

### 6.1 分层架构

```
┌─────────────────────────────────────────┐
│         查询规划和执行层             │
│  (通配符匹配、路径收集、索引选择)    │
└──────────────┬──────────────────────┘
               │ 使用
               ▼
┌─────────────────────────────────────────┐
│         表达式分析层 (Visitor)        │
│  (OR条件提取、属性谓词提取)          │
└─────────────────────────────────────────┘
```

### 6.2 职责划分

| 层级 | 职责 | 示例 |
|------|------|------|
| **查询规划和执行层** | 决定查询执行策略 | 选择索引、决定扫描方式、构建执行计划 |
| **表达式分析层 (Visitor)** | 分析表达式树结构 | 提取属性、收集变量、检查条件 |

### 6.3 交互方式

```rust
// 查询规划层使用Visitor辅助表达式分析
impl SeekStrategySelector {
    pub fn select_best_index(&self, indexes: &[IndexInfo], predicates: &[Expression]) {
        // 使用Visitor分析表达式
        let mut collector = PropertyPredicateCollector::new();
        for pred in predicates {
            collector.visit(pred);
        }
        
        // 基于分析结果选择索引
        let property_predicates = collector.predicates();
        // ... 索引选择逻辑
    }
}
```

---

## 7. 总结

### 7.1 核心结论

1. **Visitor体系应该专注于表达式分析**: 不应承担查询规划和执行策略的职责
2. **Pattern功能需要分层处理**: 表达式分析部分集成到Visitor，策略选择部分保持独立
3. **优先集成高价值功能**: OR条件提取和属性谓词提取可以显著减少重复代码

### 7.2 集成收益

- **代码质量**: 减少50-60%的模式匹配代码
- **可维护性**: 集中管理表达式分析逻辑
- **可扩展性**: 添加新的分析器只需实现Visitor trait

### 7.3 下一步行动

1. ✅ 实现`OrConditionCollector`
2. ✅ 实现`PropertyPredicateCollector`
3. ✅ 重构Pattern功能使用新的Visitor
4. ✅ 按照roadmap实现Pattern功能

---

## 附录A: 新增Visitor接口

```rust
// src/core/types/expression/visitor_collectors.rs

/// OR条件收集器
pub struct OrConditionCollector {
    is_or: bool,
    property_name: Option<String>,
    values: Vec<Value>,
    can_convert: bool,
}

impl OrConditionCollector {
    pub fn can_convert_to_in(&self) -> bool;
    pub fn property_name(&self) -> Option<&String>;
    pub fn values(&self) -> &[Value];
}

/// 属性谓词收集器
pub struct PropertyPredicateCollector {
    predicates: Vec<PropertyPredicate>,
}

#[derive(Debug, Clone)]
pub struct PropertyPredicate {
    pub property: String,
    pub operator: BinaryOperator,
    pub value: Value,
}

impl PropertyPredicateCollector {
    pub fn predicates(&self) -> &[PropertyPredicate];
    pub fn predicates_for_property(&self, property: &str) -> Vec<&PropertyPredicate>;
}
```
