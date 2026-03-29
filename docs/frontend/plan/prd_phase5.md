# GraphDB 前端 PRD - 阶段 5: Schema 管理 - 索引

**文档版本**: v1.0
**创建日期**: 2026-03-29
**最后更新**: 2026-03-29
**项目阶段**: Phase 5 - Schema 管理 - 索引
**预计工期**: 1 周
**依赖阶段**: 阶段 4 (Schema 管理 - Tag/Edge)

---

## 1. 引言（Introduction）

### 1.1 阶段目标

阶段 5 的目标是实现索引的完整管理功能，包括列表展示、创建、删除、状态查看和重建。索引是提高查询性能的关键机制，允许用户为 Tag 和 Edge 的属性创建索引以加速数据检索。

### 1.2 核心价值

- **查询优化**: 通过索引管理提升查询性能
- **性能监控**: 实时查看索引状态和构建进度
- **维护便利**: 支持索引重建以修复或优化索引

### 1.3 目标用户

- **数据库开发者**: 需要创建索引以优化查询性能
- **数据库管理员**: 需要监控索引状态和维护索引
- **性能调优人员**: 需要分析索引使用情况

### 1.4 范围

**包含范围**:
- 索引列表展示
- 索引创建（Tag/Edge 属性索引）
- 索引删除
- 索引状态查看
- 索引重建
- 索引统计信息

**不包含范围**:
- 全文索引（后续版本考虑）
- 复合索引的高级配置
- 索引使用分析和建议

---

## 2. 功能需求

### 2.1 索引列表展示

#### 2.1.1 功能概述

展示当前 Space 中所有索引，包括索引类型、关联的 Tag/Edge、索引属性、状态和创建时间。

#### 2.1.2 用户故事

**US-P5-LIST-001**: 作为数据库开发者，我想要查看所有索引，以便了解当前的索引配置。

**US-P5-LIST-002**: 作为数据库开发者，我想要看到索引的状态（构建中、完成、失败），以便了解索引是否可用。

**US-P5-LIST-003**: 作为数据库开发者，我想要快速搜索和筛选索引，以便在大量索引中定位目标。

#### 2.1.3 验收标准（EARS 格式）

**AC-P5-LIST-001**: The system shall display a list of all indexes in the current Space.

**AC-P5-LIST-002**: The system shall display the index name for each index in the list.

**AC-P5-LIST-003**: The system shall indicate whether each index is for a Tag or Edge.

**AC-P5-LIST-004**: The system shall display the associated Tag or Edge name for each index.

**AC-P5-LIST-005**: The system shall display the indexed properties for each index.

**AC-P5-LIST-006**: The system shall display the status of each index (creating, finished, failed, rebuilding).

**AC-P5-LIST-007**: The system shall display the creation time for each index.

**AC-P5-LIST-008**: The system shall provide a search input to filter indexes by name.

**AC-P5-LIST-009**: The system shall provide filter options to filter by type (Tag/Edge) and status.

**AC-P5-LIST-010**: The system shall provide a "Create Index" button in the list view.

**AC-P5-LIST-011**: The system shall provide a "Delete" button for each index.

**AC-P5-LIST-012**: The system shall provide a "Rebuild" button for indexes that support rebuilding.

**AC-P5-LIST-013**: The system shall display a loading indicator when fetching the index list.

**AC-P5-LIST-014**: The system shall display an empty state message when no indexes exist.

**AC-P5-LIST-015**: The system shall auto-refresh the index status every 5 seconds for indexes in "creating" or "rebuilding" status.

---

### 2.2 索引创建

#### 2.2.1 功能概述

提供创建新索引的功能，支持选择索引类型（Tag/Edge）、关联的 Schema 和要索引的属性。

#### 2.2.2 用户故事

**US-P5-CREATE-001**: 作为数据库开发者，我想要为 Tag 或 Edge 的属性创建索引，以便提高查询性能。

**US-P5-CREATE-002**: 作为数据库开发者，我想要在创建索引时选择多个属性，以便创建复合索引。

**US-P5-CREATE-003**: 作为数据库开发者，我想要在创建索引时预览生成的 Cypher 语句，以便学习和验证。

