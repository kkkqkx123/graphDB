# LabelIndexSeek 改进总结

## 改进概述
根据nebula-graph的实现，对 `label_index_seek.rs` 进行了全面改进，解决了原实现中的三个主要问题：

1. **重复的set_output_var()调用** - 导致前面设置的变量信息被覆盖
2. **属性和过滤条件处理不完整** - 仅改变变量名，未实现实际过滤逻辑
3. **缺少健壮的元数据存储机制** - 通过变量名传递信息不够健壮

## 具体改进

### 1. 新增IndexScanMetadata结构
```rust
#[derive(Debug, Clone)]
pub struct IndexScanMetadata {
    pub label_ids: Vec<i32>,
    pub label_names: Vec<String>,
    pub index_ids: Vec<i32>,
    pub has_property_filter: bool,
    pub property_filter: Option<Expression>,
}
```

**作用：**
- 专门存储IndexScan节点执行所需的索引信息
- 避免通过变量名传递信息的不健壮做法
- 支持属性过滤表达式的存储和传递

### 2. 重构build_plan()方法

**原问题代码：**
```rust
// 第51行、69行、107行、129行多次调用，每次覆盖前一个
index_scan_node.set_output_var(variable);
```

**改进方案：**
- 只调用一次set_output_var()设置IndexScan节点的输出
- 为属性过滤记录在metadata中（不修改节点变量名）
- **条件过滤通过创建独立的Filter节点实现**

```rust
// 处理节点过滤条件 - 创建独立的Filter节点
if let Some(filter) = &self.node_info.filter {
    let mut filter_node = Box::new(SingleInputNode::new(
        PlanNodeKind::Filter,
        index_scan_node.clone(),
    ));
    // ... 设置Filter节点
    root = filter_node;
}
```

**执行计划流程：**
```
IndexScan (tail) -> [可选Filter] (root)
    |                      |
获取顶点ID          应用条件过滤
```

### 3. 改进match_node()条件验证

**原实现：**
```rust
pub fn match_node(&self) -> bool {
    !self.node_info.labels.is_empty()  // 不完整
}
```

**改进后：**
```rust
pub fn match_node(&self) -> bool {
    // 需要同时检查标签和标签ID
    !self.node_info.labels.is_empty() && !self.node_info.tids.is_empty()
}
```

### 4. 添加验证和错误处理

在build_plan()开始时添加明确的参数验证：
```rust
if self.node_info.labels.is_empty() {
    return Err(PlannerError::InvalidAstContext(
        "节点必须有标签才能使用标签索引查找".to_string(),
    ));
}

if self.node_info.tids.is_empty() {
    return Err(PlannerError::InvalidAstContext(
        "节点标签ID列表不能为空".to_string(),
    ));
}
```

**优点：**
- 提前发现问题而非在执行期失败
- 提供清晰的错误信息便于调试

### 5. 新增get_index_metadata()公共方法

为执行器提供便捷的元数据获取接口：
```rust
pub fn get_index_metadata(&self) -> Result<IndexScanMetadata, PlannerError>
```

**用途：**
- 执行器可以调用此方法获取索引执行所需的完整信息
- 包含标签ID、索引ID、属性过滤表达式等

## 代码架构对标nebula-graph

### nebula-graph的做法
- 通过IndexQueryContext存储索引查询信息
- 过滤条件可嵌入IndexScan或作为独立节点
- 明确的参数验证和错误处理

### 本实现的改进
- 定义专用的IndexScanMetadata结构
- 属性过滤记录在元数据中
- 条件过滤通过Filter节点实现（更清晰的执行流）
- 完整的验证和错误处理机制

## 测试覆盖

改进后的测试用例覆盖以下场景：

1. **基本功能**
   - `test_build_plan_success` - 基本计划构建
   - `test_match_node_*` - 各种标签/ID条件组合

2. **边界条件**
   - `test_build_plan_without_labels` - 无标签应返回错误
   - `test_build_plan_without_tids` - 无标签ID应返回错误

3. **特殊情况**
   - `test_build_plan_with_filter` - 验证Filter节点创建
   - `test_build_plan_with_properties` - 属性过滤处理

4. **元数据相关**
   - `test_index_scan_metadata_creation` - 元数据创建
   - `test_index_scan_metadata_with_filter` - 元数据包含过滤
   - `test_get_index_metadata` - 元数据检索

## 参考资料

- nebula-graph: `src/graph/planner/match/LabelIndexSeek.cpp` (Lines 66-130)
- 核心改进点：IndexQueryContext模式 -> IndexScanMetadata结构

## 后续优化建议

1. **索引ID查询**
   - 当前简化处理：`let index_id = label_id`
   - 实际应该从元数据管理器查询真实的索引ID

2. **属性过滤优化**
   - 可以考虑将支持的属性过滤嵌入IndexScan节点
   - 避免创建额外的Filter节点以提高性能

3. **多标签索引**
   - 当前实现仅使用第一个标签
   - 可扩展支持多个标签的复合索引查询

4. **成本估算**
   - 为IndexScan节点计算准确的成本值
   - 为Filter节点估算选择度

