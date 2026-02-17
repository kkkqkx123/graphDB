# GraphDB DDL 功能增强方案

## 1. 功能必要性分析

### 1.1 TTL (Time To Live) 分析

**结论：TTL 对单节点数据库并非多余，但优先级较低**

| 场景 | 必要性 | 说明 |
|------|--------|------|
| 会话/缓存数据 | **高** | 用户登录会话、临时缓存数据需要自动过期 |
| 日志/审计数据 | **中** | 日志数据通常有保留期限，需要自动清理 |
| 业务数据 | **低** | 业务数据通常长期保留，删除需求较少 |

**单节点场景下的 TTL 价值：**
1. **自动化数据清理**：避免手动删除过期数据
2. **存储空间管理**：防止历史数据无限增长
3. **与分布式无关**：TTL 是数据生命周期管理功能，与是否分布式无关

**建议：** 保留 TTL 设计，但可作为第二阶段实现

### 1.2 功能优先级最终划分

| 优先级 | 功能 | 必要性 | 实现复杂度 |
|--------|------|--------|-----------|
| **P0 (必须)** | 默认值 (DEFAULT) | 高 | 低 |
| **P0 (必须)** | NOT NULL 约束 | 高 | 低 |
| **P1 (重要)** | SHOW CREATE 语句 | 高 | 中 |
| **P2 (建议)** | TTL 支持 | 中 | 高 |
| **P3 (可选)** | 属性 COMMENT | 低 | 低 |

---

## 2. 最终修改方案

### 2.1 阶段一：数据完整性功能（P0）

#### 2.1.1 默认值 (DEFAULT) 支持

**目标语法：**
```cypher
CREATE TAG person(
    name: STRING,
    age: INT DEFAULT 18,
    status: STRING DEFAULT "active"
)
```

**修改文件清单：**

1. **src/query/parser/core/token.rs**
   - 添加 `Default` Token（已存在，确认可用）

2. **src/query/parser/parser/ddl_parser.rs**
   - 修改 `parse_property_defs` 方法，添加 DEFAULT 解析逻辑
   - 添加 `parse_default_value` 辅助方法

3. **src/storage/schema_manager.rs**
   - 在插入数据时自动填充默认值

**实现代码示例：**

```rust
// ddl_parser.rs 修改
pub fn parse_property_defs(&mut self, ctx: &mut ParseContext) -> Result<Vec<PropertyDef>, ParseError> {
    let mut defs = Vec::new();
    if ctx.match_token(TokenKind::LParen) {
        while !ctx.match_token(TokenKind::RParen) {
            let name = ctx.expect_identifier()?;
            ctx.expect_token(TokenKind::Colon)?;
            let dtype = self.parse_data_type(ctx)?;
            
            // 解析可选的默认值
            let mut default = None;
            if ctx.match_token(TokenKind::Default) {
                default = Some(self.parse_value_literal(ctx)?);
            }
            
            defs.push(PropertyDef {
                name,
                data_type: dtype,
                nullable: true,
                default,
                comment: None,
            });
            if !ctx.match_token(TokenKind::Comma) {
                break;
            }
        }
    }
    Ok(defs)
}
```

#### 2.1.2 NOT NULL 约束支持

**目标语法：**
```cypher
CREATE TAG person(
    name: STRING NOT NULL,
    email: STRING,
    age: INT NOT NULL DEFAULT 0
)
```

**修改文件清单：**

1. **src/query/parser/core/token.rs**
   - 添加 `Nullable` Token（可选，也可复用现有的 `Null`）

2. **src/query/parser/parser/ddl_parser.rs**
   - 修改 `parse_property_defs` 方法，添加 NULL/NOT NULL 解析

3. **src/storage/schema_manager.rs**
   - 在插入/更新数据时进行 NOT NULL 校验

**实现代码示例：**

```rust
// ddl_parser.rs 修改
// 解析 NULL / NOT NULL
let mut nullable = true;
if ctx.check_token(TokenKind::Not) {
    // 向前查看是否是 NOT NULL
    ctx.next_token(); // 消费 NOT
    if ctx.check_token(TokenKind::Null) {
        ctx.next_token(); // 消费 NULL
        nullable = false;
    }
}
```

#### 2.1.3 数据校验逻辑

在存储层实现默认值填充和 NOT NULL 校验：

