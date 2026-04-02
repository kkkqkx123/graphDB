# Import 模块功能分析

## 概述

`Import` 目录是 NebulaStudio 图数据库管理工具的数据导入功能模块，提供了完整的图形化界面用于将各类外部数据源的数据导入到 NebulaGraph 图数据库中。

## 目录结构

```
Import/
├── index.tsx                    # 主入口组件，提供 TaskList 和 DatasourceList 两个子页面路由
├── index.module.less           # 主入口样式文件
├── TaskList/                   # 导入任务列表模块
│   ├── index.tsx               # 任务列表主组件，展示所有导入任务
│   ├── TaskItem/               # 单个任务项组件
│   │   ├── index.tsx           # 任务项展示组件
│   │   ├── LogModal/           # 日志查看模态框
│   │   └── AIImportItem.tsx    # AI 导入任务项
│   └── TemplateModal/          # 模板导入模态框
├── TaskCreate/                 # 导入任务创建模块
│   ├── index.tsx               # 任务创建向导主组件
│   ├── SchemaConfig/           # Schema 配置模块
│   │   ├── index.tsx           # Schema 配置入口
│   │   └── FileMapping/        # 文件映射配置
│   └── ConfigConfirmModal/     # 配置确认模态框
├── DatasourceList/             # 数据源管理模块
│   ├── index.tsx               # 数据源列表主组件
│   ├── LocalFileList/          # 本地文件列表
│   └── RemoteList/             # 远程数据源列表 (S3/SFTP)
└── AIImport/                   # AI 智能导入模块
    └── Create.tsx              # AI 导入创建界面
```

## 核心功能

### 1. 数据源管理 (DatasourceList)

支持多种数据源类型：

| 数据源类型 | 说明 |
|-----------|------|
| Local | 本地文件上传 |
| S3 | AWS S3 云存储 |
| SFTP | SFTP 远程服务器 |

- 文件预览功能
- 数据源配置管理

### 2. 导入任务管理 (TaskList)

- **任务列表展示**：显示所有导入任务及状态
- **任务状态监控**：实时刷新任务进度（Pending, Processing, Completed, Failed）
- **任务操作**：
  - 创建新任务
  - 停止正在执行的任务
  - 删除任务
  - 查看任务日志
  - 重新运行任务
- **模板导入**：支持通过配置文件模板批量导入

### 3. 任务创建向导 (TaskCreate)

创建导入任务的完整流程：

1. **基础配置**：设置任务名称
2. **Schema 配置**：
   - Tag（点类型）配置
   - Edge（边类型）配置
3. **文件映射**：将数据文件映射到对应的 Schema
4. **配置确认**：预览并确认导入配置

### 4. AI 智能导入 (AIImport)

利用大语言模型（LLM）实现智能数据导入：
- 自动解析文件内容
- 智能推荐 Schema 结构
- 自然语言描述导入需求

## 工作流程

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  添加数据源      │ ──▶ │   创建导入任务    │ ──▶ │  执行导入任务    │
│ (本地/S3/SFTP)  │     │ (配置Schema映射) │     │  (监控执行状态)  │
└─────────────────┘     └──────────────────┘     └─────────────────┘
```

## 技术栈

- **前端框架**：React + TypeScript
- **UI 组件库**：Ant Design
- **状态管理**：MobX
- **路由**：React Router
- **国际化**：i18n

## 关键接口

| 接口路径 | 功能 |
|---------|------|
| `/api/llm/import/job` | AI 智能导入 |
| 任务管理相关 API | 创建、查询、停止、删除导入任务 |

## 模块依赖

- `stores/import.ts` - 导入模块状态管理
- `stores/datasource.ts` - 数据源状态管理
- `stores/files.ts` - 文件管理状态
- `stores/schema.ts` - Schema 状态管理
- `stores/llm.ts` - LLM 配置状态
