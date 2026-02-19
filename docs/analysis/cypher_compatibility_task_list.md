# GraphDB Cypher 兼容性实施任务清单

> 基于代码分析的具体修改点，保持最简，无时间安排

---

## 一、Parser 层修改

### 1.1 Token 定义扩展
**文件**: `src/query/parser/core/token.rs`
- 添加 `Create` Token 处理（当前 CREATE 用于 DDL，需区分 CREATE 节点/边）
- 添加 `IfNotExists` Token（已存在，需复用）
- 确认 `LParen`, `RParen`, `Colon`, `LBrace`, `RBrace` 等符号 Token 已定义

### 1.2 AST 扩展
**文件**: `src/query/parser/ast/stmt.rs`
- 在 `CreateStmt` 结构体中添加变体：
  ```rust
  pub enum CreateTarget {
      Schema(CreateSchemaTarget),  // 现有：CREATE TAG/EDGE/SPACE
      Data(CreateDataTarget),      // 新增：CREATE (n:Label {props})
  }
  
  pub struct CreateDataTarget {
      pub patterns: Vec<Pattern>,  // 节点+边模式列表
  }
  ```
- 在 `Pattern` 枚举中确认支持 `NodePattern` 和 `EdgePattern`（已存在）

### 1.3 DML Parser 扩展
**文件**: `src/query/parser/parser/dml_parser.rs`
- 在 `DmlParser` 结构体中添加方法：
  ```rust
  pub fn parse_create_data_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError>
  ```
- 实现 `parse_node_pattern()` 方法解析 `(var:Label {props})`
- 实现 `parse_edge_pattern()` 方法解析 `-[var:Type {props}]->`
- 实现 `parse_path_pattern()` 方法解析完整路径模式
- 在 `parse_properties()` 中解析 `{key: value, ...}` 属性映射

### 1.4 Statement Parser 路由修改
**文件**: `src/query/parser/parser/stmt_parser.rs`
- 在 `parse_single_statement()` 的 `TokenKind::Create` 分支中：
  - 检查下一个 Token 是 `Vertex`/`Edge`/`Tag` 还是 `LParen`
  - 如果是 `LParen`，调用 `DmlParser::parse_create_data_statement()`
  - 否则保持现有 DDL 解析逻辑

### 1.5 类型推断模块
**文件**: `src/query/parser/parser/expr_parser.rs`（新增方法）
- 添加 `infer_type_from_literal()` 方法：
  ```rust
  fn infer_type_from_literal(&self, expr: &Expression) -> DataType
  ```
- 实现字符串 -> STRING、整数 -> INT、浮点数 -> FLOAT、布尔 -> BOOL 的映射

---

## 二、Validator 层修改

### 2.1 Schema 自动创建验证器
**文件**: `src/query/validator/schema_validator.rs`
- 添加方法：
  ```rust
  pub fn ensure_tag_exists_or_create(&self, space: &str, tag: &str, props: &[(String, DataType)]) -> DBResult<()>
  pub fn ensure_edge_type_exists_or_create(&self, space: &str, edge: &str, props: &[(String, DataType)]) -> DBResult<()>
  ```
- 在方法中检查 TAG/EDGE 是否存在，不存在则调用 SchemaManager 创建

### 2.2 CREATE 语句验证器
**文件**: `src/query/validator/`（新建 `create_validator.rs`）
- 创建 `CreateValidator` 结构体
- 实现 `validate()` 方法：
  - 遍历所有模式中的节点，提取 Label 和属性
  - 遍历所有模式中的边，提取 Type 和属性
  - 调用 `schema_validator.ensure_tag_exists_or_create()`
  - 调用 `schema_validator.ensure_edge_type_exists_or_create()`
- 在 `validation_factory.rs` 中注册验证器

### 2.3 变量绑定验证
**文件**: `src/query/validator/base_validator.rs`
- 添加变量作用域管理：
  ```rust
  pub struct VariableScope {
      variables: HashMap<String, VariableInfo>,
  }
  
  pub struct VariableInfo {
      pub name: String,
      pub vid: Expression,  // 节点ID表达式
      pub labels: Vec<String>,
  }
  ```
- 在验证 CREATE 语句时，将变量名和 VID 存入作用域
- 在同一会话的后续语句中，允许引用已定义的变量

---

## 三、Planner 层修改

### 3.1 CREATE 语句规划器
**文件**: `src/query/planner/statements/`（新建 `create_planner.rs`）
- 创建 `CreatePlanner` 结构体，实现 `Planner` trait
- 实现 `plan()` 方法：
  ```rust
  fn plan(&self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError>
  ```
