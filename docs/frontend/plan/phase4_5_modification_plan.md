# GraphDB 前端阶段 4、5 修改方案文档

**文档版本**: v1.0
**创建日期**: 2026-03-29
**最后更新**: 2026-03-29
**适用范围**: 阶段 4 (Schema 管理 - Tag/Edge)、阶段 5 (Schema 管理 - 索引)

---

## 1. 文档概述

### 1.1 目的

本文档基于已完成的阶段 1-3 实现，结合 [prd_phase4.md](./prd_phase4.md) 和 [prd_phase5.md](./prd_phase5.md) 的详细需求，提出阶段 4 和阶段 5 的具体修改方案，包括：

- 对现有代码的修改点
- 新增组件和模块
- 接口和类型定义调整
- 与阶段 3 的集成方案

### 1.2 参考文档

- [prd_phase4.md](./prd_phase4.md) - 阶段 4 详细 PRD
- [prd_phase5.md](./prd_phase5.md) - 阶段 5 详细 PRD
- [prd_phase3.md](./prd_phase3.md) - 阶段 3 PRD（前置依赖）
- [directory_structure.md](../architecture/directory_structure.md) - 目录结构设计
- [tech_stack.md](../architecture/tech_stack.md) - 技术栈文档

---

## 2. 阶段 4 修改方案

### 2.1 现有架构分析

阶段 3 已完成：
- Space 列表展示和管理
- Schema 页面基础布局
- Schema Store 基础结构

阶段 4 需要在此基础上扩展 Tag 和 Edge 管理功能。

### 2.2 目录结构变更

```
frontend/src/
├── pages/
│   └── Schema/                          # 已存在，需扩展
│       ├── index.tsx                    # 已存在，需修改
│       ├── index.module.less            # 已存在
│       ├── SpaceList/                   # 已存在
│       ├── SpaceCreate/                 # 已存在
│       ├── TagList/                     # 新增
│       │   ├── index.tsx
│       │   └── index.module.less
│       ├── EdgeList/                    # 新增
│       │   ├── index.tsx
│       │   └── index.module.less
│       ├── SchemaVisualization/         # 新增
│       │   ├── index.tsx
│       │   └── index.module.less
│       └── components/                  # 已存在，需扩展
│           ├── TagForm/                 # 新增
│           │   ├── index.tsx
│           │   └── index.module.less
│           ├── EdgeForm/                # 新增
│           │   ├── index.tsx
│           │   └── index.module.less
│           └── PropertyEditor/          # 新增
│               ├── index.tsx
│               └── index.module.less
├── components/                          # 已存在，需扩展
│   └── business/                        # 已存在
│       └── PropertyEditor/              # 新增（可复用组件）
├── stores/
│   └── schema.ts                        # 已存在，需扩展
├── services/
│   └── schema.ts                        # 已存在，需扩展
└── types/
    └── schema.ts                        # 已存在，需扩展
```

### 2.3 具体修改点

#### 2.3.1 Schema 页面入口 (pages/Schema/index.tsx)

**修改类型**: 修改

**修改内容**:
1. 在现有 Tabs 中添加 "Tags"、"Edges"、"Visualization" 标签页
2. 集成 TagList、EdgeList、SchemaVisualization 组件
3. 添加当前 Space 上下文传递

```typescript
// 修改后的 Tabs 配置
const tabs = [
  { key: 'spaces', label: 'Spaces', component: SpaceList },
  { key: 'tags', label: 'Tags', component: TagList },
  { key: 'edges', label: 'Edges', component: EdgeList },
  { key: 'visualization', label: 'Visualization', component: SchemaVisualization },
];
```

#### 2.3.2 Schema Store 扩展 (stores/schema.ts)

**修改类型**: 扩展

