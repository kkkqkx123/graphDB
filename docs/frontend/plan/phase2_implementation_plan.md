# GraphDB 前端阶段2执行方案

**文档版本**: v1.0  
**创建日期**: 2026-03-29  
**对应PRD**: [prd_phase2.md](./prd_phase2.md)

---

## 1. 执行概览

### 1.1 目标

实现完整的 Cypher 查询控制台功能，包括查询编辑器、执行引擎、结果展示、历史记录和收藏功能。

### 1.2 工期

预计 2 周

### 1.3 依赖

- 阶段 1 完成（基础框架和连接管理）
- 后端查询 API 可用（`/api/query/execute`）

### 1.4 交付物

- 查询控制台页面
- 查询编辑器组件
- 结果展示组件（表格/JSON）
- 查询历史和收藏功能
- 结果导出功能

---

## 2. 执行步骤

### 步骤1: 项目准备和工具函数实现

**任务**: 创建查询相关的工具函数和服务

**操作**:
1. 创建 `src/utils/gql.ts` - Cypher 查询解析和格式化工具
2. 创建 `src/utils/export.ts` - CSV/JSON 导出工具
3. 创建 `src/utils/parseData.ts` - 查询结果解析工具
4. 更新 `src/services/query.ts` - 查询 API 服务

**文件清单**:

| 文件 | 功能 | 参考来源 |
|------|------|----------|
| `utils/gql.ts` | 查询解析、格式化 | nebula-studio `utils/gql.ts`（适配 Cypher） |
| `utils/export.ts` | CSV/JSON 导出 | 新建 |
| `utils/parseData.ts` | 结果数据解析 | nebula-studio `utils/parseData.ts` |
| `services/query.ts` | 查询 API | 新建 |

**验收**:
- 工具函数单元测试通过
- CSV 导出格式正确
- JSON 导出格式正确

---

### 步骤2: 状态管理实现

**任务**: 实现控制台相关的状态管理

**操作**:
1. 创建 `src/stores/console.ts` - 控制台状态管理

**Store 设计**:
```typescript
interface ConsoleState {
  // 编辑器状态
  editorContent: string;
  isExecuting: boolean;
  
  // 执行结果
  currentResult: QueryResult | null;
  executionTime: number;
  error: QueryError | null;
  
  // 视图状态
  activeView: 'table' | 'json';
  
  // 历史记录
  history: QueryHistoryItem[];
  
  // 收藏
  favorites: QueryFavoriteItem[];
  
  // Actions
  setEditorContent: (content: string) => void;
  executeQuery: (query: string) => Promise<void>;
  clearResult: () => void;
  setActiveView: (view: 'table' | 'json') => void;
  addToHistory: (item: QueryHistoryItem) => void;
  clearHistory: () => void;
  addToFavorites: (item: QueryFavoriteItem) => void;
  removeFromFavorites: (id: string) => void;
}
```

**持久化配置**:
- 历史记录: localStorage, key=`graphdb_query_history`, max=50
- 收藏: localStorage, key=`graphdb_query_favorites`, max=30
- 编辑器草稿: localStorage, key=`graphdb_editor_draft`

**验收**:
- Store 状态可正常读写
- 历史记录持久化正常
- 收藏持久化正常

---

### 步骤3: 查询编辑器组件实现

**任务**: 实现查询编辑器组件

**操作**:
1. 创建 `src/pages/Console/components/QueryEditor/index.tsx`
2. 创建 `src/pages/Console/components/QueryEditor/index.module.less`

**组件功能**:
- 多行文本输入（Input.TextArea）
- Tab 键缩进支持
- 快捷键支持（Ctrl+Enter 执行）
- 工具栏（执行、清除、保存收藏、历史、收藏按钮）
- 状态栏（光标位置、字符数）

**组件接口**:
```typescript
interface QueryEditorProps {
  value: string;
  onChange: (value: string) => void;
  onExecute: () => void;
  onClear: () => void;
  onSaveFavorite: () => void;
  onOpenHistory: () => void;
  onOpenFavorites: () => void;
  isExecuting: boolean;
}
```

