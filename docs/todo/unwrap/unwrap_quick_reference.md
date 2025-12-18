# GraphDB unwrap() 改进快速参考

## 常见模式和改进方案

### 1. 锁操作

```rust
// ❌ 当前代码
let mut val = self.value.lock().unwrap();

// ✅ 改进方案 1：使用 expect()
let mut val = self.value.lock()
    .expect("Counter lock should not be poisoned");

// ✅ 改进方案 2：完整错误处理
match self.value.lock() {
    Ok(mut val) => *val += 1,
    Err(poisoned) => {
        log::warn!("Lock is poisoned, attempting recovery");
        *poisoned.into_inner() += 1;
    }
}
```

### 2. Option 类型

```rust
// ❌ 当前代码
let value = option.unwrap();

// ✅ 改进方案 1：使用 expect()
let value = option.expect("Value should exist");

// ✅ 改进方案 2：错误传播
let value = option.ok_or_else(|| Error::MissingValue)?;

// ✅ 改进方案 3：提供默认值
let value = option.unwrap_or(default_value);
```

### 3. Result 类型

```rust
// ❌ 当前代码
let result = operation().unwrap();

// ✅ 改进方案 1：使用 ? 操作符
let result = operation()?;

// ✅ 改进方案 2：上下文错误
let result = operation().map_err(|e| Error::OperationFailed {
    context: "Failed to initialize storage".to_string(),
    source: e,
})?;

// ✅ 改进方案 3：使用 expect()
let result = operation().expect("Operation should succeed");
```

### 4. 迭代器操作

```rust
// ❌ 当前代码
let first = collection.iter().next().unwrap();
let min = values.iter().min().unwrap();

// ✅ 改进方案 1：使用 expect()
let first = collection.iter().next()
    .expect("Collection should not be empty");
let min = values.iter().min()
    .expect("Values should not be empty when calculating min");

// ✅ 改进方案 2：提供默认值
let first = collection.iter().next().unwrap_or(&default);
let min = values.iter().min().copied().unwrap_or(0);
```

## 优先级指南

### 🔴 高优先级（立即处理）
- 所有 `Mutex::lock().unwrap()`
- 所有 `RwLock::read().unwrap()` 和 `RwLock::write().unwrap()`
- 存储初始化和关键系统操作的 `unwrap()`

### 🟡 中优先级（计划处理）
- `Option::unwrap()` 在业务逻辑中
- `Result::unwrap()` 在非关键路径
- 迭代器操作的 `unwrap()`

### 🟢 低优先级（可选处理）
- 已经充分验证的场景
- 性能关键路径（需要权衡）

## 错误信息最佳实践

### 提供上下文
```rust
// ❌ 不好的错误信息
.expect("Failed")

// ✅ 好的错误信息
.expect("Failed to acquire counter lock in stats module")
```

### 包含关键信息
```rust
// ❌ 不好的错误信息
.expect("Invalid state")

// ✅ 好的错误信息
.expect("PlanNode should have dependencies initialized")
```

## 常见反模式

### 1. 链式 unwrap()
```rust
// ❌ 反模式
let value = some_option.unwrap().get_field().unwrap().process().unwrap();

// ✅ 改进方案
let value = some_option
    .ok_or_else(|| Error::MissingValue)?
    .get_field()
    .ok_or_else(|| Error::MissingField)?
    .process()?;
```

### 2. 在循环中使用 unwrap()
```rust
// ❌ 反模式
for item in collection {
    process(item.unwrap()); // 如果任何一个失败，整个循环崩溃
}

// ✅ 改进方案
for item in collection {
    match item {
        Ok(valid_item) => process(valid_item),
        Err(e) => {
            log::error!("Failed to process item: {}", e);
            continue; // 或者决定是否中断
        }
    }
}
```

### 3. 忽略错误可能性
```rust
// ❌ 反模式
let config = load_config().unwrap(); // 假设总是成功

// ✅ 改进方案
let config = load_config().map_err(|e| {
    log::error!("Failed to load configuration: {}", e);
    Error::ConfigurationFailed(e.to_string())
})?;
```

## 测试策略

### 1. 测试错误路径
```rust
#[test]
fn test_lock_poison_recovery() {
    // 创建锁并故意污染它
    let mutex = Mutex::new(42);
    {
        let _guard = mutex.lock().unwrap();
        std::panic::catch_unwind(|| {
            let _guard = mutex.lock().unwrap();
            panic!("Intentional panic to poison the lock");
        }).unwrap_err();
    }
    
    // 测试恢复逻辑
    let result = safe_increment(&mutex);
    assert!(result.is_ok());
}
```

### 2. 测试边界条件
```rust
#[test]
fn test_empty_collection_handling() {
    let empty_vec: Vec<i32> = vec![];
    
    // 测试改进后的代码不会 panic
    let result = calculate_stats(&empty_vec);
    assert!(result.is_ok());
    
    let stats = result.unwrap();
    assert_eq!(stats.min, 0); // 默认值
    assert_eq!(stats.max, 0); // 默认值
}
```

## 代码审查检查点

- [ ] 每个 `unwrap()` 都有明确的理由
- [ ] 错误信息提供了足够的上下文
- [ ] 考虑了错误恢复的可能性
- [ ] 添加了适当的日志记录
- [ ] 测试覆盖了错误路径
- [ ] 性能影响已评估

## 工具和辅助函数

### 推荐的辅助函数
```rust
// 安全的锁操作
pub fn safe_lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<T>, Error> { ... }

// 期望值存在
pub fn expect_some<T>(option: Option<T>, msg: &str) -> Result<T, Error> { ... }

// 结果映射
pub fn map_result<T, E, F>(result: Result<T, E>, mapper: F) -> Result<T, Error> 
where F: FnOnce(E) -> Error { ... }
```

### 推荐的依赖
```toml
[dependencies]
thiserror = "1.0"  # 错误处理
log = "0.4"        # 日志记录
```

## 记住

1. **unwrap() 不是邪恶的**，但需要谨慎使用
2. **错误信息应该有用**，帮助调试和维护
3. **考虑错误恢复**，而不仅仅是失败
4. **测试错误路径**，确保它们按预期工作
5. **渐进式改进**，不要试图一次性修复所有问题