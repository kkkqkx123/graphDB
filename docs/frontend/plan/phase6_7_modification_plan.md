# GraphDB 前端阶段 6、7 修改方案文档

**文档版本**: v1.0
**创建日期**: 2026-03-29
**最后更新**: 2026-03-29
**适用范围**: 阶段 6 (图可视化)、阶段 7 (数据浏览)

---

## 1. 文档概述

### 1.1 目的

本文档基于已完成的阶段 1-5 实现，结合 [prd_phase6.md](./prd_phase6.md) 和 [prd_phase7.md](./prd_phase7.md) 的详细需求，提出阶段 6 和阶段 7 的具体修改方案，包括：

- 对现有代码的修改点
- 新增组件和模块
- 接口和类型定义调整
- 与阶段 2、3 的集成方案

### 1.2 参考文档

- [prd_phase6.md](./prd_phase6.md) - 阶段 6 详细 PRD
- [prd_phase7.md](./prd_phase7.md) - 阶段 7 详细 PRD
- [prd_phase2.md](./prd_phase2.md) - 阶段 2 PRD（阶段 6 依赖）
- [prd_phase3.md](./prd_phase3.md) - 阶段 3 PRD（阶段 7 依赖）
- [directory_structure.md](../architecture/directory_structure.md) - 目录结构设计
- [tech_stack.md](../architecture/tech_stack.md) - 技术栈文档
- [component_reuse_analysis.md](../component_reuse_analysis.md) - 组件复用分析

---

## 2. 阶段 6 修改方案

### 2.1 现有架构分析

阶段 2 已完成：
- 查询控制台基础功能
- 查询结果表格展示
- Monaco Editor 集成

阶段 6 需要在此基础上扩展图可视化功能。

### 2.2 目录结构变更

```
frontend/src/
├── pages/
│   ├── Console/                         # 已存在，需扩展
│   │   ├── index.tsx                    # 已存在，需修改
│   │   ├── index.module.less            # 已存在
│   │   ├── components/                  # 已存在，需扩展
│   │   │   ├── QueryEditor/             # 已存在
│   │   │   ├── OutputBox/               # 已存在，需修改
│   │   │   │   ├── index.tsx            # 已存在，需修改
│   │   │   │   ├── TableView.tsx        # 已存在
│   │   │   │   ├── JsonView.tsx         # 已存在
│   │   │   │   ├── GraphView.tsx        # 新增
│   │   │   │   └── index.module.less    # 已存在
│   │   │   ├── GraphVisualization/      # 新增
│   │   │   │   ├── index.tsx
│   │   │   │   ├── GraphCanvas.tsx
│   │   │   │   ├── GraphToolbar.tsx
│   │   │   │   ├── StylePanel.tsx
│   │   │   │   ├── DetailPanel.tsx
│   │   │   │   └── index.module.less
│   │   │   └── LayoutSelector/          # 新增
│   │   │       ├── index.tsx
│   │   │       └── index.module.less
│   │   └── hooks/                       # 已存在，需扩展
│   │       └── useGraphVisualization.ts # 新增
│   └── GraphVisualization/              # 新增 - 独立页面
│       ├── index.tsx
│       ├── index.module.less
│       ├── components/
│       │   ├── QueryPanel/
│       │   ├── GraphCanvas/
│       │   ├── GraphToolbar/
│       │   ├── StylePanel/
│       │   ├── DetailPanel/
│       │   └── TemplateSelector/
│       └── hooks/
│           └── useGraphPage.ts
├── components/
│   └── business/
│       ├── GraphCanvas/                 # 新增（可复用组件）
│       │   ├── index.tsx
│       │   └── index.module.less
│       ├── ColorPicker/                 # 复用阶段 4/5
│       └── EmptyGraphTip/               # 新增
│           ├── index.tsx
│           └── index.module.less
├── stores/
│   └── graph.ts                         # 新增
├── services/
│   └── graph.ts                         # 新增
├── utils/
│   ├── cytoscapeConfig.ts               # 新增
│   └── graphLayout.ts                   # 新增
└── types/
    └── graph.ts                         # 已存在，需扩展
```

### 2.3 具体修改点

#### 2.3.1 Console 页面扩展 (pages/Console/index.tsx)

**修改类型**: 修改

**修改内容**:
1. 在结果展示区域添加视图切换标签（Table / Graph）
2. 传递查询结果数据给 GraphView 组件

```typescript
// 修改后的 OutputBox 组件调用
<OutputBox
  result={queryResult}
  activeView={activeView} // 'table' | 'json' | 'graph'
  onViewChange={setActiveView}
/>
```

#### 2.3.2 OutputBox 组件扩展 (pages/Console/components/OutputBox/index.tsx)