```rust
// src/storage/schema_manager.rs
impl SchemaManager {
    /// 根据 Schema 定义填充默认值并校验
    pub fn fill_defaults_and_validate(
        &self,
        tag_name: &str,
        properties: &mut HashMap<String, Value>
    ) -> Result<(), StorageError> {
        let tag_info = self.get_tag_info(tag_name)?;
        
        for prop_def in &tag_info.properties {
            match properties.get(&prop_def.name) {
                None | Some(Value::Null) => {
                    // 属性缺失或为 NULL
                    if !prop_def.nullable && prop_def.default.is_none() {
                        return Err(StorageError::ValidationError(
                            format!("属性 '{}' 不能为 NULL", prop_def.name)
                        ));
                    }
                    // 填充默认值
                    if let Some(ref default) = prop_def.default {
                        properties.insert(prop_def.name.clone(), default.clone());
                    }
                }
                Some(_) => {
                    // 属性有值，无需处理
                }
            }
        }
        
        Ok(())
    }
}
```

---

### 2.2 阶段二：开发调试功能（P1）

#### 2.2.1 SHOW CREATE 语句

**目标语法：**
```cypher
SHOW CREATE SPACE test_space
SHOW CREATE TAG person
SHOW CREATE EDGE follow
SHOW CREATE INDEX idx_person_name
```

**修改文件清单：**

1. **src/query/parser/core/token.rs**
   - 添加 `ShowCreate` Token

2. **src/query/parser/ast/stmt.rs**
   - 添加 `ShowCreateStmt` 结构体和 `ShowCreateTarget` 枚举
   - 在 `Stmt` 枚举中添加 `ShowCreate` 变体

3. **src/query/parser/parser/ddl_parser.rs**
   - 添加 `parse_show_create_statement` 方法

4. **src/query/executor/show_executor.rs**（新增或修改）
   - 实现 SHOW CREATE 的执行逻辑

**AST 定义：**

```rust
// stmt.rs
#[derive(Debug, Clone, PartialEq)]
pub struct ShowCreateStmt {
    pub span: Span,
    pub target: ShowCreateTarget,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShowCreateTarget {
    Space(String),
    Tag(String),
    Edge(String),
    Index(String),
}
```

**执行器实现：**

```rust
// show_executor.rs
impl ShowCreateExecutor {
    pub fn execute_show_create_tag(&self, tag_name: &str) -> Result<DataSet, ExecutorError> {
        let tag_info = self.schema_manager.get_tag_info(tag_name)?;
        
        // 生成 CREATE TAG 语句字符串
        let create_stmt = self.generate_create_tag_statement(&tag_info);
        
        let mut data_set = DataSet::new();
        data_set.add_column("Tag");
        data_set.add_column("Create Tag");
        data_set.add_row(vec![
            Value::String(tag_name.to_string()),
            Value::String(create_stmt)
        ]);
        
        Ok(data_set)
    }
    
    fn generate_create_tag_statement(&self, tag_info: &TagInfo) -> String {
        let mut stmt = format!("CREATE TAG `{}` (", tag_info.tag_name);
        
        let props: Vec<String> = tag_info.properties.iter()
            .map(|p| {
                let mut prop_str = format!("`{}`: {:?}", p.name, p.data_type);
                if !p.nullable {
                    prop_str.push_str(" NOT NULL");
                }
                if let Some(ref default) = p.default {
                    prop_str.push_str(&format!(" DEFAULT {}", default));
                }
                prop_str
            })
            .collect();
        
        stmt.push_str(&props.join(", "));
        stmt.push_str(")");
        stmt
    }
}
```

---

### 2.3 阶段三：数据生命周期管理（P2，可选）

#### 2.3.1 TTL 支持

**目标语法：**
```cypher
CREATE TAG session(
    token: STRING NOT NULL,
    created_at: TIMESTAMP DEFAULT 0
) TTL_DURATION = 86400 TTL_COL = created_at
```

**修改文件清单：**

1. **src/core/types/metadata.rs**
   - 添加 `TtlConfig` 结构体
   - 修改 `TagInfo` 和 `EdgeTypeInfo` 添加 `ttl_config` 字段

2. **src/query/parser/core/token.rs**
   - 添加 `TtlDuration`, `TtlCol` Token

3. **src/query/parser/ast/stmt.rs**
   - 修改 `CreateTarget` 添加 `ttl_config` 字段

4. **src/query/parser/parser/ddl_parser.rs**
   - 修改 CREATE TAG/EDGE 解析逻辑，添加 TTL 解析

5. **src/storage/ttl_manager.rs**（新增）
   - 实现 TTL 清理后台任务

**TTL 配置结构：**

```rust
// metadata.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct TtlConfig {
    /// TTL 持续时间（秒）
    pub duration: i64,
    /// 用于 TTL 计算的属性名
    pub col: String,
}

impl TtlConfig {
    pub fn new(duration: i64, col: String) -> Self {
        Self { duration, col }
    }
    
    pub fn is_enabled(&self) -> bool {
        self.duration > 0 && !self.col.is_empty()
    }
    
    /// 计算过期时间
    pub fn calculate_expire_time(&self, property_value: i64) -> i64 {
        property_value + self.duration
    }
}
```