#### 2.2.3 验收标准（EARS 格式）

**AC-P5-CREATE-001**: The system shall provide a "Create Index" button that opens a creation form.

**AC-P5-CREATE-002**: The system shall provide an input field for index name in the creation form.

**AC-P5-CREATE-003**: The system shall validate that the index name is not empty.

**AC-P5-CREATE-004**: The system shall validate that the index name follows naming conventions (alphanumeric and underscores, starting with a letter).

**AC-P5-CREATE-005**: The system shall validate that the index name is unique within the current Space.

**AC-P5-CREATE-006**: The system shall provide a dropdown to select the index type (Tag or Edge).

**AC-P5-CREATE-007**: When the user selects "Tag" type, the system shall display a dropdown of available Tags.

**AC-P5-CREATE-008**: When the user selects "Edge" type, the system shall display a dropdown of available Edges.

**AC-P5-CREATE-009**: The system shall provide a multi-select or checkbox list to select properties to index.

**AC-P5-CREATE-010**: The system shall only display properties of the selected Tag or Edge.

**AC-P5-CREATE-011**: The system shall validate that at least one property is selected.

**AC-P5-CREATE-012**: The system shall validate that the selected properties exist in the selected Tag or Edge.

**AC-P5-CREATE-013**: The system shall provide a "Create" button to submit the form.

**AC-P5-CREATE-014**: The system shall provide a "Cancel" button to close the form.

**AC-P5-CREATE-015**: The system shall display a preview of the generated Cypher CREATE INDEX statement.

**AC-P5-CREATE-016**: When the user submits valid index creation data, the system shall execute the CREATE INDEX command.

**AC-P5-CREATE-017**: When index creation is successful, the system shall display a success message.

**AC-P5-CREATE-018**: When index creation is successful, the system shall refresh the index list.

**AC-P5-CREATE-019**: When index creation is successful, the system shall close the creation form.

**AC-P5-CREATE-020**: When index creation fails, the system shall display an error message with details.

**AC-P5-CREATE-021**: The system shall display a loading indicator during index creation.

**AC-P5-CREATE-022**: The system shall display a warning if the selected Tag/Edge has a large amount of data, indicating that index creation may take a long time.

---

### 2.3 索引删除

#### 2.3.1 功能概述

提供删除索引的功能，包含确认对话框以防止误操作。

#### 2.3.2 用户故事

**US-P5-DELETE-001**: 作为数据库开发者，我想要删除不需要的索引，以便减少存储开销。

**US-P5-DELETE-002**: 作为数据库开发者，我想要在删除前看到确认提示，以避免误删除影响查询性能。

#### 2.3.3 验收标准（EARS 格式）

**AC-P5-DELETE-001**: The system shall provide a "Delete" button for each index in the list.

**AC-P5-DELETE-002**: When the user clicks the "Delete" button, the system shall display a confirmation dialog.

**AC-P5-DELETE-003**: The confirmation dialog shall display the index name to be deleted.

**AC-P5-DELETE-004**: The confirmation dialog shall display the associated Tag or Edge name.

**AC-P5-DELETE-005**: The confirmation dialog shall display a warning message about potential query performance impact.

**AC-P5-DELETE-006**: The confirmation dialog shall provide a "Confirm" button to proceed with deletion.

**AC-P5-DELETE-007**: The confirmation dialog shall provide a "Cancel" button to abort deletion.

**AC-P5-DELETE-008**: When the user confirms deletion, the system shall execute the DROP INDEX command.

**AC-P5-DELETE-009**: When index deletion is successful, the system shall display a success message.

**AC-P5-DELETE-010**: When index deletion is successful, the system shall refresh the index list.

**AC-P5-DELETE-011**: When index deletion fails, the system shall display an error message with details.

**AC-P5-DELETE-012**: The system shall display a loading indicator during index deletion.

---

### 2.4 索引重建

#### 2.4.1 功能概述

提供重建索引的功能，用于修复损坏的索引或优化索引结构。

#### 2.4.2 用户故事

