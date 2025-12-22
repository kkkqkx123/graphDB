# Expression和Query模块迁移文件修改清单

## 概述

本文档提供了Expression模块和Query模块迁移过程中需要修改的具体文件清单，以及每个文件的修改说明。

## Expression模块迁移文件清单

### 阶段一：删除重复类型定义

#### 需要修改的文件：

1. **`src/expression/expression.rs`**
   - **修改类型：** 删除内容
   - **修改说明：** 删除所有类型定义（Expression、LiteralValue、BinaryOperator等）
   - **保留内容：** 无（文件将被清空或删除）

2. **`src/expression/mod.rs`**
   - **修改类型：** 更新导出
   - **修改说明：** 从Core模块重新导出所有类型
   - **示例修改：**
     ```rust
     // 修改前
     pub use expression::Expression;
     
     // 修改后
     pub use crate::core::Expression;
     ```

3. **所有引用Expression类型的文件**
   - **修改类型：** 更新导入路径
   - **修改说明：** 将 `crate::expression::Expression` 改为 `crate::core::Expression`
   - **影响文件：** 整个项目中的多个文件

### 阶段二：迁移表达式求值系统

#### 需要移动的文件：

1. **`src/expression/evaluator.rs`** → **`src/core/evaluator/expression.rs`**
   - **操作：** 移动文件
   - **修改说明：** 更新所有导入路径

2. **`src/expression/evaluator_trait.rs`** → **`src/core/evaluator/traits.rs`**
   - **操作：** 移动文件
   - **修改说明：** 更新所有导入路径

3. **`src/expression/binary.rs`** → **`src/core/evaluator/operations/binary.rs`**
   - **操作：** 移动文件
   - **修改说明：** 更新所有导入路径

4. **`src/expression/unary.rs`** → **`src/core/evaluator/operations/unary.rs`**
   - **操作：** 移动文件
   - **修改说明：** 更新所有导入路径

5. **`src/expression/function.rs`** → **`src/core/evaluator/operations/function.rs`**
   - **操作：** 移动文件
   - **修改说明：** 更新所有导入路径

6. **`src/expression/context/default.rs`** → **`src/core/context/expression.rs`**
   - **操作：** 移动文件
   - **修改说明：** 整合到Core模块的上下文系统

#### 需要创建的目录：

- `src/core/evaluator/`
- `src/core/evaluator/operations/`

#### 需要修改的文件：

1. **`src/core/mod.rs`**
   - **修改类型：** 添加模块导出
   - **修改说明：** 添加新的evaluator模块导出

2. **`src/core/evaluator/mod.rs`** (新文件)
   - **操作：** 创建新文件
   - **修改说明：** 导出evaluator子模块

### 阶段三：迁移专用功能

#### 需要移动的文件：

1. **`src/expression/cypher/`** → **`src/core/cypher/`**
   - **操作：** 移动整个目录
   - **修改说明：** 更新所有导入路径

2. **`src/expression/storage/`** → **`src/core/storage/expression/`**
   - **操作：** 移动整个目录
   - **修改说明：** 整合到Core模块的存储系统

3. **`src/expression/visitor.rs`** → **`src/core/visitor/expression.rs`**
   - **操作：** 移动文件
   - **修改说明：** 整合到Core模块的访问者系统

## Query模块迁移文件清单

### 阶段一：统一查询类型定义

#### 需要修改的文件：

1. **`src/query/context/execution_context.rs`**
   - **修改类型：** 整合功能
   - **修改说明：** 将功能整合到Core模块的ExecutionContext中
   - **目标文件：** `src/core/context/execution.rs`

2. **`src/query/context/ast/query_ast_context.rs`**
   - **修改类型：** 移动文件
   - **修改说明：** 移动到Core模块
   - **目标文件：** `src/core/context/query_ast.rs`

3. **`src/core/context/query.rs`**
   - **修改类型：** 扩展功能
   - **修改说明：** 整合Query模块的上下文功能

4. **`src/core/context/execution.rs`**
   - **修改类型：** 扩展功能
   - **修改说明：** 整合QueryExecutionContext的功能

5. **`src/core/types/query.rs`**
   - **修改类型：** 统一类型
   - **修改说明：** 整合ExecutionResult类型

### 阶段二：迁移查询规划系统

#### 需要移动的文件：

1. **`src/query/planner/planner.rs`** → **`src/core/query/planner/mod.rs`**
   - **操作：** 移动文件
   - **修改说明：** 更新所有导入路径

2. **`src/query/planner/plan/execution_plan.rs`** → **`src/core/query/plan/execution_plan.rs`**
   - **操作：** 移动文件
   - **修改说明：** 更新所有导入路径

