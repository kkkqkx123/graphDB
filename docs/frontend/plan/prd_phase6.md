# GraphDB 前端 PRD - 阶段 6: 图可视化

**文档版本**: v1.0
**创建日期**: 2026-03-29
**最后更新**: 2026-03-29
**项目阶段**: Phase 6 - 图可视化
**预计工期**: 2 周
**依赖阶段**: 阶段 2 (查询控制台)

---

## 1. 引言（Introduction）

### 1.1 阶段目标

阶段 6 的目标是提供查询结果的图形化展示功能，使用力导向图可视化节点和边的关系，让用户能够直观地理解图数据的结构和关联。

### 1.2 核心价值

- **直观理解**: 通过图形化方式直观展示节点和边的关系
- **交互探索**: 支持缩放、平移、拖拽等交互操作，便于探索大型图
- **样式定制**: 支持节点和边样式自定义，区分不同类型数据
- **数据洞察**: 通过可视化发现数据中的模式和异常

### 1.3 目标用户

- **数据库开发者**: 需要可视化查询结果以理解数据结构
- **数据分析师**: 需要通过图形化方式发现数据关系
- **业务人员**: 需要直观展示图数据以支持决策

### 1.4 范围

**包含范围**:
- 力导向图渲染
- 节点和边的图形化展示
- 多种布局算法支持
- 节点/边样式自定义
- 交互操作（缩放、平移、拖拽、选择）
- 节点/边详情查看
- 视图切换（表格/图形）

**不包含范围**:
- 图编辑功能（添加/删除节点和边）
- 路径分析和高亮
- 子图提取和保存
- 3D 可视化

---

## 2. 功能需求

### 2.1 图形展示

#### 2.1.1 功能概述

基于查询结果渲染力导向图，展示节点和边的关系。支持从控制台查询结果直接切换到图形视图。

#### 2.1.2 用户故事

**US-P6-GRAPH-001**: 作为数据库开发者，我想要以图形化方式查看查询结果，以便直观地理解节点和边的关系。

**US-P6-GRAPH-002**: 作为数据库开发者，我想要在控制台和图形视图之间切换，以便根据不同场景选择合适的展示方式。

**US-P6-GRAPH-003**: 作为数据分析师，我想要选择不同的布局算法，以便更好地展示不同类型的图结构。

#### 2.1.3 验收标准（EARS 格式）

**AC-P6-GRAPH-001**: The system shall provide a toggle button to switch between table and graph views in the console page.

**AC-P6-GRAPH-002**: When the user switches to graph view, the system shall render a force-directed graph based on the query result data.

**AC-P6-GRAPH-003**: The system shall display nodes as circles in the graph.

**AC-P6-GRAPH-004**: The system shall display edges as lines connecting nodes.

**AC-P6-GRAPH-005**: The system shall support the following layout algorithms: force-directed, circular, grid, and hierarchical.

**AC-P6-GRAPH-006**: The system shall provide a dropdown to select the layout algorithm.

**AC-P6-GRAPH-007**: When the user changes the layout algorithm, the system shall re-render the graph with the new layout.

**AC-P6-GRAPH-008**: The system shall display node labels based on the node ID or specified property.

**AC-P6-GRAPH-009**: The system shall display edge labels based on the edge type or specified property.

**AC-P6-GRAPH-010**: The system shall handle graphs with up to 500 nodes without significant performance degradation.

**AC-P6-GRAPH-011**: When the graph contains more than 500 nodes, the system shall display a warning message and suggest filtering the query.

**AC-P6-GRAPH-012**: The system shall display a loading indicator while rendering the graph.

**AC-P6-GRAPH-013**: The system shall display an empty state message when the query result contains no graph data.

---

### 2.2 样式自定义

#### 2.2.1 功能概述

支持自定义节点和边的样式，包括颜色、大小、标签等，以便区分不同类型和属性的数据。

#### 2.2.2 用户故事

**US-P6-STYLE-001**: 作为数据库开发者，我想要自定义节点的样式（颜色、大小），以便区分不同类型的节点。

**US-P6-STYLE-002**: 作为数据库开发者，我想要自定义边的样式（颜色、粗细），以便区分不同类型的关系。

