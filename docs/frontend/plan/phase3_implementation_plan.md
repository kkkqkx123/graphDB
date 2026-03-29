# GraphDB 前端阶段3执行方案

**文档版本**: v1.0  
**创建日期**: 2026-03-29  
**对应PRD**: [prd_phase3.md](./prd_phase3.md)

---

## 1. 执行概览

### 1.1 目标

实现 Space（图空间）的完整管理功能，包括列表展示、创建、删除、详情查看和切换功能。Space 是 GraphDB 中数据组织的顶层容器，为后续的 Tag 和 Edge 管理奠定基础。

### 1.2 工期

预计 1 周

### 1.3 依赖

- 阶段 1 完成（基础框架和连接管理）
- 阶段 2 完成（查询控制台）- 可选，但推荐完成
- 后端 Schema API 可用（`/api/schema/spaces`）

### 1.4 交付物

- Space 列表页面
- Space 创建表单
- Space 详情查看
- Space 删除功能
- Space 统计信息展示
- Space 切换功能
- Schema 状态管理

---

## 2. 执行步骤

### 步骤1: 项目准备和工具函数实现

**任务**: 创建 Schema 相关的工具函数和服务

**操作**:
1. 更新 `src/utils/gql.ts` - 添加 Space 相关的 Cypher 查询生成函数
2. 创建 `src/services/schema.ts` - Schema API 服务
3. 更新 `src/utils/constant.ts` - 添加 Space 相关常量

**文件清单**:

| 文件 | 功能 | 参考来源 |
|------|------|----------|
| `utils/gql.ts` | 更新：添加 Space 查询生成 | nebula-studio `utils/gql.ts`（适配 Cypher） |
| `services/schema.ts` | 新建：Space/Schema API 服务 | 新建 |
| `utils/constant.ts` | 更新：添加 Vid Type 常量 | 新建 |

**Space 相关 Cypher 命令**:

```typescript
// utils/gql.ts

export const spaceGQL = {
  // 列出所有 Spaces
  listSpaces: () => 'SHOW SPACES',
  
  // 创建 Space
  createSpace: (params: CreateSpaceParams) => 
    `CREATE SPACE IF NOT EXISTS ${params.name} (vid_type = ${params.vidType}, partition_num = ${params.partitionNum}, replica_factor = ${params.replicaFactor})`,
  
  // 删除 Space
  dropSpace: (name: string) => `DROP SPACE IF EXISTS ${name}`,
  
  // 描述 Space
  describeSpace: (name: string) => `DESCRIBE SPACE ${name}`,
  
  // 使用 Space
  useSpace: (name: string) => `USE ${name}`,
  
  // 获取统计信息
  showStats: () => 'SHOW STATS',
};
```

**验收**:
- Cypher 查询生成函数单元测试通过
- API 服务可以正常调用

---

### 步骤2: 状态管理实现

**任务**: 实现 Schema 相关的状态管理

**操作**:
1. 创建 `src/stores/schema.ts` - Schema 状态管理

