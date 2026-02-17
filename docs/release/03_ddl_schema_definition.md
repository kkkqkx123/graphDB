# GraphDB 数据定义语言 (DDL)

## 概述

数据定义语言 (DDL) 用于定义和管理图数据库的Schema，包括标签、边类型、索引等的创建、修改和删除。

---

## 1. CREATE TAG - 创建标签

### 功能
定义节点标签及其属性。

### 语法结构
```cypher
CREATE TAG [IF NOT EXISTS] <tag_name> (<prop_name>: <prop_type> [, <prop_name>: <prop_type> ...])
```

### 关键特性
- 支持多种数据类型
- 支持IF NOT EXISTS
- 支持默认值

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

### 示例
```cypher
CREATE TAG IF NOT EXISTS person(name: STRING, age: INT, created_at: TIMESTAMP)
CREATE TAG company(name: STRING, founded: DATE)
```

---

## 2. CREATE EDGE - 创建边类型

### 功能
定义边类型及其属性。

### 语法结构
```cypher
CREATE EDGE [IF NOT EXISTS] <edge_type> (<prop_name>: <prop_type> [, <prop_name>: <prop_type> ...])
```

### 关键特性
- 支持多种数据类型
- 支持IF NOT EXISTS
- 支持默认值

### 示例
```cypher
CREATE EDGE IF NOT EXISTS follow(degree: FLOAT, since: TIMESTAMP)
CREATE EDGE work_at(start_date: DATE, end_date: DATE)
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