**修改类型**: 修改

**修改内容**:
1. 添加 GraphView 导入
2. 添加图形视图标签
3. 传递数据给 GraphView

```typescript
// 修改后的视图切换
<Tabs activeKey={activeView} onChange={setActiveView}>
  <Tabs.TabPane tab="Table" key="table">
    <TableView data={result.data} columns={result.columns} />
  </Tabs.TabPane>
  <Tabs.TabPane tab="JSON" key="json">
    <JsonView data={result.data} />
  </Tabs.TabPane>
  <Tabs.TabPane tab="Graph" key="graph">
    <GraphView data={result.data} />
  </Tabs.TabPane>
</Tabs>
```

#### 2.3.3 Graph Store 新增 (stores/graph.ts)

**修改类型**: 新增

**新增状态**:
```typescript
interface GraphState {
  // 图数据
  graphData: GraphData | null;
  
  // 视图状态
  layout: 'force' | 'circle' | 'grid' | 'hierarchical';
  zoom: number;
  selectedNodes: string[];
  selectedEdges: string[];
  
  // 样式配置
  nodeStyles: Record<string, NodeStyle>;
  edgeStyles: Record<string, EdgeStyle>;
  
  // 详情面板
  detailPanelVisible: boolean;
  detailData: NodeDetail | EdgeDetail | null;
  
  // Actions
  setGraphData: (data: GraphData) => void;
  setLayout: (layout: GraphState['layout']) => void;
  setZoom: (zoom: number) => void;
  selectNode: (id: string, multi?: boolean) => void;
  selectEdge: (id: string, multi?: boolean) => void;
  clearSelection: () => void;
  setNodeStyle: (tag: string, style: NodeStyle) => void;
  setEdgeStyle: (type: string, style: EdgeStyle) => void;
  showDetail: (data: NodeDetail | EdgeDetail) => void;
  hideDetail: () => void;
  fitToScreen: () => void;
  resetLayout: () => void;
}

interface NodeStyle {
  color: string;
  size: 'small' | 'medium' | 'large';
  labelProperty: string;
}

interface EdgeStyle {
  color: string;
  width: 'thin' | 'medium' | 'thick';
  labelProperty: string;
}
```

#### 2.3.4 Graph Service 新增 (services/graph.ts)

**修改类型**: 新增

**新增 API**:
```typescript
import { get } from '@/utils/http';
import type { GraphData, NodeDetail, EdgeDetail } from '@/types/graph';

export const graphService = {
  // 获取节点详情
  getVertexDetail: (space: string, id: string) =>
    get<NodeDetail>(`/api/graph/vertex/${id}?space=${space}`),
  
  // 获取边详情
  getEdgeDetail: (space: string, id: string) =>
    get<EdgeDetail>(`/api/graph/edge/${id}?space=${space}`),
  
  // 获取邻居节点
  getNeighbors: (space: string, id: string) =>
    get<GraphData>(`/api/graph/neighbors?space=${space}&id=${id}`),
};
```

#### 2.3.5 类型定义扩展 (types/graph.ts)

**修改类型**: 扩展

**新增类型**:
```typescript
// 图数据
export interface GraphData {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

export interface GraphNode {
  id: string;
  tag: string;
  properties: Record<string, any>;
}

export interface GraphEdge {
  id: string;
  type: string;
  source: string;
  target: string;
  rank: number;
  properties: Record<string, any>;
}

// 节点详情
export interface NodeDetail {
  id: string;
  tag: string;
  properties: Record<string, any>;
}

// 边详情
export interface EdgeDetail {
  id: string;
  type: string;
  source: string;
  target: string;
  rank: number;
  properties: Record<string, any>;
}

// 布局类型
export type LayoutType = 'force' | 'circle' | 'grid' | 'hierarchical';

// 样式配置
export interface GraphStyleConfig {
  nodes: Record<string, {
    color: string;
    size: 'small' | 'medium' | 'large';
    labelProperty: string;
  }>;
  edges: Record<string, {
    color: string;
    width: 'thin' | 'medium' | 'thick';
    labelProperty: string;
  }>;
}
```

#### 2.3.6 Cytoscape 配置工具 (utils/cytoscapeConfig.ts)

**修改类型**: 新增

