# 图数据库全文检索应用场景分析

## 概述

全文检索与图数据库的结合，解决了**图结构数据中的文本内容搜索**问题。传统的图数据库只能通过精确匹配或正则表达式查找节点属性，而全文检索提供了**语义搜索**、**模糊匹配**和**相关性排序**的能力。

---

## 1. 核心概念：全文检索如何关联到图数据库

### 1.1 数据模型

```
图数据模型：
┌─────────────┐
│   Vertex    │
│  (顶点/节点) │
├─────────────┤
│ - id        │  ← 全文索引关联键
│ - tag/label │  ← 索引作用域
│ - properties│  ← 被索引的文本字段
│   - title   │  ← 可索引字段
│   - content │  ← 可索引字段
│   - ...     │
└─────────────┘

全文索引模型：
┌─────────────────────────────┐
│   Fulltext Index            │
├─────────────────────────────┤
│ - index_name                │
│ - space_id                  │
│ - tag_name        ← 关联点  │
│ - field_name      ← 关联点  │
│ - engine_type (bm25/inv)    │
│ - index_data (倒排索引)     │
└─────────────────────────────┘
```

### 1.2 关联机制

**关键设计原则**：
1. **索引与 Tag 绑定**：全文索引建立在特定 Tag 的特定字段上
2. **顶点 ID 作为文档 ID**：每个顶点在全文索引中是一个文档
3. **异步同步**：图数据变更后，异步更新全文索引

**关联流程**：
```
用户插入顶点
  │ INSERT VERTEX Post(id, title, content) 
  │ VALUES ("v1", "Rust 编程", "Rust 是一门系统编程语言")
  ▼
存储层 (Redb)
  │ 1. 写入图数据
  │ 2. 提交事务
  │ 3. 触发事件
  ▼
FulltextCoordinator (异步)
  │ 1. 检查 Post 标签是否有全文索引
  │ 2. 发现 idx_post_title 索引 Post.title 字段
  │ 3. 调用 engine.index("v1", "Rust 编程")
  ▼
BM25/Inversearch 引擎
  │ 添加文档到倒排索引
  │ doc_id: "v1", content: "Rust 编程"
```

### 1.3 查询时的关联

```
用户查询
  │ MATCH (p:Post) WHERE p.content MATCH "Rust" RETURN p
  ▼
查询引擎
  │ 1. 识别 WHERE 子句中的 MATCH 表达式
  │ 2. 提取全文搜索条件：p.content MATCH "Rust"
  │ 3. 查找 Post.content 的全文索引
  ▼
FulltextCoordinator
  │ 1. 调用 engine.search("Rust", limit)
  │ 2. 返回匹配的 doc_ids: ["v1", "v5", "v10"]
  ▼
查询引擎
  │ 1. 根据 doc_ids 从 Redb 查询完整顶点数据
  │ 2. 返回结果
```

---

## 2. 实际应用场景

### 2.1 场景一：内容管理系统 (CMS)

#### 业务背景
新闻网站、博客平台、知识库等需要存储和检索大量文本内容。

#### 数据模型
```sql
-- 定义标签
CREATE TAG IF NOT EXISTS Article(
    id STRING,
    title STRING,           -- 标题（需要全文索引）
    content STRING,         -- 正文（需要全文索引）
    author STRING,
    category STRING,
    publish_time TIMESTAMP,
    view_count INT
);

CREATE TAG IF NOT EXISTS Category(
    id STRING,
    name STRING,
    parent_id STRING
);

-- 定义边
CREATE EDGE IF NOT EXISTS BELONGS_TO();
CREATE EDGE IF NOT EXISTS WRITTEN_BY();
```

#### 全文索引创建
```sql
-- 为标题创建全文索引（BM25 引擎）
CREATE FULLTEXT INDEX idx_article_title 
ON Article(title) 
USING bm25;

-- 为正文创建全文索引（BM25 引擎，适合长文本）
CREATE FULLTEXT INDEX idx_article_content 
ON Article(content) 
USING bm25;

-- 为分类名称创建全文索引
CREATE FULLTEXT INDEX idx_category_name 
ON Category(name) 
USING inversearch;
```

#### 典型查询

