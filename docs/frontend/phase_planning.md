# GraphDB 前端阶段规划

**文档版本**: v2.0
**创建日期**: 2026-03-28
**最后更新**: 2026-03-28

---

## 阶段划分原则

基于 GraphDB 项目的实际需求和开发复杂度，将前端开发划分为 7 个阶段，每个阶段专注于一个核心功能模块，确保：

1. **渐进式开发**: 每个阶段都有明确的交付物和验收标准
2. **功能内聚**: 每个阶段的功能高度相关，减少上下文切换
3. **可独立交付**: 每个阶段完成后都可以独立测试和使用
4. **简化用户故事**: 纯数据操作功能整合为整体用户故事，避免过于细碎

---

## 阶段概览

| 阶段 | 名称 | 核心功能 | 预计工期 | 依赖 |
|-------|------|---------|------|
| 阶段 1 | 基础框架和连接管理 | 1 周 | 无 |
| 阶段 2 | 查询控制台 | 2 周 | 阶段 1 |
| 阶段 3 | Schema 管理 - Space | 1 周 | 阶段 1 |
| 阶段 4 | Schema 管理 - Tag/Edge | 2 周 | 阶段 3 |
| 阶段 5 | Schema 管理 - 索引 | 1 周 | 阶段 4 |
| 阶段 6 | 图可视化 | 2 周 | 阶段 2 |
| 阶段 7 | 数据浏览 | 1.5 周 | 阶段 3 |

---

## 阶段 1: 基础框架和连接管理

### 目标

搭建前端项目基础架构，实现数据库连接管理功能，为后续功能开发奠定基础。

### 核心功能

1. **项目初始化**
   - React + TypeScript + Vite 项目搭建
   - Ant Design 组件库集成
   - 基础路由配置
   - 全局样式和主题配置

2. **连接管理**
   - 连接配置界面
   - 数据库连接和认证
   - 连接状态管理
   - 连接信息持久化

3. **基础布局**
   - 应用头部（Logo、导航、用户信息）
   - 侧边栏导航
   - 登录页面

### 用户故事

**US-P1-001**: 作为数据库开发者，我想要配置数据库连接信息，以便能够连接到我的 GraphDB 实例。

**US-P1-002**: 作为数据库开发者，我想要看到当前的连接状态，以便了解数据库是否可用。

**US-P1-003**: 作为数据库开发者，我想要断开当前的数据库连接，以便释放资源或切换到其他数据库。

**US-P1-004**: 作为数据库开发者，我想要保存我的连接配置，以便下次打开应用时自动填充连接信息。

### 验收标准

**AC-P1-001**: The system shall provide a React + TypeScript + Vite project structure.

**AC-P1-002**: The system shall integrate Ant Design 5.x component library.

**AC-P1-003**: The system shall configure React Router v6 for navigation.

**AC-P1-004**: The system shall provide a login page with connection form.

**AC-P1-005**: The system shall validate connection configuration before attempting to connect.

**AC-P1-006**: When user submits valid connection credentials, the system shall establish a connection to GraphDB instance.

**AC-P1-007**: When connection is successful, the system shall display a success message and navigate to the main dashboard.

**AC-P1-008**: When connection fails, the system shall display an error message with details about the failure.

**AC-P1-009**: The system shall display current connection status (connected/disconnected) in the application header.

**AC-P1-010**: Where user is connected, the system shall display the connected database host and username.

**AC-P1-011**: When user clicks the disconnect button, the system shall terminate the database connection.

**AC-P1-012**: When connection is terminated, the system shall navigate to the login page.

**AC-P1-013**: The system shall store connection configuration in localStorage after a successful connection.

**AC-P1-014**: When the application loads, the system shall pre-fill the connection form with stored credentials if available.

**AC-P1-015**: The system shall mask the password field during input and display.

**AC-P1-016**: The system shall provide a "Remember me" option to control whether credentials are persisted.

**AC-P1-017**: The system shall provide a responsive layout that works on desktop (1366x768+) and mobile (375px+).