```typescript
import cytoscape from 'cytoscape';
import type { GraphData, GraphStyleConfig } from '@/types/graph';

// 转换 GraphData 为 Cytoscape elements
export function convertToCytoscapeElements(data: GraphData): cytoscape.ElementDefinition[] {
  const nodes = data.nodes.map(node => ({
    data: {
      id: node.id,
      label: node.tag,
      ...node.properties,
      _tag: node.tag,
    },
  }));
  
  const edges = data.edges.map(edge => ({
    data: {
      id: edge.id,
      source: edge.source,
      target: edge.target,
      label: edge.type,
      ...edge.properties,
      _type: edge.type,
      _rank: edge.rank,
    },
  }));
  
  return [...nodes, ...edges];
}

// 生成 Cytoscape 样式
export function generateCytoscapeStyle(config: GraphStyleConfig): cytoscape.Stylesheet[] {
  const nodeStyles = Object.entries(config.nodes).map(([tag, style]) => ({
    selector: `node[_tag="${tag}"]`,
    style: {
      'background-color': style.color,
      'width': getNodeSize(style.size),
      'height': getNodeSize(style.size),
      'label': `data(${style.labelProperty})`,
      'font-size': '12px',
      'text-valign': 'center',
      'text-halign': 'center',
    },
  }));
  
  const edgeStyles = Object.entries(config.edges).map(([type, style]) => ({
    selector: `edge[_type="${type}"]`,
    style: {
      'line-color': style.color,
      'width': getEdgeWidth(style.width),
      'label': `data(${style.labelProperty})`,
      'font-size': '10px',
      'curve-style': 'bezier',
      'target-arrow-shape': 'triangle',
    },
  }));
  
  return [
    {
      selector: 'node',
      style: {
        'background-color': '#666',
        'width': 40,
        'height': 40,
        'label': 'data(id)',
        'font-size': '12px',
      },
    },
    {
      selector: 'edge',
      style: {
        'width': 2,
        'line-color': '#ccc',
        'curve-style': 'bezier',
        'target-arrow-shape': 'triangle',
      },
    },
    {
      selector: ':selected',
      style: {
        'border-width': 3,
        'border-color': '#1890ff',
      },
    },
    ...nodeStyles,
    ...edgeStyles,
  ];
}

function getNodeSize(size: string): number {
  const sizes = { small: 30, medium: 40, large: 50 };
  return sizes[size as keyof typeof sizes] || 40;
}

function getEdgeWidth(width: string): number {
  const widths = { thin: 1, medium: 2, thick: 4 };
  return widths[width as keyof typeof widths] || 2;
}
```

#### 2.3.7 布局算法工具 (utils/graphLayout.ts)

**修改类型**: 新增

```typescript
import cytoscape from 'cytoscape';
import type { LayoutType } from '@/types/graph';

export function applyLayout(cy: cytoscape.Core, layout: LayoutType): cytoscape.Layouts {
  const layouts: Record<LayoutType, cytoscape.LayoutOptions> = {
    force: {
      name: 'cose',
      padding: 10,
      nodeRepulsion: 4500,
      edgeElasticity: 100,
      gravity: 0.1,
    },
    circle: {
      name: 'circle',
      padding: 30,
    },
    grid: {
      name: 'grid',
      padding: 30,
      fit: true,
    },
    hierarchical: {
      name: 'dagre',
      padding: 30,
      rankDir: 'TB',
    },
  };
  
  return cy.layout(layouts[layout]).run();
}
```

#### 2.3.8 GraphView 组件 (pages/Console/components/OutputBox/GraphView.tsx)

**修改类型**: 新增

