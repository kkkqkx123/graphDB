# 阶段四：Schema 可视化实施方案

## 1. 概述

本阶段实现 Schema 可视化功能，通过图形化方式展示 Space 中的 Tag、Edge 及其关系，帮助用户直观理解数据模型结构。

## 2. 功能需求

### 2.1 功能清单

| 功能点 | 描述 | 优先级 |
|--------|------|--------|
| 图形化展示 | 使用节点和边展示 Tag 和 Edge 关系 | 高 |
| 自动布局 | 自动计算节点位置，避免重叠 | 高 |
| 缩放平移 | 支持画布缩放和拖拽平移 | 中 |
| 节点详情 | 点击节点查看 Tag/Edge 详细信息 | 中 |
| 数据采样 | 基于真实数据生成可视化 | 中 |
| 导出图片 | 将可视化结果导出为图片 | 低 |

### 2.2 界面设计

```
+----------------------------------------------------------+
|  Schema Visualization: production_space    [Refresh] [Export]    |
+----------------------------------------------------------+
|                                                          |
|    +--------+         +--------+                        |
|    | Person |         |Company |                        |
|    |--------|         |--------|                        |
|    | name   |         | name   |                        |
|    | age    |         | founded|                        |
|    | email  |         +--------+                        |
|    +--------+              |                             |
|         | WORKS_AT         |                             |
|         | (start_date)     |                             |
|         v                  |                             |
|    +--------+              |                             |
|    | Person |              |                             |
|    +--------+              |                             |
|                                                          |
|  Legend:  [Tag]  [Edge]  [Selected]                     |
+----------------------------------------------------------+
|  Zoom: [-] 100% [+]    Pan: Drag canvas                 |
+----------------------------------------------------------+

节点详情面板:
+----------------------------------------------------------+
|  Person (Tag)                                    [Close] |
+----------------------------------------------------------+
|  Properties:                                             |
|  +----------------+------------+----------+              |
|  | Name           | Type       | Nullable |              |
|  +----------------+------------+----------+              |
|  | name           | STRING     | YES      |              |
|  | age            | INT64      | NO       |              |
|  | email          | STRING     | YES      |              |
|  +----------------+------------+----------+              |
|                                                          |
|  TTL: duration=0, col=""                                 |
|  Comment: Person entity                                  |
+----------------------------------------------------------+
```

## 3. 技术方案

### 3.1 技术选型

| 组件 | 选择 | 原因 |
|------|------|------|
| 图形引擎 | Cytoscape.js | 已在项目中使用，功能强大 |
| 布局算法 | Cytoscape.js 内置 | 支持多种布局算法 |
| 样式 | CSS + Less | 与项目一致 |

### 3.2 目录结构

```
frontend/src/pages/Schema/
├── SchemaVisualization/         # 新增：Schema 可视化页面
│   ├── components/
│   │   ├── GraphCanvas.tsx      # 图形画布组件
│   │   ├── NodeDetailPanel.tsx  # 节点详情面板
│   │   ├── ZoomControls.tsx     # 缩放控制
│   │   └── Legend.tsx           # 图例组件
│   ├── hooks/
│   │   └── useSchemaGraph.ts    # Schema 图数据处理 Hook
│   ├── utils/
│   │   └── graphLayout.ts       # 布局算法
│   ├── index.module.less
│   └── index.tsx
```

### 3.3 组件设计

#### 3.3.1 SchemaVisualization 主组件