**新增状态**:
```typescript
interface SchemaState {
  // 已存在
  currentSpace: string | null;
  spaces: Space[];
  
  // 新增
  tags: Tag[];
  edges: Edge[];
  tagLoading: boolean;
  edgeLoading: boolean;
  
  // 新增 Actions
  fetchTags: (space: string) => Promise<void>;
  fetchEdges: (space: string) => Promise<void>;
  createTag: (space: string, data: CreateTagData) => Promise<void>;
  updateTag: (space: string, name: string, data: UpdateTagData) => Promise<void>;
  deleteTag: (space: string, name: string) => Promise<void>;
  createEdge: (space: string, data: CreateEdgeData) => Promise<void>;
  updateEdge: (space: string, name: string, data: UpdateEdgeData) => Promise<void>;
  deleteEdge: (space: string, name: string) => Promise<void>;
}
```

#### 2.3.3 Schema Service 扩展 (services/schema.ts)

**修改类型**: 扩展

**新增 API**:
```typescript
export const schemaService = {
  // 已存在
  getSpaces: () => get<Space[]>('/api/schema/spaces'),
  createSpace: (data: CreateSpaceData) => post('/api/schema/spaces', data),
  deleteSpace: (name: string) => del(`/api/schema/spaces/${name}`),
  
  // 新增 - Tag API
  getTags: (space: string) => get<Tag[]>(`/api/schema/tags?space=${space}`),
  getTagDetail: (space: string, name: string) => get<Tag>(`/api/schema/tags/${name}?space=${space}`),
  createTag: (space: string, data: CreateTagData) => post('/api/schema/tags', { space, ...data }),
  updateTag: (space: string, name: string, data: UpdateTagData) => put(`/api/schema/tags/${name}`, { space, ...data }),
  deleteTag: (space: string, name: string) => del(`/api/schema/tags/${name}?space=${space}`),
  
  // 新增 - Edge API
  getEdges: (space: string) => get<Edge[]>(`/api/schema/edges?space=${space}`),
  getEdgeDetail: (space: string, name: string) => get<Edge>(`/api/schema/edges/${name}?space=${space}`),
  createEdge: (space: string, data: CreateEdgeData) => post('/api/schema/edges', { space, ...data }),
  updateEdge: (space: string, name: string, data: UpdateEdgeData) => put(`/api/schema/edges/${name}`, { space, ...data }),
  deleteEdge: (space: string, name: string) => del(`/api/schema/edges/${name}?space=${space}`),
};
```

#### 2.3.4 类型定义扩展 (types/schema.ts)

**修改类型**: 扩展

**新增类型**:
```typescript
// 数据类型枚举
export type DataType = 
  | 'STRING' 
  | 'INT64' 
  | 'DOUBLE' 
  | 'BOOL' 
  | 'DATETIME' 
  | 'DATE' 
  | 'TIME' 
  | 'TIMESTAMP';

// 属性定义
export interface Property {
  name: string;
  type: DataType;
  default_value?: string;
  nullable?: boolean;
}

// Tag 定义
export interface Tag {
  name: string;
  properties: Property[];
  created_at: string;
  comment?: string;
}

// Edge 定义
export interface Edge {
  name: string;
  properties: Property[];
  created_at: string;
  comment?: string;
}

// 创建 Tag 请求数据
export interface CreateTagData {
  name: string;
  properties: Omit<Property, 'nullable'>[];
}

// 更新 Tag 请求数据
export interface UpdateTagData {
  add_properties?: Omit<Property, 'nullable'>[];
  drop_properties?: string[];
}

// 创建 Edge 请求数据
export interface CreateEdgeData {
  name: string;
  properties: Omit<Property, 'nullable'>[];
}

// 更新 Edge 请求数据
export interface UpdateEdgeData {
  add_properties?: Omit<Property, 'nullable'>[];
  drop_properties?: string[];
}
```

### 2.4 新增组件清单

