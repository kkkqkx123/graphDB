# GraphDB 数据定义语言 (DDL)

## 概述

数据定义语言 (DDL) 用于定义和管理图数据库的Schema，包括标签、边类型、索引等的创建、修改和删除。

---

## 1. CREATE TAG - 创建标签

### 功能
定义节点标签及其属性。

### 语法结构
```cypher
CREATE TAG [IF NOT EXISTS] <tag_name> (
    <prop_name>: <prop_type> [NOT NULL | NULL] [DEFAULT <value>] [COMMENT '<text>']
    [, <prop_name>: <prop_type> [NOT NULL | NULL] [DEFAULT <value>] [COMMENT '<text>'] ...]
    [, ttl_duration=<seconds>]
    [, ttl_col=<prop_name>]
)
```

### 关键特性
- 支持多种数据类型
- 支持IF NOT EXISTS
- 支持NOT NULL约束
- 支持DEFAULT默认值
- 支持COMMENT属性注释
- 支持TTL自动过期

### 支持的数据类型
| 类型 | 说明 |
|------|------|
| INT/INT8/INT16/INT32/INT64 | 整数类型 |
| FLOAT/DOUBLE | 浮点数类型 |
| STRING/VARCHAR/TEXT | 字符串类型 |
| BOOL/BOOLEAN | 布尔类型 |
| DATE | 日期类型 |
| TIMESTAMP | 时间戳类型 |
| DATETIME | 日期时间类型 |

### 约束说明
| 约束 | 语法 | 默认值 | 说明 |
|------|------|--------|------|
| NOT NULL | `prop: TYPE NOT NULL` | 未指定时默认可空 | 属性值不能为空，插入数据时必须提供值 |
| NULL | `prop: TYPE NULL` | ✅ **默认行为** | 属性值可为空，插入数据时可不提供值 |
| DEFAULT | `prop: TYPE DEFAULT <value>` | 未指定时无默认值 | 插入数据时如未提供值，自动使用默认值 |
| COMMENT | `prop: TYPE COMMENT 'text'` | 未指定时无注释 | 属性的描述说明，仅用于文档目的 |

#### 约束默认值详细说明

**NULL 约束（默认可空）**
- 当不指定 `NOT NULL` 或 `NULL` 时，属性**默认可空**（等同于 `NULL`）
- 示例：`name: STRING` 等价于 `name: STRING NULL`

**DEFAULT 约束（默认无默认值）**
- 当不指定 `DEFAULT` 时，属性**没有默认值**
- 插入数据时如未提供值且属性可为空，则填充 `NULL`
- 如属性有 `NOT NULL` 约束且无默认值，插入时必须提供值，否则会报错

**COMMENT 约束（默认无注释）**
- 当不指定 `COMMENT` 时，属性**没有注释**
- 注释仅用于文档说明，不影响数据存储和查询

#### 约束组合规则

| 场景 | 语法示例 | 插入行为 |
|------|----------|----------|
| 仅类型 | `age: INT` | 可空，无默认值，不提供值时填充 NULL |
| NOT NULL | `age: INT NOT NULL` | 非空，无默认值，**必须**提供值 |
| NOT NULL + DEFAULT | `age: INT NOT NULL DEFAULT 0` | 非空，有默认值，不提供值时使用默认值 0 |
| DEFAULT | `age: INT DEFAULT 0` | 可空，有默认值，不提供值时使用默认值 0 |
| NULL + DEFAULT | `age: INT NULL DEFAULT 0` | 可空，有默认值，不提供值时使用默认值 0 |

### TTL说明
TTL（Time To Live）用于自动清理过期数据。

**TTL 参数：**
| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `ttl_duration` | INT | `0`（禁用TTL） | TTL持续时间（秒），0表示禁用 |
| `ttl_col` | STRING | 无 | 用于计算过期时间的属性名，必须是TIMESTAMP或INT类型 |

