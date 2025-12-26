# 错误处理机制统一方案（简化版）

## 1. 现状分析

### 1.1 现有错误类型

项目已实现统一的错误处理系统，主要错误类型包括：

- **核心错误**（`src/core/error.rs`）：
  - `DBError` - 统一的数据库错误枚举
  - `StorageError` - 存储层错误
  - `QueryError` - 查询层错误
  - `ExpressionError` - 表达式错误（结构化设计，包含位置信息）
  - `PlanNodeVisitError` - 计划节点访问错误
  - `LockError` - 锁操作错误

- **其他错误**：
  - `TypeDeductionError` - 类型推导错误（`src/query/visitor/deduce_type_visitor.rs`）
  - `ParseError` - 解析错误（`src/query/parser/core/error.rs`）

### 1.2 已实现的功能

- ✅ 统一的 `DBResult<T>` 类型别名
- ✅ `ExpressionError` 的结构化设计（包含错误类型、消息、位置）
- ✅ 完善的错误转换机制（`From` trait）
- ✅ 自定义日志系统（`src/common/log.rs`）
- ✅ 错误处理辅助函数（`src/utils/error_handling.rs`）

### 1.3 存在的问题

1. **错误消息格式不一致**：中英文混用，表述不统一
2. **部分错误缺少结构化信息**：只有 `ExpressionError` 包含位置信息
3. **错误上下文信息不足**：缺少模块、操作等上下文
4. **错误统计和监控缺失**：无法追踪错误发生频率和趋势

## 2. 统一方案设计

### 2.1 设计原则

1. **渐进式重构**：分阶段实施，避免大规模修改
2. **最小化依赖**：使用现有依赖（`thiserror`、`serde`、`log`）
3. **保持兼容**：不破坏现有 API
4. **实用主义**：优先解决实际问题，避免过度设计

### 2.2 核心改进

#### 2.2.1 统一错误消息格式

制定统一的错误消息规范：

```rust
// 中文错误消息格式
"<错误类型>: <具体描述>"

// 示例
"类型错误: 期望 Int64, 实际 String"
"存储错误: 节点未找到: 123"
"执行错误: 除零错误"
```

#### 2.2.2 扩展错误上下文

为 `DBError` 添加上下文信息：

```rust
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub module: String,           // 模块名称
    pub operation: String,         // 操作名称
    pub details: Option<String>,   // 详细信息
}

impl DBError {
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        // 将上下文信息附加到错误消息中
        self
    }
}
```

#### 2.2.3 结构化错误信息

参考 `ExpressionError` 的设计，为其他错误添加结构化信息：

```rust
// 为 StorageError 添加位置信息
#[derive(Error, Debug, Clone)]
pub enum StorageError {
    #[error("数据库错误: {message}")]
    DbError {
        message: String,
        position: Option<ErrorPosition>,
    },
    // ...
}

// 统一的位置信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPosition {
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub file: Option<String>,
}
```

#### 2.2.4 错误日志增强

集成到现有日志系统，添加错误统计：

```rust
// src/core/error_logging.rs
use crate::common::log::{LogLevel, log};

pub struct ErrorLogger {
    component: String,
}

impl ErrorLogger {
    pub fn log_error(&self, error: &DBError, context: Option<&ErrorContext>) {
        let message = format!("{:?}", error);
        log(LogLevel::Error, &self.component, &message);

        // 记录错误统计
        self.record_error_metric(error);
    }

    fn record_error_metric(&self, error: &DBError) {
        // 使用现有日志系统记录错误指标
        // 避免引入 metrics crate
    }
}
```

#### 2.2.5 简化的错误恢复

实现基础的重试机制，仅针对可恢复的错误：