```typescript
import React, { useEffect, useRef } from 'react';
import cytoscape from 'cytoscape';
import { useGraphStore } from '@/stores/graph';
import { convertToCytoscapeElements, generateCytoscapeStyle } from '@/utils/cytoscapeConfig';
import { applyLayout } from '@/utils/graphLayout';
import { GraphToolbar } from '../GraphVisualization/GraphToolbar';
import { StylePanel } from '../GraphVisualization/StylePanel';
import { DetailPanel } from '../GraphVisualization/DetailPanel';
import styles from './index.module.less';

interface GraphViewProps {
  data: any[];
}

export const GraphView: React.FC<GraphViewProps> = ({ data }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const cyRef = useRef<cytoscape.Core | null>(null);
  
  const {
    layout,
    nodeStyles,
    edgeStyles,
    setGraphData,
    selectNode,
    selectEdge,
    showDetail,
  } = useGraphStore();
  
  useEffect(() => {
    if (!containerRef.current || !data) return;
    
    // 解析数据为图数据
    const graphData = parseQueryResultToGraph(data);
    setGraphData(graphData);
    
    // 初始化 Cytoscape
    const cy = cytoscape({
      container: containerRef.current,
      elements: convertToCytoscapeElements(graphData),
      style: generateCytoscapeStyle({ nodes: nodeStyles, edges: edgeStyles }),
      layout: { name: 'cose' },
      minZoom: 0.1,
      maxZoom: 3,
    });
    
    cyRef.current = cy;
    
    // 应用布局
    applyLayout(cy, layout);
    
    // 事件监听
    cy.on('tap', 'node', (evt) => {
      const node = evt.target;
      selectNode(node.id(), evt.originalEvent?.ctrlKey || evt.originalEvent?.metaKey);
      showDetail({
        id: node.id(),
        tag: node.data('_tag'),
        properties: node.data(),
      });
    });
    
    cy.on('tap', 'edge', (evt) => {
      const edge = evt.target;
      selectEdge(edge.id(), evt.originalEvent?.ctrlKey || evt.originalEvent?.metaKey);
      showDetail({
        id: edge.id(),
        type: edge.data('_type'),
        source: edge.data('source'),
        target: edge.data('target'),
        rank: edge.data('_rank'),
        properties: edge.data(),
      });
    });
    
    return () => {
      cy.destroy();
    };
  }, [data]);
  
  // 样式更新
  useEffect(() => {
    if (!cyRef.current) return;
    cyRef.current.style().clear();
    cyRef.current.style(generateCytoscapeStyle({ nodes: nodeStyles, edges: edgeStyles }));
  }, [nodeStyles, edgeStyles]);
  
  // 布局更新
  useEffect(() => {
    if (!cyRef.current) return;
    applyLayout(cyRef.current, layout);
  }, [layout]);
  
  return (
    <div className={styles.graphView}>
      <div className={styles.graphContainer} ref={containerRef} />
      <GraphToolbar cy={cyRef.current} />
      <StylePanel />
      <DetailPanel />
    </div>
  );
};

// 解析查询结果为图数据
function parseQueryResultToGraph(data: any[]): GraphData {
  const nodes: GraphNode[] = [];
  const edges: GraphEdge[] = [];
  const nodeIds = new Set<string>();
  
  data.forEach(row => {
    // 解析节点
    if (row._verticesParsedList) {
      row._verticesParsedList.forEach((v: any) => {
        if (!nodeIds.has(v.vid)) {
          nodeIds.add(v.vid);
          nodes.push({
            id: v.vid,
            tag: v.tags?.[0] || 'unknown',
            properties: v.properties || {},
          });
        }
      });
    }
    
    // 解析边
    if (row._edgesParsedList) {
      row._edgesParsedList.forEach((e: any) => {
        edges.push({
          id: `${e.edgeName}_${e.srcID}_${e.dstID}_${e.rank}`,
          type: e.edgeName,
          source: e.srcID,
          target: e.dstID,
          rank: e.rank,
          properties: e.properties || {},
        });
      });
    }
  });
  
  return { nodes, edges };
}
```

#### 2.3.9 独立图可视化页面 (pages/GraphVisualization/index.tsx)

**修改类型**: 新增

```typescript
import React, { useState } from 'react';
import { Input, Button, Select } from 'antd';
import { useGraphStore } from '@/stores/graph';
import { GraphCanvas } from './components/GraphCanvas';
import { GraphToolbar } from './components/GraphToolbar';
import { StylePanel } from './components/StylePanel';
import { DetailPanel } from './components/DetailPanel';
import { TemplateSelector } from './components/TemplateSelector';
import { queryService } from '@/services/query';
import styles from './index.module.less';

const { TextArea } = Input;
const { Option } = Select;

const QUERY_TEMPLATES = [
  { label: 'Show all nodes (limit 50)', value: 'MATCH (n) RETURN n LIMIT 50' },
  { label: 'Show all relationships (limit 50)', value: 'MATCH ()-[r]->() RETURN r LIMIT 50' },
  { label: 'Show graph (limit 100)', value: 'MATCH (n)-[r]->(m) RETURN n, r, m LIMIT 100' },
];

export const GraphVisualizationPage: React.FC = () => {
  const [query, setQuery] = useState('MATCH (n)-[r]->(m) RETURN n, r, m LIMIT 50');
  const [loading, setLoading] = useState(false);
  const { setGraphData } = useGraphStore();
  
  const handleExecute = async () => {
    setLoading(true);
    try {
      const result = await queryService.exec({ gql: query });
      if (result.code === 0 && result.data) {
        const graphData = parseQueryResultToGraph(result.data);
        setGraphData(graphData);
      }
    } finally {
      setLoading(false);
    }
  };
  
  return (
    <div className={styles.graphPage}>
      <div className={styles.queryPanel}>
        <TextArea
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          rows={4}
          placeholder="Enter Cypher query..."
        />
        <div className={styles.queryActions}>
          <Select
            placeholder="Select template"
            onChange={(value) => setQuery(value)}
            style={{ width: 250 }}
          >
            {QUERY_TEMPLATES.map(t => (
              <Option key={t.value} value={t.value}>{t.label}</Option>
            ))}
          </Select>
          <Button type="primary" onClick={handleExecute} loading={loading}>
            Execute
          </Button>
        </div>
      </div>
      
      <div className={styles.graphArea}>
        <GraphCanvas />
        <GraphToolbar />
        <StylePanel />
        <DetailPanel />
      </div>
    </div>
  );
};
```