**快捷键映射**:
| 快捷键 | 功能 |
|--------|------|
| Ctrl/Cmd + Enter | 执行查询 |
| Shift + Enter | 执行查询 |
| Ctrl/Cmd + / | 注释/取消注释当前行 |
| Tab | 插入缩进 |
| Shift + Tab | 减少缩进 |

**验收**:
- 文本输入正常
- Tab 缩进正常
- 快捷键响应正常
- 工具栏按钮可用

---

### 步骤4: 结果展示组件实现

**任务**: 实现结果展示组件

**操作**:
1. 创建 `src/pages/Console/components/ResultTable/index.tsx` - 表格视图
2. 创建 `src/pages/Console/components/ResultJson/index.tsx` - JSON 视图
3. 创建 `src/pages/Console/components/OutputBox/index.tsx` - 结果容器

**ResultTable 组件**:
- Ant Design Table 组件
- 列自动识别
- 排序功能
- 分页（每页100行）
- 水平/垂直滚动

**ResultJson 组件**:
- JSON 格式化显示
- 语法高亮（使用 react-json-view 或原生实现）
- 复制功能

**OutputBox 组件**:
- 视图切换（Table/JSON）
- 导出按钮（CSV/JSON）
- 状态栏（执行时间、行数）
- 空状态显示
- 错误信息显示

**组件接口**:
```typescript
interface OutputBoxProps {
  result: QueryResult | null;
  error: QueryError | null;
  executionTime: number;
  activeView: 'table' | 'json';
  onViewChange: (view: 'table' | 'json') => void;
  onExport: (format: 'csv' | 'json') => void;
}

interface QueryResult {
  columns: string[];
  rows: any[][];
  rowCount: number;
}

interface QueryError {
  code: string;
  message: string;
  position?: { line: number; column: number };
}
```

**验收**:
- 表格显示正常
- JSON 显示正常
- 视图切换正常
- 导出功能正常
- 分页功能正常

---

### 步骤5: 历史记录面板实现

**任务**: 实现查询历史记录面板

**操作**:
1. 创建 `src/pages/Console/components/HistoryPanel/index.tsx`
2. 创建 `src/pages/Console/components/HistoryPanel/index.module.less`

**组件功能**:
- 侧边抽屉（Drawer）设计
- 历史列表显示
- 查询预览（截断100字符）
- 执行时间和时间戳显示
- 点击加载到编辑器
- 清空历史按钮

**历史项结构**:
```typescript
interface QueryHistoryItem {
  id: string;
  query: string;
  executionTime: number; // 执行耗时(ms)
  timestamp: number;     // 执行时间戳
  rowCount: number;      // 返回行数
}
```

**验收**:
- 面板可正常打开/关闭
- 历史列表显示正常
- 点击加载正常
- 清空历史功能正常

---

### 步骤6: 收藏面板实现

**任务**: 实现查询收藏面板

**操作**:
1. 创建 `src/pages/Console/components/FavoritePanel/index.tsx`
2. 创建 `src/pages/Console/components/FavoritePanel/index.module.less`
3. 创建 `src/pages/Console/components/SaveFavoriteModal/index.tsx` - 保存收藏弹窗

**FavoritePanel 组件功能**:
- 侧边抽屉（Drawer）设计
- 收藏列表显示
- 名称和查询预览
- 执行按钮（直接执行）
- 加载按钮（加载到编辑器）
- 删除按钮

**SaveFavoriteModal 组件功能**:
- 输入收藏名称
- 显示查询预览
- 名称唯一性验证
- 保存/取消按钮

**收藏项结构**:
```typescript
interface QueryFavoriteItem {
  id: string;
  name: string;
  query: string;
  createdAt: number;
}
```

**验收**:
- 面板可正常打开/关闭
- 收藏列表显示正常
- 保存收藏弹窗正常
- 执行/加载/删除功能正常

