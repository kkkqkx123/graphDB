# GraphDB C API 故障排除

## 常见问题

### 1. 编译错误

#### 问题：找不到 graphdb.h 头文件

**原因**：头文件未生成或路径不正确

**解决方案**：
```bash
# 确保使用 c_api 特性编译
cargo build --release --features c_api

# 检查头文件是否存在
ls include/graphdb.h

# 编译时指定正确的头文件路径
gcc -o example example.c -Iinclude -Ltarget/release -lgraphdb
```

#### 问题：找不到 libgraphdb.so 或 libgraphdb.dll

**原因**：库文件未生成或路径不正确

**解决方案**：
```bash
# 检查库文件是否存在
ls target/release/libgraphdb.so  # Linux/macOS
ls target/release/graphdb.dll      # Windows

# 编译时指定正确的库路径
gcc -o example example.c -Iinclude -Ltarget/release -lgraphdb

# 运行时设置库路径
export LD_LIBRARY_PATH=target/release:$LD_LIBRARY_PATH  # Linux
export DYLD_LIBRARY_PATH=target/release:$DYLD_LIBRARY_PATH  # macOS
```

### 2. 运行时错误

#### 问题：segmentation fault (段错误)

**原因**：使用了空指针或已释放的句柄

**解决方案**：
```c
// 检查指针是否为空
if (db == NULL) {
    fprintf(stderr, "数据库句柄为空\n");
    return 1;
}

// 检查函数返回值
int rc = graphdb_open("test.db", &db);
if (rc != GRAPHDB_OK) {
    fprintf(stderr, "打开数据库失败: %s\n", graphdb_errmsg(db));
    return 1;
}

// 不要重复释放
graphdb_close(db);
db = NULL;  // 设置为 NULL 避免重复释放
```

#### 问题：内存泄漏

**原因**：未正确释放资源

**解决方案**：
```c
// 使用 goto 进行统一清理
graphdb_t *db = NULL;
graphdb_session_t *session = NULL;
graphdb_result_t *result = NULL;

int rc = graphdb_open("test.db", &db);
if (rc != GRAPHDB_OK) {
    goto cleanup;
}

rc = graphdb_session_create(db, &session);
if (rc != GRAPHDB_OK) {
    goto cleanup;
}

rc = graphdb_execute(session, "MATCH (n) RETURN n", &result);
if (rc != GRAPHDB_OK) {
    goto cleanup;
}

// 处理结果...

cleanup:
    if (result) graphdb_result_free(result);
    if (session) graphdb_session_close(session);
    if (db) graphdb_close(db);
```

### 3. 性能问题

#### 问题：查询执行缓慢

**原因**：未使用预编译语句或未创建索引

**解决方案**：
```c
// 使用预编译语句
graphdb_stmt_t *stmt = NULL;
graphdb_prepare(session, "MATCH (n:User {id: $id}) RETURN n", &stmt);

// 重用语句
for (int i = 0; i < 1000; i++) {
    graphdb_bind_int(stmt, 1, i);
    graphdb_step(stmt);
    graphdb_reset(stmt);
}

graphdb_finalize(stmt);
```

#### 问题：内存占用过高

**原因**：结果集过大或未及时释放

**解决方案**：
```c
// 使用 LIMIT 限制结果集大小
graphdb_execute(session, "MATCH (n) RETURN n LIMIT 1000", &result);

// 及时释放结果
graphdb_result_free(result);
result = NULL;
```

### 4. 并发问题

#### 问题：多线程访问时崩溃

**原因**：句柄不是线程安全的

**解决方案**：
```c
// 数据库句柄可以跨线程共享
graphdb_t *db = NULL;
graphdb_open("test.db", &db);

// 每个线程使用独立的会话
void *thread_func(void *arg) {
    graphdb_session_t *session = NULL;
    graphdb_session_create(db, &session);

    // 使用会话...

    graphdb_session_close(session);
    return NULL;
}

// 创建多个线程
pthread_t threads[10];
for (int i = 0; i < 10; i++) {
    pthread_create(&threads[i], NULL, thread_func, NULL);
}

// 等待线程完成
for (int i = 0; i < 10; i++) {
    pthread_join(threads[i], NULL);
}

graphdb_close(db);
```

## 调试技巧

### 1. 启用详细日志

```bash
# 设置日志级别
export RUST_LOG=debug

# 运行程序
./example
```

### 2. 使用内存检查工具

```bash
# 使用 Valgrind 检测内存泄漏
valgrind --leak-check=full ./example

# 使用 AddressSanitizer 检测内存错误
gcc -fsanitize=address -g -o example example.c -Iinclude -Ltarget/release -lgraphdb
./example
```

### 3. 使用调试器

```bash
# 使用 GDB
gdb ./example
(gdb) run
(gdb) backtrace  # 查看调用栈
```

## 获取帮助

如果以上解决方案无法解决你的问题，请：

1. 查看错误信息：`graphdb_errmsg(db)`
2. 查看日志文件
3. 提交 issue 到 GitHub: https://github.com/vesoft-inc/nebula/issues
4. 提供详细的错误信息和复现步骤