**1. 搜索标题包含关键词的文章**
```sql
MATCH (a:Article)
WHERE a.title MATCH "图数据库"
RETURN a.id, a.title, a.publish_time
ORDER BY a.publish_time DESC
LIMIT 10;
```

**2. 搜索正文包含技术术语的文章**
```sql
MATCH (a:Article)
WHERE a.content MATCH "Rust 内存安全"
RETURN a.id, a.title, score(a) as relevance
ORDER BY relevance DESC
LIMIT 20;
```

**3. 组合查询：全文搜索 + 图遍历**
```sql
-- 搜索某作者写的关于"人工智能"的文章
MATCH (a:Article)-[:WRITTEN_BY]->(auth:Author)
WHERE a.content MATCH "人工智能" AND auth.name == "张三"
RETURN a.title, a.publish_time;
```

**4. 分类筛选 + 全文搜索**
```sql
MATCH (a:Article)-[:BELONGS_TO]->(c:Category)
WHERE a.content MATCH "机器学习" AND c.name == "技术博客"
RETURN a.title, a.view_count
ORDER BY a.view_count DESC;
```

#### 应用价值
- **提升用户体验**：模糊搜索，容忍拼写错误
- **提高检索效率**：倒排索引比全表扫描快 100-1000 倍
- **支持复杂查询**：全文搜索 + 图关系组合

---

### 2.2 场景二：社交网络分析

#### 业务背景
社交平台需要分析用户发布的内容、评论、话题等。

#### 数据模型
```sql
-- 用户
CREATE TAG IF NOT EXISTS User(
    id STRING,
    username STRING,
    bio STRING,             -- 个人简介（可全文索引）
    location STRING
);

-- 帖子
CREATE TAG IF NOT EXISTS Post(
    id STRING,
    content STRING,         -- 帖子内容（需要全文索引）
    hashtags STRING,
    created_at TIMESTAMP
);

-- 评论
CREATE TAG IF NOT EXISTS Comment(
    id STRING,
    content STRING,         -- 评论内容（需要全文索引）
    created_at TIMESTAMP
);

-- 边
CREATE EDGE IF NOT EXISTS POSTS();
CREATE EDGE IF NOT EXISTS COMMENTS_ON();
CREATE EDGE IF NOT EXISTS FOLLOWS();
CREATE EDGE IF NOT EXISTS LIKES();
```

#### 全文索引创建
```sql
CREATE FULLTEXT INDEX idx_user_bio ON User(bio) USING inversearch;
CREATE FULLTEXT INDEX idx_post_content ON Post(content) USING bm25;
CREATE FULLTEXT INDEX idx_comment_content ON Comment(content) USING bm25;
```

#### 典型查询

**1. 搜索包含特定话题的帖子**
```sql
MATCH (p:Post)
WHERE p.content MATCH "人工智能大会"
RETURN p.id, p.content, p.created_at
ORDER BY p.created_at DESC
LIMIT 50;
```

**2. 搜索用户及其相关内容**
```sql
-- 查找个人简介包含"数据科学家"的用户及其发布的帖子
MATCH (u:User)-[:POSTS]->(p:Post)
WHERE u.bio MATCH "数据科学家" AND p.content MATCH "机器学习"
RETURN u.username, count(p) as post_count;
```

**3. 舆情分析：搜索提及某关键词的帖子和评论**
```sql
-- 搜索提及"产品发布"的帖子和评论
MATCH (p:Post)
WHERE p.content MATCH "产品发布"
RETURN p.id, p.created_at, score(p) as relevance
UNION
MATCH (c:Comment)-[:COMMENTS_ON]->(p:Post)
WHERE c.content MATCH "产品发布"
RETURN c.id, c.created_at, score(c) as relevance;
```

**4. 影响力分析：搜索并分析传播路径**
```sql
-- 搜索包含"突发新闻"的帖子，并分析转发链
MATCH path = (original:Post)-[:RETWEETS*0..5]->(spread:Post)
WHERE original.content MATCH "突发新闻"
RETURN original.id, length(path) as spread_depth, count(spread) as total_spreads;
```

#### 应用价值
- **实时舆情监控**：快速定位热点话题
- **用户画像分析**：基于内容理解用户兴趣
- **影响力追踪**：分析信息传播路径

