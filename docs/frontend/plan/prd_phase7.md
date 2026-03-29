# GraphDB 前端 PRD - 阶段 7: 数据浏览

**文档版本**: v1.0
**创建日期**: 2026-03-29
**最后更新**: 2026-03-29
**项目阶段**: Phase 7 - 数据浏览
**预计工期**: 1.5 周
**依赖阶段**: 阶段 3 (Schema 管理 - Space)

---

## 1. 引言（Introduction）

### 1.1 阶段目标

阶段 7 的目标是提供数据浏览界面，支持按 Tag 和 Edge 类型浏览数据，并提供数据筛选和统计功能，让用户能够方便地查看和管理数据库中的实际数据。

### 1.2 核心价值

- **数据探索**: 方便地浏览数据库中的节点和边数据
- **快速定位**: 通过筛选和搜索快速找到目标数据
- **数据洞察**: 通过统计信息了解数据分布和规模
- **直观展示**: 以表格形式清晰展示数据内容

### 1.3 目标用户

- **数据库开发者**: 需要浏览和验证数据内容
- **数据分析师**: 需要探索数据以进行分析
- **数据库管理员**: 需要监控数据规模和分布

### 1.4 范围

**包含范围**:
- 按 Tag 浏览节点数据
- 按 Edge 类型浏览边数据
- 数据分页展示
- 数据排序
- 属性筛选
- 数据统计信息

**不包含范围**:
- 数据编辑功能（修改节点/边属性）
- 数据删除功能
- 复杂查询构建器
- 数据导出功能（已在阶段 2 实现）

---

## 2. 功能需求

### 2.1 数据浏览主界面

#### 2.1.1 功能概述

提供数据浏览的主界面，包含 Tag 和 Edge 的导航，以及数据展示区域。

#### 2.1.2 用户故事

**US-P7-MAIN-001**: 作为数据库开发者，我想要一个统一的数据浏览界面，以便方便地查看数据库中的数据。

**US-P7-MAIN-002**: 作为数据库开发者，我想要在节点和边数据之间快速切换，以便查看不同类型的数据。

**US-P7-MAIN-003**: 作为数据分析师，我想要看到当前 Space 的数据统计概览，以便了解数据规模。

#### 2.1.3 验收标准（EARS 格式）

**AC-P7-MAIN-001**: The system shall provide a "Data Browser" menu item in the sidebar navigation.

**AC-P7-MAIN-002**: When the user clicks the "Data Browser" menu item, the system shall navigate to the data browser page.

**AC-P7-MAIN-003**: The data browser page shall display a header with the current Space name.

**AC-P7-MAIN-004**: The system shall display tabs for switching between "Vertices" and "Edges" views.

**AC-P7-MAIN-005**: The system shall display a statistics overview panel showing:
- Total vertex count
- Total edge count
- Number of Tags
- Number of Edge types

**AC-P7-MAIN-006**: The system shall display the statistics broken down by Tag type (for vertices) and Edge type (for edges).

**AC-P7-MAIN-007**: The system shall provide a "Refresh" button to update the statistics.

**AC-P7-MAIN-008**: The system shall display a loading indicator when fetching statistics.

**AC-P7-MAIN-009**: The system shall display the last updated time for the statistics.

---

### 2.2 节点数据浏览

#### 2.2.1 功能概述

按 Tag 浏览节点数据，支持分页、排序和筛选。

#### 2.2.2 用户故事

**US-P7-VERTEX-001**: 作为数据库开发者，我想要按 Tag 浏览节点数据，以便查看特定类型的节点。

**US-P7-VERTEX-002**: 作为数据库开发者，我想要对节点数据进行排序，以便按特定属性查看数据。

**US-P7-VERTEX-003**: 作为数据分析师，我想要分页浏览大量节点数据，以便处理大型数据集。

#### 2.2.3 验收标准（EARS 格式）

**AC-P7-VERTEX-001**: The system shall display a list of all Tags in the current Space.

**AC-P7-VERTEX-002**: The system shall allow the user to select a Tag to browse its vertices.

**AC-P7-VERTEX-003**: When the user selects a Tag, the system shall display a table of vertices with that Tag.

**AC-P7-VERTEX-004**: The table shall display the vertex ID as the first column.

**AC-P7-VERTEX-005**: The table shall display all properties of the Tag as columns.

**AC-P7-VERTEX-006**: The system shall support sorting by any column in ascending or descending order.