---

### 步骤7: 控制台页面集成

**任务**: 整合所有组件到控制台页面

**操作**:
1. 更新 `src/pages/Console/index.tsx` - 控制台主页面
2. 创建 `src/pages/Console/index.module.less` - 页面样式

**页面布局**:
```
+------------------+
|   QueryEditor    |
|   (可调整高度)    |
+------------------+
|  拖拽调整条       |
+------------------+
|    OutputBox     |
|   (可调整高度)    |
+------------------+
```

**页面功能**:
- 上下分栏布局（可拖拽调整）
- 编辑器状态管理
- 查询执行逻辑
- 结果展示控制
- 历史/收藏面板控制

**路由配置**:
```typescript
// config/routes.tsx
{
  path: '/console',
  element: <ConsolePage />,
  title: '查询控制台'
}
```

**验收**:
- 页面布局正常
- 拖拽调整正常
- 组件间通信正常
- 路由访问正常

---

### 步骤8: 侧边栏导航更新

**任务**: 更新侧边栏导航，添加控制台入口

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
    label: 'Schema管理'
  }
];
```

**验收**:
- 导航项显示正常
- 点击跳转正常
- 当前项高亮正常

---

## 3. API 接口清单

| 接口 | 方法 | 用途 | 请求体 | 响应 |
|------|------|------|--------|------|
| `/api/query/execute` | POST | 执行查询 | `{ query: string, sessionId: string }` | `{ success: boolean, data?: QueryResult, error?: QueryError }` |

**请求头**:
- `X-Session-ID`: 会话ID
- `Content-Type`: application/json

**错误码**:
| 错误码 | 说明 |
|--------|------|
| `SYNTAX_ERROR` | Cypher 语法错误 |
| `CONNECTION_ERROR` | 数据库连接错误 |
| `TIMEOUT_ERROR` | 查询超时 |
| `PERMISSION_ERROR` | 权限不足 |
| `INTERNAL_ERROR` | 内部服务器错误 |

---

## 4. 组件清单

### 页面组件

| 组件 | 路径 | 用途 | 优先级 |
|------|------|------|--------|
| Console | `pages/Console/index.tsx` | 控制台主页面 | P0 |

### 业务组件

| 组件 | 路径 | 用途 | 优先级 |
|------|------|------|--------|
| QueryEditor | `pages/Console/components/QueryEditor/` | 查询编辑器 | P0 |
| OutputBox | `pages/Console/components/OutputBox/` | 结果展示容器 | P0 |
| ResultTable | `pages/Console/components/ResultTable/` | 表格结果 | P0 |
| ResultJson | `pages/Console/components/ResultJson/` | JSON 结果 | P0 |
| HistoryPanel | `pages/Console/components/HistoryPanel/` | 历史记录面板 | P1 |
| FavoritePanel | `pages/Console/components/FavoritePanel/` | 收藏面板 | P1 |
| SaveFavoriteModal | `pages/Console/components/SaveFavoriteModal/` | 保存收藏弹窗 | P1 |

### 公共组件（新增/更新）

| 组件 | 路径 | 用途 | 优先级 |
|------|------|------|--------|
| EmptyTableTip | `components/common/EmptyTableTip/` | 空表格提示 | P1 |

---

## 5. Store 清单

| Store | 路径 | 用途 |
|-------|------|------|
| console | `stores/console.ts` | 控制台状态（编辑器、结果、历史、收藏） |

**Store 状态结构**:
```typescript
interface ConsoleState {
  // Editor
  editorContent: string;
  isExecuting: boolean;
  
  // Result
  currentResult: QueryResult | null;
  executionTime: number;
  error: QueryError | null;
  activeView: 'table' | 'json';
  
  // History
  history: QueryHistoryItem[];
  
  // Favorites
  favorites: QueryFavoriteItem[];
}
```

---

## 6. 依赖清单

### 生产依赖（新增）

```
# 可选，用于 JSON 语法高亮
react-json-view

