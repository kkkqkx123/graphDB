# GraphDB 扩展类型 E2E 测试设计方案

## 概述

本文档针对 GraphDB 的扩展类型功能（地理空间、向量搜索、全文检索）设计 E2E 测试方案，验证这些高级功能的正确性和性能。

## 一、地理空间类型测试

### 1.1 功能范围

| 功能           | 说明                 | 测试优先级 |
| -------------- | -------------------- | ---------- |
| POINT 类型存储 | 经纬度坐标存储       | P0         |
| WKT 格式解析   | Well-Known Text 格式 | P0         |
| 距离计算       | Haversine 公式       | P0         |
| 方位角计算     | Bearing              | P1         |
| 边界框查询     | Point-in-rectangle   | P0         |
| 空间函数       | 10个标准 ST 函数     | P0         |

### 1.2 Schema 设计

```cypher
-- 创建地理空间测试空间
CREATE SPACE e2e_geography (vid_type=STRING)
USE e2e_geography

-- 创建带地理坐标的标签
CREATE TAG location(
    name: STRING NOT NULL,
    coord: GEOGRAPHY,              -- 地理坐标点
    address: STRING,
    category: STRING               -- 地点类别: restaurant, hotel, attraction
)

CREATE TAG city(
    name: STRING NOT NULL,
    center: GEOGRAPHY,             -- 城市中心坐标
    population: INT,
    area_km2: DOUBLE
)

-- 创建距离关系
CREATE EDGE nearby(
    distance_km: DOUBLE,           -- 实际距离
    walking_time_min: INT
)
```

### 1.3 测试数据设计思路

**数据规模**

- 地点: 200 个顶点 (餐厅、酒店、景点)
- 城市: 10 个顶点 (主要城市)
- 邻近关系: 500 条边

**数据生成策略**

- **地点数据**: 在中国主要城市范围内随机生成经纬度，确保坐标有效性(经度73-135, 纬度3-53)
- **城市数据**: 选择10个代表性城市，设置真实中心坐标和人口数据
- **邻近关系**: 基于实际距离(< 50km)建立边，模拟真实邻近关系

**数据特点**

- 覆盖不同密度区域(城市中心vs郊区)
- 包含边界情况(靠近城市边界、跨省等)
- 距离分布均匀(1km-50km)

### 1.4 测试用例

#### TC-GEO-001: 基础地理数据插入

```cypher
-- 测试 WKT 格式插入
INSERT VERTEX location(name, coord, category) VALUES "loc1":
    ("天安门", ST_GeogFromText("POINT(116.3974 39.9093)"), "attraction")

-- 测试函数创建点
INSERT VERTEX location(name, coord, category) VALUES "loc2":
    ("故宫", ST_Point(116.3972, 39.9163), "attraction")
```

#### TC-GEO-002: 距离计算验证

```cypher
-- 计算两点距离
MATCH (a:location {name: "天安门"}), (b:location {name: "故宫"})
RETURN ST_Distance(a.coord, b.coord) AS distance_km

-- 预期: 距离约 0.8km
```

#### TC-GEO-003: 范围查询

```cypher
-- 查找指定范围内的地点
MATCH (center:location {name: "天安门"})
MATCH (loc:location)
WHERE ST_DWithin(center.coord, loc.coord, 5.0)  -- 5km范围内
RETURN loc.name, ST_Distance(center.coord, loc.coord) AS distance
ORDER BY distance
```

#### TC-GEO-004: 边界框查询

```cypher
-- 查找北京市区内的地点(大致边界框)
MATCH (loc:location)
WHERE loc.coord IN BBOX(39.8, 40.0, 116.2, 116.5)
RETURN loc.name, loc.category
```

#### TC-GEO-005: 最近邻查询

```cypher
-- 查找最近的5个餐厅
MATCH (user:location {name: "天安门"})
MATCH (restaurant:location {category: "restaurant"})
RETURN restaurant.name,
       ST_Distance(user.coord, restaurant.coord) AS distance
ORDER BY distance
LIMIT 5
```

#### TC-GEO-006: 路径规划相关