**TTL 默认行为：**
- 当不指定 `ttl_duration` 和 `ttl_col` 时，**TTL 默认禁用**
- 仅指定 `ttl_duration` 而不指定 `ttl_col` 时，TTL 不会生效（需要两者配合）
- 建议同时指定 `ttl_duration` 和 `ttl_col`，或都不指定

**TTL 工作原理：**
1. 数据插入时，记录 `ttl_col` 指定属性的值作为基准时间
2. 当 `当前时间 > 基准时间 + ttl_duration` 时，数据被视为过期
3. 过期数据会在后台自动清理（或在查询时被过滤）

**TTL 使用场景：**
- 会话数据自动清理（如用户登录令牌）
- 临时数据过期删除（如验证码、缓存数据）
- 日志数据定期归档（如操作日志保留30天）

### 示例
```cypher
-- 基础创建
CREATE TAG IF NOT EXISTS person(name: STRING, age: INT, created_at: TIMESTAMP)

-- 带约束创建
CREATE TAG Person(
    id: INT NOT NULL COMMENT '主键ID',
    name: STRING NOT NULL DEFAULT 'unknown' COMMENT '姓名',
    age: INT DEFAULT 0 COMMENT '年龄',
    email: STRING NULL COMMENT '邮箱'
)

-- 带TTL创建（数据1年后自动过期）
CREATE TAG Session(
    token: STRING NOT NULL,
    user_id: INT NOT NULL,
    created_at: TIMESTAMP NOT NULL,
    ttl_duration=31536000,
    ttl_col=created_at
)
```

---

## 2. CREATE EDGE - 创建边类型

### 功能
定义边类型及其属性。

### 语法结构
```cypher
CREATE EDGE [IF NOT EXISTS] <edge_type> (
    <prop_name>: <prop_type> [NOT NULL | NULL] [DEFAULT <value>] [COMMENT '<text>']
    [, <prop_name>: <prop_type> [NOT NULL | NULL] [DEFAULT <value>] [COMMENT '<text>'] ...]
    [, ttl_duration=<seconds>]
    [, ttl_col=<prop_name>]
)
```

### 关键特性
- 支持多种数据类型
- 支持IF NOT EXISTS
- 支持NOT NULL约束
- 支持DEFAULT默认值
- 支持COMMENT属性注释
- 支持TTL自动过期

> **注意：** 约束默认值与 CREATE TAG 相同：属性默认可空，无默认值时插入 NULL，TTL 默认禁用。

### 示例
```cypher
-- 基础创建
CREATE EDGE IF NOT EXISTS follow(degree: FLOAT, since: TIMESTAMP)

-- 带约束创建
CREATE EDGE WORKS_AT(
    since: DATE NOT NULL COMMENT '入职日期',
    department: STRING DEFAULT 'unknown' COMMENT '部门',
    active: BOOL DEFAULT true COMMENT '是否在职'
)

-- 带TTL创建（数据30天后自动过期）
CREATE EDGE TempRelation(
    data: STRING,
    expire_at: TIMESTAMP NOT NULL,
    ttl_duration=2592000,
    ttl_col=expire_at
)
```

---

## 3. CREATE SPACE - 创建图空间

### 功能
创建图空间（数据库实例）。

### 语法结构
```cypher
CREATE SPACE [IF NOT EXISTS] <space_name> [(vid_type=<type>, partition_num=<n>, replica_factor=<n>, comment="<text>")]
```

### 关键特性
- 支持IF NOT EXISTS
- 可配置VID类型（INT64, FIXEDSTRING32等）
- 可配置分区数
- 可配置副本因子
- 可添加注释

### 示例
```cypher
-- 基本创建
CREATE SPACE IF NOT EXISTS test_space

-- 带参数创建
CREATE SPACE test_space(vid_type=FIXEDSTRING32, partition_num=10, replica_factor=3, comment="测试空间")
```

---

## 4. CREATE INDEX - 创建索引

### 功能
在标签或边类型上创建索引。

