# Unsafe 使用文档

本文档记录项目中所有unsafe代码的使用原因和安全性分析。

## MutexGuard 生命周期延长

### 位置
- `src/query/planner/plan/core/nodes/graph_scan_node.rs`

### 原因
`PlanNode` trait的`dependencies()`方法要求返回`&[Box<PlanNodeEnum>]`，但graph_scan_node中的依赖存储在`Mutex<Vec<Box<PlanNodeEnum>>>`中。由于`MutexGuard`的生命周期受限于方法作用域，无法直接返回其内部数据的引用。

### 使用场景
以下结构体使用了unsafe来延长MutexGuard的生命周期：
- `GetVerticesNode`
- `GetEdgesNode`
- `GetNeighborsNode`
- `ScanVerticesNode`
- `ScanEdgesNode`

### 安全性分析
这些节点的`dependencies()`方法在以下条件下是安全的：
1. 这些方法在查询计划构建阶段被调用，此时没有并发访问
2. `dependencies()`方法在查询计划执行前被调用，不会在执行过程中被调用
3. 调用`dependencies()`时不会持有锁，不会导致死锁

### 代码示例
```rust
fn dependencies(&self) -> &[Box<super::plan_node_enum::PlanNodeEnum>] {
    unsafe {
        let deps = &self.dependencies as *const Mutex<Vec<Box<super::plan_node_enum::PlanNodeEnum>>>;
        let guard = (*deps).lock().unwrap();
        std::mem::transmute::<&Vec<Box<super::plan_node_enum::PlanNodeEnum>>, &[Box<super::plan_node_enum::PlanNodeEnum>]>(
            &*guard
        )
    }
}
```

### 替代方案
如果需要完全避免unsafe，可以考虑：
1. 将依赖存储从Mutex改为普通Vec（如果不需要线程安全）
2. 使用Arc<Mutex<Vec<...>>>并返回Arc<Vec<...>>（但这会改变trait定义）
3. 重构架构，避免在trait中返回引用
