# Schema 模块功能对比分析报告

## 1. 概述

本文档对比分析了 nebula-studio-3.10.0 的 Schema 模块与当前 frontend 项目的 Schema 模块，识别出需要补充的功能点，并制定分阶段实施计划。

## 2. 目录结构对比

### 2.1 nebula-studio Schema 目录结构

```
Schema/
├── SchemaConfig/           # Schema 配置管理
│   ├── Create/             # 创建功能
│   │   ├── CommonCreate/   # Tag/Edge 通用创建
│   │   │   ├── PropertiesForm.tsx
│   │   │   ├── TTLForm.tsx
│   │   │   ├── index.module.less
│   │   │   └── index.tsx
│   │   └── IndexCreate/    # 索引创建（含拖拽字段）
│   │       ├── DraggableTags.tsx
│   │       ├── FieldSelectModal.tsx
│   │       ├── index.module.less
│   │       └── index.tsx
│   ├── DDLModal/           # DDL 导出模态框
│   │   ├── index.module.less
│   │   └── index.tsx
│   ├── Edit/               # 编辑功能
│   │   └── CommonEdit/     # Tag/Edge 通用编辑
│   │       ├── PropertiesForm.tsx
│   │       ├── PropertiesRow.tsx
│   │       ├── TTLForm.tsx
│   │       ├── index.module.less
│   │       └── index.tsx
│   └── List/               # 列表展示
│       ├── CommonLayout/   # 通用布局
│       │   ├── index.module.less
│       │   └── index.tsx
│       ├── Edge/           # Edge 列表
│       │   └── index.tsx
│       ├── Index/          # 索引列表
│       │   ├── index.module.less
│       │   └── index.tsx
│       ├── SchemaVisualization/  # Schema 可视化
│       │   ├── index.module.less
│       │   └── index.tsx
│       ├── Search/         # 搜索组件
│       │   ├── index.module.less
│       │   └── index.tsx
│       ├── SpaceStats/     # 空间统计
│       │   ├── index.module.less
│       │   └── index.tsx
│       └── Tag/            # Tag 列表
│           └── index.tsx
├── SpaceCreate/            # Space 创建
│   ├── CreateForm.tsx
│   ├── index.module.less
│   └── index.tsx
├── index.module.less
└── index.tsx
```

### 2.2 当前 frontend Schema 目录结构

```
Schema/
├── EdgeList/               # Edge 列表
│   ├── index.module.less
│   └── index.tsx
├── IndexList/              # 索引列表
│   ├── index.module.less
│   └── index.tsx
├── SpaceList/              # Space 列表
│   ├── index.module.less
│   └── index.tsx
├── TagList/                # Tag 列表
│   ├── index.module.less
│   └── index.tsx
├── components/
│   ├── SpaceCreateModal/   # Space 创建模态框
│   │   ├── index.module.less
│   │   └── index.tsx
│   └── SpaceDetailModal/   # Space 详情模态框
│       ├── index.module.less
│       └── index.tsx
└── index.tsx
```

## 3. 功能差异分析

### 3.1 功能对比矩阵

| 功能模块 | nebula-studio | 当前 frontend | 优先级 | 状态 |
|---------|---------------|---------------|--------|------|
| Space 列表管理 | ✅ | ✅ | 高 | 已完成 |
| Space 创建 | ✅ | ✅ | 高 | 已完成 |
| Space 详情查看 | ✅ | ✅ | 高 | 已完成 |
| Space 删除 | ✅ | ✅ | 高 | 已完成 |
| Space 克隆 | ✅ | ❌ | 低 | 缺失 |
| Tag 列表 | ✅ | ✅ | 高 | 已完成 |
| Tag 创建 | ✅ | ✅ | 高 | 已完成 |
| Tag 删除 | ✅ | ✅ | 高 | 已完成 |
| Tag 编辑 | ✅ | ❌ | 高 | **缺失** |
| Edge 列表 | ✅ | ✅ | 高 | 已完成 |
| Edge 创建 | ✅ | ✅ | 高 | 已完成 |
| Edge 删除 | ✅ | ✅ | 高 | 已完成 |
| Edge 编辑 | ✅ | ❌ | 高 | **缺失** |
| 索引列表 | ✅ | ✅ | 高 | 已完成 |
| 索引创建（基础） | ✅ | ✅ | 高 | 已完成 |
| 索引创建（拖拽字段） | ✅ | ❌ | 高 | **缺失** |
| 索引重建状态跟踪 | ✅ | ❌ | 低 | 缺失 |
| DDL 导出 | ✅ | ❌ | 中 | **缺失** |
| Schema 可视化 | ✅ | ❌ | 中 | **缺失** |
| Space 统计 | ✅ | ❌ | 中 | **缺失** |
| TTL 配置 | ✅ | ❌ | 中 | **缺失** |

