# NebulaGraph 查询语法完整参考

本文档整理了 NebulaGraph 3.8.0 支持的所有查询语法，基于源码分析生成。

## 目录

- [数据定义语言 (DDL)](#数据定义语言-ddl)
- [数据操作语言 (DML)](#数据操作语言-dml)
- [数据查询语言 (DQL)](#数据查询语言-dql)
- [图遍历查询](#图遍历查询)
- [图模式匹配 (MATCH)](#图模式匹配-match)
- [用户权限管理](#用户权限管理)
- [空间管理](#空间管理)
- [索引管理](#索引管理)
- [配置管理](#配置管理)
- [集群管理](#集群管理)
- [会话和查询管理](#会话和查询管理)
- [其他命令](#其他命令)

---

## 数据定义语言 (DDL)

### 创建 Tag

```sql
CREATE TAG [IF NOT EXISTS] <tag_name> (
  <prop_name> <data_type> [NULL | NOT NULL] [DEFAULT <default_value>] [COMMENT '<comment>'] [, ...]
) [TTL_DURATION = <ttl_duration>] [TTL_COL = <prop_name>] [COMMENT '<comment>']
```

**示例：**
```sql
CREATE TAG IF NOT EXISTS person (
  name string NOT NULL,
  age int DEFAULT 18,
  email string
) TTL_DURATION = 86400 TTL_COL = created_time COMMENT '用户信息'
```

### 修改 Tag

```sql
ALTER TAG <tag_name>
  | ADD (<prop_name> <data_type> [NULL | NOT NULL] [DEFAULT <default_value>] [COMMENT '<comment>'] [, ...])
  | CHANGE (<prop_name> <data_type> [NULL | NOT NULL] [DEFAULT <default_value>] [COMMENT '<comment>'] [, ...])
  | DROP (<prop_name> [, ...])
  [TTL_DURATION = <ttl_duration>] [TTL_COL = <prop_name>] [COMMENT '<comment>']
```

### 描述 Tag

```sql
DESCRIBE TAG <tag_name>
DESC TAG <tag_name>
```

### 删除 Tag

```sql
DROP TAG [IF EXISTS] <tag_name>
```

### 创建 Edge

```sql
CREATE EDGE [IF NOT EXISTS] <edge_name> (
  <prop_name> <data_type> [NULL | NOT NULL] [DEFAULT <default_value>] [COMMENT '<comment>'] [, ...]
) [TTL_DURATION = <ttl_duration>] [TTL_COL = <prop_name>] [COMMENT '<comment>']
```

**示例：**
```sql
CREATE EDGE IF NOT EXISTS follow (
  degree int DEFAULT 0,
  created_time timestamp
) TTL_DURATION = 86400 TTL_COL = created_time
```

### 修改 Edge

```sql
ALTER EDGE <edge_name>
  | ADD (<prop_name> <data_type> [NULL | NOT NULL] [DEFAULT <default_value>] [COMMENT '<comment>'] [, ...])
  | CHANGE (<prop_name> <data_type> [NULL | NOT NULL] [DEFAULT <default_value>] [COMMENT '<comment>'] [, ...])
  | DROP (<prop_name> [, ...])
  [TTL_DURATION = <ttl_duration>] [TTL_COL = <prop_name>] [COMMENT '<comment>']
```

### 描述 Edge

```sql
DESCRIBE EDGE <edge_name>
DESC EDGE <edge_name>
```

### 删除 Edge

```sql
DROP EDGE [IF EXISTS] <edge_name>
```

---

## 数据操作语言 (DML)

### 插入顶点

```sql
INSERT VERTEX [IF NOT EXISTS] <tag_name> (<prop_name> [, ...]) [IGNORE_EXISTED_INDEX]
{ VALUES | VALUE } <vid>: (<prop_value> [, ...]) [, <vid>: (<prop_value> [, ...]) ...]
```

**示例：**
```sql
INSERT VERTEX IF NOT EXISTS person(name, age) VALUES
  "100": ("Tom", 20),
  "101": ("Jerry", 22)
```

### 插入边

```sql
INSERT EDGE [IF NOT EXISTS] <edge_name> ([<prop_name> [, ...]]) [IGNORE_EXISTED_INDEX]
{ VALUES | VALUE }
<src_vid> -> <dst_vid>[@<rank>]: (<prop_value> [, ...])
[, <src_vid> -> <dst_vid>[@<rank>]: (<prop_value> [, ...]) ...]
```

**示例：**
```sql
INSERT EDGE IF NOT EXISTS follow(degree) VALUES
  "100" -> "101": (1),
  "100" -> "102"@2: (2)
```

### 更新顶点

```sql
UPDATE VERTEX <vid>
  [SET <update_item> [, ...]]
  [WHEN <condition>]
  [YIELD <return_item> [, ...]]
```

**update_item:** `<prop_name> = <expression>`

**示例：**
```sql
UPDATE VERTEX "100" SET person.age = person.age + 1 WHEN person.name == "Tom" YIELD person.name, person.age
```

### 更新边

```sql
UPDATE EDGE <src_vid> -> <dst_vid>[@<rank>] OF <edge_name>
  [SET <update_item> [, ...]]
  [WHEN <condition>]
  [YIELD <return_item> [, ...]]
```

**示例：**
```sql
UPDATE EDGE "100" -> "101" OF follow SET degree = degree + 1 YIELD degree
```

### 删除顶点

```sql
DELETE VERTEX <vid> [, <vid> ...] [WITH EDGE]
```

**示例：**
```sql
DELETE VERTEX "100", "101" WITH EDGE
```

### 删除 Tag

```sql
DELETE TAG <tag_name> [, <tag_name> ...] FROM <vid> [, <vid> ...]
```

**示例：**
```sql
DELETE TAG person FROM "100", "101"
```

### 删除边

```sql
DELETE EDGE <edge_name> <src_vid> -> <dst_vid>[@<rank>] [, <src_vid> -> <dst_vid>[@<rank>] ...]
```

**示例：**
```sql
DELETE EDGE follow "100" -> "101", "100" -> "102"@2
```

---

## 数据查询语言 (DQL)

### 查询顶点属性

```sql
FETCH PROP ON <tag_name> <vid> [, <vid> ...] [YIELD <return_item> [, ...]]
```

**示例：**
```sql
FETCH PROP ON person "100", "101" YIELD person.name, person.age
```

### 查询边属性

```sql
FETCH PROP ON <edge_name> <src_vid> -> <dst_vid>[@<rank>] [, <src_vid> -> <dst_vid>[@<rank>] ...]
  [YIELD <return_item> [, ...]]
```

**示例：**
```sql
FETCH PROP ON follow "100" -> "101", "100" -> "102"@2 YIELD follow.degree
```

### LOOKUP 查询

```sql
LOOKUP ON <tag_name|edge_name>
  [WHERE <condition>]
  [YIELD <return_item> [, ...]]
```

**示例：**
```sql
LOOKUP ON person WHERE person.age > 18 YIELD person.name, person.age
```

---

## 图遍历查询

### GO 语句

```sql
GO [<step_count> TO <step_count> STEPS] FROM <vid> [, <vid> ...]
  OVER <edge_name> [, <edge_name> ...] [REVERSELY | BIDIRECT]
  [WHERE <condition>]
  [YIELD [DISTINCT] <return_item> [, ...]]
  [REACH (<return_item> [, ...])]
```

**示例：**
```sql
GO 1 TO 3 STEPS FROM "100" OVER follow YIELD $$.person.name
GO 2 STEPS FROM "100" OVER follow REVERSELY WHERE $$.person.age > 20 YIELD $$.person.name
```

### FIND PATH

```sql
FIND [ALL | SHORTEST] PATH [WITH PROP] [NOLOOP] FROM <src_vid> [, <src_vid> ...]
  TO <dst_vid> [, <dst_vid> ...]
  OVER <edge_name> [, <edge_name> ...]
  [UPTO <step_count> STEPS]
  [WHERE <condition>]
  [YIELD <return_item> [, ...]]
```

**示例：**
```sql
FIND SHORTEST PATH FROM "100" TO "200" OVER follow YIELD path
FIND ALL PATH WITH PROP NOLOOP FROM "100" TO "200" OVER follow UPTO 5 STEPS
```

### GET SUBGRAPH

```sql
GET SUBGRAPH [WITH PROP] [<step_count> STEPS] FROM <vid> [, <vid> ...]
  [IN <edge_name> [, <edge_name> ...]]
  [OUT <edge_name> [, <edge_name> ...]]
  [BOTH <edge_name> [, <edge_name> ...]]
  [WHERE <condition>]
  [YIELD <return_item> [, ...]]
```

**示例：**
```sql
GET SUBGRAPH WITH PROP 2 STEPS FROM "100" OUT follow YIELD vertices, edges
```

---

## 图模式匹配 (MATCH)

### MATCH 语句

```sql
[MATCH] <match_pattern> [OPTIONAL]
  [WHERE <condition>]
  [RETURN [DISTINCT] <return_item> [, ...]]
  [ORDER BY <order_item> [ASC | DESC] [, ...]]
  [SKIP <skip_count>]
  [LIMIT <limit_count>]
```

**match_pattern:**
```
(<node_alias>[:<tag_name> {<prop_name>: <value>}]) [<edge_pattern>] (<node_alias>[:<tag_name> {<prop_name>: <value>}])
```

**edge_pattern:**
```
-[:<edge_name> {<prop_name>: <value>}]->
<-[:<edge_name> {<prop_name>: <value>}]-
-[:<edge_name> {<prop_name>: <value>}]-
<-[:<edge_name> {<prop_name>: <value>}]-
```

**示例：**
```sql
MATCH (p:person {name: "Tom"})-[:follow]->(f:person)
RETURN p.name, f.name
ORDER BY f.name DESC
LIMIT 10

MATCH (p:person)-[f:follow*1..3]->(friend)
WHERE p.name == "Tom"
RETURN p.name, friend.name, f.degree
```

### MATCH SET 操作

```sql
MATCH <pattern> RETURN <items>
  [UNION [ALL | DISTINCT] MATCH <pattern> RETURN <items>]
  [INTERSECT MATCH <pattern> RETURN <items>]
  [MINUS MATCH <pattern> RETURN <items>]
```

### WITH 子句

```sql
WITH <return_item> [, ...]
  [WHERE <condition>]
  [ORDER BY <order_item> [ASC | DESC] [, ...]]
  [SKIP <skip_count>]
  [LIMIT <limit_count>]
```

### UNWIND 子句

```sql
UNWIND <list_expression> AS <alias>
```

**示例：**
```sql
UNWIND [1, 2, 3] AS num RETURN num * 2
```

---

## 用户权限管理

### 创建用户

```sql
CREATE USER [IF NOT EXISTS] <user_name> [WITH PASSWORD '<password>']
```

### 修改用户

```sql
ALTER USER <user_name> WITH PASSWORD '<password>'
```

### 删除用户

```sql
DROP USER [IF EXISTS] <user_name>
```

### 修改密码

```sql
CHANGE PASSWORD <user_name> FROM '<old_password>' TO '<new_password>'
```

### 授予权限

```sql
GRANT ROLE [GOD | ADMIN | DBA | USER | GUEST] ON <space_name> TO <user_name>
```

### 撤销权限

```sql
REVOKE ROLE [GOD | ADMIN | DBA | USER | GUEST] ON <space_name> FROM <user_name>
```

---

## 空间管理

### 创建空间

```sql
CREATE SPACE [IF NOT EXISTS] <space_name>
  [PARTITION_NUM = <partition_num>]
  [REPLICA_FACTOR = <replica_factor>]
  [VID_TYPE = <vid_type>]
  [CHARSET = <charset>]
  [COLLATE = <collate>]
  [ON <zone_name> [, <zone_name> ...]]
  [COMMENT '<comment>']
```

**vid_type:** `FIXED_STRING(<length>)` | `INT64`

**示例：**
```sql
CREATE SPACE IF NOT EXISTS my_graph
  PARTITION_NUM = 10
  REPLICA_FACTOR = 1
  VID_TYPE = FIXED_STRING(32)
  COMMENT '我的图数据库'
```

### 创建空间（基于现有空间）

```sql
CREATE SPACE [IF NOT EXISTS] <new_space_name> AS <old_space_name>
```

### 描述空间

```sql
DESCRIBE SPACE <space_name>
DESC SPACE <space_name>
```

### 修改空间

```sql
ALTER SPACE <space_name> {ADD | DROP | RENAME} <zone_name> [, ...]
```

### 删除空间

```sql
DROP SPACE [IF EXISTS] <space_name>
```

### 清空空间

```sql
CLEAR SPACE [IF EXISTS] <space_name>
```

### 使用空间

```sql
USE <space_name>
```

---

## 索引管理

### 创建 Tag 索引

```sql
CREATE TAG INDEX [IF NOT EXISTS] <index_name>
  ON <tag_name> (<prop_name> [, <prop_name> ...])
  [WITH (S2_MAX_LEVEL = <level>, S2_MAX_CELLS = <cells>)]
  [COMMENT '<comment>']
```

### 创建 Edge 索引

```sql
CREATE EDGE INDEX [IF NOT EXISTS] <index_name>
  ON <edge_name> (<prop_name> [, <prop_name> ...])
  [WITH (S2_MAX_LEVEL = <level>, S2_MAX_CELLS = <cells>)]
  [COMMENT '<comment>']
```

### 创建全文索引

```sql
CREATE FULLTEXT INDEX [IF NOT EXISTS] <index_name>
  ON {TAG <tag_name> | EDGE <edge_name>} (<prop_name> [, <prop_name> ...])
```

### 描述 Tag 索引

```sql
DESCRIBE TAG INDEX <index_name>
```

### 描述 Edge 索引

```sql
DESCRIBE EDGE INDEX <index_name>
```

### 删除 Tag 索引

```sql
DROP TAG INDEX [IF EXISTS] <index_name>
```

### 删除 Edge 索引

```sql
DROP EDGE INDEX [IF EXISTS] <index_name>
```

### 删除全文索引

```sql
DROP FULLTEXT INDEX <index_name>
```

### 重建索引

```sql
REBUILD {TAG | EDGE | FULLTEXT} INDEX <index_name> [, <index_name> ...]
```

---

## 配置管理

### 查看配置

```sql
SHOW CONFIGS [GRAPH | META | STORAGE] [<config_name>]
```

### 获取配置

```sql
GET CONFIGS [GRAPH | META | STORAGE] <config_name>
```

### 设置配置

```sql
UPDATE CONFIGS [GRAPH | META | STORAGE] <config_name> = <value>
```

---

## 集群管理

### 添加主机

```sql
ADD HOSTS <host>:<port> [, <host>:<port> ...]
```

### 删除主机

```sql
DROP HOSTS <host>:<port> [, <host>:<port> ...]
```

### 查看主机

```sql
SHOW HOSTS [GRAPH | META | STORAGE]
```

### 添加监听器

```sql
ADD LISTENER [ELASTICSEARCH | <listener_type>] <host>:<port> [, <host>:<port> ...]
```

### 删除监听器

```sql
REMOVE LISTENER [ELASTICSEARCH | <listener_type>]
```

### 查看监听器

```sql
SHOW LISTENER
```

### 创建 Zone

```sql
CREATE ZONE <zone_name> (<host>:<port> [, <host>:<port> ...])
```

### 描述 Zone

```sql
DESCRIBE ZONE <zone_name>
```

### 删除 Zone

```sql
DROP ZONE <zone_name>
```

### 合并 Zone

```sql
MERGE ZONE <zone_name> INTO <zone_name>
```

### 分割 Zone

```sql
DIVIDE ZONE <zone_name> INTO (<zone_name> (<host>:<port> [, ...]) [, ...])
```

### 重命名 Zone

```sql
RENAME ZONE <old_zone_name> TO <new_zone_name>
```

### 查看 Zone

```sql
SHOW ZONES
```

### 查看 Group

```sql
SHOW GROUPS
```

---

## 会话和查询管理

### 查看会话

```sql
SHOW SESSIONS [<session_id>] [LOCAL]
```

### 终止会话

```sql
KILL SESSION <session_id>
```

### 查看查询

```sql
SHOW QUERIES [ALL]
```

### 终止查询

```sql
KILL QUERY <session_id>, <ep_id>
```

---

## 其他命令

### SHOW 命令

```sql
SHOW SPACES
SHOW TAGS
SHOW EDGES
SHOW TAG INDEXES
SHOW EDGE INDEXES
SHOW TAG INDEX STATUS
SHOW EDGE INDEX STATUS
SHOW USERS
SHOW ROLES IN <space_name>
SHOW CREATE SPACE <space_name>
SHOW CREATE TAG <tag_name>
SHOW CREATE EDGE <edge_name>
SHOW CREATE TAG INDEX <index_name>
SHOW CREATE EDGE INDEX <index_name>
SHOW PARTS [<part_id> [, <part_id> ...]]
SHOW SNAPSHOTS
SHOW CHARSET
SHOW COLLATION
SHOW STATS
SHOW SERVICE CLIENTS [ELASTICSEARCH | <service_type>]
SHOW META LEADER
```

### EXPLAIN 命令

```sql
EXPLAIN [FORMAT <format>] <sentence>
EXPLAIN [FORMAT <format>] { <sentence>; <sentence>; ... }
```

**format:** `dot` | `row`

### PROFILE 命令

```sql
PROFILE [FORMAT <format>] <sentence>
PROFILE [FORMAT <format>] { <sentence>; <sentence>; ... }
```

### 管理任务

```sql
SUBMIT JOB [COMPACT | FLUSH | STATS | REBUILD {TAG | EDGE | FULLTEXT} INDEX] <params>
SHOW JOBS [<job_id>]
RECOVER JOB <job_id>
```

### 快照管理

```sql
CREATE SNAPSHOT
DROP SNAPSHOT <snapshot_name>
```

### 服务注册

```sql
SIGN IN SERVICE [ELASTICSEARCH | <service_type>] (<host>:<port> [, <host>:<port> ...])
SIGN OUT SERVICE [ELASTICSEARCH | <service_type>]
```

### 数据导入

```sql
INGEST <hdfs_path>
```

### 下载

```sql
DOWNLOAD <hdfs_path>
```

---

## 表达式和函数

### 数据类型

- `BOOL`: 布尔值
- `INT8`, `INT16`, `INT32`, `INT64`: 整数
- `FLOAT`, `DOUBLE`: 浮点数
- `STRING`: 字符串
- `FIXED_STRING(<length>)`: 定长字符串
- `TIMESTAMP`: 时间戳
- `DATE`: 日期
- `TIME`: 时间
- `DATETIME`: 日期时间
- `DURATION`: 时间段
- `GEOGRAPHY`: 地理位置数据
- `POINT`, `LINESTRING`, `POLYGON`: 地理图形

### 运算符

**算术运算符:** `+`, `-`, `*`, `/`, `%`

**比较运算符:** `==`, `!=`, `<`, `>`, `<=`, `>=`

**逻辑运算符:** `AND`, `OR`, `XOR`, `NOT`

**字符串运算符:** `CONTAINS`, `STARTS WITH`, `ENDS WITH`, `REG`

**集合运算符:** `IN`, `NOT IN`

**空值判断:** `IS NULL`, `IS NOT NULL`, `IS EMPTY`, `IS NOT EMPTY`

### 聚合函数

- `COUNT()`
- `SUM()`
- `AVG()`
- `MAX()`
- `MIN()`
- `STD()`
- `BIT_AND()`
- `BIT_OR()`
- `BIT_XOR()`

### 数学函数

- `ABS()`, `CEIL()`, `FLOOR()`, `ROUND()`
- `EXP()`, `LOG()`, `LOG10()`, `POW()`, `SQRT()`
- `SIN()`, `COS()`, `TAN()`, `ASIN()`, `ACOS()`, `ATAN()`
- `RAND()`, `RAND32()`, `RAND64()`

### 字符串函数

- `LOWER()`, `UPPER()`, `TRIM()`, `LTRIM()`, `RTRIM()`
- `SUBSTR()`, `LENGTH()`, `REVERSE()`
- `REPLACE()`, `CONCAT()`, `SPLIT()`
- `HASH()`, `MD5()`, `SHA1()`, `SHA256()`, `SHA512()`

### 时间函数

- `NOW()`, `DATE()`, `TIME()`, `DATETIME()`
- `TIMESTAMP()`, `YEAR()`, `MONTH()`, `DAY()`
- `HOUR()`, `MINUTE()`, `SECOND()`

### 地理函数

- `ST_Distance()`, `ST_Distance_Sphere()`
- `ST_GeogFromText()`, `ST_GeomFromText()`
- `ST_Point()`, `ST_LineString()`, `ST_Polygon()`
- `ST_Contains()`, `ST_Intersects()`

### 列表函数

- `size()`, `head()`, `tail()`
- `range()`, `reverse()`, `sort()`

### Map 函数

- `keys()`, `values()`, `map()`

### 路径函数

- `length()`, `nodes()`, `relationships()`

---

## 管道操作

### 管道 (|)

```sql
<sentence> | <sentence>
```

**示例：**
```sql
GO FROM "100" OVER follow YIELD follow._dst AS id
| GO FROM $-.id OVER follow YIELD $$.person.name
```

### 赋值 (=)

```sql
<variable> = <sentence>
```

**示例：**
```sql
$var = GO FROM "100" OVER follow YIELD follow._dst
GO FROM $var.follow._dst OVER follow YIELD $$.person.name
```

### SET 操作

```sql
<sentence> UNION [ALL | DISTINCT] <sentence>
<sentence> INTERSECT <sentence>
<sentence> MINUS <sentence>
```

---

## 子句

### WHERE 子句

```sql
WHERE <condition>
```

### YIELD 子句

```sql
YIELD [DISTINCT] <return_item> [, ...]
```

### ORDER BY 子句

```sql
ORDER BY <order_item> [ASC | DESC] [, ...]
```

### LIMIT 子句

```sql
LIMIT [<offset>,] <count>
```

### GROUP BY 子句

```sql
GROUP BY <group_item> [, ...] [YIELD <return_item> [, ...]] [HAVING <condition>]
```

---

## 特殊表达式

### CASE 表达式

```sql
CASE
  WHEN <condition> THEN <expression>
  [WHEN <condition> THEN <expression> ...]
  [ELSE <expression>]
END

<condition> ? <expression> : <expression>
```

### PREDICATE 表达式

```sql
ALL (<variable> IN <list> WHERE <condition>)
ANY (<variable> IN <list> WHERE <condition>)
SINGLE (<variable> IN <list> WHERE <condition>)
NONE (<variable> IN <list> WHERE <condition>)
EXISTS (<expression>)
```

### REDUCE 表达式

```sql
REDUCE(<accumulator> = <initial>, <variable> IN <list> | <expression>)
```

### LIST COMPREHENSION 表达式

```sql
[<variable> IN <list> WHERE <condition> | <expression>]
[<variable> IN <list> | <expression>]
```

### UUID 表达式

```sql
UUID()
```

---

## 注释

```sql
-- 单行注释
```

---

## 参考文档

- 源码位置: `nebula-3.8.0/src/parser/`
- 语法文件: `parser.yy`
- 语句定义: `Sentence.h`, `TraverseSentences.h`, `MutateSentences.h`, `MaintainSentences.h`, `AdminSentences.h`, `UserSentences.h`, `MatchSentence.h`