```typescript
// frontend/src/pages/Schema/SchemaVisualization/index.tsx

import React, { useEffect, useState, useCallback, useRef } from 'react';
import { Button, Spin, message, Empty } from 'antd';
import { ReloadOutlined, ExportOutlined } from '@ant-design/icons';
import { useSchemaStore } from '@/stores/schema';
import { useGraphStore } from '@/stores/graph';
import GraphCanvas from './components/GraphCanvas';
import NodeDetailPanel from './components/NodeDetailPanel';
import ZoomControls from './components/ZoomControls';
import Legend from './components/Legend';
import { buildSchemaGraph } from './utils/graphBuilder';
import styles from './index.module.less';

const SchemaVisualization: React.FC = () => {
  const { currentSpace, tags, edgeTypes, fetchTags, fetchEdgeTypes } = useSchemaStore();
  const { initGraph, destroyGraph } = useGraphStore();
  
  const [loading, setLoading] = useState(false);
  const [graphData, setGraphData] = useState<GraphData | null>(null);
  const [selectedNode, setSelectedNode] = useState<NodeData | null>(null);
  const canvasRef = useRef<HTMLDivElement>(null);

  // 加载 Schema 数据并构建图
  const loadSchemaGraph = useCallback(async () => {
    if (!currentSpace) {
      message.warning('Please select a space first');
      return;
    }

    setLoading(true);
    try {
      // 获取 Tags 和 Edges
      await Promise.all([
        fetchTags(currentSpace),
        fetchEdgeTypes(currentSpace),
      ]);

      // 构建图数据
      const data = buildSchemaGraph(tags, edgeTypes);
      setGraphData(data);
    } catch (err) {
      message.error('Failed to load schema visualization');
    } finally {
      setLoading(false);
    }
  }, [currentSpace, fetchTags, fetchEdgeTypes, tags, edgeTypes]);

  useEffect(() => {
    loadSchemaGraph();
    return () => {
      destroyGraph();
    };
  }, [loadSchemaGraph, destroyGraph]);

  // 处理节点点击
  const handleNodeClick = useCallback((node: NodeData) => {
    setSelectedNode(node);
  }, []);

  // 导出图片
  const handleExport = useCallback(() => {
    const canvas = canvasRef.current?.querySelector('canvas');
    if (canvas) {
      const link = document.createElement('a');
      link.download = `${currentSpace}_schema.png`;
      link.href = canvas.toDataURL('image/png');
      link.click();
    }
  }, [currentSpace]);

  if (!currentSpace) {
    return (
      <Empty description="Please select a space to view schema visualization" />
    );
  }

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h2>Schema Visualization: {currentSpace}</h2>
        <div className={styles.actions}>
          <Button
            icon={<ReloadOutlined />}
            onClick={loadSchemaGraph}
            loading={loading}
          >
            Refresh
          </Button>
          <Button
            type="primary"
            icon={<ExportOutlined />}
            onClick={handleExport}
          >
            Export
          </Button>
        </div>
      </div>

      <div className={styles.content}>
        <Spin spinning={loading} tip="Loading schema visualization...">
          <div className={styles.canvasWrapper} ref={canvasRef}>
            {graphData && (
              <GraphCanvas
                data={graphData}
                onNodeClick={handleNodeClick}
              />
            )}
          </div>
        </Spin>

        <div className={styles.sidebar}>
          <Legend />
          {selectedNode && (
            <NodeDetailPanel
              node={selectedNode}
              onClose={() => setSelectedNode(null)}
            />
          )}
        </div>
      </div>

      <ZoomControls />
    </div>
  );
};

export default SchemaVisualization;
```

#### 3.3.2 GraphCanvas 组件

```typescript
// frontend/src/pages/Schema/SchemaVisualization/components/GraphCanvas.tsx

import React, { useEffect, useRef } from 'react';
import cytoscape from 'cytoscape';
import dagre from 'cytoscape-dagre';
import { useGraphStore } from '@/stores/graph';
import { getCytoscapeConfig } from '@/utils/cytoscapeConfig';
import styles from './index.module.less';

// 注册布局插件
cytoscape.use(dagre);

interface GraphCanvasProps {
  data: GraphData;
  onNodeClick: (node: NodeData) => void;
}

const GraphCanvas: React.FC<GraphCanvasProps> = ({ data, onNodeClick }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const cyRef = useRef<cytoscape.Core | null>(null);
  const { setCy } = useGraphStore();

  useEffect(() => {
    if (!containerRef.current) return;

    // 初始化 Cytoscape
    const cy = cytoscape({
      container: containerRef.current,
      elements: data.elements,
      style: getSchemaGraphStyle(),
      layout: {
        name: 'dagre',
        rankDir: 'TB',
        nodeSep: 80,
        edgeSep: 40,
        rankSep: 100,
        padding: 20,
      },
      minZoom: 0.2,
      maxZoom: 3,
      wheelSensitivity: 0.3,
    });

    cyRef.current = cy;
    setCy(cy);

    // 节点点击事件
    cy.on('tap', 'node', (evt) => {
      const node = evt.target;
      onNodeClick({
        id: node.id(),
        type: node.data('type'),
        name: node.data('name'),
        properties: node.data('properties'),
        ...node.data(),
      });
    });

    // 空白处点击取消选择
    cy.on('tap', (evt) => {
      if (evt.target === cy) {
        onNodeClick(null as any);
      }
    });

    return () => {
      cy.destroy();
      cyRef.current = null;
      setCy(null);
    };
  }, [data, onNodeClick, setCy]);

  return (
    <div
      ref={containerRef}
      className={styles.graphCanvas}
      style={{ width: '100%', height: '100%' }}
    />
  );
};

// Schema 图样式配置
const getSchemaGraphStyle = (): cytoscape.Stylesheet[] => [
  {
    selector: 'node',
    style: {
      'background-color': '#fff',
      'border-width': 2,
      'border-color': '#1890ff',
      'width': 120,
      'height': 'label',
      'padding': 10,
      'shape': 'roundrectangle',
      'label': 'data(name)',
      'text-valign': 'top',
      'text-halign': 'center',
      'font-size': 14,
      'font-weight': 'bold',
      'color': '#1890ff',
      'text-margin-y': 8,
    },
  },
  {
    selector: 'node[type="tag"]',
    style: {
      'border-color': '#52c41a',
      'color': '#52c41a',
    },
  },
  {
    selector: 'node[type="edge"]',
    style: {
      'border-color': '#faad14',
      'color': '#faad14',
      'shape': 'diamond',
    },
  },
  {
    selector: 'node:selected',
    style: {
      'border-width': 4,
      'border-color': '#f5222d',
      'shadow-blur': 10,
      'shadow-color': '#f5222d',
    },
  },
  {
    selector: 'edge',
    style: {
      'width': 2,
      'line-color': '#999',
      'target-arrow-color': '#999',
      'target-arrow-shape': 'triangle',
      'curve-style': 'bezier',
      'label': 'data(name)',
      'font-size': 12,
      'color': '#666',
      'text-background-color': '#fff',
      'text-background-opacity': 1,
      'text-background-padding': 3,
    },
  },
  {
    selector: 'edge[type="relationship"]',
    style: {
      'line-color': '#1890ff',
      'target-arrow-color': '#1890ff',
      'line-style': 'solid',
    },
  },
];

export default GraphCanvas;
```