### 交付物

- [ ] 完整的项目代码
- [ ] 连接管理功能
- [ ] 基础布局和导航
- [ ] 单元测试（覆盖率 > 70%）
- [ ] 用户文档

---

## 阶段 2: 查询控制台

### 目标

提供完整的 Cypher 查询编辑和执行环境，支持查询结果展示、历史记录和收藏功能。

### 核心功能

1. **查询编辑器**
   - Monaco Editor 集成
   - Cypher 语法高亮
   - 自动补全（关键字、Tag、Edge、属性）
   - 快捷键支持（Shift+Enter 执行）

2. **查询执行**
   - 单条查询执行
   - 批量查询执行
   - 查询进度显示
   - 错误信息展示

3. **结果展示**
   - 表格形式展示
   - JSON 格式展示
   - 结果导出（CSV、JSON）
   - 执行时间显示

4. **查询管理**
   - 查询历史记录（localStorage）
   - 查询收藏功能
   - 历史记录清理

### 用户故事

**US-P2-001**: 作为数据库开发者，我想要在一个代码编辑器中编写 Cypher 查询，以便能够方便地编辑和调试查询语句。

**US-P2-002**: 作为数据库开发者，我想要看到 Cypher 语法的高亮显示，以便更容易识别查询语句的结构。

**US-P2-003**: 作为数据库开发者，我想要获得自动补全建议，以便快速输入关键字、标签、边类型和属性名。

**US-P2-004**: 作为数据库开发者，我想要通过快捷键（Shift+Enter）执行查询，以便提高查询效率。

**US-P2-005**: 作为数据库开发者，我想要以表格形式查看查询结果，以便快速浏览数据。

**US-P2-006**: 作为数据库开发者，我想要以 JSON 格式查看查询结果，以便复制和集成到其他工具中。

**US-P2-007**: 作为数据库开发者，我想要看到查询执行过程中的错误信息，以便快速定位和修复问题。

**US-P2-008**: 作为数据库开发者，我想要查看我的查询历史记录，以便重新执行之前的查询。

**US-P2-009**: 作为数据库开发者，我想要收藏常用的查询语句，以便快速执行重复性查询。

**US-P2-010**: 作为数据库开发者，我想要导出查询结果为 CSV 或 JSON 文件，以便在本地保存和分析数据。

### 验收标准

**AC-P2-001**: The system shall provide a code editor with Monaco Editor for writing Cypher queries.

**AC-P2-002**: The system shall support Cypher syntax highlighting in the code editor.

**AC-P2-003**: The system shall provide auto-completion suggestions for Cypher keywords when typing.

**AC-P2-004**: The system shall provide auto-completion suggestions for Tag names when typing.

**AC-P2-005**: The system shall provide auto-completion suggestions for Edge types when typing.

**AC-P2-006**: The system shall provide auto-completion suggestions for property names when typing.

**AC-P2-007**: When the user presses Shift+Enter, the system shall execute the current query.

**AC-P2-008**: When the user clicks the "Execute" button, the system shall execute the current query.

**AC-P2-009**: When a query is executing, the system shall display a loading indicator.

**AC-P2-010**: When query execution completes successfully, the system shall display results in a table format by default.

**AC-P2-011**: The system shall provide a toggle to switch between table and JSON result views.

**AC-P2-012**: When displaying results in a table, the system shall show column headers based on the returned data structure.

**AC-P2-013**: When displaying results in a table, the system shall support horizontal scrolling for wide tables.

**AC-P2-014**: When displaying results in a table, the system shall support pagination if the result set exceeds 100 rows.

**AC-P2-015**: When displaying results in JSON format, the system shall format the JSON with proper indentation.

**AC-P2-016**: When a query execution fails, the system shall display an error message with error details.

**AC-P2-017**: The system shall save each executed query to the query history.

**AC-P2-018**: The system shall store query history in localStorage.

**AC-P2-019**: The system shall display a list of the last 15 executed queries.