---

### 2.3 场景三：电商商品搜索与推荐

#### 业务背景
电商平台需要支持商品搜索、个性化推荐、关联购买等功能。

#### 数据模型
```sql
-- 商品
CREATE TAG IF NOT EXISTS Product(
    id STRING,
    name STRING,            -- 商品名称（需要全文索引）
    description STRING,     -- 商品描述（需要全文索引）
    category STRING,
    price DECIMAL,
    brand STRING,
    specs STRING            -- 规格参数（可全文索引）
);

-- 用户
CREATE TAG IF NOT EXISTS Customer(
    id STRING,
    name STRING,
    preferences STRING      -- 偏好描述（可全文索引）
);

-- 类别
CREATE TAG IF NOT EXISTS Category(
    id STRING,
    name STRING,
    parent_id STRING
);

-- 边
CREATE EDGE IF NOT EXISTS BELONGS_TO();
CREATE EDGE IF NOT EXISTS VIEWED(view_time INT);
CREATE EDGE IF NOT EXISTS PURCHASED(quantity INT, price DECIMAL);
CREATE EDGE IF NOT EXISTS SIMILAR_TO(similarity FLOAT);
CREATE EDGE IF NOT EXISTS BOUGHT_TOGETHER(frequency INT);
```

#### 全文索引创建
```sql
-- 商品名称索引（精确匹配重要）
CREATE FULLTEXT INDEX idx_product_name 
ON Product(name) 
USING inversearch 
WITH TOKENIZER = 'cjk';

-- 商品描述索引（适合长文本）
CREATE FULLTEXT INDEX idx_product_description 
ON Product(description) 
USING bm25;

-- 规格参数索引
CREATE FULLTEXT INDEX idx_product_specs 
ON Product(specs) 
USING inversearch;
```

#### 典型查询

**1. 商品搜索**
```sql
-- 搜索商品名称包含"智能手机"的商品
MATCH (p:Product)
WHERE p.name MATCH "智能手机"
RETURN p.id, p.name, p.price, p.brand
ORDER BY p.price ASC
LIMIT 20;
```

**2. 全文搜索 + 价格区间过滤**
```sql
MATCH (p:Product)
WHERE p.description MATCH "轻薄笔记本" 
  AND p.price >= 3000 AND p.price <= 8000
RETURN p.id, p.name, p.price
ORDER BY score(p) DESC;
```

**3. 个性化推荐：基于用户历史行为**
```sql
-- 根据用户购买历史推荐相似商品
MATCH (c:Customer)-[:PURCHASED]->(p1:Product)
WHERE p1.category == "电子产品"
MATCH (p1)-[:SIMILAR_TO]->(p2:Product)
WHERE p2.description MATCH "高性能 长续航"
RETURN p2.id, p2.name, p2.price, count(*) as recommendation_score
ORDER BY recommendation_score DESC
LIMIT 10;
```

**4. 关联购买推荐**
```sql
-- 购买了 A 商品的用户也购买了 B 商品
MATCH (p1:Product)-[:BOUGHT_TOGETHER]->(p2:Product)
WHERE p1.name MATCH "iPhone 15"
RETURN p2.name, p2.price
ORDER BY p2.frequency DESC
LIMIT 5;
```

**5. 跨类别搜索**
```sql
-- 搜索"运动"相关的商品和类别
MATCH (p:Product)-[:BELONGS_TO]->(c:Category)
WHERE p.name MATCH "运动鞋" OR p.description MATCH "运动"
RETURN p.name, c.name as category, p.price;
```

#### 应用价值
- **提升转化率**：精准的商品搜索
- **增加客单价**：关联购买推荐
- **个性化体验**：基于用户行为的智能推荐

---

### 2.4 场景四：知识图谱与语义搜索

#### 业务背景
企业知识库、学术研究、医疗诊断等需要理解概念之间的关系。