#### 3.3.3 图数据构建器

```typescript
// frontend/src/pages/Schema/SchemaVisualization/utils/graphBuilder.ts

import type { Tag, EdgeType } from '@/types/schema';

export interface GraphNode {
  data: {
    id: string;
    type: 'tag' | 'edge';
    name: string;
    properties: Array<{ name: string; type: string }>;
    [key: string]: any;
  };
}

export interface GraphEdge {
  data: {
    id: string;
    source: string;
    target: string;
    type: 'relationship';
    name: string;
    [key: string]: any;
  };
}

export interface GraphData {
  elements: (GraphNode | GraphEdge)[];
}

/**
 * 构建 Schema 图数据
 * 
 * 策略：
 * 1. 每个 Tag 作为一个节点
 * 2. 每个 EdgeType 作为一个节点
 * 3. 根据 EdgeType 的名称推断关系（简化版）
 * 4. 或者通过采样数据获取真实的 src/dst Tag 关系
 */
export const buildSchemaGraph = (
  tags: Tag[],
  edgeTypes: EdgeType[],
  sampleData?: SampleData
): GraphData => {
  const elements: (GraphNode | GraphEdge)[] = [];

  // 添加 Tag 节点
  tags.forEach(tag => {
    elements.push({
      data: {
        id: `tag_${tag.name}`,
        type: 'tag',
        name: tag.name,
        properties: tag.properties.map(p => ({
          name: p.name,
          type: p.type,
        })),
        comment: tag.comment,
      },
    });
  });

  // 添加 Edge 节点
  edgeTypes.forEach(edge => {
    elements.push({
      data: {
        id: `edge_${edge.name}`,
        type: 'edge',
        name: edge.name,
        properties: edge.properties.map(p => ({
          name: p.name,
          type: p.type,
        })),
        comment: edge.comment,
      },
    });
  });

  // 添加关系边
  // 如果有采样数据，使用真实关系
  if (sampleData) {
    sampleData.edges.forEach((edge, index) => {
      const srcTags = sampleData.vidToTags[edge.src];
      const dstTags = sampleData.vidToTags[edge.dst];

      srcTags?.forEach(srcTag => {
        dstTags?.forEach(dstTag => {
          elements.push({
            data: {
              id: `rel_${index}_${srcTag}_${dstTag}`,
              source: `tag_${srcTag}`,
              target: `edge_${edge.name}`,
              type: 'relationship',
              name: '',
            },
          });
          elements.push({
            data: {
              id: `rel_${index}_${dstTag}_${srcTag}`,
              source: `edge_${edge.name}`,
              target: `tag_${dstTag}`,
              type: 'relationship',
              name: '',
            },
          });
        });
      });
    });
  } else {
    // 简化版：Edge 连接到所有 Tag（表示可能的连接）
    edgeTypes.forEach(edge => {
      tags.forEach(tag => {
        elements.push({
          data: {
            id: `rel_${edge.name}_${tag.name}`,
            source: `edge_${edge.name}`,
            target: `tag_${tag.name}`,
            type: 'relationship',
            name: '',
          },
        });
      });
    });
  }

  return { elements };
};
```