**AC-P2-020**: When the user selects a query from history, the system shall load it into the editor.

**AC-P2-021**: The system shall provide a "Clear History" button to remove all history records.

**AC-P2-022**: The system shall provide a "Save to Favorites" button for the current query.

**AC-P2-023**: The system shall display a list of saved favorite queries.

**AC-P2-024**: When the user clicks a favorite query, the system shall load it into the editor and execute it.

**AC-P2-025**: The system shall provide a "Delete" button for each favorite query.

**AC-P2-026**: The system shall provide an "Export to CSV" button for the current result set.

**AC-P2-027**: The system shall provide an "Export to JSON" button for the current result set.

**AC-P2-028**: When exporting to CSV, the system shall include column headers as the first row.

**AC-P2-029**: When exporting to JSON, the system shall preserve the data structure and formatting.

**AC-P2-030**: The system shall display the execution time for each query.

**AC-P2-031**: The system shall support executing multiple queries separated by semicolons.

**AC-P2-032**: When executing multiple queries, the system shall display results for each query separately.

### 交付物

- [ ] 查询编辑器组件
- [ ] 查询执行功能
- [ ] 结果展示组件（表格、JSON）
- [ ] 查询历史和收藏功能
- [ ] 结果导出功能
- [ ] 单元测试（覆盖率 > 75%）
- [ ] 集成测试

---

## 阶段 3: Schema 管理 - Space

### 目标

实现 Space（图空间）的完整管理功能，包括列表展示、创建、删除和详情查看。

### 核心功能

1. **Space 列表**
   - 展示所有 Space
   - 显示 Space 基本信息
   - Space 统计信息

2. **Space 操作**
   - 创建 Space（配置 Vid Type、分区数、副本数）
   - 删除 Space（带确认）
   - 查看 Space 详情

### 用户故事

**US-P3-001**: 作为数据库开发者，我想要查看所有可用的 Space，以便了解数据库的结构。

**US-P3-002**: 作为数据库开发者，我想要创建新的 Space，以便组织不同类型的数据。

**US-P3-003**: 作为数据库开发者，我想要查看 Space 的详细信息，包括 Vid Type 和副本数，以便了解配置。

**US-P3-004**: 作为数据库开发者，我想要删除不需要的 Space，以便释放存储空间。

**US-P3-005**: 作为数据库开发者，我想要查看 Space 的统计信息，包括节点和边的数量，以便监控数据规模。

### 验收标准

**AC-P3-001**: The system shall display a list of all Spaces in the database.

**AC-P3-002**: The system shall provide a "Create Space" button.

**AC-P3-003**: When the user clicks "Create Space", the system shall display a form with fields for Space name, Vid Type, partition number, and replica factor.

**AC-P3-004**: The system shall validate that the Space name is not empty and follows naming conventions.

**AC-P3-005**: The system shall provide options for Vid Type (INT64, FIXED_STRING, etc.).

**AC-P3-006**: When the user submits valid Space creation data, the system shall execute the CREATE SPACE command.

**AC-P3-007**: When Space creation is successful, the system shall display a success message and refresh the Space list.

**AC-P3-008**: When Space creation fails, the system shall display an error message with details.

**AC-P3-009**: The system shall provide a "Delete" button for each Space in the list.

**AC-P3-010**: When the user clicks "Delete Space", the system shall display a confirmation dialog.

**AC-P3-011**: When the user confirms Space deletion, the system shall execute the DROP SPACE command.

**AC-P3-012**: The system shall provide a "View Details" option for each Space.

**AC-P3-013**: When the user views Space details, the system shall display Vid Type, partition number, replica factor, and creation time.

**AC-P3-014**: The system shall display statistics for each Space including the number of vertices and edges.

**AC-P3-015**: The system shall provide a "Refresh Statistics" button to update the statistics.

**AC-P3-016**: The system shall display a breadcrumb navigation showing the current Space.

**AC-P3-017**: The system shall allow the user to switch between Spaces.

