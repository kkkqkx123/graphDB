# GraphDB 其他语句

## 概述

本文档包含不属于DQL、DML、DDL、DCL分类的其他实用语句。

---

## 1. USE - 切换图空间

### 功能
切换到指定的图空间。

### 语法结构
```cypher
USE <space_name>
```

### 示例
```cypher
USE test_space
USE social_network
```

---

## 2. EXPLAIN - 查询计划

### 功能
显示查询的执行计划，用于性能分析和优化。

### 语法结构
```cypher
EXPLAIN <statement>
```

### 关键特性
- 显示查询执行计划
- 显示预计成本
- 显示使用的索引
- 不实际执行查询

### 示例
```cypher
EXPLAIN MATCH (p:Person {name: 'Alice'}) RETURN p
EXPLAIN GO 2 STEPS FROM "101" OVER follow
```

---

## 3. 管道操作符

### 功能
使用管道符 `|` 连接多个语句，将前一个语句的结果传递给后一个语句。

### 语法结构
```cypher
<statement1> | <statement2> | <statement3>
```

### 示例
```cypher
GO FROM "101" OVER follow | YIELD $^.follow._dst AS dst | GO FROM dst OVER like
```

---

## 4. 表达式支持

### 4.1 字面量
- 字符串: `'hello'`, `"world"`
- 整数: `123`, `-456`
- 浮点数: `3.14`, `-0.5`
- 布尔值: `true`, `false`
- NULL: `NULL`

### 4.2 属性访问
```cypher
$^.tag.prop        -- 访问源点属性
$$.tag.prop        -- 访问目标点属性
$-.prop            -- 访问边属性
variable.prop      -- 访问变量属性
```

### 4.3 运算符
| 类型 | 运算符 |
|------|--------|
| 算术 | `+`, `-`, `*`, `/`, `%` |
| 比较 | `=`, `==`, `!=`, `<>`, `<`, `>`, `<=`, `>=` |
| 逻辑 | `AND`, `OR`, `NOT`, `XOR` |
| 字符串 | `+` (连接) |
| 列表 | `IN` |

### 4.4 函数
| 类别 | 函数 |
|------|------|
| 聚合 | `count()`, `sum()`, `avg()`, `max()`, `min()`, `collect()` |
| 字符串 | `concat()`, `substring()`, `lower()`, `upper()`, `trim()` |
| 数学 | `abs()`, `round()`, `floor()`, `ceil()`, `sqrt()`, `pow()` |
| 时间 | `now()`, `timestamp()`, `date()`, `datetime()` |
| 图相关 | `id()`, `tags()`, `type()`, `src()`, `dst()`, `rank()` |

---

## 5. 查询语句完整示例

### 5.1 复杂MATCH查询
```cypher
MATCH (p:Person {name: 'Alice'})-[:FRIEND*1..3]->(f:Person)
WHERE f.age > 25 AND f.city = 'Beijing'
RETURN f.name, f.age, count(*) AS friend_count
ORDER BY friend_count DESC
LIMIT 10
```

### 5.2 多语句管道
```cypher
GO 2 STEPS FROM "player100" OVER follow 
WHERE follow.degree > 0.5 
| YIELD follow._dst AS friend_id, follow.degree AS degree
| GO FROM friend_id OVER serve 
WHERE serve.start_year > 2020
| YIELD $^.serve._dst AS team_id, degree
```

### 5.3 使用UNWIND展开列表
```cypher
UNWIND [1, 2, 3] AS n
RETURN n * 2 AS doubled
```

### 5.4 使用WITH传递中间结果
```cypher
MATCH (p:Person)-[:FRIEND]->(f)
WITH p, count(f) AS friend_count
WHERE friend_count > 5
RETURN p.name, friend_count
ORDER BY friend_count DESC
```