#### 2.3.10 路由配置更新 (config/routes.tsx)

**修改类型**: 修改

```typescript
// 添加图可视化页面路由
{
  path: '/graph',
  element: <GraphVisualizationPage />,
  title: 'Graph Visualization',
}
```

#### 2.3.11 侧边栏导航更新

**修改类型**: 修改

```typescript
// 添加 Graph 菜单项
const menuItems = [
  // ... 现有菜单项
  {
    key: '/graph',
    icon: <Icon type="icon-graph" />,
    label: 'Graph Visualization',
  },
];
```

---

## 3. 阶段 7 修改方案

### 3.1 现有架构分析

阶段 3 已完成：
- Space 列表展示和管理
- Schema 页面基础布局

阶段 7 需要在此基础上添加数据浏览功能。

### 3.2 目录结构变更

```
frontend/src/
├── pages/
│   └── DataBrowser/                     # 新增
│       ├── index.tsx
│       ├── index.module.less
│       ├── components/
│       │   ├── StatisticsPanel/         # 统计面板
│       │   │   ├── index.tsx
│       │   │   └── index.module.less
│       │   ├── VertexBrowser/           # 节点浏览
│       │   │   ├── index.tsx
│       │   │   ├── VertexTable.tsx
│       │   │   └── index.module.less
│       │   ├── EdgeBrowser/             # 边浏览
│       │   │   ├── index.tsx
│       │   │   ├── EdgeTable.tsx
│       │   │   └── index.module.less
│       │   ├── FilterPanel/             # 筛选面板
│       │   │   ├── index.tsx
│       │   │   └── index.module.less
│       │   └── DataDetailModal/         # 详情弹窗
│       │       ├── index.tsx
│       │       └── index.module.less
│       └── hooks/
│           └── useDataBrowser.ts
├── stores/
│   └── dataBrowser.ts                   # 新增
├── services/
│   └── dataBrowser.ts                   # 新增
└── types/
    └── dataBrowser.ts                   # 新增
```

### 3.3 具体修改点

#### 3.3.1 DataBrowser Store 新增 (stores/dataBrowser.ts)

**修改类型**: 新增

```typescript
import { create } from 'zustand';
import type { VertexData, EdgeData, FilterGroup, Statistics } from '@/types/dataBrowser';

interface DataBrowserState {
  // 当前视图
  activeTab: 'vertices' | 'edges';
  
  // 节点浏览状态
  selectedTag: string | null;
  vertices: VertexData[];
  vertexTotal: number;
  vertexPage: number;
  vertexPageSize: number;
  vertexSort: { field: string; order: 'asc' | 'desc' } | null;
  
  // 边浏览状态
  selectedEdgeType: string | null;
  edges: EdgeData[];
  edgeTotal: number;
  edgePage: number;
  edgePageSize: number;
  edgeSort: { field: string; order: 'asc' | 'desc' } | null;
  
  // 筛选状态
  filters: FilterGroup;
  filterPanelVisible: boolean;
  
  // 统计信息
  statistics: Statistics | null;
  
  // 详情弹窗
  detailModalVisible: boolean;
  detailData: VertexData | EdgeData | null;
  
  // Actions
  setActiveTab: (tab: 'vertices' | 'edges') => void;
  setSelectedTag: (tag: string | null) => void;
  setSelectedEdgeType: (type: string | null) => void;
  setVertices: (vertices: VertexData[], total: number) => void;
  setEdges: (edges: EdgeData[], total: number) => void;
  setVertexPage: (page: number) => void;
  setEdgePage: (page: number) => void;
  setVertexPageSize: (size: number) => void;
  setEdgePageSize: (size: number) => void;
  setVertexSort: (sort: { field: string; order: 'asc' | 'desc' } | null) => void;
  setEdgeSort: (sort: { field: string; order: 'asc' | 'desc' } | null) => void;
  setFilters: (filters: FilterGroup) => void;
  toggleFilterPanel: () => void;
  setStatistics: (stats: Statistics) => void;
  showDetail: (data: VertexData | EdgeData) => void;
  hideDetail: () => void;
}

export const useDataBrowserStore = create<DataBrowserState>((set) => ({
  activeTab: 'vertices',
  selectedTag: null,
  vertices: [],
  vertexTotal: 0,
  vertexPage: 1,
  vertexPageSize: 50,
  vertexSort: null,
  selectedEdgeType: null,
  edges: [],
  edgeTotal: 0,
  edgePage: 1,
  edgePageSize: 50,
  edgeSort: null,
  filters: { conditions: [], logic: 'AND' },
  filterPanelVisible: false,
  statistics: null,
  detailModalVisible: false,
  detailData: null,
  
  setActiveTab: (tab) => set({ activeTab: tab }),
  setSelectedTag: (tag) => set({ selectedTag: tag, vertexPage: 1 }),
  setSelectedEdgeType: (type) => set({ selectedEdgeType: type, edgePage: 1 }),
  setVertices: (vertices, total) => set({ vertices, vertexTotal: total }),
  setEdges: (edges, total) => set({ edges, edgeTotal: total }),
  setVertexPage: (page) => set({ vertexPage: page }),
  setEdgePage: (page) => set({ edgePage: page }),
  setVertexPageSize: (size) => set({ vertexPageSize: size, vertexPage: 1 }),
  setEdgePageSize: (size) => set({ edgePageSize: size, edgePage: 1 }),
  setVertexSort: (sort) => set({ vertexSort: sort }),
  setEdgeSort: (sort) => set({ edgeSort: sort }),
  setFilters: (filters) => set({ filters }),
  toggleFilterPanel: () => set((state) => ({ filterPanelVisible: !state.filterPanelVisible })),
  setStatistics: (statistics) => set({ statistics }),
  showDetail: (data) => set({ detailData: data, detailModalVisible: true }),
  hideDetail: () => set({ detailModalVisible: false, detailData: null }),
}));
```