**US-P5-REBUILD-001**: 作为数据库开发者，我想要重建索引，以便修复损坏的索引或优化索引性能。

**US-P5-REBUILD-002**: 作为数据库开发者，我想要看到重建进度，以便了解操作何时完成。

#### 2.4.3 验收标准（EARS 格式）

**AC-P5-REBUILD-001**: The system shall provide a "Rebuild" button for each index that supports rebuilding.

**AC-P5-REBUILD-002**: The "Rebuild" button shall be disabled for indexes that do not support rebuilding.

**AC-P5-REBUILD-003**: When the user clicks the "Rebuild" button, the system shall display a confirmation dialog.

**AC-P5-REBUILD-004**: The confirmation dialog shall display the index name to be rebuilt.

**AC-P5-REBUILD-005**: The confirmation dialog shall display a warning message that the index will be unavailable during rebuilding.

**AC-P5-REBUILD-006**: The confirmation dialog shall provide a "Confirm" button to proceed with rebuilding.

**AC-P5-REBUILD-007**: The confirmation dialog shall provide a "Cancel" button to abort rebuilding.

**AC-P5-REBUILD-008**: When the user confirms rebuilding, the system shall execute the REBUILD INDEX command.

**AC-P5-REBUILD-009**: When index rebuilding starts, the system shall display a success message indicating the rebuild has started.

**AC-P5-REBUILD-010**: The system shall update the index status to "rebuilding" in the list.

**AC-P5-REBUILD-011**: The system shall auto-refresh the index status to show rebuild progress.

**AC-P5-REBUILD-012**: When index rebuilding completes, the system shall update the index status to "finished".

**AC-P5-REBUILD-013**: When index rebuilding fails, the system shall update the index status to "failed" and display an error message.

**AC-P5-REBUILD-014**: The system shall display a loading indicator during the rebuild initiation.

---

### 2.5 索引详情查看

#### 2.5.1 功能概述

提供查看索引详细信息的功能，包括索引配置和构建统计。

#### 2.5.2 用户故事

**US-P5-DETAIL-001**: 作为数据库开发者，我想要查看索引的详细信息，以便了解索引的配置和状态。

**US-P5-DETAIL-002**: 作为数据库开发者，我想要看到索引的构建进度，以便了解索引何时可用。

#### 2.5.3 验收标准（EARS 格式）

**AC-P5-DETAIL-001**: The system shall provide a "View Details" option for each index.

**AC-P5-DETAIL-002**: When viewing index details, the system shall display the index name.

**AC-P5-DETAIL-003**: When viewing index details, the system shall display the index type (Tag or Edge).

**AC-P5-DETAIL-004**: When viewing index details, the system shall display the associated Tag or Edge name.

**AC-P5-DETAIL-005**: When viewing index details, the system shall display the list of indexed properties.

**AC-P5-DETAIL-006**: When viewing index details, the system shall display the current status.

**AC-P5-DETAIL-007**: When viewing index details, the system shall display the creation time.

**AC-P5-DETAIL-008**: When viewing index details, the system shall display the last rebuild time (if applicable).

**AC-P5-DETAIL-009**: For indexes in "creating" or "rebuilding" status, the system shall display a progress indicator.

**AC-P5-DETAIL-010**: The system shall provide a "Close" button to close the detail view.

**AC-P5-DETAIL-011**: The system shall provide a "Rebuild" button in the detail view (if supported).

---

### 2.6 索引统计信息

#### 2.6.1 功能概述

提供索引的统计信息展示，帮助用户了解索引的使用情况和效率。

#### 2.6.2 用户故事

**US-P5-STATS-001**: 作为数据库开发者，我想要看到索引的统计信息，以便评估索引的效果。

#### 2.6.3 验收标准（EARS 格式）

**AC-P5-STATS-001**: The system shall display the total number of indexes in the current Space.

**AC-P5-STATS-002**: The system shall display the count of indexes by type (Tag indexes, Edge indexes).

**AC-P5-STATS-003**: The system shall display the count of indexes by status (finished, creating, failed, rebuilding).

**AC-P5-STATS-004**: The system shall display the index statistics in the Schema overview page.