**US-P6-STYLE-003**: 作为数据分析师，我想要根据节点属性自动设置样式，以便快速识别关键数据。

#### 2.2.3 验收标准（EARS 格式）

**AC-P6-STYLE-001**: The system shall provide a "Style Settings" panel for customizing graph appearance.

**AC-P6-STYLE-002**: The system shall allow the user to customize node color for each Tag type.

**AC-P6-STYLE-003**: The system shall provide a color picker for selecting node colors.

**AC-P6-STYLE-004**: The system shall allow the user to customize node size (small, medium, large).

**AC-P6-STYLE-005**: The system shall allow the user to select which property to display as the node label.

**AC-P6-STYLE-006**: The system shall allow the user to customize edge color for each Edge type.

**AC-P6-STYLE-007**: The system shall allow the user to customize edge width (thin, medium, thick).

**AC-P6-STYLE-008**: The system shall allow the user to select which property to display as the edge label.

**AC-P6-STYLE-009**: The system shall save style preferences in localStorage.

**AC-P6-STYLE-010**: When the user reloads the page, the system shall restore the saved style preferences.

**AC-P6-STYLE-011**: The system shall provide a "Reset to Default" button to restore default styles.

**AC-P6-STYLE-012**: The system shall apply style changes in real-time without re-rendering the entire graph.

---

### 2.3 交互操作

#### 2.3.1 功能概述

提供丰富的交互操作，包括缩放、平移、拖拽、选择等，便于用户探索和分析图数据。

#### 2.3.2 用户故事

**US-P6-INTERACT-001**: 作为数据库开发者，我想要通过缩放和平移操作来浏览大型图，以便查看细节和整体结构。

**US-P6-INTERACT-002**: 作为数据库开发者，我想要拖拽节点来调整图布局，以便更好地展示关系。

**US-P6-INTERACT-003**: 作为数据分析师，我想要选择多个节点和边进行分析，以便发现数据模式。

#### 2.3.3 验收标准（EARS 格式）

**AC-P6-INTERACT-001**: The system shall support zooming in and out using the mouse wheel.

**AC-P6-INTERACT-002**: The system shall display the current zoom level as a percentage.

**AC-P6-INTERACT-003**: The system shall provide zoom in and zoom out buttons in the toolbar.

**AC-P6-INTERACT-004**: The system shall support panning the graph by dragging the background.

**AC-P6-INTERACT-005**: The system shall allow the user to drag nodes to adjust their position.

**AC-P6-INTERACT-006**: The system shall maintain the node position after dragging.

**AC-P6-INTERACT-007**: The system shall support selecting a node by clicking on it.

**AC-P6-INTERACT-008**: The system shall highlight the selected node with a distinct border.

**AC-P6-INTERACT-009**: The system shall support selecting multiple nodes using Ctrl/Cmd + click.

**AC-P6-INTERACT-010**: The system shall support box selection by dragging a selection box.

**AC-P6-INTERACT-011**: The system shall provide a "Fit to Screen" button to center and scale the graph to fit the viewport.

**AC-P6-INTERACT-012**: The system shall provide a "Reset Layout" button to restore the initial layout.

**AC-P6-INTERACT-013**: The system shall provide a "Clear Selection" button to deselect all nodes and edges.

**AC-P6-INTERACT-014**: The system shall display the number of selected nodes and edges in the toolbar.

---

### 2.4 详情查看

#### 2.4.1 功能概述

支持点击节点或边查看其详细信息，包括属性、类型、ID 等，便于用户了解具体的数据内容。

#### 2.4.2 用户故事

**US-P6-DETAIL-001**: 作为数据库开发者，我想要点击节点查看其详细信息，以便了解具体的数据内容。

**US-P6-DETAIL-002**: 作为数据库开发者，我想要点击边查看其详细信息，以便了解关系的属性。

**US-P6-DETAIL-003**: 作为数据分析师，我想要复制节点或边的 ID，以便在查询中使用。

#### 2.4.3 验收标准（EARS 格式）

**AC-P6-DETAIL-001**: When the user clicks a node, the system shall display a detail panel with node information.

**AC-P6-DETAIL-002**: The detail panel shall display the node ID.