```cypher
-- 计算从A到B的方位角
MATCH (a:location {name: "天安门"}), (b:location {name: "故宫"})
RETURN ST_Bearing(a.coord, b.coord) AS bearing_degrees

-- 验证: 方位角应该在合理范围内
```

#### TC-GEO-007: EXPLAIN 地理查询计划

```cypher
-- 查看地理查询的执行计划
\explain MATCH (loc:location)
WHERE ST_DWithin(ST_Point(116.4, 39.9), loc.coord, 10.0)
RETURN loc.name

-- 验证: 计划应显示 Filter 节点和地理函数调用
```

### 1.5 范围查询测试

#### 数值范围查询

```cypher
-- 单字段范围
MATCH (c:city)
WHERE c.population > 1000000 AND c.population < 10000000
RETURN c.name, c.population

-- 双字段范围(矩形区域)
MATCH (loc:location)
WHERE loc.coord.longitude > 116.0 AND loc.coord.longitude < 117.0
  AND loc.coord.latitude > 39.5 AND loc.coord.latitude < 40.0
RETURN loc.name
```

#### 时间范围查询

```cypher
-- 创建带时间属性的标签
CREATE TAG event(
    name: STRING,
    start_time: TIMESTAMP,
    end_time: TIMESTAMP,
    location: GEOGRAPHY
)

-- 时间范围查询
MATCH (e:event)
WHERE e.start_time >= datetime("2024-01-01")
  AND e.end_time <= datetime("2024-12-31")
RETURN e.name, e.start_time
```

## 二、向量搜索测试

### 2.1 功能范围

| 功能       | 说明              | 测试优先级 |
| ---------- | ----------------- | ---------- |
| 向量存储   | 高维向量数据      | P0         |
| 相似度搜索 | ANN 近似最近邻    | P0         |
| 距离度量   | Cosine/Euclid/Dot | P0         |
| 过滤搜索   | 带条件的向量搜索  | P0         |
| 批量操作   | 批量插入/搜索     | P1         |
| 索引管理   | HNSW 索引         | P1         |

### 2.2 Schema 设计

```cypher
-- 创建向量测试空间
CREATE SPACE e2e_vector (vid_type=STRING)
USE e2e_vector

-- 创建带向量属性的标签
CREATE TAG product_vector(
    product_id: STRING NOT NULL,
    name: STRING,
    category: STRING,
    embedding: VECTOR(128),        -- 128维向量
    price: DOUBLE,
    tags: LIST<STRING>
)

CREATE TAG image_vector(
    image_id: STRING NOT NULL,
    url: STRING,
    feature: VECTOR(512),          -- 512维图像特征
    labels: LIST<STRING>
)

CREATE TAG text_embedding(
    doc_id: STRING NOT NULL,
    content: STRING,
    embedding: VECTOR(768),         -- 768维文本嵌入(BERT)
    category: STRING
)
```

### 2.3 测试数据设计思路

**数据规模**

- 商品向量: 1000 个顶点，128维
- 图像向量: 500 个顶点，512维
- 文本嵌入: 2000 个顶点，768维

**数据生成策略**

- **商品向量**: 使用随机正态分布生成，模拟商品相似度
- **图像向量**: 基于真实图像特征分布，包含聚类结构(模拟不同类别)
- **文本嵌入**: 模拟BERT输出，不同类别文本在向量空间有明显区分

**数据特点**

- 向量维度覆盖常见场景(128/512/768)
- 包含明显的聚类结构(用于验证搜索结果)
- 混合属性支持过滤搜索

### 2.4 测试用例

#### TC-VEC-001: 向量数据插入

```cypher
-- 插入带向量的顶点
INSERT VERTEX product_vector(product_id, name, category, embedding, price)
VALUES "p1": ("PROD001", "iPhone 15", "electronics",
    [0.1, 0.2, ..., 0.05], 5999.00)

-- 批量插入
INSERT VERTEX product_vector(product_id, name, category, embedding, price)
VALUES "p2": ("PROD002", "MacBook Pro", "electronics", [...], 14999.00),
       "p3": ("PROD003", "AirPods", "electronics", [...], 1999.00)
```