---

## 3. 非功能需求

### 3.1 性能需求

**NF-P5-PERF-001**: The system shall load the index list within 2 seconds for up to 100 indexes.

**NF-P5-PERF-002**: The system shall update index status in real-time or near real-time (within 5 seconds).

**NF-P5-PERF-003**: The system shall handle index creation for large datasets with appropriate progress indication.

### 3.2 可用性需求

**NF-P5-UX-001**: The system shall clearly indicate which properties can be indexed based on data type.

**NF-P5-UX-002**: The system shall provide tooltips explaining index types and their use cases.

**NF-P5-UX-003**: The system shall disable actions that are not applicable (e.g., rebuild for unsupported indexes).

**NF-P5-UX-004**: The system shall provide clear visual indicators for different index statuses.

### 3.3 兼容性需求

**NF-P5-COMPAT-001**: The system shall work on modern browsers (Chrome, Firefox, Safari, Edge) latest 2 versions.

**NF-P5-COMPAT-002**: The system shall handle different index capabilities based on GraphDB version.

### 3.4 安全需求

**NF-P5-SEC-001**: The system shall validate all user inputs to prevent injection attacks.

**NF-P5-SEC-002**: The system shall prevent deletion of system indexes.

### 3.5 可靠性需求

**NF-P5-REL-001**: The system shall handle network errors during index status polling gracefully.

**NF-P5-REL-002**: The system shall recover from temporary API failures during long-running index operations.

---

## 4. 用户界面需求

### 4.1 页面布局

#### 4.1.1 索引管理主页面

```
+----------------------------------------------------------+
|  Index Management                                         |
+----------------------------------------------------------+
|  [Space Selector]  [Tabs: Tags | Edges | Indexes | Viz]  |
+----------------------------------------------------------+
|                                                           |
|  +----------------------------------------------------+  |
|  | Search: [____________]  [Type: All ▼] [Status: All ▼]|
|  +----------------------------------------------------+  |
|  |                                                    |  |
|  |  Index List Table                                  |  |
|  |  +------+------+----------+--------+--------+------+|  |
|  |  | Name | Type | Schema   | Props  | Status |Action||  |
|  |  +------+------+----------+--------+--------+------+|  |
|  |  | idx1 | Tag  | person   | name   | ✓ Fin  |Del/Reb|  |
|  |  | idx2 | Edge | follow   | since  | ⟳ Cre  |Del   |  |
|  |  | idx3 | Tag  | company  | code   | ✗ Fail |Del/Reb|  |
|  |  +------+------+----------+--------+--------+------+|  |
|  |                                                    |  |
|  +----------------------------------------------------+  |
|  [+ Create Index]                                        |
|                                                           |
+----------------------------------------------------------+
```

#### 4.1.2 索引创建表单

```
+----------------------------------------------------------+
|  Create Index                                [X]         |
+----------------------------------------------------------+
|                                                           |
|  Index Name: [____________________]                      |
|                                                           |
|  Index Type:                                             |
|  ( ) Tag    ( ) Edge                                     |
|                                                           |
|  Select Tag/Edge: [Dropdown ▼]                          |
|                                                           |
|  Select Properties to Index:                             |
|  +----------------------------------------------------+  |
|  | ☑ name        | STRING    |                        |  |
|  | ☐ age         | INT64     |                        |  |
|  | ☑ email       | STRING    |                        |  |
|  +----------------------------------------------------+  |
|                                                           |
|  Cypher Preview:                                          |
|  +----------------------------------------------------+  |
|  | CREATE TAG INDEX idx_person ON person(name, email);|  |
|  +----------------------------------------------------+  |
|                                                           |
|  ⚠️ Warning: Index creation may take a long time for     |
|     large datasets.                                      |
|                                                           |
|  [Cancel]                    [Create]                    |
|                                                           |
+----------------------------------------------------------+
```

### 4.2 状态指示器

| 状态 | 图标 | 颜色 | 说明 |
|------|------|------|------|
| Finished | ✓ | 绿色 | 索引可用 |
| Creating | ⟳ | 蓝色 | 正在构建 |
| Rebuilding | ⟳ | 橙色 | 正在重建 |
| Failed | ✗ | 红色 | 构建失败 |