**Store 设计**:
```typescript
// stores/schema.ts
import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface Space {
  name: string;
  vidType: 'INT64' | 'FIXED_STRING(32)';
  partitionNum: number;
  replicaFactor: number;
  createdAt: string;
  vertexCount: number;
  edgeCount: number;
}

interface SchemaState {
  // Space 列表
  spaces: Space[];
  isLoadingSpaces: boolean;
  spacesError: string | null;
  
  // 当前 Space
  currentSpace: string | null;
  
  // Space 详情
  spaceDetails: Record<string, Space>;
  
  // Actions
  fetchSpaces: () => Promise<void>;
  createSpace: (params: CreateSpaceParams) => Promise<void>;
  deleteSpace: (name: string) => Promise<void>;
  setCurrentSpace: (name: string) => void;
  fetchSpaceStats: (name: string) => Promise<void>;
}

export const useSchemaStore = create<SchemaState>()(
  persist(
    (set, get) => ({
      spaces: [],
      isLoadingSpaces: false,
      spacesError: null,
      currentSpace: null,
      spaceDetails: {},
      
      fetchSpaces: async () => {
        set({ isLoadingSpaces: true, spacesError: null });
        try {
          const spaces = await schemaService.getSpaces();
          set({ spaces, isLoadingSpaces: false });
        } catch (error) {
          set({ spacesError: error.message, isLoadingSpaces: false });
        }
      },
      
      createSpace: async (params) => {
        await schemaService.createSpace(params);
        await get().fetchSpaces();
      },
      
      deleteSpace: async (name) => {
        await schemaService.deleteSpace(name);
        await get().fetchSpaces();
        if (get().currentSpace === name) {
          set({ currentSpace: null });
        }
      },
      
      setCurrentSpace: (name) => {
        set({ currentSpace: name });
      },
      
      fetchSpaceStats: async (name) => {
        const stats = await schemaService.getSpaceStats(name);
        set((state) => ({
          spaceDetails: {
            ...state.spaceDetails,
            [name]: { ...state.spaceDetails[name], ...stats },
          },
        }));
      },
    }),
    {
      name: 'schema-storage',
      partialize: (state) => ({ currentSpace: state.currentSpace }),
    }
  )
);
```

**持久化配置**:
- 当前 Space: localStorage, key=`schema-storage`

**验收**:
- Store 状态可正常读写
- 当前 Space 持久化正常

---

### 步骤3: Space 列表页面实现

**任务**: 实现 Space 列表页面

**操作**:
1. 创建 `src/pages/Schema/SpaceList/index.tsx`
2. 创建 `src/pages/Schema/SpaceList/index.module.less`

**页面功能**:
- Space 列表表格展示
- 显示基本信息（名称、Vid Type、分区数、副本数、创建时间）
- 显示统计信息（节点数、边数）
- 刷新按钮
- 创建 Space 按钮
- 操作按钮（查看详情、删除）

**表格列定义**:
```typescript
const columns: ColumnsType<Space> = [
  {
    title: 'Name',
    dataIndex: 'name',
    key: 'name',
    sorter: (a, b) => a.name.localeCompare(b.name),
  },
  {
    title: 'Vid Type',
    dataIndex: 'vidType',
    key: 'vidType',
  },
  {
    title: 'Partitions',
    dataIndex: 'partitionNum',
    key: 'partitionNum',
  },
  {
    title: 'Replicas',
    dataIndex: 'replicaFactor',
    key: 'replicaFactor',
  },
  {
    title: 'Vertices',
    dataIndex: 'vertexCount',
    key: 'vertexCount',
  },
  {
    title: 'Edges',
    dataIndex: 'edgeCount',
    key: 'edgeCount',
  },
  {
    title: 'Created At',
    dataIndex: 'createdAt',
    key: 'createdAt',
    render: (date) => formatDate(date),
  },
  {
    title: 'Actions',
    key: 'actions',
    render: (_, record) => (
      <Space>
        <Button onClick={() => showDetails(record)}>Details</Button>
        <Button danger onClick={() => confirmDelete(record)}>Delete</Button>
      </Space>
    ),
  },
];
```

**验收**:
- 列表显示正常
- 排序功能正常
- 刷新功能正常
- 空状态显示正常

---

### 步骤4: Space 创建表单实现

**任务**: 实现 Space 创建表单

**操作**:
1. 创建 `src/pages/Schema/components/SpaceCreateModal/index.tsx`
2. 创建 `src/pages/Schema/components/SpaceCreateModal/index.module.less`

**组件功能**:
- Modal 弹窗形式
- Space 名称输入（必填，验证命名规则）
- Vid Type 选择（下拉：INT64、FIXED_STRING(32)）
- 分区数输入（默认 100，正整数）
- 副本数输入（默认 1，正整数）
- 创建/取消按钮