**AC-P6-DETAIL-003**: The detail panel shall display the node Tag type.

**AC-P6-DETAIL-004**: The detail panel shall display all node properties in a key-value format.

**AC-P6-DETAIL-005**: The detail panel shall provide a "Copy ID" button to copy the node ID to clipboard.

**AC-P6-DETAIL-006**: When the user clicks an edge, the system shall display a detail panel with edge information.

**AC-P6-DETAIL-007**: The detail panel shall display the edge ID (combination of srcId, dstId, edgeType, and rank).

**AC-P6-DETAIL-008**: The detail panel shall display the edge type.

**AC-P6-DETAIL-009**: The detail panel shall display the source and destination node IDs.

**AC-P6-DETAIL-010**: The detail panel shall display all edge properties in a key-value format.

**AC-P6-DETAIL-011**: The detail panel shall provide a "Copy ID" button to copy the edge ID to clipboard.

**AC-P6-DETAIL-012**: The system shall highlight the selected node or edge in the graph.

**AC-P6-DETAIL-013**: The system shall provide a close button to hide the detail panel.

**AC-P6-DETAIL-014**: When multiple nodes or edges are selected, the system shall display a summary in the detail panel.

---

### 2.5 独立图可视化页面

#### 2.5.1 功能概述

提供独立的图可视化页面，支持直接输入查询或选择预设查询来展示图数据。

#### 2.5.2 用户故事

**US-P6-PAGE-001**: 作为数据库开发者，我想要一个独立的图可视化页面，以便专注于图数据的探索。

**US-P6-PAGE-002**: 作为数据库开发者，我想要在图可视化页面直接输入查询，以便快速查看结果。

**US-P6-PAGE-003**: 作为数据分析师，我想要使用预设查询模板，以便快速生成常见的图可视化。

#### 2.5.3 验收标准（EARS 格式）

**AC-P6-PAGE-001**: The system shall provide a "Graph" menu item in the sidebar navigation.

**AC-P6-PAGE-002**: When the user clicks the "Graph" menu item, the system shall navigate to the graph visualization page.

**AC-P6-PAGE-003**: The graph visualization page shall provide a query input area.

**AC-P6-PAGE-004**: The system shall provide a "Execute" button to run the query and display the graph.

**AC-P6-PAGE-005**: The system shall provide preset query templates (e.g., "Show all nodes", "Show relationships of selected node").

**AC-P6-PAGE-006**: The system shall display the graph visualization in the main area of the page.

**AC-P6-PAGE-007**: The system shall display the style settings panel on the right side.

**AC-P6-PAGE-008**: The system shall display the detail panel when a node or edge is selected.

---

## 3. 非功能需求

### 3.1 性能需求

**NF-P6-PERF-001**: The system shall render graphs with up to 100 nodes within 1 second.

**NF-P6-PERF-002**: The system shall render graphs with up to 500 nodes within 3 seconds.

**NF-P6-PERF-003**: The system shall maintain 60 FPS during pan and zoom operations for graphs with up to 500 nodes.

**NF-P6-PERF-004**: The system shall display a progress indicator for graphs with more than 200 nodes.

### 3.2 兼容性需求

**NF-P6-COMPAT-001**: The graph visualization shall work in Chrome, Firefox, Safari, and Edge browsers.

**NF-P6-COMPAT-002**: The graph visualization shall support touch interactions on mobile devices.

### 3.3 可访问性需求

**NF-P6-ACCESS-001**: The system shall provide keyboard navigation for the graph view.

**NF-P6-ACCESS-002**: The system shall ensure color contrast ratios meet WCAG 2.1 AA standards.

---

## 4. 界面设计

### 4.1 控制台页面 - 图形视图