**AC-P3-018**: When the user switches Spaces, the system shall refresh the Schema data for the new Space.

### 交付物

- [ ] Space 列表页面
- [ ] Space 创建表单
- [ ] Space 详情查看
- [ ] Space 删除功能
- [ ] Space 统计信息展示
- [ ] 单元测试（覆盖率 > 70%）
- [ ] 集成测试

---

## 阶段 4: Schema 管理 - Tag/Edge

### 目标

实现 Tag（标签）和 Edge（边类型）的完整管理功能，包括列表展示、创建、修改、删除和详情查看。

### 核心功能

1. **Tag 管理**
   - Tag 列表展示
   - 创建 Tag（定义属性）
   - 修改 Tag（添加/删除属性）
   - 删除 Tag
   - Tag 详情查看

2. **Edge 管理**
   - Edge 列表展示
   - 创建 Edge（定义属性）
   - 修改 Edge（添加/删除属性）
   - 删除 Edge
   - Edge 详情查看

### 用户故事

**US-P4-001**: 作为数据库开发者，我想要查看所有 Tag 及其属性，以便了解数据模型。

**US-P4-002**: 作为数据库开发者，我想要创建新的 Tag 并定义其属性，以便为节点添加结构化数据。

**US-P4-003**: 作为数据库开发者，我想要修改 Tag 的属性，包括添加新属性或删除现有属性，以便适应数据模型的变化。

**US-P4-004**: 作为数据库开发者，我想要删除不需要的 Tag，以便清理 Schema。

**US-P4-005**: 作为数据库开发者，我想要查看所有 Edge 类型及其属性，以便了解关系模型。

**US-P4-006**: 作为数据库开发者，我想要创建新的 Edge 类型并定义其属性，以便为关系添加结构化数据。

**US-P4-007**: 作为数据库开发者，我想要修改 Edge 类型的属性，包括添加新属性或删除现有属性，以便适应关系模型的变化。

**US-P4-008**: 作为数据库开发者，我想要删除不需要的 Edge 类型，以便清理 Schema。

### 验收标准

**Tag 管理**

**AC-P4-001**: The system shall display a list of all Tags in the current Space.

**AC-P4-002**: The system shall provide a "Create Tag" button.

**AC-P4-003**: When the user clicks "Create Tag", the system shall display a form with fields for Tag name and properties.

**AC-P4-004**: The system shall allow the user to add multiple properties to a Tag.

**AC-P4-005**: For each property, the system shall provide fields for property name, data type, and default value.

**AC-P4-006**: The system shall provide options for data types (string, int, float, bool, datetime, etc.).

**AC-P4-007**: The system shall validate that property names are unique within a Tag.

**AC-P4-008**: When the user submits valid Tag creation data, the system shall execute the CREATE TAG command.

**AC-P4-009**: When Tag creation is successful, the system shall display a success message and refresh the Tag list.

**AC-P4-010**: The system shall provide an "Edit" button for each Tag.

**AC-P4-011**: When the user clicks "Edit Tag", the system shall display the Tag's current properties.

**AC-P4-012**: The system shall allow the user to add new properties to an existing Tag.

**AC-P4-013**: The system shall allow the user to delete properties from an existing Tag.

**AC-P4-014**: When the user submits Tag modifications, the system shall execute the ALTER TAG command.

**AC-P4-015**: The system shall provide a "Delete" button for each Tag.

**AC-P4-016**: When the user clicks "Delete Tag", the system shall display a confirmation dialog.

**AC-P4-017**: When the user confirms Tag deletion, the system shall execute the DROP TAG command.

**AC-P4-018**: The system shall provide a "View Details" option for each Tag.

**AC-P4-019**: When the user views Tag details, the system shall display all properties with their data types and default values.

**Edge 管理**

**AC-P4-020**: The system shall display a list of all Edge types in the current Space.

**AC-P4-021**: The system shall provide a "Create Edge" button.

**AC-P4-022**: When the user clicks "Create Edge", the system shall display a form with fields for Edge name and properties.