#### 数据模型
```sql
-- 实体
CREATE TAG IF NOT EXISTS Entity(
    id STRING,
    name STRING,            -- 实体名称（需要全文索引）
    type STRING,            -- 实体类型
    description STRING,     -- 实体描述（需要全文索引）
    aliases STRING          -- 别名（可全文索引）
);

-- 关系类型
CREATE TAG IF NOT EXISTS RelationType(
    id STRING,
    name STRING,
    category STRING
);

-- 边（带属性的关系）
CREATE EDGE IF NOT EXISTS RELATES_TO(
    relation_type STRING,
    confidence FLOAT,
    source STRING
);

CREATE EDGE IF NOT EXISTS IS_A();      -- 是一种
CREATE EDGE IF NOT EXISTS PART_OF();   -- 部分属于
CREATE EDGE IF NOT EXISTS CAUSES();    -- 导致
CREATE EDGE IF NOT EXISTS TREATS();    -- 治疗
```

#### 全文索引创建
```sql
CREATE FULLTEXT INDEX idx_entity_name ON Entity(name) USING inversearch;
CREATE FULLTEXT INDEX idx_entity_description ON Entity(description) USING bm25;
CREATE FULLTEXT INDEX idx_entity_aliases ON Entity(aliases) USING inversearch;
```

#### 典型查询

**1. 实体搜索**
```sql
-- 搜索名称包含"机器学习"的实体
MATCH (e:Entity)
WHERE e.name MATCH "机器学习"
RETURN e.id, e.name, e.type, e.description;
```

**2. 语义扩展搜索**
```sql
-- 搜索"人工智能"及其相关概念
MATCH path = (e1:Entity)-[:RELATES_TO*0..2]-(e2:Entity)
WHERE e1.name MATCH "人工智能"
RETURN e2.name, e2.type, length(path) as relevance_depth;
```

**3. 知识推理**
```sql
-- 查找可能的新关系（知识补全）
MATCH (e1:Entity)-[:RELATES_TO]->(e2:Entity), 
      (e1)-[:RELATES_TO]->(e3:Entity)
WHERE e2 != e3 
  AND NOT (e2)-[:RELATES_TO]->(e3)
  AND e1.description MATCH "疾病治疗"
RETURN e2.name as potential_cause, 
       e3.name as potential_effect;
```

**4. 多跳关系查询**
```sql
-- 医学知识图谱：查找某种疾病的所有可能治疗方法
MATCH path = (disease:Entity)-[:CAUSES*0..3]->(symptom:Entity),
           (treatment:Entity)-[:TREATS]->(disease)
WHERE disease.name MATCH "糖尿病"
RETURN treatment.name, treatment.description, length(path) as symptom_count;
```

#### 应用价值
- **智能问答**：理解问题语义，返回相关知识
- **知识发现**：通过关系推理发现新知识
- **决策支持**：提供全面的相关知识网络

---

### 2.5 场景五：日志分析与安全监控

#### 业务背景
IT 运维、安全分析需要从海量日志中快速定位问题。

#### 数据模型
```sql
-- 日志条目
CREATE TAG IF NOT EXISTS LogEntry(
    id STRING,
    message STRING,         -- 日志内容（需要全文索引）
    level STRING,           -- ERROR/WARN/INFO
    service STRING,
    host STRING,
    timestamp TIMESTAMP
);

-- 告警
CREATE TAG IF NOT EXISTS Alert(
    id STRING,
    title STRING,           -- 告警标题（需要全文索引）
    description STRING,     -- 告警描述（需要全文索引）
    severity STRING,
    status STRING,
    created_at TIMESTAMP
);

-- 边
CREATE EDGE IF NOT EXISTS TRIGGERED_BY();  -- 告警由日志触发
CREATE EDGE IF NOT EXISTS CORRELATED_WITH(); -- 告警关联
CREATE EDGE IF NOT EXISTS DEPLOYED_ON();   -- 服务部署在主机
```

#### 全文索引创建
```sql
CREATE FULLTEXT INDEX idx_log_message ON LogEntry(message) USING bm25;
CREATE FULLTEXT INDEX idx_alert_title ON Alert(title) USING inversearch;
CREATE FULLTEXT INDEX idx_alert_description ON Alert(description) USING bm25;
```

#### 典型查询

**1. 日志搜索**
```sql
-- 搜索包含错误信息的日志
MATCH (l:LogEntry)
WHERE l.message MATCH "连接超时 数据库"
  AND l.level == "ERROR"
RETURN l.id, l.service, l.host, l.timestamp
ORDER BY l.timestamp DESC
LIMIT 50;
```

