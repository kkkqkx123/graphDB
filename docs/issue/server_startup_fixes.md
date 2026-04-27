# 服务器启动问题修复记录

## 修复概要

本次修复解决了 GraphDB 服务器启动过程中的多个关键问题，使服务器能够正常启动并处理请求。

## 已修复问题

### 1. Tokio Runtime 未运行

**问题描述：**
启动服务器时出现错误：
```
thread 'main' panicked at src\api\mod.rs:101:26:
there is no reactor running, must be called from the context of a Tokio 1.x runtime
```

**根本原因：**
`start_service_with_config` 是同步函数，但内部使用了 `tokio::runtime::Handle::current()` 来获取当前 runtime，而此时没有 runtime 在运行。

**修复方案：**
- 将 `start_service_with_config` 改为 async 函数
- 在 `main.rs` 中创建 Tokio runtime 并使用 `rt.block_on()` 调用

**修改文件：**
- `src/main.rs`
- `src/api/mod.rs`

---

### 2. 嵌套 Runtime 错误

**问题描述：**
修复问题 #1 后出现新错误：
```
Cannot start a runtime from within a runtime. This happens because a function (like `block_on`) 
attempted to block the current thread while the thread is being used to drive asynchronous tasks.
```

**根本原因：**
`start_service_with_config` 函数内部又创建了一个新的 Tokio runtime（`tokio::runtime::Runtime::new()`），导致嵌套 runtime 错误。

**修复方案：**
- 移除 `start_service_with_config` 内部的 runtime 创建
- 直接使用 async/await 模式

**修改文件：**
- `src/api/mod.rs`

---

### 3. VectorManager 连接失败导致 Panic

**问题描述：**
当 Qdrant 未运行时，服务器启动会 panic：
```
Failed to create VectorManager: ConnectionFailed("Connection failed: Failed to connect to Qdrant at http://localhost:6333")
```

**根本原因：**
代码使用 `expect()` 处理 `VectorManager::new()` 的结果，连接失败时直接 panic。

**修复方案：**
- 使用 `match` 处理 `VectorManager::new()` 的结果
- 连接失败时记录警告日志，继续启动（禁用向量搜索功能）

**修改文件：**
- `src/api/mod.rs`

**代码变更：**
```rust
// 修复前
let vector_manager = Arc::new(
    rt.block_on(VectorManager::new(config.vector.clone()))
        .expect("Failed to create VectorManager"),
);

// 修复后
match VectorManager::new(config.vector.clone()).await {
    Ok(vm) => {
        let vector_manager = Arc::new(vm);
        // ... 启用向量搜索
        info!("Vector index sync enabled");
    }
    Err(e) => {
        warn!("Failed to create VectorManager: {}. Vector search will be disabled.", e);
    }
}
```

---

### 4. shutdown_signal 同步/异步不匹配

**问题描述：**
`shutdown_signal` 函数是同步的，但在 async 上下文中调用。

**修复方案：**
- 将 `shutdown_signal` 改为 async 函数
- 移除内部创建的临时 runtime

**修改文件：**
- `src/api/mod.rs`

---

### 5. 认证不工作（Login 未创建 Session）

**问题描述：**
E2E 测试中认证成功（返回 session_id），但后续查询返回 401 Unauthorized。

**根本原因：**
`login` 处理函数只是返回一个模拟的 session_id（12345），并没有真正在 session_manager 中创建 session。

**修复方案：**
- 修改 `login` 函数，调用 `session_manager.create_session()` 真正创建 session
- 返回真实的 session_id

**修改文件：**
- `src/api/server/http/handlers/auth.rs`

**代码变更：**
```rust
// 修复前
pub async fn login<S: StorageClient + Clone + Send + Sync + 'static>(
    State(_state): State<AppState<S>>,  // 注意：state 未被使用
    Json(request): Json<LoginRequest>,
) -> Result<JsonResponse<LoginResponse>, HttpError> {
    // 只是返回模拟结果
    Ok(JsonResponse(LoginResponse {
        session_id: 12345,  // 模拟 ID
        username: request.username,
        expires_at: None,
    }))
}

// 修复后
pub async fn login<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Json(request): Json<LoginRequest>,
) -> Result<JsonResponse<LoginResponse>, HttpError> {
    let session_manager = state.server.get_session_manager();
    
    let session = session_manager
        .create_session(request.username.clone(), "127.0.0.1".to_string())
        .await
        .map_err(|e| HttpError::InternalError(format!("Failed to create session: {}", e)))?;
    
    let session_id = session.id();
    info!("Created session {} for user {}", session_id, request.username);
    
    Ok(JsonResponse(LoginResponse {
        session_id,
        username: request.username,
        expires_at: None,
    }))
}
```

---

## 验证结果

### 服务器启动测试
```
============================================================
GraphDB Server Startup Integration Test
============================================================
✓ PASS: test_01_server_binary_exists - OK
✓ PASS: test_02_config_file_exists - OK
✓ PASS: test_03_port_available - OK
✓ PASS: test_04_start_server - OK
✓ PASS: test_05_health_endpoint - OK
✓ PASS: test_06_api_endpoints - OK
✓ PASS: test_07_graceful_shutdown - OK

Total: 7 tests, 7 passed, 0 failed
```

### E2E 基础验证
```
============================================================
E2E Verification Summary
============================================================
✓ PASS: Server Startup
✓ PASS: Health Check
✓ PASS: Data Generation
✓ PASS: Basic Query (6/6 查询通过)
✓ PASS: Cleanup

Total: 5/5 steps passed
✓ E2E Verification PASSED
```

## 遗留问题

详见 `e2e_test_failures.md`，主要包括：
- GQL 语法与解析器兼容性问题
- 部分查询功能（MATCH, GO, LOOKUP）实现不完整
- EXPLAIN/PROFILE 功能待完善

## 相关文件

- 启动测试：`tests/server_startup_test.py`
- E2E 验证：`tests/e2e_verify.py`
- E2E 客户端：`tests/e2e/graphdb_client.py`
