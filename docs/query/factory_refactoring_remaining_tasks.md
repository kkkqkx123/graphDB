# Factory 重构剩余任务清单

## 当前状态
已完成大部分构建器模块的修复，但 executor_factory.rs 存在大量编译错误需要修复。

## 主要问题

### 1. executor_factory.rs - 61个错误
- **问题**: 尝试调用 `n.input()` 方法，但节点类型的 `input` 字段是私有的
- **解决方案**: 使用 `dependencies()` 方法或 `SingleInputNode` trait 的 `input()` 方法
- **注意**: 需要正确导入 trait

### 2. data_processing_builder.rs - 11个错误
- **FilterExecutor::new**: 参数数量不匹配 (需要3个，提供了4个)
- **ProjectExecutor::new**: 类型不匹配 (需要 `Vec<ProjectionColumn>`，提供了 `Vec<(String, Expression)>`)
- **SortExecutor::new**: 返回类型不匹配 (返回 `Result`，但期望直接值)
- **Expression::Null**: 变体不存在

### 3. traversal_builder.rs - 6个错误
- **ExpandExecutor::new**: 参数数量不匹配 (需要6个，提供了8个)
- **ExpandAllExecutor::new**: 参数数量不匹配 (需要7个，提供了8个)
- **TraverseExecutor::new**: 参数数量不匹配 (需要7个，提供了8个)
- 类型转换问题 (`EdgeDirection` 到 `Option<usize>`, `&str` 到 `bool`)

### 4. set_operation_builder.rs - 3个错误
- **UnionExecutor/MinusExecutor/IntersectExecutor::new**: 类型不匹配 (需要 `Vec<String>`，提供了 `String`)

### 5. admin_builder.rs - 11个错误
- 多个节点缺少方法 (如 `ShowTagIndexesNode.space_name()`, `CreateUserNode.user_name()` 等)
- **CreateUserExecutor/AlterUserExecutor::new**: 参数数量不匹配

### 6. plan_executor.rs - 1个错误
- **ExecutorEnum::execute**: 方法不存在

### 7. query_pipeline_manager.rs - 1个错误
- **ExecutorFactory::execute_plan**: 方法不存在

## 修复优先级

### 高优先级 (阻塞编译)
1. 修复 executor_factory.rs 的 input() 方法调用
2. 修复 data_processing_builder.rs 的参数和类型问题
3. 修复 traversal_builder.rs 的参数数量问题

### 中优先级
4. 修复 set_operation_builder.rs 的类型问题
5. 修复 admin_builder.rs 的方法缺失问题

### 低优先级
6. 修复 plan_executor.rs 和 query_pipeline_manager.rs 的方法缺失

## 修复策略

对于 executor_factory.rs，需要：
1. 导入 `SingleInputNode`, `BinaryInputNode` traits
2. 使用 `node.dependencies()` 获取依赖，而不是直接调用 `input()`
3. 或者使用 trait 方法 `input()`, `left_input()`, `right_input()`

对于构建器，需要：
1. 检查每个 Executor::new 的实际签名
2. 修正参数数量和类型
3. 使用正确的类型转换