```rust
// src/core/error_recovery.rs
use std::time::Duration;
use tokio::time::sleep;

pub async fn retry_with_backoff<F, T, E>(
    mut operation: F,
    max_retries: u32,
    base_delay_ms: u64,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Debug,
{
    let mut attempt = 0;
    loop {
        match operation() {
            Ok(result) => return Ok(result),
            Err(error) => {
                if attempt >= max_retries {
                    return Err(error);
                }

                let delay = Duration::from_millis(base_delay_ms * 2_u64.pow(attempt));
                sleep(delay).await;
                attempt += 1;
            }
        }
    }
}
```

## 3. 实施计划

### 3.1 第一阶段：错误类型统一（1-2天）

**任务**：
1. 审查所有错误类型，识别重复和冗余
2. 统一错误命名规范（中文，驼峰命名）
3. 统一错误消息格式
4. 完善错误转换逻辑

**交付物**：
- 更新的错误类型定义
- 错误消息格式规范文档
- 错误转换测试

### 3.2 第二阶段：错误上下文增强（2-3天）

**任务**：
1. 实现 `ErrorContext` 结构体
2. 为 `DBError` 添加 `with_context()` 方法
3. 为关键错误添加位置信息
4. 集成到现有日志系统

**交付物**：
- `ErrorContext` 实现
- 增强的错误日志功能
- 错误上下文测试

### 3.3 第三阶段：错误恢复和监控（1-2天）

**任务**：
1. 实现基础的重试机制
2. 添加错误统计功能
3. 实现错误告警机制
4. 编写集成测试

**交付物**：
- 错误恢复模块
- 错误统计和告警功能
- 集成测试

### 3.4 第四阶段：文档和测试（1天）

**任务**：
1. 更新 API 文档
2. 编写错误处理最佳实践文档
3. 完善单元测试和集成测试
4. 性能测试

**交付物**：
- 完整的 API 文档
- 最佳实践文档
- 测试覆盖率 > 80%

## 4. 预期收益

### 4.1 代码质量

- **一致性**：统一的错误处理模式和消息格式
- **可维护性**：减少重复代码，简化错误处理逻辑
- **可读性**：清晰的错误类型和一致的命名规范

### 4.2 开发效率

- **简化开发**：开发者只需要处理一种错误类型
- **减少 bug**：消除错误转换中的遗漏和错误
- **快速定位**：统一的错误上下文便于问题定位

### 4.3 系统可靠性

- **错误恢复**：基础的重试机制提高系统健壮性
- **监控告警**：实时的错误统计和告警
- **性能优化**：减少错误处理开销

## 5. 风险评估

### 5.1 技术风险

- **兼容性问题**：现有代码可能需要少量修改
- **性能影响**：错误上下文可能带来轻微开销
- **测试覆盖**：需要确保所有错误路径都有测试

### 5.2 缓解措施

- **渐进式迁移**：分阶段实施，避免一次性大规模修改
- **充分测试**：每个阶段都要进行充分的测试验证
- **性能监控**：实施过程中持续监控性能指标
- **回滚机制**：保留回滚到原错误处理机制的能力

### 5.3 成功标准

- 所有错误消息格式统一，中英文一致
- 错误处理代码减少 20-25%
- 错误日志一致性达到 90% 以上
- 错误恢复成功率达到 60-70%
- 系统性能下降 < 5%
- 测试覆盖率 > 80%

## 6. 技术选型

### 6.1 依赖管理

- **保留现有依赖**：
  - `thiserror` - 错误处理
  - `serde` - 序列化
  - `log` - 日志
  - `tokio` - 异步运行时

- **不引入新依赖**：
  - ❌ `tracing` - 使用现有 `log` 系统
  - ❌ `metrics` - 使用自定义统计

### 6.2 设计模式

- **访问者模式**：用于错误类型转换
- **策略模式**：用于错误恢复策略
- **观察者模式**：用于错误监控和告警

## 7. 实施注意事项

1. **保持向后兼容**：不破坏现有 API，使用 `#[deprecated]` 标记旧 API
2. **充分测试**：每个阶段都要进行充分的测试验证
3. **文档同步**：及时更新文档，保持与代码同步
4. **性能监控**：实施过程中持续监控性能指标
5. **团队沟通**：及时与团队沟通进展和问题
