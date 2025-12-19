# 编译错误分类文档

## 1. 未解析导入错误（E0432）

### 1.1 query/executor/data_processing模块相关
- `query/executor/factory.rs`: 导入`data_processing::filter`, `data_processing::pagination`, `data_processing::sort`, `data_processing::aggregation`失败
- 需要检查这些模块是否存在以及正确导出

### 1.2 query/validator模块相关
- `query/planner/match_planning/core/cypher_clause_planner.rs`: 导入`validator::common_structs`失败
- `query/planner/match_planning/paths/match_path_planner.rs`: 导入`validator::Column`, `validator::Variable`失败
- `query/planner/match_planning/paths/shortest_path_planner.rs`: 导入`validator::Column`, `validator::Variable`失败
- `query/planner/match_planning/clauses/clause_planner.rs`: 导入`validator::common_structs`失败
- `query/planner/match_planning/clauses/where_clause_planner.rs`: 导入`validator::common_structs`失败
- `query/planner/match_planning/clauses/return_clause_planner.rs`: 导入`validator::common_structs`失败
- `query/planner/match_planning/clauses/order_by_planner.rs`: 导入`validator::common_structs`失败
- `query/planner/match_planning/clauses/pagination_planner.rs`: 导入`validator::common_structs`失败
- `query/planner/match_planning/clauses/yield_planner.rs`: 导入`validator::common_structs`失败
- `query/visitor/deduce_type_visitor.rs`: 导入`validator::ValidateContext`失败

### 1.3 query/context模块相关
- `query/query_pipeline_manager.rs`: 导入`context::RequestContext`失败
- `query/executor/cypher/context.rs`: 导入`context::ast_context::CypherAstContext`失败
- `query/planner/match_planning/utils/connection_builder.rs`: 导入`context::ast_context::base`失败
- `query/planner/match_planning/utils/connection_strategy.rs`: 导入`context::ast_context::base`失败
- `query/planner/ngql/fetch_edges_planner.rs`: 导入`context::ast_context::FetchEdgesContext`失败
- `query/planner/ngql/fetch_vertices_planner.rs`: 导入`context::ast_context::FetchVerticesContext`失败
- `query/planner/ngql/go_planner.rs`: 导入`context::ast_context::GoContext`失败
- `query/planner/ngql/lookup_planner.rs`: 导入`context::ast_context::LookupContext`失败
- `query/planner/ngql/maintain_planner.rs`: 导入`context::ast_context::MaintainContext`失败
- `query/planner/ngql/path_planner.rs`: 导入`context::ast_context::PathContext`失败
- `query/planner/ngql/subgraph_planner.rs`: 导入`context::ast_context::SubgraphContext`失败

### 1.4 其他导入错误
- `query/executor/mod.rs`: 导入`result_processing::ResultProcessorFactory`失败

## 2. 未知字段错误（E0609）

### 2.1 base_validator::NodeInfo字段缺失
- `query/planner/match_planning/paths/match_path_planner.rs`: 访问`.tids`, `.props`, `.filter`字段失败
- `query/planner/match_planning/paths/shortest_path_planner.rs`: 访问`.tids`, `.props`, `.filter`字段失败
- `query/planner/match_planning/seeks/scan_seek.rs`: 访问`.filter`, `.props`字段失败
- `query/planner/match_planning/seeks/index_seek.rs`: 访问`.props`, `.filter`, `.tids`字段失败
- `query/planner/match_planning/utils/finder.rs`: 访问`.filter`, `.props`字段失败

### 2.2 base_validator::EdgeInfo字段缺失
- `query/planner/match_planning/paths/match_path_planner.rs`: 访问`.edge_types`, `.anonymous`, `.filter`, `.range`字段失败
- `query/planner/match_planning/paths/shortest_path_planner.rs`: 访问`.edge_types`, `.anonymous`字段失败

### 2.3 上下文结构体字段缺失
- `query/planner/match_planning/clauses/with_clause_planner.rs`: 访问`YieldClauseContext`的`.yield_columns`字段失败
- `query/planner/match_planning/clauses/order_by_planner.rs`: 访问`OrderByClauseContext`的`.indexed_order_factors`字段失败

## 3. 无法找到方法错误（E0599）

### 3.1 ExpressionContext的set_variable方法缺失
- 多个文件中出现此错误，包括：
  - `query/executor/data_processing/transformations/append_vertices.rs`
  - `query/executor/data_processing/loops.rs`
  - `query/executor/result_processing/aggregation.rs`
  - `query/executor/data_processing/transformations/assign.rs`
  - `query/executor/data_processing/transformations/unwind.rs`
  - `query/executor/data_access.rs`
  - `query/executor/result_processing/projection.rs`
  - `query/executor/result_processing/sort.rs`
  - `query/executor/result_processing/filter.rs`
  - `query/executor/result_processing/topn.rs`
  - `query/executor/tag_filter.rs`
  - 以及其他相关文件