### 语法结构
```cypher
CREATE INDEX [IF NOT EXISTS] <index_name> ON <tag_or_edge_name> (<prop_list>)
```

### 示例
```cypher
CREATE INDEX IF NOT EXISTS idx_person_name ON person(name)
CREATE INDEX idx_follow_degree ON follow(degree)
```

---

## 5. ALTER TAG - 修改标签

### 功能
修改标签定义。

### 语法结构
```cypher
ALTER TAG <tag_name> ADD (<prop_name>: <prop_type> [, <prop_name>: <prop_type> ...])
ALTER TAG <tag_name> DROP (<prop_name> [, <prop_name> ...])
ALTER TAG <tag_name> CHANGE (<old_prop> <new_prop>: <prop_type>)
```

### 关键特性
- 支持添加属性
- 支持删除属性
- 支持重命名属性
- 支持修改属性类型

### 示例
```cypher
ALTER TAG person ADD (email: STRING, phone: STRING)
ALTER TAG person DROP (temp_field)
ALTER TAG person CHANGE (old_name new_name: STRING)
```

---

## 6. ALTER EDGE - 修改边类型

### 功能
修改边类型定义。

### 语法结构
```cypher
ALTER EDGE <edge_type> ADD (<prop_name>: <prop_type> [, <prop_name>: <prop_type> ...])
ALTER EDGE <edge_type> DROP (<prop_name> [, <prop_name> ...])
ALTER EDGE <edge_type> CHANGE (<old_prop> <new_prop>: <prop_type>)
```

### 关键特性
- 支持添加属性
- 支持删除属性
- 支持重命名属性
- 支持修改属性类型

### 示例
```cypher
ALTER EDGE follow ADD (note: STRING)
ALTER EDGE follow DROP (old_field)
```

---

## 7. DROP TAG - 删除标签

### 功能
删除标签定义。

### 语法结构
```cypher
DROP TAG [IF EXISTS] <tag_name> [, <tag_name> ...]
```

### 关键特性
- 支持IF EXISTS
- 支持批量删除
- 级联删除相关数据

### 示例
```cypher
DROP TAG IF EXISTS person, company
```

---

## 8. DROP EDGE - 删除边类型

### 功能
删除边类型定义。

### 语法结构
```cypher
DROP EDGE [IF EXISTS] <edge_type> [, <edge_type> ...]
```

### 关键特性
- 支持IF EXISTS
- 支持批量删除
- 级联删除相关数据

### 示例
```cypher
DROP EDGE IF EXISTS follow, like
```

---

## 9. DROP SPACE - 删除图空间

### 功能
删除图空间。

### 语法结构
```cypher
DROP SPACE [IF EXISTS] <space_name>
```

### 示例
```cypher
DROP SPACE IF EXISTS test_space
```

---

## 10. DROP INDEX - 删除索引

### 功能
删除索引。

### 语法结构
```cypher
DROP INDEX [IF EXISTS] <index_name> [ON <space_name>]
DROP TAG INDEX [IF EXISTS] <index_name> [ON <space_name>]
DROP EDGE INDEX [IF EXISTS] <index_name> [ON <space_name>]
```

### 示例
```cypher
DROP INDEX IF EXISTS idx_person_name
DROP TAG INDEX idx_person_name ON test_space
```

---

## 11. DESC/DESCRIBE - 描述对象

### 功能
显示标签、边类型或用户的定义。

### 语法结构
```cypher
DESCRIBE TAG <tag_name> [IN <space_name>]
DESCRIBE EDGE <edge_type> [IN <space_name>]
DESCRIBE SPACE <space_name>
```

### 关键特性
- 显示属性列表
- 显示属性类型
- 显示索引信息

### 示例
```cypher
DESCRIBE TAG person
DESCRIBE EDGE follow
DESCRIBE SPACE test_space
```

---

## 12. SHOW - 显示信息

### 功能
显示数据库中的各种信息。

### 语法结构
```cypher
SHOW SPACES
SHOW TAGS
SHOW EDGES
SHOW INDEXES
```