**表单验证规则**:
```typescript
const validationRules = {
  name: [
    { required: true, message: 'Please input Space name' },
    { pattern: /^[a-zA-Z][a-zA-Z0-9_]*$/, message: 'Name must start with a letter and contain only alphanumeric characters and underscores' },
    { max: 64, message: 'Name must be less than 64 characters' },
  ],
  vidType: [
    { required: true, message: 'Please select Vid Type' },
  ],
  partitionNum: [
    { required: true, message: 'Please input partition number' },
    { type: 'number', min: 1, message: 'Partition number must be a positive integer' },
  ],
  replicaFactor: [
    { required: true, message: 'Please input replica factor' },
    { type: 'number', min: 1, message: 'Replica factor must be a positive integer' },
  ],
};
```

**组件接口**:
```typescript
interface SpaceCreateModalProps {
  visible: boolean;
  onCancel: () => void;
  onSuccess: () => void;
}
```

**验收**:
- 表单验证正常
- 创建成功刷新列表
- 创建失败显示错误
- 加载状态正常

---

### 步骤5: Space 详情模态框实现

**任务**: 实现 Space 详情查看模态框

**操作**:
1. 创建 `src/pages/Schema/components/SpaceDetailModal/index.tsx`
2. 创建 `src/pages/Schema/components/SpaceDetailModal/index.module.less`

**组件功能**:
- Modal 弹窗形式
- 分区域展示信息：
  - 基本信息：名称、创建时间
  - 配置信息：Vid Type、分区数、副本数
  - 统计信息：节点数、边数（带刷新按钮）

**组件接口**:
```typescript
interface SpaceDetailModalProps {
  visible: boolean;
  space: Space | null;
  onClose: () => void;
  onRefreshStats: (name: string) => void;
}
```

**验收**:
- 详情显示正常
- 统计刷新正常
- 关闭功能正常

---

### 步骤6: Space 删除确认实现

**任务**: 实现 Space 删除确认功能

**操作**:
1. 使用 Ant Design Modal.confirm 实现
2. 在 SpaceList 页面集成

**确认对话框内容**:
```typescript
const showDeleteConfirm = (space: Space) => {
  Modal.confirm({
    title: 'Delete Space',
    icon: <ExclamationCircleOutlined />,
    content: (
      <div>
        <p>Are you sure you want to delete Space &quot;{space.name}&quot;?</p>
        <p style={{ color: '#ff4d4f' }}>
          This action cannot be undone and all data in this Space will be lost.
        </p>
      </div>
    ),
    okText: 'Delete',
    okType: 'danger',
    cancelText: 'Cancel',
    onOk: async () => {
      await deleteSpace(space.name);
      message.success(`Space "${space.name}" deleted successfully`);
    },
  });
};
```

**验收**:
- 确认对话框显示正常
- 删除成功刷新列表
- 删除失败显示错误

---

### 步骤7: Space 选择器组件实现

**任务**: 实现 Space 选择器组件，用于在 Header 或 Sidebar 中切换 Space

**操作**:
1. 创建 `src/components/business/SpaceSelector/index.tsx`
2. 创建 `src/components/business/SpaceSelector/index.module.less`

**组件功能**:
- 下拉选择器形式
- 显示当前选中的 Space
- 列出所有可用 Space
- 切换时更新当前 Space

**组件接口**:
```typescript
interface SpaceSelectorProps {
  spaces: Space[];
  currentSpace: string | null;
  onChange: (spaceName: string) => void;
  loading?: boolean;
}
```

**使用示例**:
```typescript
<SpaceSelector
  spaces={spaces}
  currentSpace={currentSpace}
  onChange={setCurrentSpace}
  loading={isLoadingSpaces}
/>
```

**验收**:
- 选择器显示正常
- 切换功能正常
- 当前 Space 高亮正常

---

### 步骤8: Schema 页面集成

**任务**: 整合所有组件到 Schema 页面

**操作**:
1. 创建 `src/pages/Schema/index.tsx` - Schema 主页面
2. 创建 `src/pages/Schema/index.module.less` - 页面样式

**页面布局**:
```
+------------------+
|  Page Header     |
|  (Title + Create |
|   Button)        |
+------------------+
|                  |
|  Space List      |
|  Table           |
|                  |
+------------------+
```

