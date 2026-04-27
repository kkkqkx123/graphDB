# GraphDB 优化器验证 E2E 测试设计方案

## 概述

本文档针对 GraphDB 查询优化器的各项策略，设计通过 EXPLAIN 命令验证优化器行为的 E2E 测试方案，确保优化器实际执行符合预期设计。

## 一、优化器策略验证矩阵

| 优化策略 | 验证方式 | 预期计划特征 | 优先级 |
|---------|---------|-------------|--------|
| 索引选择 | EXPLAIN + 实际执行 | IndexScan vs SeqScan | P0 |
| 连接顺序 | EXPLAIN 计划结构 | Join 节点顺序 | P0 |
| 连接算法 | EXPLAIN 节点类型 | HashJoin/IndexJoin/NestedLoop | P0 |
| 聚合策略 | EXPLAIN + PROFILE | Hash/Sort/Streaming Aggregate | P1 |
| 双向遍历 | EXPLAIN 遍历节点 | BidirectionalTraverse | P1 |
| 遍历方向 | EXPLAIN 方向标记 | Traverse 方向参数 | P1 |
| TopN 优化 | EXPLAIN 节点类型 | TopN vs Sort+Limit | P0 |
| 子查询解关联 | EXPLAIN 计划结构 | HashJoin vs Apply | P1 |
| CTE 物化 | EXPLAIN 物化标记 | Materialize 节点 | P1 |
| 表达式预计算 | EXPLAIN 表达式简化 | 简化后的表达式 | P2 |
| 范围查询优化 | EXPLAIN 索引使用 | 范围扫描 vs 全表扫描 | P0 |
| 多条件优化 | EXPLAIN 谓词下推 | Filter 节点位置 | P0 |

## 二、索引选择优化验证

### 2.1 单字段索引选择

```cypher
-- 准备数据
CREATE SPACE e2e_optimizer (vid_type=STRING)
USE e2e_optimizer

CREATE TAG person(name: STRING, age: INT, city: STRING)
CREATE TAG INDEX idx_person_name ON person(name)
CREATE TAG INDEX idx_person_age ON person(age)

-- 插入测试数据
INSERT VERTEX person(name, age, city) VALUES 
    "p1": ("Alice", 30, "Beijing"),
    "p2": ("Bob", 25, "Shanghai"),
    ... -- 1000条数据
```

#### TC-IDX-001: 等值查询使用索引
```cypher
-- 测试: 等值条件应使用 IndexScan
\explain MATCH (p:person {name: "Alice"}) RETURN p.age

-- 预期计划:
-- IndexScan (idx_person_name)
--   - 条件: name == "Alice"
--   - 成本: 较低
-- Project
--   - 输出: age
```

#### TC-IDX-002: 范围查询使用索引
```cypher
-- 测试: 范围条件应使用 IndexScan
\explain MATCH (p:person) WHERE p.age > 25 AND p.age < 35 RETURN p.name

-- 预期计划:
-- IndexScan (idx_person_age)
--   - 条件: age > 25 AND age < 35
--   - 扫描范围: (25, 35)
-- Project
```

#### TC-IDX-003: 无索引时全表扫描
```cypher
-- 测试: 无索引字段应使用 SeqScan
\explain MATCH (p:person) WHERE p.city == "Beijing" RETURN p.name

-- 预期计划:
-- SeqScan (person)
--   - 条件: city == "Beijing"
--   - 成本: 较高
-- Filter
-- Project
```

#### TC-IDX-004: 复合条件索引选择
```cypher
-- 测试: 选择最优索引
\explain MATCH (p:person) 
WHERE p.name == "Alice" AND p.age > 25 
RETURN p.city

-- 预期计划:
-- IndexScan (idx_person_name) -- 选择选择性更高的索引
--   - 条件: name == "Alice"
-- Filter
--   - 条件: age > 25
-- Project
```

### 2.2 复合索引验证

```cypher
-- 创建复合索引
CREATE TAG INDEX idx_person_name_age ON person(name, age)

-- 测试前缀匹配使用复合索引
\explain MATCH (p:person) 
WHERE p.name == "Alice" AND p.age == 30 
RETURN p.city

-- 预期计划:
-- IndexScan (idx_person_name_age)
--   - 条件: name == "Alice", age == 30
--   - 使用复合索引的两列
```

### 2.3 索引覆盖扫描

```cypher
-- 测试: 查询列都在索引中时使用覆盖扫描
\explain MATCH (p:person) WHERE p.name == "Alice" RETURN p.name

-- 预期计划:
-- IndexOnlyScan (idx_person_name) -- 无需回表
--   - 输出: name
```