# 或轻量级替代方案
# 使用原生 JSON.stringify + CSS 高亮
```

### 开发依赖（无新增）

使用现有开发依赖即可。

---

## 7. 目录结构

阶段 2 涉及的目录结构：

```
src/
├── pages/
│   └── Console/
│       ├── index.tsx                    # 控制台主页面
│       ├── index.module.less            # 页面样式
│       └── components/                  # 页面组件
│           ├── QueryEditor/             # 查询编辑器
│           │   ├── index.tsx
│           │   └── index.module.less
│           ├── OutputBox/               # 结果容器
│           │   ├── index.tsx
│           │   └── index.module.less
│           ├── ResultTable/             # 表格结果
│           │   ├── index.tsx
│           │   └── index.module.less
│           ├── ResultJson/              # JSON 结果
│           │   ├── index.tsx
│           │   └── index.module.less
│           ├── HistoryPanel/            # 历史面板
│           │   ├── index.tsx
│           │   └── index.module.less
│           ├── FavoritePanel/           # 收藏面板
│           │   ├── index.tsx
│           │   └── index.module.less
│           └── SaveFavoriteModal/       # 保存收藏弹窗
│               ├── index.tsx
│               └── index.module.less
├── stores/
│   └── console.ts                       # 控制台状态管理
├── services/
│   └── query.ts                         # 查询 API 服务
└── utils/
    ├── gql.ts                           # Cypher 查询工具
    ├── export.ts                        # 导出工具
    └── parseData.ts                     # 数据解析工具
```

---

## 8. 验收检查表

### 功能验收

- [ ] 查询编辑器可正常输入文本
- [ ] Tab 键缩进功能正常
- [ ] Ctrl+Enter 快捷键可执行查询
- [ ] 查询执行结果显示正常
- [ ] 表格视图可正常显示数据
- [ ] JSON 视图可正常显示数据
- [ ] 视图切换功能正常
- [ ] 表格排序功能正常
- [ ] 表格分页功能正常
- [ ] CSV 导出功能正常
- [ ] JSON 导出功能正常
- [ ] 历史记录保存正常
- [ ] 历史记录加载正常
- [ ] 历史记录清空正常
- [ ] 收藏保存正常
- [ ] 收藏加载正常
- [ ] 收藏删除正常
- [ ] 收藏直接执行正常
- [ ] 页面布局可拖拽调整
- [ ] 侧边栏导航正常

### 代码质量

- [ ] 无 ESLint 错误
- [ ] TypeScript 类型完整
- [ ] 代码格式统一
- [ ] 单元测试覆盖率 > 75%

### 性能验收

- [ ] 查询执行响应时间 < 1s（简单查询）
- [ ] 结果渲染时间 < 1s（100行）
- [ ] 编辑器响应流畅（10000字符）

---

## 9. 风险与应对

| 风险 | 可能性 | 影响 | 应对 |
|------|--------|------|------|
| 后端 API 未就绪 | 中 | 高 | 使用 Mock 数据开发，定义好接口契约 |
| 大数据量渲染性能问题 | 中 | 中 | 实现虚拟滚动，限制最大返回行数 |
| TextArea 编辑器体验不佳 | 低 | 中 | 阶段 3+ 可升级为 Monaco Editor |
| 复杂查询超时 | 中 | 中 | 设置合理的超时时间，提供取消功能 |

---

## 10. 参考文档

- [阶段2 PRD](./prd_phase2.md)
- [技术栈设计](../architecture/tech_stack.md)
- [目录结构设计](../architecture/directory_structure.md)
- [阶段1执行方案](./phase1_implementation_plan.md)
- [Web API文档](../../api/web/web_api_overview.md)

---

## 11. 变更历史

| 版本 | 日期 | 变更内容 | 作者 |
|------|------|----------|------|
| v1.0 | 2026-03-29 | 初始版本 | - |

---

**文档结束**