### 3.2 详细功能差异

#### 3.2.1 核心缺失功能（高优先级）

1. **Tag/Edge 编辑功能**
   - 编辑属性（添加、删除、修改）
   - 编辑 TTL 配置
   - 编辑注释
   - 实时 GQL 预览

2. **索引创建增强**
   - 拖拽字段排序
   - 字段选择模态框
   - 索引字段顺序配置

#### 3.2.2 重要增强功能（中优先级）

3. **DDL 导出功能**
   - 导出完整 Schema DDL
   - 支持复制到剪贴板
   - 支持下载为 .ngql 文件
   - 包含 Space、Tag、Edge、Index 的创建语句

4. **Schema 可视化**
   - 图形化展示 Schema 结构
   - 显示 Tag、Edge 之间的关系
   - 基于真实数据采样生成可视化
   - 支持缩放和平移

5. **Space 统计功能**
   - 显示各类型数据量统计
   - 支持手动触发统计任务
   - 显示统计更新时间
   - 统计任务状态跟踪

6. **TTL 配置功能**
   - TTL 持续时间设置
   - TTL 列选择
   - 在创建/编辑时配置

#### 3.2.3 优化功能（低优先级）

7. **索引重建状态跟踪**
   - 实时监控重建进度
   - 自动轮询状态
   - 成功/失败提示

8. **Space 克隆功能**
   - 基于现有 Space 创建副本
   - 复制 Schema 结构

## 4. 技术实现参考

### 4.1 关键组件参考

| 功能 | 参考文件 |
|------|----------|
| Tag/Edge 编辑 | `ref/nebula-studio-3.10.0/app/pages/Schema/SchemaConfig/Edit/CommonEdit/index.tsx` |
| 索引创建（拖拽） | `ref/nebula-studio-3.10.0/app/pages/Schema/SchemaConfig/Create/IndexCreate/` |
| DDL 导出 | `ref/nebula-studio-3.10.0/app/pages/Schema/SchemaConfig/DDLModal/index.tsx` |
| Schema 可视化 | `ref/nebula-studio-3.10.0/app/pages/Schema/SchemaConfig/List/SchemaVisualization/index.tsx` |
| Space 统计 | `ref/nebula-studio-3.10.0/app/pages/Schema/SchemaConfig/List/SpaceStats/index.tsx` |
| TTL 配置 | `ref/nebula-studio-3.10.0/app/pages/Schema/SchemaConfig/Create/CommonCreate/TTLForm.tsx` |

### 4.2 API 接口需求

根据功能分析，需要后端提供以下 API 支持：

1. **Tag/Edge 编辑**
   - `ALTER TAG <tag_name> ...`
   - `ALTER EDGE <edge_name> ...`
   - `DESCRIBE TAG <tag_name>`
   - `DESCRIBE EDGE <edge_name>`

2. **DDL 导出**
   - `SHOW CREATE SPACE <space_name>`
   - `SHOW CREATE TAG <tag_name>`
   - `SHOW CREATE EDGE <edge_name>`
   - `SHOW CREATE INDEX <index_name>`

3. **Space 统计**
   - `SUBMIT JOB STATS`
   - `SHOW STATS`
   - `SHOW JOB <job_id>`

4. **Schema 可视化**
   - 采样查询接口
   - 节点-Tag 映射接口

## 5. 实施阶段规划

### 阶段一：核心功能完善（2-3 周）
- Tag/Edge 编辑功能
- 索引创建增强（拖拽字段）

### 阶段二：数据管理增强（2 周）
- DDL 导出功能
- TTL 配置功能

### 阶段三：可视化与监控（2-3 周）
- Schema 可视化
- Space 统计功能

### 阶段四：优化功能（1-2 周）
- 索引重建状态跟踪
- Space 克隆功能

## 6. 相关文档

- [阶段一：Tag/Edge 编辑功能](./02_phase1_tag_edge_edit.md)
- [阶段二：索引创建增强](./03_phase2_index_enhancement.md)
- [阶段三：DDL 导出功能](./04_phase3_ddl_export.md)
- [阶段四：Schema 可视化](./05_phase4_schema_visualization.md)
- [阶段五：Space 统计与 TTL 配置](./06_phase5_stats_and_ttl.md)