```
+----------------------------------------------------------+
|  Query Editor                                            |
|  +----------------------------------------------------+  |
|  | MATCH (n)-[r]->(m) RETURN n, r, m                 |  |
|  +----------------------------------------------------+  |
|  [Execute] [Table | Graph]                               |
+----------------------------------------------------------+
|  Result Area - Graph View                                |
|  +----------------------------------------------------+  |
|  |  +----------------------------------------------+  |  |
|  |  |                                              |  |
|  |  |     (A) -----> (B)                           |  |
|  |  |      |         ^                             |  |
|  |  |      v         |                             |  |
|  |  |     (C) -------+                             |  |
|  |  |                                              |  |
|  |  |   [Zoom: 100%] [Fit] [Reset] [Layout ▼]      |  |
|  |  +----------------------------------------------+  |  |
|  |                                                    |  |
|  |  +--------------+  +------------------------+      |  |
|  |  | Style Panel  |  | Detail Panel           |      |  |
|  |  | - Node Colors|  | Node: Person            |      |  |
|  |  | - Edge Colors|  | ID: 123                 |      |  |
|  |  | - Labels     |  | Name: John              |      |  |
|  |  +--------------+  +------------------------+      |  |
|  +----------------------------------------------------+  |
+----------------------------------------------------------+
```

### 4.2 独立图可视化页面

```
+----------------------------------------------------------+
|  Sidebar | Graph Visualization Page                      |
|          +----------------------------------------------------------+
|          |  Query Input                                    |
|          |  +------------------------------------------+  |
|          |  | MATCH (n:Person) RETURN n LIMIT 50      |  |
|          |  +------------------------------------------+  |
|          |  [Execute] [Templates ▼]                        |
|          +----------------------------------------------------------+
|          |  +------------------------------------------+  |
|          |  |                                          |  |
|          |  |           Graph Visualization            |  |
|          |  |                                          |  |
|          |  +------------------------------------------+  |
|          |  [Toolbar: Zoom | Fit | Reset | Layout]        |
|          +----------------------------------------------------------+
|          |  Style Panel  |  Detail Panel                  |
|          +----------------------------------------------------------+
```

---

## 5. 技术实现

### 5.1 技术选型

| 组件/功能 | 技术选择 | 说明 |
|-----------|----------|------|
| 图渲染引擎 | Cytoscape.js | 高性能图可视化库 |
| 布局算法 | Cytoscape.js 内置 | force, circle, grid, dagre |
| 样式管理 | CSS-in-JS + localStorage | 动态样式和持久化 |

### 5.2 数据结构

```typescript
// 图数据接口
interface GraphData {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

interface GraphNode {
  id: string;
  tag: string;
  properties: Record<string, any>;
}

interface GraphEdge {
  id: string;
  type: string;
  source: string;
  target: string;
  rank: number;
  properties: Record<string, any>;
}

// 样式配置接口
interface GraphStyle {
  nodes: {
    [tag: string]: {
      color: string;
      size: 'small' | 'medium' | 'large';
      labelProperty: string;
    };
  };
  edges: {
    [type: string]: {
      color: string;
      width: 'thin' | 'medium' | 'thick';
      labelProperty: string;
    };
  };
}
```

### 5.3 API 接口

```typescript
// 查询执行（复用阶段 2 接口）
POST /api/query/exec
Request: { gql: string, space?: string }
Response: { data: any[], columns: string[], code: number, message?: string }

// 获取节点详情
GET /api/graph/vertex/:id?space=:space
Response: { id: string, tag: string, properties: Record<string, any> }

// 获取边详情
GET /api/graph/edge/:id?space=:space
Response: { id: string, type: string, src: string, dst: string, rank: number, properties: Record<string, any> }
```

---

## 6. 交付物

- [ ] 图可视化组件 (GraphVisualization)
- [ ] 布局算法实现
- [ ] 样式配置面板 (StylePanel)
- [ ] 详情查看面板 (DetailPanel)
- [ ] 控制台图形视图集成
- [ ] 独立图可视化页面
- [ ] 单元测试（覆盖率 > 70%）
- [ ] 集成测试
- [ ] 用户文档

---

## 7. 风险与缓解

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| 大数据集渲染性能问题 | 高 | 中 | 限制节点数量，提供警告提示；实现虚拟渲染 |
| Cytoscape.js 学习曲线 | 中 | 高 | 预留时间学习文档；参考官方示例 |
| 布局算法效果不理想 | 中 | 中 | 提供多种布局选项；允许用户手动调整 |
| 浏览器兼容性问题 | 低 | 低 | 使用成熟的 Cytoscape.js 库；充分测试 |

---

**文档结束**