## 三、连接优化验证

### 3.1 连接顺序优化

```cypher
-- 准备数据
CREATE TAG company(name: STRING, industry: STRING)
CREATE TAG employee(name: STRING, salary: INT)
CREATE EDGE works_at(position: STRING)

-- 插入数据: company(100), employee(10000)
```

#### TC-JOIN-001: 小表驱动大表
```cypher
-- 测试: 小表应作为驱动表
\explain 
MATCH (c:company)-[:works_at]->(e:employee)
WHERE c.industry == "Tech"
RETURN c.name, e.name

-- 预期计划:
-- IndexScan (company) -- 小表先扫描
--   - 条件: industry == "Tech"
-- HashJoin
--   - 左: company
--   - 右: IndexScan (works_at) + GetVertices (employee)
```

#### TC-JOIN-002: 多表连接顺序
```cypher
-- 测试: 三表连接的最优顺序
\explain
MATCH (c:company)-[:works_at]->(e:employee)-[:manages]->(d:department)
WHERE c.name == "TechCorp"
RETURN e.name, d.name

-- 预期计划:
-- 从选择性最高的表开始(company)
-- 逐步连接其他表
```

### 3.2 连接算法选择

#### TC-JOIN-003: HashJoin 选择
```cypher
-- 测试: 大表连接使用 HashJoin
\explain
MATCH (e:employee)-[:works_at]->(c:company)
WHERE e.salary > 50000
RETURN e.name, c.name

-- 预期计划:
-- SeqScan (employee) -- 过滤后仍较大
--   - 条件: salary > 50000
-- HashJoin
--   - Build: company (小表)
--   - Probe: filtered employees
```

#### TC-JOIN-004: IndexJoin 选择
```cypher
-- 测试: 有索引时使用 IndexJoin
\explain
MATCH (e:employee)-[:works_at]->(c:company {name: "TechCorp"})
RETURN e.name

-- 预期计划:
-- IndexScan (company)
--   - 条件: name == "TechCorp"
-- IndexJoin
--   - 使用 works_at 边索引
```

#### TC-JOIN-005: NestedLoop 选择
```cypher
-- 测试: 小结果集使用 NestedLoop
\explain
MATCH (e:employee)-[:works_at]->(c:company)
WHERE e.name == "Alice"
RETURN c.name

-- 预期计划:
-- IndexScan (employee)
--   - 条件: name == "Alice"
--   - 结果: 1行
-- NestedLoopJoin
--   - 外层: employee
--   - 内层: GetEdge + GetVertex (company)
```

## 四、聚合策略验证

### 4.1 聚合策略选择

```cypher
-- 准备数据
CREATE TAG sales(product: STRING, amount: INT, date: DATE)
```

#### TC-AGG-001: HashAggregate 选择
```cypher
-- 测试: 无排序输入使用 HashAggregate
\explain
MATCH (s:sales)
RETURN s.product, sum(s.amount) AS total
GROUP BY s.product

-- 预期计划:
-- SeqScan (sales)
-- HashAggregate
--   - 分组键: product
--   - 聚合: sum(amount)
```

#### TC-AGG-002: SortAggregate 选择
```cypher
-- 测试: 有排序输入使用 SortAggregate
\explain
MATCH (s:sales)
RETURN s.product, sum(s.amount) AS total
GROUP BY s.product
ORDER BY s.product

-- 预期计划:
-- SeqScan (sales)
-- Sort
--   - 键: product
-- SortAggregate
--   - 利用已排序数据
```

#### TC-AGG-003: StreamingAggregate 选择
```cypher
-- 测试: 索引有序输入使用 StreamingAggregate
CREATE TAG INDEX idx_sales_product ON sales(product)

\explain
MATCH (s:sales)
RETURN s.product, sum(s.amount) AS total
GROUP BY s.product

-- 预期计划:
-- IndexScan (sales) -- 按 product 有序
--   - 顺序: product ASC
-- StreamingAggregate
--   - 无需额外排序
```

## 五、图遍历优化验证

### 5.1 遍历方向优化

```cypher
-- 准备数据: 社交网络
CREATE TAG user(name: STRING, follower_count: INT, following_count: INT)
CREATE EDGE follows(created_at: TIMESTAMP)

-- 数据分布: 少数大V(100万粉丝), 多数普通用户(<100粉丝)
```

#### TC-TRAV-001: 出度优先遍历
```cypher
-- 测试: 从出度小的节点开始遍历
\explain
GO 1 STEP FROM "normal_user" OVER follows
YIELD follows.created_at

-- 预期计划:
-- Traverse
--   - 方向: OUT (出度小)
--   - 起点: normal_user (出度 < 100)
--   - 策略: 正向遍历
```