**页面功能**:
- 页面标题和创建按钮
- Space 列表表格
- 创建模态框控制
- 详情模态框控制
- 删除确认控制

**路由配置**:
```typescript
// config/routes.tsx
{
  path: '/schema',
  element: <SchemaPage />,
  title: 'Schema管理'
}
```

**验收**:
- 页面布局正常
- 组件间通信正常
- 路由访问正常

---

### 步骤9: 侧边栏导航更新

**任务**: 更新侧边栏导航，添加 Schema 管理入口

**操作**:
1. 更新 `src/pages/MainPage/Sidebar/index.tsx`

**导航项**:
```typescript
const menuItems = [
  {
    key: '/console',
    icon: <ConsoleIcon />,
    label: '查询控制台'
  },
  {
    key: '/schema',
    icon: <SchemaIcon />,
    label: 'Schema管理',
    children: [
      { key: '/schema/spaces', label: 'Space管理' },
      { key: '/schema/tags', label: 'Tag管理' },
      { key: '/schema/edges', label: 'Edge管理' },
      { key: '/schema/indexes', label: '索引管理' },
    ]
  }
];
```

**验收**:
- 导航项显示正常
- 子菜单展开正常
- 点击跳转正常
- 当前项高亮正常

---

### 步骤10: Header 集成 Space 选择器

**任务**: 在 Header 中集成 Space 选择器

**操作**:
1. 更新 `src/pages/MainPage/Header/index.tsx`

**Header 布局更新**:
```
+--------------------------------------------------+
| Logo  |  Space Selector  |       |  Status | User |
+--------------------------------------------------+
```

**功能**:
- 在 Logo 右侧添加 Space 选择器
- 选择器显示当前 Space
- 切换 Space 时更新全局状态

**验收**:
- 选择器显示正常
- 切换功能正常
- 与 Space 列表页面同步

---

## 3. API 接口清单

| 接口 | 方法 | 用途 | 请求体 | 响应 |
|------|------|------|--------|------|
| `/api/schema/spaces` | GET | 获取 Space 列表 | - | `{ spaces: Space[] }` |
| `/api/schema/spaces` | POST | 创建 Space | `{ name, vidType, partitionNum, replicaFactor }` | `{ success: boolean }` |
| `/api/schema/spaces/{name}` | DELETE | 删除 Space | - | `{ success: boolean }` |
| `/api/schema/spaces/{name}/stats` | GET | 获取 Space 统计 | - | `{ vertexCount, edgeCount }` |
| `/api/query/execute` | POST | 执行 Cypher | `{ query, sessionId }` | 查询结果 |

**请求头**:
- `X-Session-ID`: 会话ID
- `Content-Type`: application/json

**错误码**:
| 错误码 | 说明 |
|--------|------|
| `SPACE_EXISTS` | Space 已存在 |
| `SPACE_NOT_FOUND` | Space 不存在 |
| `SPACE_IN_USE` | Space 正在使用中 |
| `INVALID_NAME` | 无效的 Space 名称 |
| `PERMISSION_DENIED` | 权限不足 |

---

## 4. 组件清单

### 页面组件

| 组件 | 路径 | 用途 | 优先级 |
|------|------|------|--------|
| Schema | `pages/Schema/index.tsx` | Schema 主页面 | P0 |
| SpaceList | `pages/Schema/SpaceList/` | Space 列表 | P0 |

### 业务组件

| 组件 | 路径 | 用途 | 优先级 |
|------|------|------|--------|
| SpaceCreateModal | `pages/Schema/components/SpaceCreateModal/` | 创建 Space 弹窗 | P0 |
| SpaceDetailModal | `pages/Schema/components/SpaceDetailModal/` | Space 详情弹窗 | P0 |
| SpaceSelector | `components/business/SpaceSelector/` | Space 选择器 | P0 |

### 公共组件（新增/更新）

| 组件 | 路径 | 用途 | 优先级 |
|------|------|------|--------|
| EmptyTableTip | `components/common/EmptyTableTip/` | 空表格提示 | P1 |

---

## 5. Store 清单