**2. 告警关联分析**
```sql
-- 搜索相关告警并分析关联
MATCH (a1:Alert)-[:CORRELATED_WITH]->(a2:Alert)
WHERE a1.description MATCH "CPU 使用率过高"
RETURN a1.title, a2.title, a2.severity;
```

**3. 根因分析**
```sql
-- 查找告警的源头日志
MATCH (a:Alert)<-[:TRIGGERED_BY]-(l:LogEntry)
WHERE a.title MATCH "服务不可用"
  AND l.message MATCH "异常 错误"
RETURN l.message, l.service, l.timestamp
ORDER BY l.timestamp ASC
LIMIT 10;
```

**4. 影响范围分析**
```sql
-- 某台主机故障，分析影响的服务
MATCH (l:LogEntry)-[:DEPLOYED_ON]->(h:Host)
WHERE l.message MATCH "磁盘空间不足" 
  AND h.hostname == "prod-db-01"
MATCH (s:Service)-[:DEPLOYED_ON]->(h)
RETURN DISTINCT s.service_name as affected_services;
```

#### 应用价值
- **快速故障定位**：秒级搜索海量日志
- **智能告警关联**：减少告警噪音
- **根因分析**：快速定位问题源头

---

## 3. 技术优势对比

### 3.1 传统方案 vs 图数据库 + 全文检索

| 功能 | 传统关系数据库 | 搜索引擎 (Elasticsearch) | 图数据库 + 全文检索 |
|------|---------------|------------------------|-------------------|
| **精确匹配** | ✅ 优秀 | ✅ 优秀 | ✅ 优秀 |
| **模糊搜索** | ❌ LIKE 性能差 | ✅ 优秀 | ✅ 优秀 |
| **全文检索** | ❌ 有限支持 | ✅ 专业 | ✅ 良好 |
| **关系查询** | ⚠️ JOIN 开销大 | ❌ 不支持 | ✅ 原生支持 |
| **多跳查询** | ❌ 性能极差 | ❌ 不支持 | ✅ 高效 |
| **相关性评分** | ❌ 不支持 | ✅ BM25 | ✅ BM25 |
| **图遍历 + 搜索** | ❌ 无法实现 | ❌ 无法实现 | ✅ 原生支持 |

### 3.2 性能对比示例

**场景**：在 100 万篇文章中搜索包含"图数据库"的文章，并找出作者的其他文章

| 方案 | 耗时 | 说明 |
|------|------|------|
| MySQL LIKE | 5000ms+ | 全表扫描 |
| MySQL + ES | 50ms (搜索) + 200ms (JOIN) | 需要应用层关联 |
| **GraphDB + BM25** | **30ms** | 原生集成，一次查询 |

---

## 4. 最佳实践建议

### 4.1 索引设计原则

1. **只索引必要字段**：不是所有文本字段都需要全文索引
2. **根据字段长度选择引擎**：
   - 短文本（标题、名称）→ Inversearch
   - 长文本（正文、描述）→ BM25
3. **避免过度索引**：每个索引都会增加写入开销

### 4.2 查询优化建议

1. **先过滤后搜索**：
   ```sql
   -- 好的做法
   MATCH (p:Product)
   WHERE p.category == "电子产品" AND p.name MATCH "手机"
   
   -- 避免
   MATCH (p:Product)
   WHERE p.name MATCH "手机" AND p.category == "电子产品"
   ```

2. **限制结果数量**：
   ```sql
   MATCH (a:Article)
   WHERE a.content MATCH "关键词"
   RETURN a LIMIT 100;  -- 始终添加 LIMIT
   ```

3. **使用评分排序**：
   ```sql
   MATCH (a:Article)
   WHERE a.content MATCH "关键词"
   RETURN a, score(a) as relevance
   ORDER BY relevance DESC;
   ```

### 4.3 数据同步策略

1. **实时同步**：适用于对实时性要求高的场景
   ```rust
   // 事务提交后立即同步
   on_vertex_inserted(vertex) -> coordinator.index()
   ```