#### 3.3.2 DataBrowser Service 新增 (services/dataBrowser.ts)

**修改类型**: 新增

```typescript
import { get } from '@/utils/http';
import type { 
  VertexData, 
  EdgeData, 
  FilterGroup, 
  Statistics,
  VertexListResponse,
  EdgeListResponse 
} from '@/types/dataBrowser';

export const dataBrowserService = {
  // 获取节点列表
  getVertices: (
    space: string,
    tag: string,
    page: number,
    pageSize: number,
    sort?: { field: string; order: 'asc' | 'desc' },
    filters?: FilterGroup
  ) => get<VertexListResponse>('/api/data/vertices', {
    params: {
      space,
      tag,
      page,
      pageSize,
      sortField: sort?.field,
      sortOrder: sort?.order,
      filters: filters ? JSON.stringify(filters) : undefined,
    },
  }),
  
  // 获取边列表
  getEdges: (
    space: string,
    type: string,
    page: number,
    pageSize: number,
    sort?: { field: string; order: 'asc' | 'desc' },
    filters?: FilterGroup
  ) => get<EdgeListResponse>('/api/data/edges', {
    params: {
      space,
      type,
      page,
      pageSize,
      sortField: sort?.field,
      sortOrder: sort?.order,
      filters: filters ? JSON.stringify(filters) : undefined,
    },
  }),
  
  // 获取统计信息
  getStatistics: (space: string) =>
    get<Statistics>(`/api/data/statistics?space=${space}`),
};
```

#### 3.3.3 类型定义新增 (types/dataBrowser.ts)

**修改类型**: 新增

```typescript
// 节点数据
export interface VertexData {
  id: string;
  tag: string;
  properties: Record<string, any>;
}

// 边数据
export interface EdgeData {
  id: string;
  type: string;
  src: string;
  dst: string;
  rank: number;
  properties: Record<string, any>;
}

// 筛选条件
export interface FilterCondition {
  property: string;
  operator: 'eq' | 'ne' | 'gt' | 'lt' | 'ge' | 'le' | 'contains' | 'startsWith' | 'endsWith';
  value: string | number | boolean;
}

// 筛选组
export interface FilterGroup {
  conditions: FilterCondition[];
  logic: 'AND' | 'OR';
}

// 统计信息
export interface Statistics {
  totalVertices: number;
  totalEdges: number;
  tagCount: number;
  edgeTypeCount: number;
  tagDistribution: { tag: string; count: number }[];
  edgeTypeDistribution: { type: string; count: number }[];
}

// API 响应类型
export interface VertexListResponse {
  data: VertexData[];
  total: number;
  page: number;
  pageSize: number;
}

export interface EdgeListResponse {
  data: EdgeData[];
  total: number;
  page: number;
  pageSize: number;
}
```

#### 3.3.4 DataBrowser 页面 (pages/DataBrowser/index.tsx)

**修改类型**: 新增