#### TC-TRAV-002: 入度优先遍历
```cypher
-- 测试: 反向遍历大V的粉丝
\explain
GO 1 STEP FROM "big_v" OVER follows REVERSELY
YIELD follows.created_at

-- 预期计划:
-- Traverse
--   - 方向: IN (入度大, 100万)
--   - 起点: big_v
--   - 策略: 反向遍历(从粉丝指向big_v的边)
```

### 5.2 双向遍历优化

#### TC-BI-001: 双向BFS选择
```cypher
-- 测试: 长路径使用双向BFS
\explain
FIND SHORTEST PATH FROM "user_a" TO "user_z" OVER follows

-- 预期计划:
-- ShortestPath
--   - 算法: BidirectionalBFS
--   - 前向深度: 3
--   - 后向深度: 3
--   - 原因: 深度较大, 双向搜索更优
```

#### TC-BI-002: 单向遍历选择
```cypher
-- 测试: 短路径使用单向BFS
\explain
FIND SHORTEST PATH FROM "user_a" TO "user_b" OVER follows UPTO 2 STEPS

-- 预期计划:
-- ShortestPath
--   - 算法: BFS
--   - 深度: 2
--   - 原因: 深度小, 单向搜索足够
```

### 5.3 遍历起点选择

#### TC-START-001: 选择性高的起点
```cypher
-- 测试: 选择数据量小的标签作为起点
\explain
MATCH (u:user)-[:follows]->(v:vip)
WHERE u.city == "Beijing" AND v.level == "gold"
RETURN u.name, v.name

-- 预期计划:
-- 从 vip (数量少) 开始遍历
-- 而不是从 user (数量多) 开始
```

## 六、TopN 优化验证

### 6.1 Sort + Limit 转 TopN

```cypher
-- 准备数据
CREATE TAG product(name: STRING, price: INT, sales: INT)
```

#### TC-TOPN-001: 简单 TopN 优化
```cypher
-- 测试: Sort + Limit 应转为 TopN
\explain
MATCH (p:product)
RETURN p.name, p.price
ORDER BY p.price DESC
LIMIT 10

-- 预期计划:
-- SeqScan (product)
-- TopN
--   - 排序键: price DESC
--   - 限制: 10
--   - 内存: O(N) 其中 N=10
```

#### TC-TOPN-002: 带过滤的 TopN
```cypher
-- 测试: 过滤后 TopN
\explain
MATCH (p:product)
WHERE p.category == "electronics"
RETURN p.name, p.sales
ORDER BY p.sales DESC
LIMIT 5

-- 预期计划:
-- IndexScan (product) -- 如果有 category 索引
-- TopN
--   - 排序键: sales DESC
--   - 限制: 5
```

#### TC-TOPN-003: 多键排序 TopN
```cypher
-- 测试: 多键排序的 TopN
\explain
MATCH (p:product)
RETURN p.name
ORDER BY p.category ASC, p.price DESC
LIMIT 20

-- 预期计划:
-- SeqScan (product)
-- TopN
--   - 排序键: category ASC, price DESC
--   - 限制: 20
```

## 七、子查询优化验证

### 7.1 子查询解关联

```cypher
-- 准备数据
CREATE TAG department(name: STRING, budget: INT)
CREATE TAG employee(name: STRING, salary: INT, dept: STRING)
```

#### TC-SUBQ-001: 相关子查询解关联
```cypher
-- 测试: 相关子查询应转为 HashJoin
\explain
MATCH (d:department)
WHERE d.budget > (
    SELECT sum(e.salary) 
    FROM employee e 
    WHERE e.dept == d.name
)
RETURN d.name

-- 预期计划:
-- SeqScan (department)
-- HashJoin (解关联后的连接)
--   - 左: department
--   - 右: Aggregate (employee)
-- Filter
--   - 条件: budget > sum_salary
```

#### TC-SUBQ-002: EXISTS 子查询优化
```cypher
-- 测试: EXISTS 应转为 SemiJoin
\explain
MATCH (d:department)
WHERE EXISTS (
    MATCH (e:employee)
    WHERE e.dept == d.name AND e.salary > 100000
)
RETURN d.name

-- 预期计划:
-- SeqScan (department)
-- SemiJoin
--   - 左: department
--   - 右: IndexScan (employee) + Filter
```

## 八、CTE 物化验证

### 8.1 CTE 物化决策