#### TC-VEC-002: 基础相似度搜索

```cypher
-- 查找最相似的商品
MATCH (p:product_vector)
WHERE p.category == "electronics"
ORDER BY cosine_similarity(p.embedding, [0.1, 0.2, ...]) DESC
LIMIT 10
```

#### TC-VEC-003: 带过滤条件的向量搜索

```cypher
-- 在价格范围内搜索相似商品
MATCH (p:product_vector)
WHERE p.price < 5000
  AND p.category == "electronics"
ORDER BY cosine_similarity(p.embedding, $query_vector) DESC
LIMIT 5
```

#### TC-VEC-004: 多距离度量测试

```cypher
-- Cosine 相似度
MATCH (p:product_vector)
ORDER BY cosine_similarity(p.embedding, $v) DESC
LIMIT 5

-- Euclidean 距离
MATCH (p:product_vector)
ORDER BY l2_distance(p.embedding, $v) ASC
LIMIT 5

-- Dot 积
MATCH (p:product_vector)
ORDER BY dot_product(p.embedding, $v) DESC
LIMIT 5
```

#### TC-VEC-005: 向量索引测试

```cypher
-- 创建向量索引
CREATE VECTOR INDEX idx_product_embedding ON product_vector(embedding)
    WITH (dimension=128, metric=cosine, hnsw_m=16, ef_construction=200)

-- 验证索引使用
\explain MATCH (p:product_vector)
ORDER BY cosine_similarity(p.embedding, $v) DESC
LIMIT 10
```

#### TC-VEC-006: EXPLAIN 向量查询计划

```cypher
-- 查看向量搜索计划
\explain MATCH (p:product_vector)
WHERE p.category == "electronics"
ORDER BY cosine_similarity(p.embedding, $query_vector) DESC
LIMIT 10

-- 验证: 计划应显示 VectorSearch 节点
```

## 三、全文检索测试

### 3.1 功能范围

| 功能     | 说明             | 测试优先级 |
| -------- | ---------------- | ---------- |
| 全文索引 | BM25/Inversearch | P0         |
| 文本搜索 | MATCH/SEARCH     | P0         |
| 分词     | CJK/标准分词器   | P0         |
| 高亮     | 搜索结果高亮     | P1         |
| 评分     | 相关性评分       | P1         |
| 布尔查询 | AND/OR/NOT       | P0         |

### 3.2 Schema 设计

```cypher
-- 创建全文检索测试空间
CREATE SPACE e2e_fulltext (vid_type=STRING)
USE e2e_fulltext

-- 创建带全文索引的标签
CREATE TAG article(
    doc_id: STRING NOT NULL,
    title: STRING,
    content: STRING,
    author: STRING,
    publish_date: TIMESTAMP,
    tags: LIST<STRING>
)

CREATE TAG product_desc(
    sku: STRING NOT NULL,
    name: STRING,
    description: STRING,
    category: STRING
)

-- 创建全文索引
CREATE FULLTEXT INDEX idx_article_content ON article(content)
    WITH (engine=bm25, analyzer=standard)

CREATE FULLTEXT INDEX idx_article_title ON article(title)
    WITH (engine=inversearch, analyzer=cjk)
```

### 3.3 测试数据设计思路

**数据规模**

- 文章: 500 个顶点
- 商品描述: 1000 个顶点

**数据生成策略**

- **文章数据**: 混合中英文内容，长度500-5000字，覆盖技术、新闻、博客等类型
- **商品描述**: 电商风格文本，包含商品特性、规格、使用说明
- **查询词**: 准备高频词、低频词、中文词组、英文短语

**数据特点**

- 中英文混合(测试分词)
- 不同长度文档(测试评分)
- 多字段覆盖(测试多字段搜索)

### 3.4 测试用例

#### TC-FT-001: 全文索引创建