```typescript
import React, { useEffect } from 'react';
import { Tabs, Card } from 'antd';
import { useDataBrowserStore } from '@/stores/dataBrowser';
import { useSchemaStore } from '@/stores/schema';
import { StatisticsPanel } from './components/StatisticsPanel';
import { VertexBrowser } from './components/VertexBrowser';
import { EdgeBrowser } from './components/EdgeBrowser';
import { dataBrowserService } from '@/services/dataBrowser';
import styles from './index.module.less';

const { TabPane } = Tabs;

export const DataBrowserPage: React.FC = () => {
  const { currentSpace } = useSchemaStore();
  const { activeTab, setActiveTab, setStatistics } = useDataBrowserStore();
  
  // 加载统计信息
  const loadStatistics = async () => {
    if (!currentSpace) return;
    try {
      const stats = await dataBrowserService.getStatistics(currentSpace);
      setStatistics(stats);
    } catch (error) {
      console.error('Failed to load statistics:', error);
    }
  };
  
  useEffect(() => {
    loadStatistics();
    // 自动刷新统计信息
    const interval = setInterval(loadStatistics, 60000);
    return () => clearInterval(interval);
  }, [currentSpace]);
  
  return (
    <div className={styles.dataBrowser}>
      <Card className={styles.header}>
        <h2>Data Browser - {currentSpace || 'No Space Selected'}</h2>
      </Card>
      
      <StatisticsPanel onRefresh={loadStatistics} />
      
      <Card className={styles.content}>
        <Tabs activeKey={activeTab} onChange={(key) => setActiveTab(key as 'vertices' | 'edges')}>
          <TabPane tab="Vertices" key="vertices">
            <VertexBrowser />
          </TabPane>
          <TabPane tab="Edges" key="edges">
            <EdgeBrowser />
          </TabPane>
        </Tabs>
      </Card>
    </div>
  );
};
```

#### 3.3.5 VertexBrowser 组件 (pages/DataBrowser/components/VertexBrowser/index.tsx)

**修改类型**: 新增

```typescript
import React, { useEffect, useCallback } from 'react';
import { Select, Table, Button, Space } from 'antd';
import { useDataBrowserStore } from '@/stores/dataBrowser';
import { useSchemaStore } from '@/stores/schema';
import { dataBrowserService } from '@/services/dataBrowser';
import { FilterPanel } from '../FilterPanel';
import { DataDetailModal } from '../DataDetailModal';
import styles from './index.module.less';

const { Option } = Select;

export const VertexBrowser: React.FC = () => {
  const { currentSpace, tags } = useSchemaStore();
  const {
    selectedTag,
    vertices,
    vertexTotal,
    vertexPage,
    vertexPageSize,
    vertexSort,
    filters,
    filterPanelVisible,
    setSelectedTag,
    setVertices,
    setVertexPage,
    setVertexPageSize,
    setVertexSort,
    toggleFilterPanel,
    showDetail,
  } = useDataBrowserStore();
  
  // 加载节点数据
  const loadVertices = useCallback(async () => {
    if (!currentSpace || !selectedTag) return;
    try {
      const response = await dataBrowserService.getVertices(
        currentSpace,
        selectedTag,
        vertexPage,
        vertexPageSize,
        vertexSort || undefined,
        filters.conditions.length > 0 ? filters : undefined
      );
      setVertices(response.data, response.total);
    } catch (error) {
      console.error('Failed to load vertices:', error);
    }
  }, [currentSpace, selectedTag, vertexPage, vertexPageSize, vertexSort, filters]);
  
  useEffect(() => {
    loadVertices();
  }, [loadVertices]);
  
  // 生成表格列
  const generateColumns = () => {
    const tag = tags.find(t => t.name === selectedTag);
    if (!tag) return [];
    
    const columns = [
      {
        title: 'ID',
        dataIndex: 'id',
        key: 'id',
        sorter: true,
        fixed: 'left',
      },
      ...tag.properties.map(prop => ({
        title: prop.name,
        dataIndex: ['properties', prop.name],
        key: prop.name,
        sorter: true,
      })),
      {
        title: 'Actions',
        key: 'actions',
        fixed: 'right',
        render: (_: any, record: VertexData) => (
          <Button type="link" onClick={() => showDetail(record)}>
            View Details
          </Button>
        ),
      },
    ];
    
    return columns;
  };
  
  // 处理表格变化（分页、排序）
  const handleTableChange = (pagination: any, _filters: any, sorter: any) => {
    setVertexPage(pagination.current);
    setVertexPageSize(pagination.pageSize);
    if (sorter.field) {
      setVertexSort({
        field: sorter.field,
        order: sorter.order === 'ascend' ? 'asc' : 'desc',
      });
    } else {
      setVertexSort(null);
    }
  };
  
  return (
    <div className={styles.vertexBrowser}>
      <div className={styles.toolbar}>
        <Space>
          <Select
            placeholder="Select Tag"
            value={selectedTag}
            onChange={setSelectedTag}
            style={{ width: 200 }}
          >
            {tags.map(tag => (
              <Option key={tag.name} value={tag.name}>{tag.name}</Option>
            ))}
          </Select>
          <Button onClick={toggleFilterPanel}>
            Filter {filters.conditions.length > 0 && `(${filters.conditions.length})`}
          </Button>
          <Button onClick={loadVertices}>Refresh</Button>
        </Space>
      </div>
      
      {filterPanelVisible && (
        <FilterPanel
          properties={tags.find(t => t.name === selectedTag)?.properties || []}
          filters={filters}
          onChange={(newFilters) => useDataBrowserStore.setState({ filters: newFilters })}
          onApply={() => {
            setVertexPage(1);
            loadVertices();
          }}
        />
      )}
      
      <Table
        columns={generateColumns()}
        dataSource={vertices}
        rowKey="id"
        pagination={{
          current: vertexPage,
          pageSize: vertexPageSize,
          total: vertexTotal,
          showSizeChanger: true,
          pageSizeOptions: ['20', '50', '100'],
        }}
        onChange={handleTableChange}
        scroll={{ x: true }}
      />
      
      <DataDetailModal />
    </div>
  );
};
```