### 4.3 组件需求

| 组件 | 描述 | 来源 |
|------|------|------|
| IndexList | 索引列表展示组件 | 新建 |
| IndexForm | 索引创建表单组件 | 新建 |
| IndexStatusBadge | 索引状态标签组件 | 新建 |
| DeleteConfirmModal | 删除确认弹窗 | 复用 Ant Design |
| RebuildConfirmModal | 重建确认弹窗 | 新建 |
| DetailDrawer | 详情抽屉组件 | 复用 Ant Design |

---

## 5. 数据需求

### 5.1 数据模型

#### 5.1.1 索引数据模型

```typescript
interface Index {
  name: string;
  type: 'TAG' | 'EDGE';
  schemaName: string;  // Tag name or Edge name
  properties: string[];
  status: IndexStatus;
  created_at: string;
  updated_at?: string;
  progress?: number;  // For creating/rebuilding status (0-100)
  errorMessage?: string;  // For failed status
}

type IndexStatus = 'creating' | 'finished' | 'failed' | 'rebuilding';

interface IndexStats {
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
```

### 5.2 数据验证规则

| 字段 | 规则 | 错误消息 |
|------|------|----------|
| Index Name | 必填，字母开头，字母数字下划线 | "索引名必须以字母开头" |
| Index Name | 唯一性检查 | "该索引名已存在" |
| Index Type | 必选 | "请选择索引类型" |
| Schema | 必选 | "请选择 Tag 或 Edge" |
| Properties | 至少选择一项 | "请至少选择一个属性" |

### 5.3 数据持久化

- 索引定义存储在 GraphDB 中
- 前端缓存索引列表，定期刷新状态
- 创建/删除操作后立即刷新列表

---

## 6. API 集成需求

### 6.1 API 端点列表

| 端点 | 方法 | 描述 | 请求参数 | 响应数据 |
|------|------|------|----------|----------|
| /api/schema/indexes | GET | 获取所有索引 | space: string | Index[] |
| /api/schema/indexes | POST | 创建索引 | space: string, name: string, type: string, schemaName: string, properties: string[] | { success: boolean, jobId?: string } |
| /api/schema/indexes/:name | GET | 获取索引详情 | space: string | Index |
| /api/schema/indexes/:name | DELETE | 删除索引 | space: string | { success: boolean } |
| /api/schema/indexes/:name/rebuild | POST | 重建索引 | space: string | { success: boolean, jobId?: string } |
| /api/schema/indexes/:name/status | GET | 获取索引状态 | space: string | { status: IndexStatus, progress?: number } |
| /api/schema/index-stats | GET | 获取索引统计 | space: string | IndexStats |

### 6.2 Cypher 语句示例

```cypher
-- 创建 Tag 索引
CREATE TAG INDEX idx_person_name ON person(name);

-- 创建复合索引
CREATE TAG INDEX idx_person_name_age ON person(name, age);

-- 创建 Edge 索引
CREATE EDGE INDEX idx_follow_since ON follow(since);

-- 查看所有索引
SHOW INDEXES;

-- 查看索引详情
DESCRIBE INDEX idx_person_name;

-- 删除索引
DROP INDEX idx_person_name;

-- 重建索引
REBUILD INDEX idx_person_name;
```

---

## 7. 测试需求

### 7.1 单元测试

| 测试项 | 描述 | 覆盖率要求 |
|--------|------|------------|
| IndexList 组件 | 列表渲染、搜索、筛选、空状态 | > 80% |
| IndexForm 组件 | 表单验证、类型切换、属性选择 | > 85% |
| IndexStatusBadge 组件 | 状态显示、颜色正确 | > 90% |
| Schema Store 扩展 | 索引状态管理、API 调用 | > 80% |
| 验证函数 | 名称验证、属性验证 | > 90% |
| 状态轮询逻辑 | 自动刷新、错误处理 | > 80% |

### 7.2 集成测试