### 3.2 QueryContext方法缺失
- `query/optimizer/optimizer.rs`: `clone`方法缺失于`&mut QueryContext`
- `query/query_pipeline_manager.rs`: `with_request_context`关联函数缺失
- `query/query_pipeline_manager.rs`: `gen_id`方法缺失于`&mut QueryContext`

### 3.3 AstContext方法缺失
- `query/planner/planner.rs`: `statement_type`方法缺失
- `query/planner/go_planner.rs`: `statement_type`方法缺失
- `query/planner/lookup_planner.rs`: `statement_type`方法缺失
- `query/planner/path_planner.rs`: `statement_type`方法缺失
- `query/planner/subgraph_planner.rs`: `statement_type`方法缺失
- `query/planner/match_planning/match_planner.rs`: `statement_type`方法缺失
- `query/planner/ngql/fetch_edges_planner.rs`: `statement_type`方法缺失
- `query/planner/ngql/fetch_vertices_planner.rs`: `statement_type`方法缺失
- `query/planner/ngql/go_planner.rs`: `statement_type`方法缺失
- `query/planner/ngql/lookup_planner.rs`: `statement_type`方法缺失
- `query/planner/ngql/maintain_planner.rs`: `statement_type`方法缺失
- `query/planner/ngql/path_planner.rs`: `statement_type`方法缺失
- `query/planner/ngql/subgraph_planner.rs`: `statement_type`方法缺失

### 3.4 CypherClauseContext方法缺失
- `query/planner/match_planning/core/match_clause_planner.rs`: `kind`方法缺失
- `query/planner/match_planning/clauses/with_clause_planner.rs`: `kind`方法缺失
- `query/planner/match_planning/clauses/unwind_planner.rs`: `kind`方法缺失
- `query/planner/match_planning/match_planner.rs`: `kind`方法缺失

## 4. 类型不匹配错误（E0308）

### 4.1 Option类型相关
- `query/query_pipeline_manager.rs`: 期望`String`，得到`Option<String>`
- `query/optimizer/elimination_rules.rs`: 比较`String`与`Option<String>`

### 4.2 类型转换问题
- `query/context/expression_context.rs`: 期望`Edge`，得到`Box<Edge>`
- `query/context/expression_context.rs`: 期望`&Value`，得到`Value`

## 5. 泛型参数缺失错误（E0107）

### 5.1 Result类型缺少泛型参数
- `query/context/execution_context.rs`: `std::result::Result`缺少2个泛型参数，出现在多行

## 6. 实现/特征相关错误（E0782, E0310）

### 6.1 类型vs特征错误
- `query/query_pipeline_manager.rs`: 期望类型，得到特征
- `query/executor/result_processing/aggregation.rs`: 生命周期问题

## 7. 迭代器构建错误（E0277）

### 7.1 Vec构建错误
- `query/optimizer/elimination_rules.rs`: 无法从`Option<String>`迭代器构建`Vec<String>`
- `query/planner/plan/core/nodes/project_node.rs`: 无法从`Option<String>`迭代器构建`Vec<String>`

## 8. 移动/借用错误（E0382, E0502, E0507）

### 8.1 借用冲突
- `query/executor/result_processing/aggregation.rs`: 不可变借用与可变借用冲突
- `query/planner/match_planning/paths/match_path_planner.rs`: 部分移动后借用
- `query/planner/match_planning/clauses/clause_planner.rs`: 无法从共享引用中移出值

## 9. 参数错误（E0061）

### 9.1 函数参数数量不匹配
- `query/executor/result_processing/projection.rs`: 函数期望2个参数，提供1个参数（多处出现）
- `query/executor/result_processing/filter.rs`: 函数期望2个参数，提供1个参数（多处出现）

## 10. 类型注解缺失错误（E0282）

### 10.1 Option类型参数需要注解
- `query/planner/ngql/go_planner.rs`: Option类型需要类型注解
- `query/planner/ngql/lookup_planner.rs`: Option类型需要类型注解
- `query/planner/ngql/path_planner.rs`: Option类型需要类型注解
- `query/planner/ngql/subgraph_planner.rs`: Option类型需要类型注解

## 11. 类型不匹配错误（E0308）

### 11.1 usize与Expression类型不匹配
- `query/planner/match_planning/core/match_clause_planner.rs`: 期望`usize`，得到`Expression`

### 11.2 Option<usize>与整数类型不匹配
- `query/planner/match_planning/clauses/projection_planner.rs`: 期望`Option<usize>`，得到整数（多处出现）