**AC-P4-023**: The system shall allow the user to add multiple properties to an Edge.

**AC-P4-024**: For each property, the system shall provide fields for property name, data type, and default value.

**AC-P4-025**: The system shall provide options for data types (string, int, float, bool, datetime, etc.).

**AC-P4-026**: The system shall validate that property names are unique within an Edge.

**AC-P4-027**: When the user submits valid Edge creation data, the system shall execute the CREATE EDGE command.

**AC-P4-028**: When Edge creation is successful, the system shall display a success message and refresh the Edge list.

**AC-P4-029**: The system shall provide an "Edit" button for each Edge.

**AC-P4-030**: When the user clicks "Edit Edge", the system shall display the Edge's current properties.

**AC-P4-031**: The system shall allow the user to add new properties to an existing Edge.

**AC-P4-032**: The system shall allow the user to delete properties from an existing Edge.

**AC-P4-033**: When the user submits Edge modifications, the system shall execute the ALTER EDGE command.

**AC-P4-034**: The system shall provide a "Delete" button for each Edge.

**AC-P4-035**: When the user clicks "Delete Edge", the system shall display a confirmation dialog.

**AC-P4-036**: When the user confirms Edge deletion, the system shall execute the DROP EDGE command.

**AC-P4-037**: The system shall provide a "View Details" option for each Edge.

**AC-P4-038**: When the user views Edge details, the system shall display all properties with their data types and default values.

**通用要求**

**AC-P4-039**: The system shall provide tabs or sections to switch between Space, Tag, and Edge management.

**AC-P4-040**: The system shall display loading indicators when fetching Schema data.

**AC-P4-041**: The system shall display error messages when Schema operations fail.

### 交付物

- [ ] Tag 列表页面
- [ ] Tag 创建和编辑表单
- [ ] Edge 列表页面
- [ ] Edge 创建和编辑表单
- [ ] Tag/Edge 删除功能
- [ ] Tag/Edge 详情查看
- [ ] 单元测试（覆盖率 > 75%）
- [ ] 集成测试

---

## 阶段 5: Schema 管理 - 索引

### 目标

实现索引的完整管理功能，包括列表展示、创建、删除和状态查看。

### 核心功能

1. **索引列表**
   - 展示所有索引
   - 显示索引类型（Tag/Edge）
   - 显示索引属性
   - 显示索引状态

2. **索引操作**
   - 创建索引（选择 Tag/Edge 和属性）
   - 删除索引（带确认）
   - 重建索引

### 用户故事

**US-P5-001**: 作为数据库开发者，我想要查看所有索引，以便了解当前的索引配置。

**US-P5-002**: 作为数据库开发者，我想要为 Tag 或 Edge 的属性创建索引，以便提高查询性能。

**US-P5-003**: 作为数据库开发者，我想要删除不需要的索引，以便减少存储开销。

**US-P5-004**: 作为数据库开发者，我想要查看索引的状态，以便了解索引是否可用。

### 验收标准

**AC-P5-001**: The system shall display a list of all indexes in the current Space.

**AC-P5-002**: The system shall indicate whether each index is for a Tag or Edge.

**AC-P5-003**: The system shall display the indexed properties for each index.

**AC-P5-004**: The system shall provide a "Create Index" button.

**AC-P5-005**: When the user clicks "Create Index", the system shall display a form with fields for index name, type (Tag/Edge), and properties.

**AC-P5-006**: The system shall allow the user to select a Tag or Edge for the index.

**AC-P5-007**: The system shall allow the user to select one or multiple properties to index.

**AC-P5-008**: The system shall validate that the selected properties exist in the selected Tag or Edge.

**AC-P5-009**: When the user submits valid index creation data, the system shall execute the CREATE INDEX command.

**AC-P5-010**: When index creation is successful, the system shall display a success message and refresh the index list.

**AC-P5-011**: The system shall provide a "Delete" button for each index.

**AC-P5-012**: When the user clicks "Delete Index", the system shall display a confirmation dialog.