| 组件 | 路径 | 描述 | 复杂度 |
|------|------|------|--------|
| TagList | pages/Schema/TagList/ | Tag 列表页面 | 中 |
| EdgeList | pages/Schema/EdgeList/ | Edge 列表页面 | 中 |
| TagForm | pages/Schema/components/TagForm/ | Tag 创建/编辑表单 | 高 |
| EdgeForm | pages/Schema/components/EdgeForm/ | Edge 创建/编辑表单 | 高 |
| PropertyEditor | components/business/PropertyEditor/ | 属性编辑器（可复用） | 高 |
| SchemaVisualization | pages/Schema/SchemaVisualization/ | Schema 可视化 | 中 |

### 2.5 与阶段 3 的集成

#### 2.5.1 Space 上下文传递

阶段 3 的 SpaceList 需要支持选择当前 Space，并将选择传递给 TagList 和 EdgeList：

```typescript
// Schema Store 中已存在的 currentSpace 状态
// 在 TagList 和 EdgeList 中监听 currentSpace 变化
useEffect(() => {
  if (currentSpace) {
    fetchTags(currentSpace);
    fetchEdges(currentSpace);
  }
}, [currentSpace]);
```

#### 2.5.2 路由参数同步

可选：将当前 Space 同步到 URL 参数，支持刷新后保持状态：

```typescript
// 在 Schema 页面中
const [searchParams, setSearchParams] = useSearchParams();

useEffect(() => {
  const spaceFromUrl = searchParams.get('space');
  if (spaceFromUrl && spaceFromUrl !== currentSpace) {
    setCurrentSpace(spaceFromUrl);
  }
}, []);

const handleSpaceChange = (space: string) => {
  setCurrentSpace(space);
  setSearchParams({ space });
};
```

---

## 3. 阶段 5 修改方案

### 3.1 现有架构分析

阶段 4 完成后，Schema 管理页面已包含：
- Space 管理
- Tag 管理
- Edge 管理
- Schema 可视化

阶段 5 需要添加索引管理功能，与 Tag/Edge 管理紧密关联。

### 3.2 目录结构变更

```
frontend/src/
├── pages/
│   └── Schema/
│       ├── index.tsx                    # 修改 - 添加 Index 标签页
│       ├── IndexList/                   # 新增
│       │   ├── index.tsx
│       │   └── index.module.less
│       └── components/
│           ├── IndexForm/               # 新增
│           │   ├── index.tsx
│           │   └── index.module.less
│           └── RebuildConfirmModal/     # 新增
│               ├── index.tsx
│               └── index.module.less
├── components/
│   └── business/
│       └── IndexStatusBadge/            # 新增
│           ├── index.tsx
│           └── index.module.less
├── stores/
│   └── schema.ts                        # 扩展 - 添加索引状态
├── services/
│   └── schema.ts                        # 扩展 - 添加索引 API
└── types/
    └── schema.ts                        # 扩展 - 添加索引类型
```

### 3.3 具体修改点

#### 3.3.1 Schema 页面入口 (pages/Schema/index.tsx)

**修改类型**: 修改

**修改内容**:
在 Tabs 中添加 "Indexes" 标签页：

```typescript
const tabs = [
  { key: 'spaces', label: 'Spaces', component: SpaceList },
  { key: 'tags', label: 'Tags', component: TagList },
  { key: 'edges', label: 'Edges', component: EdgeList },
  { key: 'indexes', label: 'Indexes', component: IndexList },  // 新增
  { key: 'visualization', label: 'Visualization', component: SchemaVisualization },
];
```

#### 3.3.2 Schema Store 扩展 (stores/schema.ts)

**修改类型**: 扩展

