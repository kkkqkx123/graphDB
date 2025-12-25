# 错误处理机制统一方案

## 1. 现状分析

### 1.1 错误类型分布

当前项目中存在多个独立的错误处理系统，导致错误处理逻辑分散且不一致：

#### 核心错误系统
- `DBError` - 主要的错误枚举，定义在 `src/core/error.rs`
- `QueryError` - 查询相关错误，定义在 `src/query/error.rs`
- `StorageError` - 存储相关错误，定义在 `src/storage/error.rs`

#### 分布式错误定义
```rust
// 核心错误定义 (src/core/error.rs)
pub enum DBError {
    Storage(StorageError),
    Query(QueryError),
    // ... 其他错误类型
}

// 查询错误定义 (src/query/error.rs)
pub enum QueryError {
    SyntaxError(String),
    ExecutionError(String),
    // ... 其他错误类型
}

// 存储错误定义 (src/storage/error.rs)
pub enum StorageError {
    IoError(std::io::Error),
    InvalidData(String),
    // ... 其他错误类型
}
```

#### 访问者模式中的错误定义
```rust
// src/query/visitor/deduce_type_visitor.rs
#[derive(Debug, Clone)]
pub enum PlanNodeVisitError {
    TypeDeductionError(String),
    UnsupportedNode(String),
    InvalidInput(String),
}

// src/query/parser/ast/visitor.rs
#[derive(Debug, Clone)]
pub enum VisitError {
    SyntaxError(String),
    TypeError(String),
    ValidationError(String),
}

// src/expression/visitor.rs
#[derive(Debug, Clone)]
pub enum ExpressionVisitError {
    EvaluationError(String),
    TypeMismatch(String),
    UndefinedVariable(String),
}
```

### 1.2 问题识别

#### 1.2.1 错误类型重复
多个模块定义了功能相似的错误类型：

```rust
// 重复的类型错误定义
PlanNodeVisitError::TypeDeductionError  // 计划节点访问错误
VisitError::TypeError                   // AST访问错误  
ExpressionVisitError::TypeMismatch      // 表达式访问错误
QueryError::TypeError                  // 查询错误
```

#### 1.2.2 错误转换复杂
不同错误类型之间缺乏统一的转换机制：

```rust
// 当前复杂的错误转换
impl From<PlanNodeVisitError> for QueryError {
    fn from(error: PlanNodeVisitError) -> Self {
        match error {
            PlanNodeVisitError::TypeDeductionError(msg) => {
                QueryError::TypeError(msg)
            }
            PlanNodeVisitError::UnsupportedNode(msg) => {
                QueryError::ExecutionError(msg)
            }
            // ... 更多转换逻辑
        }
    }
}
```

#### 1.2.3 错误信息不一致
相同类型的错误在不同模块中有不同的描述方式：

```rust
// 类型不匹配错误的多种表述
"类型推断失败: 无法匹配类型"
"Type deduction failed: cannot match types"
"类型错误: 期望的类型不匹配"
"Type mismatch in expression evaluation"
```

## 2. 统一方案设计

### 2.1 核心错误枚举扩展

扩展 `DBError` 枚举，包含所有必要的错误变体：

```rust
// src/core/error.rs
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum DBError {
    // 存储相关错误
    #[error("存储错误: {0}")]
    Storage(#[from] StorageError),
    
    // 查询相关错误
    #[error("查询错误: {0}")]
    Query(#[from] QueryError),
    
    // 网络相关错误
    #[error("网络错误: {0}")]
    Network(String),
    
    // 配置相关错误
    #[error("配置错误: {0}")]
    Configuration(String),
    
    // 验证相关错误
    #[error("验证错误: {0}")]
    Validation(String),
    
    // 类型相关错误
    #[error("类型错误: {0}")]
    Type(String),
    
    // 执行相关错误
    #[error("执行错误: {0}")]
    Execution(String),
    
    // 语法相关错误
    #[error("语法错误: {0}")]
    Syntax(String),
    
    // 未定义错误
    #[error("未定义错误: {0}")]
    Undefined(String),
    
    // IO错误
    #[error("IO错误: {0}")]
    Io(String),
    
    // 内部错误
    #[error("内部错误: {0}")]
    Internal(String),
    
    // 不支持的操作
    #[error("不支持的操作: {0}")]
    Unsupported(String),
    
    // 超时错误
    #[error("超时错误: {0}")]
    Timeout(String),
    
    // 权限错误
    #[error("权限错误: {0}")]
    Permission(String),
}

// 统一的Result类型
pub type DBResult<T> = Result<T, DBError>;
```

### 2.2 错误上下文信息

为错误添加上下文信息，便于调试和日志记录：