**AC-P5-013**: When the user confirms index deletion, the system shall execute the DROP INDEX command.

**AC-P5-014**: The system shall display the status of each index (creating, finished, failed).

**AC-P5-015**: The system shall provide a "Rebuild Index" button for indexes that need rebuilding.

**AC-P5-016**: When the user clicks "Rebuild Index", the system shall execute the REBUILD INDEX command.

**AC-P5-017**: The system shall display loading indicators when creating or rebuilding indexes.

**AC-P5-018**: The system shall display error messages when index operations fail.

### 交付物

- [ ] 索引列表页面
- [ ] 索引创建表单
- [ ] 索引删除功能
- [ ] 索引状态显示
- [ ] 索引重建功能
- [ ] 单元测试（覆盖率 > 70%）
- [ ] 集成测试

---

## 阶段 6: 图可视化

### 目标

提供查询结果的图形化展示功能，使用力导向图可视化节点和边的关系。

### 核心功能

1. **图形展示**
   - 力导向图渲染
   - 节点和边的显示
   - 布局算法选择

2. **样式自定义**
   - 节点样式（颜色、大小）
   - 边样式（颜色、粗细）
   - 标签显示

3. **交互操作**
   - 缩放和平移
   - 节点拖拽
   - 节点/边选择
   - 详情查看

### 用户故事

**US-P6-001**: 作为数据库开发者，我想要以图形化方式查看查询结果，以便直观地理解节点和边的关系。

**US-P6-002**: 作为数据库开发者，我想要自定义节点的样式（颜色、大小），以便区分不同类型的节点。

**US-P6-003**: 作为数据库开发者，我想要自定义边的样式（颜色、粗细），以便区分不同类型的关系。

**US-P6-004**: 作为数据库开发者，我想要通过缩放和平移操作来浏览大型图，以便查看细节和整体结构。

**US-P6-005**: 作为数据库开发者，我想要拖拽节点来调整图布局，以便更好地展示关系。

**US-P6-006**: 作为数据库开发者，我想要点击节点或边查看其详细信息，以便了解具体的数据内容。

### 验收标准

**AC-P6-001**: The system shall provide a force-directed graph visualization for query results.

**AC-P6-002**: The system shall display nodes and edges based on the query result data.

**AC-P6-003**: The system shall support different layout algorithms (force-directed, circular, hierarchical).

**AC-P6-004**: The system shall allow the user to customize node styles (color, size, label).

**AC-P6-005**: The system shall allow the user to customize edge styles (color, width, label).

**AC-P6-006**: The system shall support zooming in and out of the graph.

**AC-P6-007**: The system shall support panning the graph view.

**AC-P6-008**: The system shall allow the user to drag nodes to adjust the layout.

**AC-P6-009**: When the user clicks a node, the system shall display a detail panel with node properties.

**AC-P6-010**: When the user clicks an edge, the system shall display a detail panel with edge properties.

**AC-P6-011**: The system shall support selecting multiple nodes and edges.

**AC-P6-012**: The system shall provide a toggle to switch between table and graph views.

**AC-P6-013**: The system shall display node labels based on specified properties.

**AC-P6-014**: The system shall display edge labels based on specified properties.

**AC-P6-015**: The system shall handle graphs with up to 500 nodes without significant performance degradation.

**AC-P6-016**: The system shall provide a "Fit to Screen" button to center and scale the graph.

**AC-P6-017**: The system shall provide a "Reset Layout" button to restore the initial layout.

### 交付物

- [ ] 图可视化组件
- [ ] 节点/边样式配置
- [ ] 图布局算法
- [ ] 交互操作（缩放、拖拽、选择）
- [ ] 详情查看面板
- [ ] 单元测试（覆盖率 > 70%）
- [ ] 集成测试

---

## 阶段 7: 数据浏览

### 目标

提供数据浏览界面，支持按 Tag 和 Edge 浏览数据，并提供数据筛选和统计功能。

### 核心功能