**AC-P7-VERTEX-007**: When the user clicks a column header, the system shall sort the data by that column.

**AC-P7-VERTEX-008**: The system shall display a sort indicator (arrow) in the column header.

**AC-P7-VERTEX-009**: The system shall implement pagination with a default page size of 50 rows.

**AC-P7-VERTEX-010**: The system shall allow the user to change the page size (options: 20, 50, 100).

**AC-P7-VERTEX-011**: The system shall display pagination controls (first, previous, next, last, page numbers).

**AC-P7-VERTEX-012**: The system shall display the total number of vertices and the current page range.

**AC-P7-VERTEX-013**: The system shall provide a "View Details" button for each vertex row.

**AC-P7-VERTEX-014**: When the user clicks "View Details", the system shall display a modal with all vertex properties.

**AC-P7-VERTEX-015**: The system shall provide a "Copy ID" button in the details modal.

**AC-P7-VERTEX-016**: The system shall display a loading indicator when fetching vertex data.

**AC-P7-VERTEX-017**: The system shall display an empty state message when no vertices exist for the selected Tag.

---

### 2.3 边数据浏览

#### 2.3.1 功能概述

按 Edge 类型浏览边数据，支持分页、排序和筛选。

#### 2.3.2 用户故事

**US-P7-EDGE-001**: 作为数据库开发者，我想要按 Edge 类型浏览边数据，以便查看特定类型的关系。

**US-P7-EDGE-002**: 作为数据库开发者，我想要看到边的源节点和目标节点信息，以便理解关系方向。

**US-P7-EDGE-003**: 作为数据分析师，我想要查看边的 Rank 值，以便区分同一对节点间的多条边。

#### 2.3.3 验收标准（EARS 格式）

**AC-P7-EDGE-001**: The system shall display a list of all Edge types in the current Space.

**AC-P7-EDGE-002**: The system shall allow the user to select an Edge type to browse its edges.

**AC-P7-EDGE-003**: When the user selects an Edge type, the system shall display a table of edges with that type.

**AC-P7-EDGE-004**: The table shall display the following columns:
- Edge ID (composite of srcId, dstId, type, rank)
- Source Vertex ID
- Destination Vertex ID
- Rank

**AC-P7-EDGE-005**: The table shall display all properties of the Edge type as additional columns.

**AC-P7-EDGE-006**: The system shall support sorting by any column in ascending or descending order.

**AC-P7-EDGE-007**: The system shall implement pagination with a default page size of 50 rows.

**AC-P7-EDGE-008**: The system shall allow the user to change the page size (options: 20, 50, 100).

**AC-P7-EDGE-009**: The system shall provide a "View Details" button for each edge row.

**AC-P7-EDGE-010**: When the user clicks "View Details", the system shall display a modal with all edge properties.

**AC-P7-EDGE-011**: The details modal shall display the source and destination vertex information.

**AC-P7-EDGE-012**: The system shall provide a "Copy ID" button in the details modal.

**AC-P7-EDGE-013**: The system shall display a loading indicator when fetching edge data.

**AC-P7-EDGE-014**: The system shall display an empty state message when no edges exist for the selected type.

---

### 2.4 数据筛选

#### 2.4.1 功能概述

提供数据筛选功能，支持基于属性值的简单筛选和高级多条件筛选。

#### 2.4.2 用户故事

**US-P7-FILTER-001**: 作为数据库开发者，我想要筛选数据以查找特定的节点或边，以便快速定位目标数据。

**US-P7-FILTER-002**: 作为数据分析师，我想要使用多个条件进行高级筛选，以便进行复杂的数据查询。

**US-P7-FILTER-003**: 作为数据库开发者，我想要保存常用的筛选条件，以便快速应用。

#### 2.4.3 验收标准（EARS 格式）

**AC-P7-FILTER-001**: The system shall provide a "Filter" button in the data browser toolbar.

**AC-P7-FILTER-002**: When the user clicks the "Filter" button, the system shall display a filter panel.

**AC-P7-FILTER-003**: The filter panel shall allow the user to add filter conditions.

**AC-P7-FILTER-004**: Each filter condition shall include:
- Property selection dropdown
- Operator selection (equals, not equals, greater than, less than, contains, etc.)
- Value input field

**AC-P7-FILTER-005**: The system shall display appropriate operators based on the property data type.

**AC-P7-FILTER-006**: The system shall support the following operators:
- String: equals, not equals, contains, starts with, ends with
- Number: equals, not equals, greater than, less than, greater or equal, less or equal
- Boolean: equals