**新增状态**:
```typescript
interface SchemaState {
  // 阶段 4 已存在
  tags: Tag[];
  edges: Edge[];
  
  // 新增
  indexes: Index[];
  indexLoading: boolean;
  indexStats: IndexStats | null;
  
  // 新增 Actions
  fetchIndexes: (space: string) => Promise<void>;
  fetchIndexStats: (space: string) => Promise<void>;
  createIndex: (space: string, data: CreateIndexData) => Promise<void>;
  deleteIndex: (space: string, name: string) => Promise<void>;
  rebuildIndex: (space: string, name: string) => Promise<void>;
  pollIndexStatus: (space: string, name: string) => Promise<void>;
}
```

#### 3.3.3 Schema Service 扩展 (services/schema.ts)

**修改类型**: 扩展

**新增 API**:
```typescript
export const schemaService = {
  // 阶段 4 已存在...
  
  // 新增 - Index API
  getIndexes: (space: string) => get<Index[]>(`/api/schema/indexes?space=${space}`),
  getIndexDetail: (space: string, name: string) => get<Index>(`/api/schema/indexes/${name}?space=${space}`),
  getIndexStatus: (space: string, name: string) => get<IndexStatusResponse>(`/api/schema/indexes/${name}/status?space=${space}`),
  createIndex: (space: string, data: CreateIndexData) => post('/api/schema/indexes', { space, ...data }),
  deleteIndex: (space: string, name: string) => del(`/api/schema/indexes/${name}?space=${space}`),
  rebuildIndex: (space: string, name: string) => post(`/api/schema/indexes/${name}/rebuild`, { space }),
  getIndexStats: (space: string) => get<IndexStats>(`/api/schema/index-stats?space=${space}`),
};
```

#### 3.3.4 类型定义扩展 (types/schema.ts)

**修改类型**: 扩展

**新增类型**:
```typescript
// 索引状态
export type IndexStatus = 'creating' | 'finished' | 'failed' | 'rebuilding';

// 索引定义
export interface Index {
  name: string;
  type: 'TAG' | 'EDGE';
  schemaName: string;
  properties: string[];
  status: IndexStatus;
  created_at: string;
  updated_at?: string;
  progress?: number;
  errorMessage?: string;
}

// 索引统计
export interface IndexStats {
  total: number;
  byType: {
    tag: number;
    edge: number;
  };
  byStatus: {
    creating: number;
    finished: number;
    failed: number;
    rebuilding: number;
  };
}

// 创建索引请求数据
export interface CreateIndexData {
  name: string;
  type: 'TAG' | 'EDGE';
  schemaName: string;
  properties: string[];
}

// 索引状态响应
export interface IndexStatusResponse {
  status: IndexStatus;
  progress?: number;
}
```

### 3.4 新增组件清单

| 组件 | 路径 | 描述 | 复杂度 |
|------|------|------|--------|
| IndexList | pages/Schema/IndexList/ | 索引列表页面 | 中 |
| IndexForm | pages/Schema/components/IndexForm/ | 索引创建表单 | 中 |
| IndexStatusBadge | components/business/IndexStatusBadge/ | 索引状态标签 | 低 |
| RebuildConfirmModal | pages/Schema/components/RebuildConfirmModal/ | 重建确认弹窗 | 低 |

### 3.5 与阶段 4 的集成

#### 3.5.1 属性选择器

索引创建表单需要选择 Tag/Edge 的属性，需要复用阶段 4 的 Tag/Edge 数据：

```typescript
// IndexForm 组件中
const { tags, edges, currentSpace } = useSchemaStore();

// 根据选择的类型（Tag/Edge）和名称，获取可用属性
const availableProperties = useMemo(() => {
  if (indexType === 'TAG') {
    const tag = tags.find(t => t.name === selectedSchema);
    return tag?.properties || [];
  } else {
    const edge = edges.find(e => e.name === selectedSchema);
    return edge?.properties || [];
  }
}, [indexType, selectedSchema, tags, edges]);
```

#### 3.5.2 状态轮询机制

索引创建和重建是异步操作，需要实现状态轮询：