```rust
// src/core/error.rs
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub module: String,
    pub operation: String,
    pub details: Option<serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl DBError {
    /// 为错误添加上下文信息
    pub fn with_context(self, context: ErrorContext) -> Self {
        // 实现上下文附加逻辑
        self
    }
    
    /// 获取错误严重程度
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            DBError::Internal(_) => ErrorSeverity::Critical,
            DBError::Storage(_) => ErrorSeverity::High,
            DBError::Query(_) => ErrorSeverity::Medium,
            DBError::Syntax(_) => ErrorSeverity::Low,
            _ => ErrorSeverity::Medium,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}
```

### 2.3 访问者错误统一

将所有访问者相关的错误统一到 `DBError` 中：

```rust
// src/query/visitor/mod.rs
use crate::core::error::{DBError, DBResult};

/// 统一的访问者错误处理
trait VisitorErrorHandler {
    /// 将访问者错误转换为DBError
    fn handle_visitor_error(&self, error: VisitorError) -> DBError {
        match error {
            VisitorError::TypeMismatch { expected, actual, context } => {
                DBError::Type(format!(
                    "类型不匹配: 期望 {:?}, 实际 {:?}, 上下文: {}",
                    expected, actual, context
                ))
            }
            VisitorError::UndefinedVariable(name) => {
                DBError::Undefined(format!("未定义变量: {}", name))
            }
            VisitorError::UnsupportedOperation(op) => {
                DBError::Unsupported(format!("不支持的操作: {}", op))
            }
            VisitorError::ValidationFailed(msg) => {
                DBError::Validation(msg)
            }
            VisitorError::EvaluationError(msg) => {
                DBError::Execution(format!("求值错误: {}", msg))
            }
        }
    }
}

/// 统一的访问者错误类型
#[derive(Debug, Clone)]
pub enum VisitorError {
    TypeMismatch {
        expected: String,
        actual: String,
        context: String,
    },
    UndefinedVariable(String),
    UnsupportedOperation(String),
    ValidationFailed(String),
    EvaluationError(String),
}
```

### 2.4 错误恢复机制

实现错误恢复和重试机制：

```rust
// src/core/error_recovery.rs
use crate::core::error::{DBError, ErrorSeverity};

/// 错误恢复策略
#[derive(Debug, Clone)]
pub struct ErrorRecoveryPolicy {
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub recoverable_errors: Vec<ErrorType>,
}

impl ErrorRecoveryPolicy {
    /// 判断错误是否可恢复
    pub fn is_recoverable(&self, error: &DBError) -> bool {
        match error {
            DBError::Network(_) => true,
            DBError::Timeout(_) => true,
            DBError::Storage(_) => matches!(error.severity(), ErrorSeverity::Medium),
            _ => false,
        }
    }
    
    /// 获取下次重试延迟
    pub fn get_retry_delay(&self, attempt: u32) -> Duration {
        let base_delay = Duration::from_millis(self.retry_delay_ms);
        let multiplier = self.backoff_multiplier.powi(attempt as i32);
        base_delay.mul_f64(multiplier)
    }
}

/// 带错误恢复的执行器
pub struct RecoveringExecutor<F, T> {
    operation: F,
    policy: ErrorRecoveryPolicy,
    _phantom: std::marker::PhantomData<T>,
}

impl<F, T> RecoveringExecutor<F, T>
where
    F: Fn() -> Result<T, DBError>,
    T: Clone,
{
    pub fn new(operation: F, policy: ErrorRecoveryPolicy) -> Self {
        Self {
            operation,
            policy,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// 执行带恢复的操作
    pub async fn execute(&self) -> Result<T, DBError> {
        let mut attempt = 0;
        loop {
            match (self.operation)() {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if !self.policy.is_recoverable(&error) || attempt >= self.policy.max_retries {
                        return Err(error);
                    }
                    
                    let delay = self.policy.get_retry_delay(attempt);
                    tokio::time::sleep(delay).await;
                    attempt += 1;
                }
            }
        }
    }
}
```

### 2.5 错误日志和监控

实现统一的错误日志和监控：