#### 3.3.6 路由配置更新 (config/routes.tsx)

**修改类型**: 修改

```typescript
// 添加数据浏览页面路由
{
  path: '/data-browser',
  element: <DataBrowserPage />,
  title: 'Data Browser',
}
```

#### 3.3.7 侧边栏导航更新

**修改类型**: 修改

```typescript
// 添加 Data Browser 菜单项
const menuItems = [
  // ... 现有菜单项
  {
    key: '/data-browser',
    icon: <Icon type="icon-table" />,
    label: 'Data Browser',
  },
];
```

---

## 4. 依赖安装

### 4.1 阶段 6 依赖

```bash
# Cytoscape.js 及其布局插件
npm install cytoscape cytoscape-cose-base cytoscape-dagre

# 类型定义
npm install -D @types/cytoscape
```

### 4.2 阶段 7 依赖

阶段 7 主要使用 Ant Design 已有组件，无需额外依赖。

---

## 5. 复用组件清单

### 5.1 从 Nebula Studio 复用

| 组件 | 来源 | 修改内容 |
|------|------|----------|
| ColorPicker | components/ColorPicker | 直接使用 |
| parseData.ts | utils/parseData.ts | 适配 GraphDB 数据结构 |
| fetch.ts | utils/fetch.ts | 适配 GraphDB API |

### 5.2 项目内复用

| 组件 | 来源 | 用途 |
|------|------|------|
| EmptyTableTip | components/common/EmptyTableTip | 空数据提示 |
| Icon | components/common/Icon | 图标展示 |
| http.ts | utils/http.ts | HTTP 请求 |

---

## 6. 测试策略

### 6.1 单元测试

- Graph Canvas 渲染测试
- 数据解析函数测试
- Store 状态管理测试
- 筛选逻辑测试

### 6.2 集成测试

- 图可视化与查询控制台集成
- 数据浏览与 Schema 管理集成
- 分页和排序功能测试

### 6.3 E2E 测试

- 完整的图可视化流程
- 数据浏览和筛选流程

---

## 7. 交付物清单

### 7.1 阶段 6 交付物

- [ ] Graph Store (stores/graph.ts)
- [ ] Graph Service (services/graph.ts)
- [ ] 类型定义扩展 (types/graph.ts)
- [ ] Cytoscape 配置工具 (utils/cytoscapeConfig.ts)
- [ ] 布局算法工具 (utils/graphLayout.ts)
- [ ] GraphView 组件
- [ ] GraphCanvas 组件
- [ ] GraphToolbar 组件
- [ ] StylePanel 组件
- [ ] DetailPanel 组件
- [ ] 独立图可视化页面
- [ ] 路由和导航更新

### 7.2 阶段 7 交付物

- [ ] DataBrowser Store (stores/dataBrowser.ts)
- [ ] DataBrowser Service (services/dataBrowser.ts)
- [ ] 类型定义 (types/dataBrowser.ts)
- [ ] DataBrowser 页面
- [ ] StatisticsPanel 组件
- [ ] VertexBrowser 组件
- [ ] EdgeBrowser 组件
- [ ] FilterPanel 组件
- [ ] DataDetailModal 组件
- [ ] 路由和导航更新

---

## 8. 风险与缓解

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| Cytoscape.js 性能瓶颈 | 高 | 中 | 限制节点数量；实现虚拟渲染 |
| 大数据集分页性能 | 中 | 中 | 使用游标分页；后端优化 |
| 筛选条件复杂度过高 | 低 | 中 | 限制筛选条件数量 |
| API 响应延迟 | 中 | 中 | 实现加载状态；添加缓存 |

---

**文档结束**