```typescript
// stores/schema.ts 中
pollIndexStatus: async (space: string, name: string) => {
  const poll = async () => {
    const response = await schemaService.getIndexStatus(space, name);
    
    set((state) => ({
      indexes: state.indexes.map(idx =>
        idx.name === name 
          ? { ...idx, status: response.status, progress: response.progress }
          : idx
      )
    }));
    
    // 如果仍在创建或重建中，继续轮询
    if (response.status === 'creating' || response.status === 'rebuilding') {
      setTimeout(() => poll(), 5000);
    }
  };
  
  poll();
}
```

---

## 4. 复用组件分析

### 4.1 可复用的阶段 3 组件

| 组件 | 来源 | 复用方式 | 修改需求 |
|------|------|----------|----------|
| DeleteConfirmModal | Ant Design Modal | 直接使用 | 无 |
| DetailDrawer | Ant Design Drawer | 直接使用 | 无 |
| EmptyTableTip | components/common/ | 直接使用 | 无 |
| LoadingSpinner | Ant Design Spin | 直接使用 | 无 |

### 4.2 阶段 4 新增可复用组件

| 组件 | 复用位置 | 说明 |
|------|----------|------|
| PropertyEditor | TagForm, EdgeForm | 属性编辑器，阶段 4 内部复用 |
| SchemaForm (抽象) | TagForm, EdgeForm | 可抽象出通用的 Schema 表单逻辑 |

### 4.3 阶段 5 复用阶段 4 组件

| 组件 | 复用位置 | 说明 |
|------|----------|------|
| Property 选择逻辑 | IndexForm | 复用属性列表获取和展示 |
| DeleteConfirmModal | IndexList | 复用删除确认弹窗 |
| DetailDrawer | IndexList | 复用详情抽屉 |

---

## 5. 接口变更汇总

### 5.1 Store 接口变更

```typescript
// 阶段 3 -> 阶段 4 -> 阶段 5 的 Store 演进

interface SchemaState {
  // 阶段 3
  currentSpace: string | null;
  spaces: Space[];
  spaceLoading: boolean;
  
  // 阶段 4 新增
  tags: Tag[];
  edges: Edge[];
  tagLoading: boolean;
  edgeLoading: boolean;
  
  // 阶段 5 新增
  indexes: Index[];
  indexLoading: boolean;
  indexStats: IndexStats | null;
  
  // Actions...
}
```

### 5.2 Service 接口变更

| 阶段 | 新增 API | 数量 |
|------|----------|------|
| 阶段 3 | Space 相关 | 4 |
| 阶段 4 | Tag 相关 | 5 |
| 阶段 4 | Edge 相关 | 5 |
| 阶段 5 | Index 相关 | 7 |

---

## 6. 测试策略

### 6.1 单元测试新增

| 阶段 | 测试文件 | 覆盖率目标 |
|------|----------|------------|
| 阶段 4 | TagList.test.tsx | > 80% |
| 阶段 4 | EdgeList.test.tsx | > 80% |
| 阶段 4 | TagForm.test.tsx | > 85% |
| 阶段 4 | EdgeForm.test.tsx | > 85% |
| 阶段 4 | PropertyEditor.test.tsx | > 85% |
| 阶段 5 | IndexList.test.tsx | > 80% |
| 阶段 5 | IndexForm.test.tsx | > 85% |
| 阶段 5 | IndexStatusBadge.test.tsx | > 90% |

### 6.2 集成测试场景

| 阶段 | 测试场景 |
|------|----------|
| 阶段 4 | Space 切换 -> Tag/Edge 列表刷新 |
| 阶段 4 | 创建 Tag -> 列表更新 -> 详情查看 |
| 阶段 4 | 修改 Edge -> 属性增删 -> 验证提交 |
| 阶段 5 | 创建索引 -> 状态轮询 -> 完成检测 |
| 阶段 5 | 重建索引 -> 状态变化 -> 结果验证 |

---

## 7. 实施建议

### 7.1 阶段 4 实施顺序