3. **`src/query/planner/match_planning/`** → **`src/core/query/planners/match/`**
   - **操作：** 移动整个目录
   - **修改说明：** 更新所有导入路径

4. **`src/query/planner/ngql/`** → **`src/core/query/planners/ngql/`**
   - **操作：** 移动整个目录
   - **修改说明：** 更新所有导入路径

#### 需要创建的目录：

- `src/core/query/`
- `src/core/query/planner/`
- `src/core/query/plan/`
- `src/core/query/planners/`

### 阶段三：迁移查询执行系统

#### 需要移动的文件：

1. **`src/query/executor/traits.rs`** → **`src/core/query/executor/traits.rs`**
   - **操作：** 移动文件
   - **修改说明：** 更新所有导入路径

2. **`src/query/executor/factory.rs`** → **`src/core/query/executor/factory.rs`**
   - **操作：** 移动文件
   - **修改说明：** 更新所有导入路径

3. **`src/query/executor/data_access/`** → **`src/core/query/executor/data_access/`**
   - **操作：** 移动整个目录
   - **修改说明：** 更新所有导入路径

4. **`src/query/executor/result_processing/`** → **`src/core/query/executor/result_processing/`**
   - **操作：** 移动整个目录
   - **修改说明：** 更新所有导入路径

5. **`src/query/executor/data_processing/`** → **`src/core/query/executor/data_processing/`**
   - **操作：** 移动整个目录
   - **修改说明：** 更新所有导入路径

6. **`src/query/executor_factory.rs`** → **`src/core/query/executor_factory.rs`**
   - **操作：** 移动文件
   - **修改说明：** 更新所有导入路径

#### 需要创建的目录：

- `src/core/query/executor/`
- `src/core/query/executor/data_access/`
- `src/core/query/executor/result_processing/`
- `src/core/query/executor/data_processing/`

### 阶段四：迁移查询支持系统

#### 需要移动的文件：

1. **`src/query/validator/`** → **`src/core/query/validator/`**
   - **操作：** 移动整个目录
   - **修改说明：** 更新所有导入路径

2. **`src/query/scheduler/`** → **`src/core/query/scheduler/`**
   - **操作：** 移动整个目录
   - **修改说明：** 更新所有导入路径

3. **`src/query/visitor/`** → **`src/core/query/visitor/`**
   - **操作：** 移动整个目录
   - **修改说明：** 更新所有导入路径

4. **`src/query/query_pipeline_manager.rs`** → **`src/core/query/pipeline_manager.rs`**
   - **操作：** 移动文件
   - **修改说明：** 更新所有导入路径

#### 需要创建的目录：

- `src/core/query/validator/`
- `src/core/query/scheduler/`
- `src/core/query/visitor/`

## 核心文件修改清单

### 需要修改的核心文件：

1. **`src/core/mod.rs`**
   - **修改类型：** 更新模块导出
   - **修改说明：** 添加新的query、evaluator、cypher等模块导出

2. **`src/core/context/mod.rs`**
   - **修改类型：** 更新模块导出
   - **修改说明：** 添加expression、query_ast等模块导出

3. **`src/lib.rs`**
   - **修改类型：** 更新模块导出
   - **修改说明：** 确保所有新模块正确导出

4. **`Cargo.toml`**
   - **修改类型：** 检查依赖
   - **修改说明：** 确保所有依赖关系正确

## 清理阶段文件清单

### 需要删除的文件和目录：

1. **`src/expression/`** - 整个目录（在确认所有功能迁移后）
2. **`src/query/`** - 部分文件（保留必要的文件，删除重复代码）

### 需要清理的文件：

1. **`src/query/mod.rs`**
   - **修改类型：** 清理导出
   - **修改说明：** 删除已迁移到Core模块的导出

## 验证清单

### 每个阶段完成后需要验证的项目：

1. **编译验证**
   - 确保代码能够编译
   - 检查是否有编译错误或警告

2. **单元测试**
   - 运行所有单元测试
   - 确保所有测试通过

3. **导入路径验证**
   - 检查所有导入路径是否正确
   - 确保没有遗漏的路径更新

4. **功能验证**
   - 运行功能测试
   - 确保所有功能正常工作

## 注意事项

1. **备份重要文件** - 在开始迁移前备份所有重要文件
2. **分阶段进行** - 严格按照阶段顺序进行迁移
3. **及时测试** - 每个阶段完成后立即进行测试
4. **文档更新** - 及时更新相关文档和注释
5. **团队沟通** - 与团队成员保持沟通，确保所有人都了解迁移进度

## 总结

本文件清单提供了Expression模块和Query模块迁移过程中需要修改的所有文件的详细信息。通过按照这个清单进行操作，可以确保迁移过程的系统性和完整性，减少遗漏和错误的可能性。