```cypher
-- 准备数据
CREATE TAG orders(order_id: STRING, customer: STRING, amount: INT)
CREATE TAG items(item_id: STRING, order_id: STRING, price: INT)
```

#### TC-CTE-001: 多次引用物化
```cypher
-- 测试: 多次引用的 CTE 应物化
\explain
WITH high_value_orders AS (
    SELECT * FROM orders WHERE amount > 10000
)
SELECT * FROM high_value_orders h1
JOIN high_value_orders h2 ON h1.customer == h2.customer
WHERE h1.order_id != h2.order_id

-- 预期计划:
-- CTE Scan (high_value_orders)
--   - 物化: true
--   - 原因: 被引用2次
-- HashJoin
--   - 左: high_value_orders (物化结果)
--   - 右: high_value_orders (物化结果)
```

#### TC-CTE-002: 单次引用不物化
```cypher
-- 测试: 单次引用的 CTE 不物化
\explain
WITH recent_orders AS (
    SELECT * FROM orders WHERE date > today() - 7
)
SELECT * FROM recent_orders

-- 预期计划:
-- SeqScan (orders)
--   - 条件: date > today() - 7
-- 无物化节点(内联展开)
```

## 九、范围查询优化验证

### 9.1 范围查询索引使用

```cypher
-- 准备数据
CREATE TAG event(name: STRING, start_time: TIMESTAMP, end_time: TIMESTAMP)
CREATE TAG INDEX idx_event_start ON event(start_time)
CREATE TAG INDEX idx_event_end ON event(end_time)
```

#### TC-RANGE-001: 时间范围查询
```cypher
-- 测试: 时间范围使用索引
\explain
MATCH (e:event)
WHERE e.start_time >= datetime("2024-01-01")
  AND e.start_time < datetime("2024-02-01")
RETURN e.name

-- 预期计划:
-- IndexRangeScan (idx_event_start)
--   - 范围: [2024-01-01, 2024-02-01)
-- Project
```

#### TC-RANGE-002: 多字段范围
```cypher
-- 测试: 多字段范围查询
\explain
MATCH (e:event)
WHERE e.start_time >= datetime("2024-01-01")
  AND e.end_time < datetime("2024-06-01")
RETURN e.name

-- 预期计划:
-- IndexRangeScan (idx_event_start) -- 选择选择性更高的索引
-- Filter
--   - 条件: end_time < 2024-06-01
```

### 9.2 范围合并优化

#### TC-RANGE-003: 范围合并
```cypher
-- 测试: 重叠范围合并
\explain
MATCH (e:event)
WHERE (e.start_time >= datetime("2024-01-01") 
       AND e.start_time < datetime("2024-03-01"))
   OR (e.start_time >= datetime("2024-02-01") 
       AND e.start_time < datetime("2024-04-01"))
RETURN e.name

-- 预期计划:
-- IndexRangeScan (idx_event_start)
--   - 合并后范围: [2024-01-01, 2024-04-01)
```

## 十、多条件查询优化验证

### 10.1 谓词下推

```cypher
-- 准备数据
CREATE TAG user(name: STRING, age: INT, city: STRING, status: STRING)
CREATE TAG INDEX idx_user_city ON user(city)
CREATE TAG INDEX idx_user_age ON user(age)
```

#### TC-PRED-001: 谓词下推到扫描
```cypher
-- 测试: 谓词应下推到扫描节点
\explain
MATCH (u:user)
WHERE u.city == "Beijing"
  AND u.age > 25
  AND u.status == "active"
RETURN u.name

-- 预期计划:
-- IndexScan (user)
--   - 索引: idx_user_city
--   - 下推谓词: city == "Beijing"
-- Filter
--   - 剩余谓词: age > 25, status == "active"
```

#### TC-PRED-002: 连接条件下推
```cypher
-- 测试: 连接条件下推到子查询
\explain
MATCH (u:user)-[:friend]->(f:user)
WHERE u.city == "Beijing"
  AND f.city == "Shanghai"
RETURN u.name, f.name

-- 预期计划:
-- IndexScan (u)
--   - 条件: city == "Beijing"
-- IndexJoin
--   - 连接条件: friend 关系
--   - 下推: f.city == "Shanghai" 到右侧扫描
```

### 10.2 条件重写优化

#### TC-REWRITE-001: 范围重写
```cypher
-- 测试: IN 重写为范围
\explain
MATCH (u:user)
WHERE u.age IN (25, 26, 27, 28, 29)
RETURN u.name

-- 预期计划:
-- IndexScan (user)
--   - 重写为: age >= 25 AND age <= 29
--   - 或保持为多个等值 OR
```