| 测试场景 | 描述 |
|----------|------|
| 创建索引完整流程 | 打开表单 -> 填写信息 -> 提交 -> 验证列表更新 -> 验证状态变化 |
| 删除索引完整流程 | 点击删除 -> 确认 -> 验证列表更新 |
| 重建索引完整流程 | 点击重建 -> 确认 -> 验证状态变为 rebuilding -> 验证状态变为 finished |
| 状态轮询 | 创建索引后验证状态自动更新 |
| 表单验证 | 各种验证错误的提示和阻止提交 |
| 网络错误处理 | 网络异常时的错误提示和重试 |

### 7.3 端到端测试

| 测试场景 | 描述 |
|----------|------|
| Tag 索引 CRUD | 完整的 Tag 索引增删改查操作 |
| Edge 索引 CRUD | 完整的 Edge 索引增删改查操作 |
| 索引重建 | 重建流程和状态监控 |

---

## 8. 交付物

### 8.1 代码交付物

- [ ] 索引列表页面组件 (`pages/Schema/IndexList/`)
- [ ] 索引创建表单组件 (`pages/Schema/components/IndexForm/`)
- [ ] 索引状态标签组件 (`components/IndexStatusBadge/`)
- [ ] 重建确认弹窗组件 (`components/RebuildConfirmModal/`)
- [ ] Schema Store 索引相关扩展 (`stores/schema.ts`)
- [ ] 索引服务 (`services/index.ts`)
- [ ] 类型定义扩展 (`types/schema.ts`)
- [ ] 索引工具函数 (`utils/index.ts`)

### 8.2 测试交付物

- [ ] 单元测试文件（覆盖率 > 70%）
- [ ] 集成测试文件
- [ ] E2E 测试脚本

### 8.3 文档交付物

- [ ] 组件使用文档
- [ ] API 集成文档

---

## 9. 验收标准

### 9.1 功能验收标准

| 验收项 | 标准 | 验证方法 |
|--------|------|----------|
| 索引列表 | 正确显示所有索引，支持搜索和筛选 | 手动测试 |
| 索引创建 | 能成功创建 Tag 和 Edge 索引 | 手动测试 + 自动化测试 |
| 索引删除 | 能删除索引并刷新列表 | 手动测试 + 自动化测试 |
| 索引重建 | 能重建索引并监控状态 | 手动测试 + 自动化测试 |
| 状态监控 | 状态实时更新，进度显示正确 | 手动测试 |
| 表单验证 | 各种验证场景正确处理 | 自动化测试 |

### 9.2 质量验收标准

- 代码覆盖率 > 70%
- 无严重 Bug
- 无性能问题（页面加载 < 2s，状态更新 < 5s）
- 通过代码审查

---

## 10. 风险和假设

### 10.1 风险

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 索引操作耗时较长 | 中 | 提供进度指示，支持异步状态查询 |
| 后端不支持某些索引类型 | 中 | 确认 GraphDB 索引能力，适配实现 |
| 状态同步延迟 | 低 | 合理设置轮询间隔，提供手动刷新 |

### 10.2 假设

- 后端 API 支持异步索引操作和状态查询
- GraphDB 支持标准 Cypher 索引语法
- 索引操作有合理的超时机制

---

## 11. 附录

### 11.1 术语表

| 术语 | 定义 |
|------|------|
| Index | 索引，用于加速属性查询的数据结构 |
| Tag Index | 标签索引，为 Tag 的属性创建的索引 |
| Edge Index | 边索引，为 Edge 的属性创建的索引 |
| Composite Index | 复合索引，包含多个属性的索引 |
| Rebuild | 重建，重新构建索引数据 |

### 11.2 参考文档

- [GraphDB Cypher 文档](../api/cypher.md)
- [阶段 4 PRD](./prd_phase4.md)
- [nebula-studio 索引实现参考](../../ref/nebula-studio-3.10.0/app/pages/Schema/)

### 11.3 变更历史

| 版本 | 日期 | 变更内容 | 作者 |
|------|------|----------|------|
| v1.0 | 2026-03-29 | 初始版本 | - |

---

**文档结束**