```cypher
-- 创建 BM25 索引
CREATE FULLTEXT INDEX idx_content ON article(content)
    WITH (engine=bm25, analyzer=standard)

-- 创建 Inversearch 索引
CREATE FULLTEXT INDEX idx_title ON article(title)
    WITH (engine=inversearch, analyzer=cjk)

-- 验证索引
SHOW INDEXES
```

#### TC-FT-002: 基础全文搜索

```cypher
-- 单关键词搜索
SEARCH IN article.content FOR "数据库"
RETURN article.doc_id, article.title, score()

-- 多关键词搜索
SEARCH IN article.content FOR "图数据库 性能优化"
RETURN article.doc_id, article.title, score()
ORDER BY score() DESC
```

#### TC-FT-003: 布尔查询

```cypher
-- AND 查询
SEARCH IN article.content FOR "图数据库 AND 索引"
RETURN article.doc_id, article.title

-- OR 查询
SEARCH IN article.content FOR "MySQL OR PostgreSQL"
RETURN article.doc_id, article.title

-- NOT 查询
SEARCH IN article.content FOR "数据库 NOT MySQL"
RETURN article.doc_id, article.title
```

#### TC-FT-004: 中文分词测试

```cypher
-- CJK 分词测试
SEARCH IN article.title FOR "全文检索"
RETURN article.title, score()

-- 长词组测试
SEARCH IN article.content FOR "分布式图数据库"
RETURN article.doc_id, article.title
```

#### TC-FT-005: 结合图遍历的搜索

```cypher
-- 搜索文章及其相关作者的其他文章
SEARCH IN article.content FOR "优化" YIELD doc_id, score
MATCH (a:article {doc_id: doc_id})-[:written_by]->(author:author)
MATCH (author)<-[:written_by]-(other:article)
WHERE other.doc_id != doc_id
RETURN a.title, author.name, collect(other.title) AS other_articles
```

#### TC-FT-006: 多字段搜索

```cypher
-- 同时搜索标题和内容
SEARCH IN article.title, article.content FOR "GraphDB"
RETURN article.doc_id, article.title,
       highlight(article.title), highlight(article.content)
```

#### TC-FT-007: EXPLAIN 全文搜索计划

```cypher
-- 查看全文搜索计划
\explain SEARCH IN article.content FOR "数据库"
RETURN article.doc_id, score()

-- 验证: 计划应显示 FulltextSearch 节点
```

## 四、多条件组合查询测试

### 4.1 复杂条件组合

```cypher
-- 场景: 查找指定区域内、价格范围内、相似度高的商品
MATCH (p:product_vector)
WHERE p.price >= 1000 AND p.price <= 5000
  AND p.category IN ("electronics", "computers")
  AND ST_DWithin(p.location, $user_location, 50.0)
ORDER BY cosine_similarity(p.embedding, $query_vector) DESC
LIMIT 20
```

### 4.2 混合查询测试用例

#### TC-MIX-001: 地理+向量组合

```cypher
-- 查找附近相似的餐厅
MATCH (r:restaurant_vector)
WHERE ST_DWithin(r.location, $user_location, 10.0)
  AND r.rating > 4.0
ORDER BY cosine_similarity(r.taste_vector, $preference_vector) DESC
LIMIT 10
```

#### TC-MIX-002: 全文+范围组合

```cypher
-- 搜索特定时间段内的文章
SEARCH IN article.content FOR "GraphDB" YIELD doc_id, score
MATCH (a:article {doc_id: doc_id})
WHERE a.publish_date >= datetime("2024-01-01")
  AND a.publish_date < datetime("2024-07-01")
RETURN a.title, a.publish_date, score
ORDER BY score DESC
```

#### TC-MIX-003: 三类型组合

```cypher
-- 全文搜索 + 向量相似 + 地理范围
SEARCH IN product_desc.description FOR "高性能" YIELD sku, score AS text_score
MATCH (p:product_vector {sku: sku})
WHERE ST_DWithin(p.store_location, $user_location, 100.0)
WITH p, text_score,
     cosine_similarity(p.embedding, $query_vector) AS vec_score
RETURN p.name, p.price, text_score, vec_score,
       (text_score * 0.5 + vec_score * 0.5) AS combined_score
ORDER BY combined_score DESC
LIMIT 10
```