- 在方法中：
  - 提取 `CreateDataTarget` 中的模式列表
  - 为每个节点模式创建 `InsertVerticesNode`
  - 为每个边模式创建 `InsertEdgesNode`
  - 使用 `ArgumentNode` 连接节点插入和边插入（处理依赖关系）

### 3.2 Planner 工厂注册
**文件**: `src/query/planner/planner.rs`
- 在 `get_planner_for_ast()` 方法中添加：
  ```rust
  Stmt::Create(create_stmt) => {
      if matches!(create_stmt.target, CreateTarget::Data(_)) {
          CreatePlanner::make()
      } else {
          // 现有 DDL 规划器
      }
  }
  ```

### 3.3 插入节点扩展
**文件**: `src/query/planner/plan/core/nodes/insert_nodes.rs`
- 确认 `InsertVerticesNode` 和 `InsertEdgesNode` 支持批量插入
- 如有必要，添加 `if_not_exists` 字段支持

---

## 四、Executor 层修改

### 4.1 Schema 管理器扩展
**文件**: `src/storage/metadata/schema_manager.rs`（或等效文件）
- 添加方法：
  ```rust
  fn create_tag_if_not_exists(&self, space: &str, tag: &str, props: Vec<PropertyDef>) -> DBResult<()>
  fn create_edge_type_if_not_exists(&self, space: &str, edge: &str, props: Vec<PropertyDef>) -> DBResult<()>
  ```
- 方法内部使用事务确保原子性

### 4.2 执行器上下文
**文件**: `src/query/executor/`（确认上下文实现）
- 确保执行器上下文支持变量表存储
- 在节点插入后，将变量名 -> VID 映射存入上下文

---

## 五、测试文件

### 5.1 Parser 测试
**文件**: `src/query/parser/parser/tests.rs`
- 添加测试用例：
  - `test_parse_create_node_simple()` - 解析 `CREATE (:Person {name: 'Alice'})`
  - `test_parse_create_node_with_var()` - 解析 `CREATE (p:Person {name: 'Alice'})`
  - `test_parse_create_path()` - 解析 `CREATE (a)-[:FRIEND]->(b)`
  - `test_parse_create_full_path()` - 解析 `CREATE (a:Person)-[:FRIEND {since: 2020}]->(b:Person)`

### 5.2 集成测试
**目录**: `tests/`（新建 `cypher_compatibility_test.rs`）
- 测试完整流程：
  - 首次 CREATE 自动创建 Schema
  - 重复 CREATE 使用已有 Schema
  - 类型不匹配报错

---

## 六、文档更新

### 6.1 语法文档
**文件**: `docs/release/02_dml_data_manipulation.md`
- 在 "6. MERGE" 节后添加 "7. CREATE (Cypher风格)" 章节
- 包含语法说明、示例、与原生语法的对比

### 6.2 兼容性文档
**文件**: `docs/analysis/cypher_compatibility_analysis.md`
- 更新实施状态章节
- 标记已完成的功能

---

## 七、关键代码修改点汇总

| 模块 | 文件路径 | 修改类型 | 具体修改 |
|------|---------|---------|---------|
| Parser | `src/query/parser/ast/stmt.rs` | 修改 | 扩展 `CreateTarget` 枚举 |
| Parser | `src/query/parser/parser/dml_parser.rs` | 新增 | 添加 `parse_create_data_statement()` 等方法 |
| Parser | `src/query/parser/parser/stmt_parser.rs` | 修改 | 路由 CREATE 语句到 DML Parser |
| Parser | `src/query/parser/parser/expr_parser.rs` | 新增 | 添加类型推断方法 |
| Validator | `src/query/validator/create_validator.rs` | 新建 | 实现 CREATE 语句验证 |
| Validator | `src/query/validator/schema_validator.rs` | 修改 | 添加 Schema 自动创建方法 |
| Validator | `src/query/validator/validation_factory.rs` | 修改 | 注册 CREATE 验证器 |
| Planner | `src/query/planner/statements/create_planner.rs` | 新建 | 实现 CREATE 语句规划 |
| Planner | `src/query/planner/planner.rs` | 修改 | 注册 CREATE 规划器 |
| Storage | `src/storage/metadata/schema_manager.rs` | 修改 | 添加 `create_tag_if_not_exists()` 等方法 |
| Test | `src/query/parser/parser/tests.rs` | 新增 | 添加 Parser 测试用例 |
| Test | `tests/cypher_compatibility_test.rs` | 新建 | 添加集成测试 |
| Doc | `docs/release/02_dml_data_manipulation.md` | 修改 | 添加 CREATE 语法文档 |

---

*任务清单结束*