#### TC-REWRITE-002: 布尔表达式简化
```cypher
-- 测试: 布尔表达式简化
\explain
MATCH (u:user)
WHERE (u.age > 18 AND u.age < 60) OR u.age >= 60
RETURN u.name

-- 预期计划:
-- IndexScan (user)
--   - 简化后: age > 18
```

## 十一、EXPLAIN 输出验证清单

### 11.1 计划结构验证

对于每个测试用例，验证以下计划特征:

```yaml
验证项:
  - 节点类型: 是否符合预期 (IndexScan/SeqScan/HashJoin 等)
  - 节点顺序: 执行顺序是否合理
  - 成本估算: 是否有合理的成本值
  - 行数估算: 是否有行数估计
  - 条件信息: 谓词条件是否正确显示
  - 索引信息: 使用的索引名称是否正确
  - 方向信息: 遍历方向是否正确
```

### 11.2 成本对比验证

```cypher
-- 比较不同执行计划的成本
\explain MATCH (p:person {name: "Alice"})
-- 预期: IndexScan 成本 < SeqScan 成本

\explain MATCH (p:person) WHERE p.age > 25
-- 如果有索引: IndexScan 成本应较低
-- 如果无索引: SeqScan + Filter
```

## 十二、PROFILE 性能验证

### 12.1 实际执行验证

```cypher
-- 验证计划与实际执行一致
\profile MATCH (p:person {name: "Alice"}) RETURN p.age

-- 验证项:
-- - 实际扫描行数是否符合预期
-- - 实际执行时间是否合理
-- - 是否使用了预期的索引
```

### 12.2 优化效果量化

```cypher
-- 对比优化前后的性能
-- 无索引
\profile MATCH (p:person) WHERE p.city == "Beijing"

-- 创建索引后
CREATE TAG INDEX idx_person_city ON person(city)
\profile MATCH (p:person) WHERE p.city == "Beijing"

-- 预期: 有索引时扫描行数显著减少，执行时间缩短
```

## 十三、测试数据准备

```cypher
-- 优化器测试数据初始化
CREATE SPACE IF NOT EXISTS e2e_optimizer (vid_type=STRING)
USE e2e_optimizer

-- 人员表
CREATE TAG IF NOT EXISTS person(
    name: STRING,
    age: INT,
    city: STRING,
    salary: INT,
    department: STRING
)

-- 公司表
CREATE TAG IF NOT EXISTS company(
    name: STRING,
    industry: STRING,
    size: INT
)

-- 索引
CREATE TAG INDEX IF NOT EXISTS idx_person_name ON person(name)
CREATE TAG INDEX IF NOT EXISTS idx_person_age ON person(age)
CREATE TAG INDEX IF NOT EXISTS idx_person_city ON person(city)

-- 边
CREATE EDGE IF NOT EXISTS works_at(position: STRING, salary: INT)
CREATE EDGE IF NOT EXISTS manages(since: DATE)

-- 数据生成(使用脚本批量插入)
-- person: 10000条
-- company: 100条
-- works_at: 10000条
```

## 十四、自动化验证脚本模板

```bash
#!/bin/bash
# optimizer_test.sh

# 测试索引选择
echo "Testing index selection..."
graphdb-cli -c "\explain MATCH (p:person {name: 'Alice'}) RETURN p.age" | grep -q "IndexScan"
if [ $? -eq 0 ]; then
    echo "✓ TC-IDX-001 passed"
else
    echo "✗ TC-IDX-001 failed"
fi

# 测试连接算法
echo "Testing join algorithm..."
graphdb-cli -c "\explain MATCH (p:person)-[:works_at]->(c:company) RETURN p.name, c.name" | grep -q "HashJoin\|IndexJoin"
if [ $? -eq 0 ]; then
    echo "✓ TC-JOIN-001 passed"
else
    echo "✗ TC-JOIN-001 failed"
fi

# 更多测试...
```

## 十五、常见问题排查

### 15.1 优化器未选择预期索引

排查步骤:
1. 检查索引是否存在: `SHOW INDEXES`
2. 检查索引类型是否匹配查询条件
3. 检查统计信息是否最新
4. 使用 `\explain format=json` 查看详细成本

### 15.2 连接顺序不符合预期

排查步骤:
1. 检查表大小统计信息
2. 检查选择性估计
3. 查看 `join_order` 决策日志
4. 验证外键关系

### 15.3 聚合策略选择不当

排查步骤:
1. 检查输入数据排序状态
2. 检查内存限制配置
3. 检查分组键基数估计
4. 使用 `PROFILE` 对比实际性能