## 五、EXPLAIN 验证要点

### 5.1 地理空间查询计划验证

```cypher
\explain MATCH (loc:location)
WHERE ST_DWithin($center, loc.coord, 10.0)
RETURN loc.name
```

**预期计划特征**:

- Filter 节点包含地理函数
- 可能使用地理索引(如果存在)
- 成本估算考虑距离计算开销

### 5.2 向量查询计划验证

```cypher
\explain MATCH (p:product_vector)
ORDER BY cosine_similarity(p.embedding, $v) DESC
LIMIT 10
```

**预期计划特征**:

- VectorSearch 节点
- 显示使用的距离度量
- 显示 HNSW 索引访问(如果存在)

### 5.3 全文查询计划验证

```cypher
\explain SEARCH IN article.content FOR "数据库"
```

**预期计划特征**:

- FulltextSearch 节点
- 显示使用的搜索引擎(BM25/Inversearch)
- 显示分词器类型

## 六、性能基准

| 测试项              | 数据规模  | 预期响应时间 |
| ------------------- | --------- | ------------ |
| 地理范围查询        | 200 地点  | < 100ms      |
| 最近邻查询          | 200 地点  | < 50ms       |
| 向量相似搜索(128维) | 1000 向量 | < 200ms      |
| 向量相似搜索(512维) | 500 向量  | < 300ms      |
| 全文搜索            | 500 文档  | < 100ms      |
| 布尔全文查询        | 500 文档  | < 150ms      |
| 混合查询(地理+向量) | 1000 数据 | < 500ms      |

## 七、数据准备脚本模板

```cypher
-- 地理数据初始化
CREATE SPACE IF NOT EXISTS e2e_geography (vid_type=STRING)
USE e2e_geography

CREATE TAG IF NOT EXISTS location(
    name: STRING NOT NULL,
    coord: GEOGRAPHY,
    address: STRING,
    category: STRING
)

-- 批量插入地理数据(使用脚本生成)
-- INSERT VERTEX location(...) VALUES ...

-- 向量数据初始化
CREATE SPACE IF NOT EXISTS e2e_vector (vid_type=STRING)
USE e2e_vector

CREATE TAG IF NOT EXISTS product_vector(
    product_id: STRING NOT NULL,
    name: STRING,
    category: STRING,
    embedding: VECTOR(128),
    price: DOUBLE
)

-- 全文数据初始化
CREATE SPACE IF NOT EXISTS e2e_fulltext (vid_type=STRING)
USE e2e_fulltext

CREATE TAG IF NOT EXISTS article(
    doc_id: STRING NOT NULL,
    title: STRING,
    content: STRING,
    author: STRING,
    publish_date: TIMESTAMP
)

CREATE FULLTEXT INDEX IF NOT EXISTS idx_article_content
ON article(content) WITH (engine=bm25)
```

## 八、故障场景测试

### 8.1 地理空间错误

```
TC-ERR-GEO-001: 无效坐标
  COMMAND: INSERT VERTEX location(coord) VALUES "1": (ST_Point(200, 100))
  EXPECT: 返回坐标越界错误

TC-ERR-GEO-002: 类型不匹配
  COMMAND: ST_Distance(ST_Point(0,0), "invalid")
  EXPECT: 返回类型错误
```

### 8.2 向量错误

```
TC-ERR-VEC-001: 维度不匹配
  COMMAND: cosine_similarity([1,2,3], [1,2,3,4])
  EXPECT: 返回维度不匹配错误

TC-ERR-VEC-002: 空向量
  COMMAND: cosine_similarity([], [1,2,3])
  EXPECT: 返回空向量错误
```

### 8.3 全文检索错误

```
TC-ERR-FT-001: 索引不存在
  COMMAND: SEARCH IN nonexistent.field FOR "test"
  EXPECT: 返回索引不存在错误

TC-ERR-FT-002: 语法错误
  COMMAND: SEARCH IN article.content FOR "AND OR NOT"
  EXPECT: 返回查询语法错误
```