1. **类型定义** (1h)
   - 扩展 types/schema.ts

2. **Service 层** (2h)
   - 扩展 services/schema.ts

3. **Store 层** (2h)
   - 扩展 stores/schema.ts

4. **基础组件** (4h)
   - PropertyEditor 组件

5. **Tag 管理** (8h)
   - TagList 页面
   - TagForm 组件

6. **Edge 管理** (6h)
   - EdgeList 页面
   - EdgeForm 组件

7. **Schema 可视化** (4h)
   - SchemaVisualization 页面

8. **集成测试** (4h)

**总计**: 约 31 小时 (4 天)

### 7.2 阶段 5 实施顺序

1. **类型定义** (0.5h)
   - 扩展 types/schema.ts

2. **Service 层** (1h)
   - 扩展 services/schema.ts

3. **Store 层** (2h)
   - 扩展 stores/schema.ts
   - 实现轮询逻辑

4. **基础组件** (2h)
   - IndexStatusBadge
   - RebuildConfirmModal

5. **索引管理** (6h)
   - IndexList 页面
   - IndexForm 组件

6. **集成测试** (2h)

**总计**: 约 13.5 小时 (2 天)

---

## 8. 风险评估

### 8.1 技术风险

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| PropertyEditor 复杂度 | 中 | 中 | 提前设计，分步骤实现 |
| 后端 API 不匹配 | 低 | 高 | 及时沟通，Mock 数据先行 |
| 状态管理复杂度 | 低 | 中 | 使用 Zustand 简化 |

### 8.2 进度风险

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| 阶段 4 超期 | 中 | 中 | Schema 可视化可作为可选功能 |
| 阶段 5 依赖延迟 | 低 | 低 | 可独立开发，Mock 数据测试 |

---

## 9. 附录

### 9.1 命名规范

| 类型 | 规范 | 示例 |
|------|------|------|
| 组件目录 | PascalCase | `TagList/`, `IndexForm/` |
| 组件文件 | index.tsx | `index.tsx` |
| 样式文件 | index.module.less | `index.module.less` |
| 类型定义 | PascalCase | `Tag`, `Index`, `Property` |
| Store 方法 | camelCase | `createTag`, `deleteIndex` |
| Service 方法 | camelCase | `getTags`, `rebuildIndex` |

### 9.2 文件清单

#### 阶段 4 新增文件 (11)
- `pages/Schema/TagList/index.tsx`
- `pages/Schema/TagList/index.module.less`
- `pages/Schema/EdgeList/index.tsx`
- `pages/Schema/EdgeList/index.module.less`
- `pages/Schema/SchemaVisualization/index.tsx`
- `pages/Schema/SchemaVisualization/index.module.less`
- `pages/Schema/components/TagForm/index.tsx`
- `pages/Schema/components/TagForm/index.module.less`
- `pages/Schema/components/EdgeForm/index.tsx`
- `pages/Schema/components/EdgeForm/index.module.less`
- `components/business/PropertyEditor/index.tsx`
- `components/business/PropertyEditor/index.module.less`

#### 阶段 4 修改文件 (4)
- `pages/Schema/index.tsx`
- `stores/schema.ts`
- `services/schema.ts`
- `types/schema.ts`

#### 阶段 5 新增文件 (7)
- `pages/Schema/IndexList/index.tsx`
- `pages/Schema/IndexList/index.module.less`
- `pages/Schema/components/IndexForm/index.tsx`
- `pages/Schema/components/IndexForm/index.module.less`
- `pages/Schema/components/RebuildConfirmModal/index.tsx`
- `components/business/IndexStatusBadge/index.tsx`
- `components/business/IndexStatusBadge/index.module.less`

#### 阶段 5 修改文件 (4)
- `pages/Schema/index.tsx`
- `stores/schema.ts`
- `services/schema.ts`
- `types/schema.ts`

---

**文档结束**