```rust
// src/core/error_logging.rs
use crate::core::error::{DBError, ErrorSeverity, ErrorContext};
use tracing::{error, warn, info, debug};

/// 错误日志记录器
pub struct ErrorLogger {
    component: String,
    enable_metrics: bool,
}

impl ErrorLogger {
    pub fn new(component: String) -> Self {
        Self {
            component,
            enable_metrics: true,
        }
    }
    
    /// 记录错误
    pub fn log_error(&self, error: &DBError, context: Option<&ErrorContext>) {
        let severity = error.severity();
        let error_msg = format!("[{}] {:?}: {}", self.component, severity, error);
        
        match severity {
            ErrorSeverity::Critical => {
                error!("{}", error_msg);
                self.record_metric("critical_errors", 1);
            }
            ErrorSeverity::High => {
                error!("{}", error_msg);
                self.record_metric("high_severity_errors", 1);
            }
            ErrorSeverity::Medium => {
                warn!("{}", error_msg);
                self.record_metric("medium_severity_errors", 1);
            }
            ErrorSeverity::Low => {
                info!("{}", error_msg);
                self.record_metric("low_severity_errors", 1);
            }
        }
        
        if let Some(ctx) = context {
            debug!("错误上下文: {:?}", ctx);
        }
    }
    
    /// 记录错误指标
    fn record_metric(&self, metric_name: &str, value: u64) {
        if self.enable_metrics {
            // 集成指标收集系统
            metrics::counter!(metric_name, value, "component" => self.component.clone());
        }
    }
    
    /// 记录错误恢复
    pub fn log_recovery(&self, original_error: &DBError, recovery_action: &str) {
        info!(
            "[{}] 错误恢复: 原始错误 {:?}, 恢复动作: {}",
            self.component, original_error, recovery_action
        );
        self.record_metric("error_recoveries", 1);
    }
}
```

## 3. 实施计划

### 3.1 第一阶段：核心错误枚举扩展

1. **扩展 DBError 枚举**
   - 添加新的错误变体（Type, Execution, Syntax等）
   - 实现 From 转换trait
   - 添加错误严重程度判断

2. **统一 Result 类型**
   - 定义 `pub type DBResult<T> = Result<T, DBError>`
   - 替换所有函数签名中的 Result<T, SomeError>

3. **时间估算**：2-3天

### 3.2 第二阶段：错误上下文和恢复

1. **实现 ErrorContext**
   - 添加错误上下文结构体
   - 实现上下文附加逻辑
   - 集成到主要错误处理路径

2. **实现错误恢复机制**
   - 创建 ErrorRecoveryPolicy
   - 实现 RecoveringExecutor
   - 添加重试和退避逻辑

3. **时间估算**：3-4天

### 3.3 第三阶段：访问者错误统一

1. **统一访问者错误**
   - 创建统一的 VisitorError 枚举
   - 实现错误转换逻辑
   - 替换所有访问者相关的错误类型

2. **更新访问者实现**
   - 修改所有访问者trait实现
   - 统一错误处理模式
   - 添加错误恢复支持

3. **时间估算**：4-5天

### 3.4 第四阶段：日志和监控集成

1. **实现 ErrorLogger**
   - 创建错误日志记录器
   - 集成 tracing 日志系统
   - 添加指标收集支持

2. **集成监控**
   - 添加错误指标收集
   - 实现错误告警机制
   - 创建错误报告功能

3. **时间估算**：2-3天

### 3.5 第五阶段：测试和验证

1. **错误处理测试**
   - 编写错误转换测试
   - 测试错误恢复机制
   - 验证错误日志功能

2. **集成测试**
   - 测试完整的错误处理流程
   - 验证错误监控功能
   - 性能测试和优化

3. **时间估算**：3-4天

## 4. 预期收益

### 4.1 代码质量提升
- **一致性**：统一的错误处理模式和类型定义
- **可维护性**：减少重复代码，简化错误处理逻辑
- **可读性**：清晰的错误类型和一致的命名规范

### 4.2 开发效率提升
- **简化开发**：开发者只需要处理一种错误类型
- **减少bug**：消除错误转换中的遗漏和错误
- **快速定位**：统一的错误上下文便于问题定位

### 4.3 系统可靠性提升
- **错误恢复**：自动重试和错误恢复机制
- **监控告警**：实时的错误监控和告警
- **性能优化**：减少错误处理开销

### 4.4 运维能力提升
- **日志统一**：一致的错误日志格式和内容
- **指标监控**：全面的错误指标收集和分析
- **故障诊断**：详细的错误上下文便于故障诊断

## 5. 风险评估

### 5.1 技术风险
- **兼容性问题**：现有代码可能需要大量修改
- **性能影响**：错误转换可能带来额外开销
- **复杂性增加**：统一的错误处理可能增加系统复杂性

### 5.2 缓解措施
- **渐进式迁移**：分阶段实施，避免一次性大规模修改
- **充分测试**：每个阶段都要进行充分的测试验证
- **性能监控**：实施过程中持续监控性能指标
- **回滚机制**：保留回滚到原错误处理机制的能力

### 5.3 成功标准
- 所有错误类型统一到 DBError 枚举
- 错误处理代码减少 30% 以上
- 错误日志一致性达到 95% 以上
- 错误恢复成功率达到 80% 以上
- 系统性能无明显下降