#### 3.3.4 NodeDetailPanel 组件

```typescript
// frontend/src/pages/Schema/SchemaVisualization/components/NodeDetailPanel.tsx

import React from 'react';
import { Card, Table, Tag, Button } from 'antd';
import { CloseOutlined } from '@ant-design/icons';
import styles from './index.module.less';

interface NodeDetailPanelProps {
  node: NodeData;
  onClose: () => void;
}

const NodeDetailPanel: React.FC<NodeDetailPanelProps> = ({ node, onClose }) => {
  const isTag = node.type === 'tag';

  const propertyColumns = [
    { title: 'Name', dataIndex: 'name', key: 'name' },
    { title: 'Type', dataIndex: 'type', key: 'type', render: (type: string) => (
      <Tag color="blue">{type}</Tag>
    )},
  ];

  return (
    <Card
      className={styles.detailPanel}
      title={
        <div className={styles.header}>
          <span>
            {node.name}
            <Tag color={isTag ? 'green' : 'orange'} style={{ marginLeft: 8 }}>
              {isTag ? 'Tag' : 'Edge'}
            </Tag>
          </span>
          <Button
            type="text"
            size="small"
            icon={<CloseOutlined />}
            onClick={onClose}
          />
        </div>
      }
    >
      {node.properties && node.properties.length > 0 && (
        <div className={styles.section}>
          <h4>Properties</h4>
          <Table
            dataSource={node.properties}
            columns={propertyColumns}
            pagination={false}
            size="small"
            rowKey="name"
          />
        </div>
      )}

      {node.comment && (
        <div className={styles.section}>
          <h4>Comment</h4>
          <p className={styles.comment}>{node.comment}</p>
        </div>
      )}

      {node.ttl && (
        <div className={styles.section}>
          <h4>TTL Configuration</h4>
          <p>Duration: {node.ttl.duration}s</p>
          <p>Column: {node.ttl.col}</p>
        </div>
      )}
    </Card>
  );
};

export default NodeDetailPanel;
```

## 4. 样式文件

```less
// frontend/src/pages/Schema/SchemaVisualization/index.module.less

.container {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 16px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;

  h2 {
    margin: 0;
  }

  .actions {
    display: flex;
    gap: 8px;
  }
}

.content {
  display: flex;
  flex: 1;
  gap: 16px;
  overflow: hidden;
}

.canvasWrapper {
  flex: 1;
  background: #f5f5f5;
  border-radius: 8px;
  overflow: hidden;
}

.sidebar {
  width: 300px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}
```

## 5. 实现步骤

### 步骤 1: 创建目录结构 (0.5 天)

```bash
mkdir -p frontend/src/pages/Schema/SchemaVisualization/{components,hooks,utils}
```

### 步骤 2: 实现核心组件 (3-4 天)

1. 实现 `graphBuilder.ts` 数据构建器
2. 实现 `GraphCanvas.tsx` 画布组件
3. 实现 `NodeDetailPanel.tsx` 详情面板
4. 实现 `ZoomControls.tsx` 和 `Legend.tsx`
5. 实现主页面 `index.tsx`

### 步骤 3: 添加路由 (0.5 天)

```typescript
// frontend/src/config/routes.tsx

{
  path: '/schema/visualization',
  element: <SchemaVisualization />,
}
```

### 步骤 4: 集成到 Schema 导航 (0.5 天)

在 Schema 页面添加可视化入口

### 步骤 5: 测试与优化 (2 天)

1. 测试不同 Schema 的展示效果
2. 优化布局算法
3. 性能优化

## 6. 注意事项

1. **性能考虑**: 对于大型 Schema，需要考虑懒加载或分层展示
2. **布局算法**: dagre 适合层次结构，可考虑 force 布局作为备选
3. **交互体验**: 添加适当的动画和过渡效果
4. **响应式设计**: 确保在不同屏幕尺寸下正常显示

## 7. 参考文档

- [总体分析文档](./01_schema_analysis.md)
- [阶段三：DDL 导出功能](./04_phase3_ddl_export.md)
- [Cytoscape.js 官方文档](https://js.cytoscape.org/)
