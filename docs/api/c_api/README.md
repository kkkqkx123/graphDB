# GraphDB C API

GraphDB C API 提供了 GraphDB 图数据库的 C 语言接口，允许 C/C++ 程序直接调用 GraphDB 的功能。

## 版本信息

- **版本**: 0.1.0
- **许可证**: Apache-2.0
- **仓库**: https://github.com/vesoft-inc/nebula

## 快速开始

### 编译

```bash
# 编译 C 库
cargo build --release --features c_api

# 头文件会自动生成到 include/graphdb.h
```

### 基本使用

```c
#include <stdio.h>
#include "graphdb.h"

int main() {
    graphdb_t *db = NULL;
    int rc = graphdb_open("test.db", &db);
    if (rc != GRAPHDB_OK) {
        fprintf(stderr, "无法打开数据库: %s\n", graphdb_errmsg(db));
        return 1;
    }

    // 使用数据库...

    graphdb_close(db);
    return 0;
}
```

## API 参考

### 数据库管理

- `graphdb_open` - 打开数据库
- `graphdb_open_memory` - 打开内存数据库
- `graphdb_close` - 关闭数据库
- `graphdb_errcode` - 获取错误码
- `graphdb_errmsg` - 获取错误信息

### 错误码

- `GRAPHDB_OK` - 成功
- `GRAPHDB_ERROR` - 一般错误
- `GRAPHDB_IOERR` - IO 错误
- `GRAPHDB_NOMEM` - 内存不足
- ... 更多错误码请参考头文件

## 更多信息

- [API 参考文档](api_reference.md)
- [使用指南](user_guide.md)
- [示例代码](examples/)
- [故障排除](troubleshooting.md)