### 示例
```cypher
SHOW SPACES
SHOW TAGS
SHOW EDGES
```

---

## 13. SHOW CREATE - 显示创建语句

### 功能
显示对象的完整创建语句（DDL），便于查看对象定义或迁移数据。

### 语法结构
```cypher
SHOW CREATE SPACE <space_name>
SHOW CREATE TAG <tag_name>
SHOW CREATE EDGE <edge_type>
SHOW CREATE INDEX <index_name>
```

### 关键特性
- 显示完整的CREATE语句
- 包含所有属性定义
- 包含约束条件（NOT NULL, DEFAULT等）
- 包含TTL配置
- 包含注释信息

### 示例
```cypher
-- 查看图空间创建语句
SHOW CREATE SPACE test_space

-- 查看标签创建语句
SHOW CREATE TAG Person

-- 查看边类型创建语句
SHOW CREATE EDGE KNOWS

-- 查看索引创建语句
SHOW CREATE INDEX idx_person_name
```

### 返回结果
```
+------------------------------------------------------------------------+
| create_statement                                                       |
+------------------------------------------------------------------------+
| CREATE TAG IF NOT EXISTS Person(                                       |
|     id: INT NOT NULL COMMENT '主键ID',                                 |
|     name: STRING NOT NULL DEFAULT 'unknown' COMMENT '姓名',            |
|     age: INT DEFAULT 0 COMMENT '年龄',                                 |
|     created_at: TIMESTAMP,                                             |
|     ttl_duration=31536000,                                             |
|     ttl_col=created_at                                                 |
| )                                                                      |
+------------------------------------------------------------------------+
```

---

## 功能汇总表

### 支持的特性

| 功能 | CREATE TAG | CREATE EDGE | 说明 |
|------|------------|-------------|------|
| IF NOT EXISTS | ✅ | ✅ | 避免重复创建错误 |
| NOT NULL | ✅ | ✅ | 非空约束 |
| DEFAULT | ✅ | ✅ | 默认值 |
| COMMENT | ✅ | ✅ | 属性注释 |
| TTL | ✅ | ✅ | 自动过期 |

### 默认值汇总

| 特性 | 默认值 | 说明 |
|------|--------|------|
| **NULL 约束** | `NULL`（可空） | 不指定时属性默认可空 |
| **DEFAULT 约束** | 无 | 不指定时无默认值，插入 NULL |
| **COMMENT 约束** | 无 | 不指定时无注释 |
| **TTL** | 禁用 | 不指定 `ttl_duration` 时 TTL 禁用 |
| **IF NOT EXISTS** | 无 | 不指定时重复创建会报错 |

### 完整示例

```cypher
-- 创建一个完整的用户标签
CREATE TAG IF NOT EXISTS User(
    user_id: INT NOT NULL COMMENT '用户ID',
    username: STRING NOT NULL COMMENT '用户名',
    email: STRING NOT NULL DEFAULT '' COMMENT '邮箱',
    age: INT NULL DEFAULT 0 COMMENT '年龄',
    status: STRING DEFAULT 'active' COMMENT '状态',
    created_at: TIMESTAMP NOT NULL COMMENT '创建时间',
    updated_at: TIMESTAMP COMMENT '更新时间',
    ttl_duration=31536000,
    ttl_col=created_at
);

-- 创建关注关系边
CREATE EDGE IF NOT EXISTS FOLLOWS(
    follow_id: INT NOT NULL COMMENT '关注ID',
    source_user: INT NOT NULL COMMENT '关注者ID',
    target_user: INT NOT NULL COMMENT '被关注者ID',
    created_at: TIMESTAMP NOT NULL COMMENT '关注时间',
    degree: DOUBLE DEFAULT 1.0 COMMENT '关系程度',
    ttl_duration=0
);

-- 查看创建语句
SHOW CREATE TAG User;
SHOW CREATE EDGE FOLLOWS;
```