**AC-P7-FILTER-007**: The system shall allow the user to add multiple filter conditions.

**AC-P7-FILTER-008**: The system shall provide AND/OR logic selection between conditions.

**AC-P7-FILTER-009**: The system shall provide an "Apply" button to apply the filter.

**AC-P7-FILTER-010**: When the user applies a filter, the system shall refresh the data table with filtered results.

**AC-P7-FILTER-011**: The system shall display the active filter count in the toolbar.

**AC-P7-FILTER-012**: The system shall provide a "Clear All" button to remove all filters.

**AC-P7-FILTER-013**: The system shall provide a remove button for each individual filter condition.

**AC-P7-FILTER-014**: The system shall validate filter values before applying.

**AC-P7-FILTER-015**: When filter validation fails, the system shall display an error message.

---

### 2.5 数据统计

#### 2.5.1 功能概述

提供详细的数据统计信息，包括总体统计和按类型的分布统计。

#### 2.5.2 用户故事

**US-P7-STAT-001**: 作为数据库开发者，我想要查看数据库的统计信息，包括节点和边的总数，以便了解数据规模。

**US-P7-STAT-002**: 作为数据库管理员，我想要查看按 Tag 和 Edge 类型的统计分布，以便监控数据增长。

**US-P7-STAT-003**: 作为数据分析师，我想要看到统计信息的变化趋势，以便分析数据增长模式。

#### 2.5.3 验收标准（EARS 格式）

**AC-P7-STAT-001**: The system shall display a statistics panel in the data browser page.

**AC-P7-STAT-002**: The statistics panel shall display the following overall statistics:
- Total number of vertices
- Total number of edges
- Number of Tags
- Number of Edge types

**AC-P7-STAT-003**: The system shall display a breakdown of vertices by Tag type.

**AC-P7-STAT-004**: The system shall display a breakdown of edges by Edge type.

**AC-P7-STAT-005**: The system shall display the count for each Tag and Edge type.

**AC-P7-STAT-006**: The system shall provide a bar chart visualization for the Tag distribution.

**AC-P7-STAT-007**: The system shall provide a bar chart visualization for the Edge type distribution.

**AC-P7-STAT-008**: The system shall provide a "Refresh" button to update the statistics.

**AC-P7-STAT-009**: The system shall display the last updated time for the statistics.

**AC-P7-STAT-010**: The system shall auto-refresh statistics every 60 seconds when the page is active.

**AC-P7-STAT-011**: The system shall display a loading indicator when fetching statistics.

---

## 3. 非功能需求

### 3.1 性能需求

**NF-P7-PERF-001**: The system shall load the first page of data within 2 seconds.

**NF-P7-PERF-002**: The system shall support browsing datasets with millions of rows through pagination.

**NF-P7-PERF-003**: The system shall apply filters within 3 seconds for datasets up to 100,000 rows.

**NF-P7-PERF-004**: The system shall maintain responsive UI during data loading.

### 3.2 可用性需求

**NF-P7-UX-001**: The system shall provide clear empty states when no data is available.

**NF-P7-UX-002**: The system shall provide meaningful error messages when data loading fails.

**NF-P7-UX-003**: The system shall preserve the user's current page and filters when refreshing data.

---

## 4. 界面设计

### 4.1 数据浏览页面 - 概览

```
+----------------------------------------------------------+
|  Data Browser - Space: production                        |
+----------------------------------------------------------+
|  Statistics Overview                                     |
|  +----------------+  +----------------+  +--------------+|
|  | Vertices: 1500 |  | Edges: 3200    |  | Tags: 5      ||
|  +----------------+  +----------------+  +--------------+|
|  | Edge Types: 8  |  | [Refresh]      |                ||
|  +----------------+                                     |
+----------------------------------------------------------+
|  [Vertices | Edges]  [Filter ▼]  [Refresh]               |
+----------------------------------------------------------+
|  Tag Selection | Data Table                              |
|  +------------+ +--------------------------------------+ |
|  | Person     | | ID | Name  | Age | Email            | |
|  | Company    | |----|-------|-----|------------------| |
|  | Product    | | 1  | John  | 30  | john@example.com | |
|  | ...        | | 2  | Jane  | 25  | jane@example.com | |
|  +------------+ | ... | ...   | ... | ...              | |
|                 +--------------------------------------+ |
|                 | Page 1 of 30  [1][2][3]...[30]        | |
|                 +--------------------------------------+ |
+----------------------------------------------------------+
```

