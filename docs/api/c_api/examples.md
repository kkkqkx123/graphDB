# GraphDB C API 使用示例

## 示例 1：基本查询

```c
#include <stdio.h>
#include "graphdb.h"

int main() {
    graphdb_t *db = NULL;
    graphdb_session_t *session = NULL;
    graphdb_result_t *result = NULL;

    // 打开数据库
    int rc = graphdb_open("test.db", &db);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "无法打开数据库: %s\n", graphdb_errmsg(db));
        return 1;
    }

    // 创建会话
    rc = graphdb_session_create(db, &session);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "无法创建会话: %s\n", graphdb_errmsg(db));
        goto cleanup;
    }

    // 切换图空间
    rc = graphdb_session_use_space(session, "my_space");
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "无法切换图空间: %s\n", graphdb_errmsg(db));
        goto cleanup;
    }

    // 执行查询
    rc = graphdb_execute(session, "MATCH (n) RETURN n LIMIT 10", &result);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "查询失败: %s\n", graphdb_errmsg(db));
        goto cleanup;
    }

    // 处理结果
    int row_count = graphdb_row_count(result);
    printf("查询结果: %d 行\n", row_count);

cleanup:
    if (result) graphdb_result_free(result);
    if (session) graphdb_session_close(session);
    if (db) graphdb_close(db);

    return 0;
}
```

## 示例 2：预编译语句

```c
#include <stdio.h>
#include "graphdb.h"

int main() {
    graphdb_t *db = NULL;
    graphdb_session_t *session = NULL;
    graphdb_stmt_t *stmt = NULL;

    graphdb_open("test.db", &db);
    graphdb_session_create(db, &session);
    graphdb_session_use_space(session, "my_space");

    // 准备语句
    int rc = graphdb_prepare(session, "MATCH (n:User {id: $id}) RETURN n", &stmt);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "准备语句失败: %s\n", graphdb_errmsg(db));
        goto cleanup;
    }

    // 绑定参数并执行
    for (int i = 1; i <= 10; i++) {
        graphdb_bind_int(stmt, 1, i);

        rc = graphdb_step(stmt);
        if (rc == GRAPHDB_OK) {
            printf("查询用户 %d 成功\n", i);
        }

        graphdb_reset(stmt);
    }

cleanup:
    if (stmt) graphdb_finalize(stmt);
    if (session) graphdb_session_close(session);
    if (db) graphdb_close(db);

    return 0;
}
```

## 示例 3：事务操作

```c
#include <stdio.h>
#include "graphdb.h"

int main() {
    graphdb_t *db = NULL;
    graphdb_session_t *session = NULL;
    graphdb_txn_t *txn = NULL;

    graphdb_open("test.db", &db);
    graphdb_session_create(db, &session);
    graphdb_session_use_space(session, "my_space");

    // 开始事务
    int rc = graphdb_txn_begin(session, &txn);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "无法开始事务: %s\n", graphdb_errmsg(db));
        goto cleanup;
    }

    // 在事务中执行操作
    rc = graphdb_txn_execute(txn, "CREATE TAG user(name string)", NULL);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "创建标签失败: %s\n", graphdb_errmsg(db));
        graphdb_txn_rollback(txn);
        goto cleanup;
    }

    // 提交事务
    rc = graphdb_txn_commit(txn);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "提交事务失败: %s\n", graphdb_errmsg(db));
    } else {
        printf("事务提交成功\n");
    }

cleanup:
    if (session) graphdb_session_close(session);
    if (db) graphdb_close(db);

    return 0;
}
```

## 编译示例

```bash
# Linux/macOS
gcc -o example example.c -Iinclude -Ltarget/release -lgraphdb

# Windows (MSVC)
cl example.c /Iinclude /LIBPATH:target\release graphdb.lib

# 运行
./example
```