2. **批量同步**：适用于大批量导入场景
   ```rust
   // 批量导入后统一重建索引
   batch_insert_vertices() -> coordinator.rebuild_index()
   ```

3. **异步队列**：适用于高并发场景
   ```rust
   // 写入消息队列，后台消费
   on_vertex_inserted() -> queue.push() -> worker.consume()
   ```

---

## 5. 总结

### 5.1 核心价值

图数据库 + 全文检索的组合提供了**独一无二的查询能力**：

1. **语义搜索**：理解内容含义，不仅仅是关键词匹配
2. **关系感知**：在搜索结果基础上进行图遍历和分析
3. **一站式解决方案**：无需在多个系统间切换

### 5.2 适用场景特征

当你的应用场景同时具备以下特征时，图数据库 + 全文检索是理想选择：

- ✅ 数据包含丰富的文本内容需要搜索
- ✅ 数据之间存在复杂的关系网络
- ✅ 需要结合内容搜索和关系分析
- ✅ 对查询性能有较高要求

### 5.3 技术选型建议

| 场景特征 | 推荐引擎 | 理由 |
|----------|---------|------|
| 中文内容为主 | Inversearch | 原生 CJK 分词支持 |
| 长文本搜索 | BM25 | Tantivy 针对长文本优化 |
| 需要高亮显示 | Inversearch | 强大的高亮功能 |
| 企业级应用 | BM25 | 更成熟稳定 |
| 混合场景 | 两者结合 | 不同字段使用不同引擎 |

---

## 附录：完整示例代码

### A.1 创建完整的知识图谱搜索系统

```sql
-- 1. 创建标签
CREATE TAG IF NOT EXISTS Article(
    id STRING,
    title STRING,
    content STRING,
    author_id STRING,
    created_at TIMESTAMP
);

CREATE TAG IF NOT EXISTS Author(
    id STRING,
    name STRING,
    bio STRING,
    expertise STRING
);

-- 2. 创建边
CREATE EDGE IF NOT EXISTS WRITTEN_BY();
CREATE EDGE IF NOT EXISTS CITED();
CREATE EDGE IF NOT EXISTS RELATED_TO();

-- 3. 创建全文索引
CREATE FULLTEXT INDEX idx_article_title ON Article(title) USING inversearch;
CREATE FULLTEXT INDEX idx_article_content ON Article(content) USING bm25;
CREATE FULLTEXT INDEX idx_author_name ON Author(name) USING inversearch;

-- 4. 插入数据
INSERT VERTEX Author(id, name, bio, expertise) 
VALUES ("a1", "张三", "人工智能专家，专注于机器学习研究", "AI,ML,DL");

INSERT VERTEX Article(id, title, content, author_id, created_at)
VALUES ("art1", "深度学习入门", "深度学习是机器学习的一个分支...", "a1", now());

INSERT EDGE WRITTEN_BY() VALUES "art1" -> "a1";

-- 5. 搜索
MATCH (a:Article)-[:WRITTEN_BY]->(auth:Author)
WHERE a.content MATCH "深度学习" AND auth.expertise MATCH "人工智能"
RETURN a.title, auth.name, score(a) as relevance;
```

### A.2 性能测试基准

```sql
-- 测试数据集：100 万篇文章，10 万作者

-- 测试 1：纯全文搜索
MATCH (a:Article) WHERE a.content MATCH "关键词" RETURN a LIMIT 100;
-- 预期：< 50ms

-- 测试 2：全文搜索 + 属性过滤
MATCH (a:Article) 
WHERE a.content MATCH "关键词" AND a.created_at > now() - 86400000
RETURN a LIMIT 100;
-- 预期：< 100ms

-- 测试 3：全文搜索 + 图遍历
MATCH (a:Article)-[:WRITTEN_BY]->(auth:Author)
WHERE a.content MATCH "关键词" AND auth.name MATCH "张"
RETURN a.title, auth.name LIMIT 100;
-- 预期：< 200ms

-- 测试 4：多跳全文搜索
MATCH (a1:Article)-[:CITED]->(a2:Article)
WHERE a1.content MATCH "机器学习" AND a2.content MATCH "深度学习"
RETURN a1.title, a2.title LIMIT 100;
-- 预期：< 500ms
```