### 4.2 数据浏览页面 - 筛选面板

```
+----------------------------------------------------------+
|  Filter Panel                                            |
|  +----------------------------------------------------+  |
|  |  Property    Operator    Value              [X]    |  |
|  |  [Age ▼]     [> ▼]       [25]                      |  |
|  |                                                    |  |
|  |  [AND | OR]                                        |  |
|  |                                                    |  |
|  |  Property    Operator    Value              [X]    |  |
|  |  [Name ▼]    [contains ▼] [John]                   |  |
|  |                                                    |  |
|  |  [+ Add Condition]                                 |  |
|  |                                                    |  |
|  |  [Clear All]              [Apply] [Cancel]         |  |
|  +----------------------------------------------------+  |
+----------------------------------------------------------+
```

### 4.3 数据详情弹窗

```
+--------------------------------------------------+
|  Vertex Details - Person                    [X]  |
+--------------------------------------------------+
|  ID: 123                                         |
|  Tag: Person                                     |
|                                                  |
|  Properties:                                     |
|  +------------------+--------------------------+ |
|  | Name             | John Smith               | |
|  | Age              | 30                       | |
|  | Email            | john@example.com         | |
|  | Created At       | 2024-01-15 10:30:00      | |
|  +------------------+--------------------------+ |
|                                                  |
|  [Copy ID]  [View in Graph]  [Close]             |
+--------------------------------------------------+
```

---

## 5. 技术实现

### 5.1 技术选型

| 组件/功能 | 技术选择 | 说明 |
|-----------|----------|------|
| 表格组件 | Ant Design Table | 成熟稳定，支持分页、排序、筛选 |
| 图表组件 | Ant Design Charts | 与 Ant Design 集成 |
| 状态管理 | Zustand | 管理浏览状态 |

### 5.2 数据结构

```typescript
// 浏览数据接口
interface BrowseData {
  vertices: VertexData[];
  edges: EdgeData[];
  total: number;
  page: number;
  pageSize: number;
}

interface VertexData {
  id: string;
  tag: string;
  properties: Record<string, any>;
}

interface EdgeData {
  id: string;
  type: string;
  src: string;
  dst: string;
  rank: number;
  properties: Record<string, any>;
}

// 筛选条件接口
interface FilterCondition {
  property: string;
  operator: 'eq' | 'ne' | 'gt' | 'lt' | 'ge' | 'le' | 'contains' | 'startsWith' | 'endsWith';
  value: string | number | boolean;
}

interface FilterGroup {
  conditions: FilterCondition[];
  logic: 'AND' | 'OR';
}

// 统计信息接口
interface Statistics {
  totalVertices: number;
  totalEdges: number;
  tagCount: number;
  edgeTypeCount: number;
  tagDistribution: { tag: string; count: number }[];
  edgeTypeDistribution: { type: string; count: number }[];
}
```

### 5.3 API 接口

```typescript
// 获取节点数据
GET /api/data/vertices?space=:space&tag=:tag&page=:page&pageSize=:pageSize&sort=:sort&filters=:filters
Response: {
  data: VertexData[];
  total: number;
  page: number;
  pageSize: number;
}

// 获取边数据
GET /api/data/edges?space=:space&type=:type&page=:page&pageSize=:pageSize&sort=:sort&filters=:filters
Response: {
  data: EdgeData[];
  total: number;
  page: number;
  pageSize: number;
}

// 获取统计信息
GET /api/data/statistics?space=:space
Response: Statistics
```

---

## 6. 交付物

- [ ] 数据浏览页面 (DataBrowser)
- [ ] 节点数据浏览组件 (VertexBrowser)
- [ ] 边数据浏览组件 (EdgeBrowser)
- [ ] 筛选面板组件 (FilterPanel)
- [ ] 统计面板组件 (StatisticsPanel)
- [ ] 数据详情弹窗 (DataDetailModal)
- [ ] 单元测试（覆盖率 > 70%）
- [ ] 集成测试
- [ ] 用户文档

---

## 7. 风险与缓解

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| 大数据集分页性能问题 | 中 | 中 | 使用游标分页；后端优化查询 |
| 复杂筛选条件查询性能 | 中 | 中 | 限制筛选条件数量；优化索引 |
| 数据权限控制 | 低 | 低 | 复用现有权限机制 |
| 实时数据一致性 | 低 | 中 | 提供手动刷新；显示最后更新时间 |

---

**文档结束**