| Store | 路径 | 用途 |
|-------|------|------|
| schema | `stores/schema.ts` | Schema 状态（Space 列表、当前 Space、统计信息） |

**Store 状态结构**:
```typescript
interface SchemaState {
  // Space 列表
  spaces: Space[];
  isLoadingSpaces: boolean;
  spacesError: string | null;
  
  // 当前 Space
  currentSpace: string | null;
  
  // Space 详情
  spaceDetails: Record<string, Space>;
}
```

---

## 6. 依赖清单

### 生产依赖（无新增）

使用现有依赖即可：
- antd
- zustand
- axios

### 开发依赖（无新增）

使用现有开发依赖即可。

---

## 7. 目录结构

阶段 3 涉及的目录结构：

```
src/
├── pages/
│   └── Schema/
│       ├── index.tsx                    # Schema 主页面
│       ├── index.module.less            # 页面样式
│       ├── SpaceList/                   # Space 列表
│       │   ├── index.tsx
│       │   └── index.module.less
│       └── components/                  # 页面组件
│           ├── SpaceCreateModal/        # 创建 Space 弹窗
│           │   ├── index.tsx
│           │   └── index.module.less
│           └── SpaceDetailModal/        # Space 详情弹窗
│               ├── index.tsx
│               └── index.module.less
├── components/
│   └── business/
│       └── SpaceSelector/               # Space 选择器
│           ├── index.tsx
│           └── index.module.less
├── stores/
│   └── schema.ts                        # Schema 状态管理
├── services/
│   └── schema.ts                        # Schema API 服务
└── utils/
    ├── gql.ts                           # 更新：添加 Space 查询
    └── constant.ts                      # 更新：添加常量
```

---

## 8. 验收检查表

### 功能验收

- [ ] Space 列表可正常显示
- [ ] Space 列表排序功能正常
- [ ] Space 列表刷新功能正常
- [ ] 空状态显示正常
- [ ] 创建 Space 弹窗可正常打开
- [ ] 创建 Space 表单验证正常
- [ ] 创建 Space 成功刷新列表
- [ ] 创建 Space 失败显示错误
- [ ] 删除 Space 确认对话框正常
- [ ] 删除 Space 成功刷新列表
- [ ] 删除 Space 失败显示错误
- [ ] Space 详情弹窗显示正常
- [ ] Space 统计刷新功能正常
- [ ] Space 选择器显示正常
- [ ] Space 切换功能正常
- [ ] 当前 Space 持久化正常
- [ ] 侧边栏导航正常
- [ ] Header Space 选择器正常

### 代码质量

- [ ] 无 ESLint 错误
- [ ] TypeScript 类型完整
- [ ] 代码格式统一
- [ ] 单元测试覆盖率 > 70%

### 性能验收

- [ ] Space 列表加载时间 < 1s
- [ ] Space 创建响应时间 < 2s
- [ ] Space 删除响应时间 < 2s
- [ ] 页面切换流畅

---

## 9. 风险与应对

| 风险 | 可能性 | 影响 | 应对 |
|------|--------|------|------|
| 后端 Schema API 未就绪 | 中 | 高 | 使用 Mock 数据开发，定义好接口契约 |
| Space 创建耗时较长 | 中 | 中 | 显示加载状态，支持异步创建 |
| Space 删除失败（有数据） | 中 | 中 | 显示详细错误信息，引导用户先清理数据 |
| 大量 Space 时性能问题 | 低 | 中 | 实现分页或虚拟滚动 |

---

## 10. 参考文档

- [阶段3 PRD](./prd_phase3.md)
- [技术栈设计](../architecture/tech_stack.md)
- [目录结构设计](../architecture/directory_structure.md)
- [阶段1执行方案](./phase1_implementation_plan.md)
- [阶段2执行方案](./phase2_implementation_plan.md)
- [Web API文档](../../api/web/web_api_overview.md)

---

## 11. 变更历史

| 版本 | 日期 | 变更内容 | 作者 |
|------|------|----------|------|
| v1.0 | 2026-03-29 | 初始版本 | - |

---

**文档结束**