**TTL 管理器：**

```rust
// ttl_manager.rs
pub struct TtlManager {
    storage: Arc<StorageEngine>,
    check_interval: Duration,
}

impl TtlManager {
    pub async fn run_cleanup_task(&self) {
        let mut interval = tokio::time::interval(self.check_interval);
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.cleanup_expired_data().await {
                error!("TTL 清理任务失败: {}", e);
            }
        }
    }
    
    async fn cleanup_expired_data(&self) -> Result<(), StorageError> {
        let now = chrono::Utc::now().timestamp();
        
        // 获取所有带 TTL 的 Tag
        let ttl_tags = self.storage.get_ttl_tags()?;
        
        for tag_info in ttl_tags {
            if let Some(ref ttl) = tag_info.ttl_config {
                // 查找过期数据并删除
                let expired_vertices = self.storage
                    .find_vertices_by_ttl(&tag_info.tag_name, &ttl.col, now)?;
                
                for vertex_id in expired_vertices {
                    self.storage.delete_vertex(&vertex_id)?;
                }
            }
        }
        
        Ok(())
    }
}
```

---

### 2.4 阶段四：文档化功能（P3，可选）

#### 2.4.1 属性 COMMENT 支持

**目标语法：**
```cypher
CREATE TAG person(
    name: STRING NOT NULL COMMENT "用户姓名",
    email: STRING COMMENT "邮箱地址"
)
```

**说明：** `PropertyDef` 结构已包含 `comment` 字段，只需在解析器中解析即可。

---

## 3. 修改文件汇总

### 阶段一（必须实现）

| 文件 | 修改类型 | 修改内容 |
|------|---------|---------|
| `src/query/parser/parser/ddl_parser.rs` | 修改 | 添加 DEFAULT 和 NOT NULL 解析逻辑 |
| `src/storage/schema_manager.rs` | 修改 | 添加默认值填充和 NOT NULL 校验 |

### 阶段二（重要）

| 文件 | 修改类型 | 修改内容 |
|------|---------|---------|
| `src/query/parser/core/token.rs` | 修改 | 添加 ShowCreate Token |
| `src/query/parser/ast/stmt.rs` | 修改 | 添加 ShowCreateStmt 和 ShowCreateTarget |
| `src/query/parser/parser/ddl_parser.rs` | 修改 | 添加 SHOW CREATE 解析 |
| `src/query/executor/show_executor.rs` | 新增 | 实现 SHOW CREATE 执行逻辑 |

### 阶段三（可选）

| 文件 | 修改类型 | 修改内容 |
|------|---------|---------|
| `src/core/types/metadata.rs` | 修改 | 添加 TtlConfig，修改 TagInfo/EdgeTypeInfo |
| `src/query/parser/core/token.rs` | 修改 | 添加 TtlDuration, TtlCol Token |
| `src/query/parser/ast/stmt.rs` | 修改 | 修改 CreateTarget 添加 ttl_config |
| `src/query/parser/parser/ddl_parser.rs` | 修改 | 添加 TTL 解析 |
| `src/storage/ttl_manager.rs` | 新增 | TTL 清理任务管理器 |

---

## 4. 实施建议

### 4.1 开发顺序

1. **第一周**：实现阶段一（DEFAULT + NOT NULL）
   - 修改解析器
   - 修改存储层校验逻辑
   - 编写单元测试

2. **第二周**：实现阶段二（SHOW CREATE）
   - 添加 AST 节点
   - 实现解析器
   - 实现执行器

3. **第三周及以后**：阶段三（TTL）
   - 设计 TTL 存储格式
   - 实现 TTL 管理器
   - 性能测试

### 4.2 兼容性考虑

- 所有新增功能均为可选语法，不影响现有语句
- 默认值和 NOT NULL 约束在存储层校验，不修改存储格式
- TTL 需要存储额外元数据，需考虑版本兼容性

### 4.3 测试策略

| 功能 | 测试类型 | 测试内容 |
|------|---------|---------|
| DEFAULT | 单元测试 | 解析、存储、查询 |
| NOT NULL | 单元测试 | 约束校验、错误处理 |
| SHOW CREATE | 集成测试 | 语句生成准确性 |
| TTL | 集成测试 | 过期清理正确性 |

---

## 5. 总结

本方案针对单节点图数据库场景，优先实现数据完整性功能（DEFAULT、NOT NULL）和开发调试功能（SHOW CREATE），将 TTL 作为可选功能后续实现。这样既能满足核心需求，又保持系统的简洁性。
