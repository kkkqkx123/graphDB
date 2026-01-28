# Unsafe 使用文档

本文档记录项目中所有unsafe代码的使用原因和安全性分析。

## 内存工具函数中的 unsafe 使用

### 位置
- `src/common/memory.rs` - `memory_utils` 模块

### 原因
内存工具函数需要直接操作内存指针，这是底层内存操作的标准做法。

### 使用场景
以下函数使用了unsafe代码：

1. **`copy_memory(src: *const u8, dest: *mut u8, size: usize)`**
   - **功能**：从源指针复制内存到目标指针
   - **安全性保证**：使用 `ptr::copy_nonoverlapping`，确保源和目标内存区域不重叠
   - **使用场景**：需要高效复制内存块时使用

2. **`set_memory(ptr: *mut u8, value: u8, size: usize)`**
   - **功能**：用指定值填充内存区域
   - **安全性保证**：使用 `ptr::write_bytes`，标准库提供的内存填充函数
   - **使用场景**：需要快速初始化内存区域时使用

### 安全性分析
这些函数的安全性依赖于调用者：
1. **调用者责任**：调用者必须确保指针有效且内存区域已正确分配
2. **不重叠保证**：`copy_memory` 使用 `copy_nonoverlapping`，自动防止重叠问题
3. **标准库保证**：底层使用标准库函数，已经过充分测试和验证

### 代码示例
```rust
pub unsafe fn copy_memory(src: *const u8, dest: *mut u8, size: usize) {
    ptr::copy_nonoverlapping(src, dest, size);
}

pub unsafe fn set_memory(ptr: *mut u8, value: u8, size: usize) {
    ptr::write_bytes(ptr, value, size);
}
```

### 替代方案
如果需要完全避免unsafe，可以考虑：
1. 使用 `slice.copy_from_slice()` 替代 `copy_memory`（需要先转换为slice）
2. 使用 `slice.fill()` 替代 `set_memory`（需要先转换为slice）
3. 但这些替代方案会增加额外的边界检查和转换开销

### 为什么保留 unsafe
1. **性能考虑**：直接指针操作避免了额外的边界检查和转换
2. **灵活性**：支持任意内存地址的操作，不限于slice
3. **标准实践**：底层内存操作使用unsafe是Rust生态的标准做法
4. **明确责任**：unsafe标记明确告知调用者需要确保安全性

## UUID 非标准转换使用

### 位置
- `src/query/planner/statements/statement_planner.rs`
- `src/query/planner/statements/match_planner.rs`
- `src/query/planner/statements/match_statement_planner.rs`

### 使用原因
生成执行计划ID时，需要一个唯一的标识符。当前实现将UUID v4的前8字节转换为i64。

### 代码示例
```rust
let uuid = uuid::Uuid::new_v4();
let uuid_bytes = uuid.as_bytes();
let id = i64::from_ne_bytes([
    uuid_bytes[0],
    uuid_bytes[1],
    uuid_bytes[2],
    uuid_bytes[3],
    uuid_bytes[4],
    uuid_bytes[5],
    uuid_bytes[6],
    uuid_bytes[7],
]);
plan.set_id(id);
```

### 潜在问题
1. **碰撞风险**：仅使用UUID的8字节（64位），相比完整UUID（128位）碰撞概率增加
2. **非标准做法**：UUID转换为i64不是标准做法，可能导致兼容性问题
3. **可预测性**：如果系统需要真正的不可预测ID，这种方式可能不够安全

### 使用场景
此ID主要用于：
- 执行计划的内部标识
- 日志和调试输出
- 计划缓存的键（如果需要）

### 何时需要修改
1. 如果需要真正的全局唯一ID，考虑使用完整的UUID字符串
2. 如果需要更高的安全性，考虑使用加密安全的随机数生成器
3. 如果需要分布式环境下的唯一性，考虑使用snowflake算法或类似方案