1. **数据浏览**
   - 按 Tag 浏览节点
   - 按 Edge 浏览边
   - 分页展示
   - 数据排序

2. **数据筛选**
   - 属性筛选
   - 条件查询
   - 高级筛选

3. **数据统计**
   - 节点数量统计
   - 边数量统计
   - 按类型统计

### 用户故事

**US-P7-001**: 作为数据库开发者，我想要按 Tag 浏览节点数据，以便查看特定类型的节点。

**US-P7-002**: 作为数据库开发者，我想要按 Edge 浏览边数据，以便查看特定类型的关系。

**US-P7-003**: 作为数据库开发者，我想要筛选数据以查找特定的节点或边，以便快速定位目标数据。

**US-P7-004**: 作为数据库开发者，我想要查看数据库的统计信息，包括节点和边的总数，以便了解数据规模。

### 验收标准

**AC-P7-001**: The system shall provide a data browser interface.

**AC-P7-002**: The system shall allow the user to browse nodes by Tag.

**AC-P7-003**: The system shall allow the user to browse edges by Edge type.

**AC-P7-004**: The system shall display data in a paginated table format.

**AC-P7-005**: The system shall support sorting data by any column.

**AC-P7-006**: The system shall provide filtering options based on property values.

**AC-P7-007**: The system shall support advanced filtering with multiple conditions.

**AC-P7-008**: The system shall display statistics for the current Space including total vertex count and edge count.

**AC-P7-009**: The system shall display statistics broken down by Tag and Edge types.

**AC-P7-010**: The system shall provide a "Refresh" button to update the data.

**AC-P7-011**: The system shall allow the user to view details of a specific node or edge.

**AC-P7-012**: The system shall display loading indicators when fetching data.

**AC-P7-013**: The system shall handle large datasets with pagination (default 100 rows per page).

### 交付物

- [ ] 数据浏览页面
- [ ] Tag 数据浏览
- [ ] Edge 数据浏览
- [ ] 数据筛选功能
- [ ] 数据统计展示
- [ ] 单元测试（覆盖率 > 70%）
- [ ] 集成测试

---

## 总结

### 阶段对比

| 阶段 | 用户故事数 | 验收标准数 | 预计工期 | 复杂度 |
|-------|----------|------------|---------|--------|
| 阶段 1 | 4 | 17 | 1 周 | 低 |
| 阶段 2 | 10 | 32 | 2 周 | 中 |
| 阶段 3 | 5 | 18 | 1 周 | 低 |
| 阶段 4 | 8 | 41 | 2 周 | 高 |
| 阶段 5 | 4 | 18 | 1 周 | 中 |
| 阶段 6 | 6 | 17 | 2 周 | 高 |
| 阶段 7 | 4 | 13 | 1.5 周 | 中 |
| **总计** | **41** | **156** | **10.5 周** | - |

### 关键里程碑

- **里程碑 1**（阶段 1 完成后）: 基础框架可用，可以连接到数据库
- **里程碑 2**（阶段 2 完成后）: 可以执行查询并查看结果
- **里程碑 3**（阶段 3 完成后）: 可以管理 Space
- **里程碑 4**（阶段 4 完成后）: 可以管理 Tag 和 Edge
- **里程碑 5**（阶段 5 完成后）: 可以管理索引
- **里程碑 6**（阶段 6 完成后）: 可以图形化展示查询结果
- **里程碑 7**（阶段 7 完成后）: 可以浏览和筛选数据

### 风险和缓解措施

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| GraphDB 后端 API 不完整 | 高 | 中 | 在阶段 1 完成后验证所有 API 端点 |
| Cypher 语法支持复杂 | 中 | 高 | 使用成熟的 Monaco Editor 配置 |
| 大数据集性能问题 | 中 | 中 | 实现分页和虚拟滚动 |
| 图可视化性能问题 | 中 | 中 | 限制节点数量，提供警告 |
| 跨阶段依赖问题 | 高 | 低 | 每个阶段独立可交付 |

---

**文档结束**
