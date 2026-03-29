# GraphDB 前端 PRD - 阶段 4: Schema 管理 - Tag/Edge

**文档版本**: v1.0
**创建日期**: 2026-03-29
**最后更新**: 2026-03-29
**项目阶段**: Phase 4 - Schema 管理 - Tag/Edge
**预计工期**: 2 周
**依赖阶段**: 阶段 3 (Schema 管理 - Space)

---

## 1. 引言（Introduction）

### 1.1 阶段目标

阶段 4 的目标是实现 Tag（标签）和 Edge（边类型）的完整管理功能，包括列表展示、创建、修改、删除和详情查看。Tag 和 Edge 是 GraphDB 数据模型的核心组成部分，Tag 定义节点的类型和属性，Edge 定义关系的类型和属性。

### 1.2 核心价值

- **数据建模**: 提供直观的 Tag 和 Edge 管理界面，支持灵活的数据建模
- **Schema 完整性**: 支持属性定义和修改，适应数据模型的演进
- **开发效率**: 通过可视化界面简化 Schema 定义，降低 Cypher 语句编写门槛

### 1.3 目标用户

- **数据库开发者**: 需要定义和管理 Tag/Edge 来构建数据模型
- **数据架构师**: 需要设计和调整 Schema 结构
- **系统管理员**: 需要监控和维护 Schema 配置

### 1.4 范围

**包含范围**:
- Tag 列表展示和管理
- Tag 创建（定义属性）
- Tag 修改（添加/删除属性）
- Tag 删除
- Tag 详情查看
- Edge 列表展示和管理
- Edge 创建（定义属性）
- Edge 修改（添加/删除属性）
- Edge 删除
- Edge 详情查看
- Schema 可视化预览

**不包含范围**:
- Space 管理（阶段 3）
- 索引管理（阶段 5）
- 数据导入功能

---

## 2. 功能需求

### 2.1 Tag 管理

#### 2.1.1 Tag 列表展示

**功能概述**

展示当前 Space 中所有 Tag，包括基本信息和属性概览。

**用户故事**

**US-P4-TAG-LIST-001**: 作为数据库开发者，我想要查看所有 Tag 及其属性，以便了解数据模型。

**US-P4-TAG-LIST-002**: 作为数据库开发者，我想要看到 Tag 的属性数量和创建时间，以便评估模型复杂度。

**US-P4-TAG-LIST-003**: 作为数据库开发者，我想要快速搜索 Tag，以便在大量 Tag 中定位目标。

**验收标准（EARS 格式）**

**AC-P4-TAG-LIST-001**: The system shall display a list of all Tags in the current Space.

**AC-P4-TAG-LIST-002**: The system shall display the Tag name for each Tag in the list.

**AC-P4-TAG-LIST-003**: The system shall display the number of properties for each Tag.

**AC-P4-TAG-LIST-004**: The system shall display the creation time for each Tag.

**AC-P4-TAG-LIST-005**: The system shall provide a search input to filter Tags by name.

**AC-P4-TAG-LIST-006**: The system shall provide a "Create Tag" button in the list view.

**AC-P4-TAG-LIST-007**: The system shall provide an "Edit" button for each Tag.

**AC-P4-TAG-LIST-008**: The system shall provide a "Delete" button for each Tag.

**AC-P4-TAG-LIST-009**: The system shall provide a "View Details" option for each Tag.

**AC-P4-TAG-LIST-010**: The system shall display a loading indicator when fetching the Tag list.

**AC-P4-TAG-LIST-011**: The system shall display an empty state message when no Tags exist.

---

#### 2.1.2 Tag 创建

**功能概述**

提供创建新 Tag 的功能，支持定义 Tag 名称和属性列表。

**用户故事**

**US-P4-TAG-CREATE-001**: 作为数据库开发者，我想要创建新的 Tag 并定义其属性，以便为节点添加结构化数据。

**US-P4-TAG-CREATE-002**: 作为数据库开发者，我想要为 Tag 添加多个属性，每个属性指定数据类型和默认值，以便构建完整的数据模型。

**US-P4-TAG-CREATE-003**: 作为数据库开发者，我想要在创建 Tag 时预览生成的 Cypher 语句，以便学习和验证。

**验收标准（EARS 格式）**

**AC-P4-TAG-CREATE-001**: The system shall provide a "Create Tag" button that opens a creation form.

**AC-P4-TAG-CREATE-002**: The system shall provide an input field for Tag name in the creation form.

**AC-P4-TAG-CREATE-003**: The system shall validate that the Tag name is not empty.

**AC-P4-TAG-CREATE-004**: The system shall validate that the Tag name follows naming conventions (alphanumeric and underscores, starting with a letter).

**AC-P4-TAG-CREATE-005**: The system shall validate that the Tag name is unique within the current Space.

**AC-P4-TAG-CREATE-006**: The system shall provide a section to define properties for the Tag.

**AC-P4-TAG-CREATE-007**: The system shall provide an "Add Property" button to add new properties.

**AC-P4-TAG-CREATE-008**: For each property, the system shall provide an input field for property name.

**AC-P4-TAG-CREATE-009**: The system shall validate that property names are unique within the Tag.

**AC-P4-TAG-CREATE-010**: For each property, the system shall provide a dropdown for data type selection.

**AC-P4-TAG-CREATE-011**: The system shall provide the following data type options: STRING, INT64, DOUBLE, BOOL, DATETIME, DATE, TIME, TIMESTAMP.

**AC-P4-TAG-CREATE-012**: For each property, the system shall provide an optional input field for default value.

**AC-P4-TAG-CREATE-013**: The system shall validate that the default value matches the selected data type.

**AC-P4-TAG-CREATE-014**: The system shall provide a "Remove" button for each property to delete it from the list.

**AC-P4-TAG-CREATE-015**: The system shall provide a "Create" button to submit the form.

**AC-P4-TAG-CREATE-016**: The system shall provide a "Cancel" button to close the form.

**AC-P4-TAG-CREATE-017**: The system shall display a preview of the generated Cypher CREATE TAG statement.

**AC-P4-TAG-CREATE-018**: When the user submits valid Tag creation data, the system shall execute the CREATE TAG command.

**AC-P4-TAG-CREATE-019**: When Tag creation is successful, the system shall display a success message.

**AC-P4-TAG-CREATE-020**: When Tag creation is successful, the system shall refresh the Tag list.

**AC-P4-TAG-CREATE-021**: When Tag creation is successful, the system shall close the creation form.

**AC-P4-TAG-CREATE-022**: When Tag creation fails, the system shall display an error message with details.

**AC-P4-TAG-CREATE-023**: The system shall display a loading indicator during Tag creation.

---

#### 2.1.3 Tag 修改

**功能概述**

提供修改现有 Tag 的功能，支持添加新属性和删除现有属性。

**用户故事**

**US-P4-TAG-EDIT-001**: 作为数据库开发者，我想要修改 Tag 的属性，包括添加新属性或删除现有属性，以便适应数据模型的变化。

**US-P4-TAG-EDIT-002**: 作为数据库开发者，我想要看到 Tag 的现有属性列表，以便了解当前配置。

**验收标准（EARS 格式）**

**AC-P4-TAG-EDIT-001**: The system shall provide an "Edit" button for each Tag that opens an edit form.

**AC-P4-TAG-EDIT-002**: The system shall display the Tag name as read-only in the edit form.

**AC-P4-TAG-EDIT-003**: The system shall display the list of existing properties for the Tag.

**AC-P4-TAG-EDIT-004**: For each existing property, the system shall display the property name, data type, and default value (if any).

**AC-P4-TAG-EDIT-005**: The system shall provide an "Add Property" button to add new properties to the Tag.

**AC-P4-TAG-EDIT-006**: When adding a new property, the system shall provide fields for property name, data type, and default value.

**AC-P4-TAG-EDIT-007**: The system shall validate that new property names are unique within the Tag.

**AC-P4-TAG-EDIT-008**: The system shall provide a "Delete" button for each existing property to remove it from the Tag.

**AC-P4-TAG-EDIT-009**: When the user clicks "Delete Property", the system shall display a confirmation dialog.

**AC-P4-TAG-EDIT-010**: The system shall provide a "Save" button to submit the modifications.

**AC-P4-TAG-EDIT-011**: The system shall provide a "Cancel" button to discard changes.

**AC-P4-TAG-EDIT-012**: The system shall display a preview of the generated Cypher ALTER TAG statements.

**AC-P4-TAG-EDIT-013**: When the user submits Tag modifications, the system shall execute the appropriate ALTER TAG commands.

**AC-P4-TAG-EDIT-014**: When Tag modification is successful, the system shall display a success message.

**AC-P4-TAG-EDIT-015**: When Tag modification is successful, the system shall refresh the Tag list.

**AC-P4-TAG-EDIT-016**: When Tag modification is successful, the system shall close the edit form.

**AC-P4-TAG-EDIT-017**: When Tag modification fails, the system shall display an error message with details.

---

#### 2.1.4 Tag 删除

**功能概述**

提供删除 Tag 的功能，包含确认对话框以防止误操作。

**用户故事**

**US-P4-TAG-DELETE-001**: 作为数据库开发者，我想要删除不需要的 Tag，以便清理 Schema。

**US-P4-TAG-DELETE-002**: 作为数据库开发者，我想要在删除前看到确认提示，以避免误删除数据。

**验收标准（EARS 格式）**

**AC-P4-TAG-DELETE-001**: The system shall provide a "Delete" button for each Tag in the list.

**AC-P4-TAG-DELETE-002**: When the user clicks the "Delete" button, the system shall display a confirmation dialog.

**AC-P4-TAG-DELETE-003**: The confirmation dialog shall display the Tag name to be deleted.

**AC-P4-TAG-DELETE-004**: The confirmation dialog shall display a warning message about data loss.

**AC-P4-TAG-DELETE-005**: The confirmation dialog shall display the number of vertices using this Tag (if available).

**AC-P4-TAG-DELETE-006**: The confirmation dialog shall provide a "Confirm" button to proceed with deletion.

**AC-P4-TAG-DELETE-007**: The confirmation dialog shall provide a "Cancel" button to abort deletion.

**AC-P4-TAG-DELETE-008**: When the user confirms deletion, the system shall execute the DROP TAG command.

**AC-P4-TAG-DELETE-009**: When Tag deletion is successful, the system shall display a success message.

**AC-P4-TAG-DELETE-010**: When Tag deletion is successful, the system shall refresh the Tag list.

**AC-P4-TAG-DELETE-011**: When Tag deletion fails, the system shall display an error message with details.

**AC-P4-TAG-DELETE-012**: The system shall display a loading indicator during Tag deletion.

---

#### 2.1.5 Tag 详情查看

**功能概述**

提供查看 Tag 详细信息的功能，包括完整属性列表和相关统计。

**用户故事**

**US-P4-TAG-DETAIL-001**: 作为数据库开发者，我想要查看 Tag 的详细信息，包括所有属性的数据类型和默认值，以便了解完整的数据模型。

**US-P4-TAG-DETAIL-002**: 作为数据库开发者，我想要看到使用特定 Tag 的节点数量，以便评估数据分布。

**验收标准（EARS 格式）**

**AC-P4-TAG-DETAIL-001**: The system shall provide a "View Details" option for each Tag.

**AC-P4-TAG-DETAIL-002**: When viewing Tag details, the system shall display the Tag name.

**AC-P4-TAG-DETAIL-003**: When viewing Tag details, the system shall display the creation time.

**AC-P4-TAG-DETAIL-004**: When viewing Tag details, the system shall display a table of all properties.

**AC-P4-TAG-DETAIL-005**: The properties table shall include columns for property name, data type, and default value.

**AC-P4-TAG-DETAIL-006**: When viewing Tag details, the system shall display the count of vertices using this Tag.

**AC-P4-TAG-DETAIL-007**: The system shall provide a "Close" button to close the detail view.

**AC-P4-TAG-DETAIL-008**: The system shall provide an "Edit" button in the detail view to open the edit form.

---

### 2.2 Edge 管理

#### 2.2.1 Edge 列表展示

**功能概述**

展示当前 Space 中所有 Edge 类型，包括基本信息和属性概览。

**用户故事**

**US-P4-EDGE-LIST-001**: 作为数据库开发者，我想要查看所有 Edge 类型及其属性，以便了解关系模型。

**US-P4-EDGE-LIST-002**: 作为数据库开发者，我想要快速搜索 Edge 类型，以便在大量 Edge 中定位目标。

**验收标准（EARS 格式）**

**AC-P4-EDGE-LIST-001**: The system shall display a list of all Edge types in the current Space.

**AC-P4-EDGE-LIST-002**: The system shall display the Edge name for each Edge in the list.

**AC-P4-EDGE-LIST-003**: The system shall display the number of properties for each Edge.

**AC-P4-EDGE-LIST-004**: The system shall display the creation time for each Edge.

**AC-P4-EDGE-LIST-005**: The system shall provide a search input to filter Edges by name.

**AC-P4-EDGE-LIST-006**: The system shall provide a "Create Edge" button in the list view.

**AC-P4-EDGE-LIST-007**: The system shall provide an "Edit" button for each Edge.

**AC-P4-EDGE-LIST-008**: The system shall provide a "Delete" button for each Edge.

**AC-P4-EDGE-LIST-009**: The system shall provide a "View Details" option for each Edge.

**AC-P4-EDGE-LIST-010**: The system shall display a loading indicator when fetching the Edge list.

**AC-P4-EDGE-LIST-011**: The system shall display an empty state message when no Edges exist.

---

#### 2.2.2 Edge 创建

**功能概述**

提供创建新 Edge 类型的功能，支持定义 Edge 名称和属性列表。

**用户故事**

**US-P4-EDGE-CREATE-001**: 作为数据库开发者，我想要创建新的 Edge 类型并定义其属性，以便为关系添加结构化数据。

**US-P4-EDGE-CREATE-002**: 作为数据库开发者，我想要在创建 Edge 时预览生成的 Cypher 语句，以便学习和验证。

**验收标准（EARS 格式）**

**AC-P4-EDGE-CREATE-001**: The system shall provide a "Create Edge" button that opens a creation form.

**AC-P4-EDGE-CREATE-002**: The system shall provide an input field for Edge name in the creation form.

**AC-P4-EDGE-CREATE-003**: The system shall validate that the Edge name is not empty.

**AC-P4-EDGE-CREATE-004**: The system shall validate that the Edge name follows naming conventions (alphanumeric and underscores, starting with a letter).

**AC-P4-EDGE-CREATE-005**: The system shall validate that the Edge name is unique within the current Space.

**AC-P4-EDGE-CREATE-006**: The system shall provide a section to define properties for the Edge.

**AC-P4-EDGE-CREATE-007**: The system shall provide an "Add Property" button to add new properties.

**AC-P4-EDGE-CREATE-008**: For each property, the system shall provide an input field for property name.

**AC-P4-EDGE-CREATE-009**: The system shall validate that property names are unique within the Edge.

**AC-P4-EDGE-CREATE-010**: For each property, the system shall provide a dropdown for data type selection.

**AC-P4-EDGE-CREATE-011**: The system shall provide the following data type options: STRING, INT64, DOUBLE, BOOL, DATETIME, DATE, TIME, TIMESTAMP.

**AC-P4-EDGE-CREATE-012**: For each property, the system shall provide an optional input field for default value.

**AC-P4-EDGE-CREATE-013**: The system shall validate that the default value matches the selected data type.

**AC-P4-EDGE-CREATE-014**: The system shall provide a "Remove" button for each property to delete it from the list.

**AC-P4-EDGE-CREATE-015**: The system shall provide a "Create" button to submit the form.

**AC-P4-EDGE-CREATE-016**: The system shall provide a "Cancel" button to close the form.

**AC-P4-EDGE-CREATE-017**: The system shall display a preview of the generated Cypher CREATE EDGE statement.

**AC-P4-EDGE-CREATE-018**: When the user submits valid Edge creation data, the system shall execute the CREATE EDGE command.

**AC-P4-EDGE-CREATE-019**: When Edge creation is successful, the system shall display a success message.

**AC-P4-EDGE-CREATE-020**: When Edge creation is successful, the system shall refresh the Edge list.

**AC-P4-EDGE-CREATE-021**: When Edge creation is successful, the system shall close the creation form.

**AC-P4-EDGE-CREATE-022**: When Edge creation fails, the system shall display an error message with details.

**AC-P4-EDGE-CREATE-023**: The system shall display a loading indicator during Edge creation.

---

#### 2.2.3 Edge 修改

**功能概述**

提供修改现有 Edge 类型的功能，支持添加新属性和删除现有属性。

**用户故事**

**US-P4-EDGE-EDIT-001**: 作为数据库开发者，我想要修改 Edge 类型的属性，包括添加新属性或删除现有属性，以便适应关系模型的变化。

**验收标准（EARS 格式）**

**AC-P4-EDGE-EDIT-001**: The system shall provide an "Edit" button for each Edge that opens an edit form.

**AC-P4-EDGE-EDIT-002**: The system shall display the Edge name as read-only in the edit form.

**AC-P4-EDGE-EDIT-003**: The system shall display the list of existing properties for the Edge.

**AC-P4-EDGE-EDIT-004**: For each existing property, the system shall display the property name, data type, and default value (if any).

**AC-P4-EDGE-EDIT-005**: The system shall provide an "Add Property" button to add new properties to the Edge.

**AC-P4-EDGE-EDIT-006**: When adding a new property, the system shall provide fields for property name, data type, and default value.

**AC-P4-EDGE-EDIT-007**: The system shall validate that new property names are unique within the Edge.

**AC-P4-EDGE-EDIT-008**: The system shall provide a "Delete" button for each existing property to remove it from the Edge.

**AC-P4-EDGE-EDIT-009**: When the user clicks "Delete Property", the system shall display a confirmation dialog.

**AC-P4-EDGE-EDIT-010**: The system shall provide a "Save" button to submit the modifications.

**AC-P4-EDGE-EDIT-011**: The system shall provide a "Cancel" button to discard changes.

**AC-P4-EDGE-EDIT-012**: The system shall display a preview of the generated Cypher ALTER EDGE statements.

**AC-P4-EDGE-EDIT-013**: When the user submits Edge modifications, the system shall execute the appropriate ALTER EDGE commands.

**AC-P4-EDGE-EDIT-014**: When Edge modification is successful, the system shall display a success message.

**AC-P4-EDGE-EDIT-015**: When Edge modification is successful, the system shall refresh the Edge list.

**AC-P4-EDGE-EDIT-016**: When Edge modification is successful, the system shall close the edit form.

**AC-P4-EDGE-EDIT-017**: When Edge modification fails, the system shall display an error message with details.

---

#### 2.2.4 Edge 删除

**功能概述**

提供删除 Edge 类型的功能，包含确认对话框以防止误操作。

**用户故事**

**US-P4-EDGE-DELETE-001**: 作为数据库开发者，我想要删除不需要的 Edge 类型，以便清理 Schema。

**验收标准（EARS 格式）**

**AC-P4-EDGE-DELETE-001**: The system shall provide a "Delete" button for each Edge in the list.

**AC-P4-EDGE-DELETE-002**: When the user clicks the "Delete" button, the system shall display a confirmation dialog.

**AC-P4-EDGE-DELETE-003**: The confirmation dialog shall display the Edge name to be deleted.

**AC-P4-EDGE-DELETE-004**: The confirmation dialog shall display a warning message about data loss.

**AC-P4-EDGE-DELETE-005**: The confirmation dialog shall display the number of edges using this Edge type (if available).

**AC-P4-EDGE-DELETE-006**: The confirmation dialog shall provide a "Confirm" button to proceed with deletion.

**AC-P4-EDGE-DELETE-007**: The confirmation dialog shall provide a "Cancel" button to abort deletion.

**AC-P4-EDGE-DELETE-008**: When the user confirms deletion, the system shall execute the DROP EDGE command.

**AC-P4-EDGE-DELETE-009**: When Edge deletion is successful, the system shall display a success message.

**AC-P4-EDGE-DELETE-010**: When Edge deletion is successful, the system shall refresh the Edge list.

**AC-P4-EDGE-DELETE-011**: When Edge deletion fails, the system shall display an error message with details.

**AC-P4-EDGE-DELETE-012**: The system shall display a loading indicator during Edge deletion.

---

#### 2.2.5 Edge 详情查看

**功能概述**

提供查看 Edge 类型详细信息的功能，包括完整属性列表和相关统计。

**用户故事**

**US-P4-EDGE-DETAIL-001**: 作为数据库开发者，我想要查看 Edge 类型的详细信息，包括所有属性的数据类型和默认值，以便了解完整的关系模型。

**验收标准（EARS 格式）**

**AC-P4-EDGE-DETAIL-001**: The system shall provide a "View Details" option for each Edge.

**AC-P4-EDGE-DETAIL-002**: When viewing Edge details, the system shall display the Edge name.

**AC-P4-EDGE-DETAIL-003**: When viewing Edge details, the system shall display the creation time.

**AC-P4-EDGE-DETAIL-004**: When viewing Edge details, the system shall display a table of all properties.

**AC-P4-EDGE-DETAIL-005**: The properties table shall include columns for property name, data type, and default value.

**AC-P4-EDGE-DETAIL-006**: When viewing Edge details, the system shall display the count of edges using this Edge type.

**AC-P4-EDGE-DETAIL-007**: The system shall provide a "Close" button to close the detail view.

**AC-P4-EDGE-DETAIL-008**: The system shall provide an "Edit" button in the detail view to open the edit form.

---

### 2.3 Schema 可视化预览

**功能概述**

提供简单的 Schema 可视化预览，展示 Tag 和 Edge 之间的关系。

**用户故事**

**US-P4-VISUAL-001**: 作为数据库开发者，我想要以图形化方式查看 Schema 结构，以便直观理解 Tag 和 Edge 之间的关系。

**验收标准（EARS 格式）**

**AC-P4-VISUAL-001**: The system shall provide a "Schema Visualization" tab in the Schema management page.

**AC-P4-VISUAL-002**: The system shall display all Tags as nodes in the visualization.

**AC-P4-VISUAL-003**: The system shall display all Edges as connections between Tags (if relationship information is available).

**AC-P4-VISUAL-004**: The system shall allow the user to zoom and pan the visualization.

**AC-P4-VISUAL-005**: When the user clicks a Tag node, the system shall display the Tag details.

**AC-P4-VISUAL-006**: The system shall provide a "Refresh" button to update the visualization.

---

## 3. 非功能需求

### 3.1 性能需求

**NF-P4-PERF-001**: The system shall load the Tag/Edge list within 2 seconds for up to 100 items.

**NF-P4-PERF-002**: The system shall support creating Tags/Edges with up to 50 properties without performance degradation.

**NF-P4-PERF-003**: The system shall provide immediate feedback for user actions (button clicks, form inputs).

### 3.2 可用性需求

**NF-P4-UX-001**: The system shall provide clear error messages when validation fails.

**NF-P4-UX-002**: The system shall prevent form submission when validation errors exist.

**NF-P4-UX-003**: The system shall provide tooltips for complex fields (e.g., data types).

**NF-P4-UX-004**: The system shall support keyboard navigation in forms.

### 3.3 兼容性需求

**NF-P4-COMPAT-001**: The system shall work on modern browsers (Chrome, Firefox, Safari, Edge) latest 2 versions.

**NF-P4-COMPAT-002**: The system shall be responsive for screen sizes from 1366x768 to 1920x1080.

### 3.4 安全需求

**NF-P4-SEC-001**: The system shall validate all user inputs to prevent injection attacks.

**NF-P4-SEC-002**: The system shall sanitize displayed data to prevent XSS attacks.

### 3.5 可靠性需求

**NF-P4-REL-001**: The system shall handle network errors gracefully with retry options.

**NF-P4-REL-002**: The system shall maintain form state during temporary network failures.

---

## 4. 用户界面需求

### 4.1 页面布局

#### 4.1.1 Schema 管理主页面

```
+----------------------------------------------------------+
|  Schema Management                                        |
+----------------------------------------------------------+
|  [Space Selector]  [Tabs: Tags | Edges | Visualization]  |
+----------------------------------------------------------+
|                                                           |
|  +----------------------------------------------------+  |
|  | Search: [____________]  [Create Tag/Edge Button]   |  |
|  +----------------------------------------------------+  |
|  |                                                    |  |
|  |  Tag/Edge List Table                               |  |
|  |  +------+----------+----------+----------+------+ |  |
|  |  | Name | Properties| Created  | Actions  |      |  |
|  |  +------+----------+----------+----------+------+ |  |
|  |  | ...  | ...      | ...      | Edit/Del |      |  |
|  |  +------+----------+----------+----------+------+ |  |
|  |                                                    |  |
|  +----------------------------------------------------+  |
|                                                           |
+----------------------------------------------------------+
```

#### 4.1.2 Tag/Edge 创建/编辑表单

```
+----------------------------------------------------------+
|  Create/Edit Tag/Edge                        [X]         |
+----------------------------------------------------------+
|                                                           |
|  Name: [____________________]                            |
|                                                           |
|  Properties:                                              |
|  +----------------------------------------------------+  |
|  | Name        | Type      | Default    | Action      |  |
|  +----------------------------------------------------+  |
|  | [________]  | [Dropdown]| [________] | [Remove]   |  |
|  | [________]  | [Dropdown]| [________] | [Remove]   |  |
|  +----------------------------------------------------+  |
|  [+ Add Property]                                        |
|                                                           |
|  Cypher Preview:                                          |
|  +----------------------------------------------------+  |
|  | CREATE TAG user (                                   |  |
|  |   name string,                                      |  |
|  |   age int                                           |  |
|  | );                                                  |  |
|  +----------------------------------------------------+  |
|                                                           |
|  [Cancel]                    [Create/Save]               |
|                                                           |
+----------------------------------------------------------+
```

### 4.2 组件需求

| 组件 | 描述 | 来源 |
|------|------|------|
| TagList | Tag 列表展示组件 | 新建 |
| EdgeList | Edge 列表展示组件 | 新建 |
| TagForm | Tag 创建/编辑表单 | 新建 |
| EdgeForm | Edge 创建/编辑表单 | 新建 |
| PropertyEditor | 属性编辑器（可复用） | 新建 |
| SchemaVisualization | Schema 可视化组件 | 新建 |
| DeleteConfirmModal | 删除确认弹窗 | 复用 Ant Design |
| DetailDrawer | 详情抽屉组件 | 复用 Ant Design |

---

## 5. 数据需求

### 5.1 数据模型

#### 5.1.1 Tag 数据模型

```typescript
interface Tag {
  name: string;
  properties: Property[];
  created_at: string;
  comment?: string;
}

interface Property {
  name: string;
  type: DataType;
  default_value?: string;
  nullable: boolean;
}

type DataType = 
  | 'STRING' 
  | 'INT64' 
  | 'DOUBLE' 
  | 'BOOL' 
  | 'DATETIME' 
  | 'DATE' 
  | 'TIME' 
  | 'TIMESTAMP';
```

#### 5.1.2 Edge 数据模型

```typescript
interface Edge {
  name: string;
  properties: Property[];
  created_at: string;
  comment?: string;
}
```

### 5.2 数据验证规则

| 字段 | 规则 | 错误消息 |
|------|------|----------|
| Tag/Edge Name | 必填，字母开头，字母数字下划线 | "名称必须以字母开头，只能包含字母、数字和下划线" |
| Tag/Edge Name | 唯一性检查 | "该名称已存在" |
| Property Name | 必填，字母开头，字母数字下划线 | "属性名必须以字母开头" |
| Property Name | 同一 Schema 内唯一 | "属性名不能重复" |
| Data Type | 必选 | "请选择数据类型" |
| Default Value | 与数据类型匹配 | "默认值与数据类型不匹配" |

### 5.3 数据持久化

- Tag 和 Edge 定义存储在 GraphDB 中
- 前端不缓存 Schema 定义（每次从服务器获取最新数据）
- 表单草稿可存储在 sessionStorage 中防止意外刷新丢失

---

## 6. API 集成需求

### 6.1 API 端点列表

| 端点 | 方法 | 描述 | 请求参数 | 响应数据 |
|------|------|------|----------|----------|
| /api/schema/tags | GET | 获取所有 Tag | space: string | Tag[] |
| /api/schema/tags | POST | 创建 Tag | space: string, name: string, properties: Property[] | { success: boolean } |
| /api/schema/tags/:name | GET | 获取 Tag 详情 | space: string | Tag |
| /api/schema/tags/:name | PUT | 修改 Tag | space: string, properties: Property[] | { success: boolean } |
| /api/schema/tags/:name | DELETE | 删除 Tag | space: string | { success: boolean } |
| /api/schema/edges | GET | 获取所有 Edge | space: string | Edge[] |
| /api/schema/edges | POST | 创建 Edge | space: string, name: string, properties: Property[] | { success: boolean } |
| /api/schema/edges/:name | GET | 获取 Edge 详情 | space: string | Edge |
| /api/schema/edges/:name | PUT | 修改 Edge | space: string, properties: Property[] | { success: boolean } |
| /api/schema/edges/:name | DELETE | 删除 Edge | space: string | { success: boolean } |
| /api/schema/stats | GET | 获取 Schema 统计 | space: string | { tagCount: number, edgeCount: number, vertexCounts: Record<string, number>, edgeCounts: Record<string, number> } |

### 6.2 Cypher 语句示例

```cypher
-- 创建 Tag
CREATE TAG user (name string, age int, email string);

-- 修改 Tag（添加属性）
ALTER TAG user ADD (phone string);

-- 修改 Tag（删除属性）
ALTER TAG user DROP (phone);

-- 删除 Tag
DROP TAG user;

-- 创建 Edge
CREATE EDGE follow (since datetime, strength double);

-- 修改 Edge
ALTER EDGE follow ADD (note string);

-- 删除 Edge
DROP EDGE follow;

-- 查看所有 Tags
SHOW TAGS;

-- 查看 Tag 详情
DESCRIBE TAG user;

-- 查看所有 Edges
SHOW EDGES;

-- 查看 Edge 详情
DESCRIBE EDGE follow;
```

---

## 7. 测试需求

### 7.1 单元测试

| 测试项 | 描述 | 覆盖率要求 |
|--------|------|------------|
| TagList 组件 | 列表渲染、搜索、空状态 | > 80% |
| EdgeList 组件 | 列表渲染、搜索、空状态 | > 80% |
| TagForm 组件 | 表单验证、提交、属性编辑 | > 85% |
| EdgeForm 组件 | 表单验证、提交、属性编辑 | > 85% |
| PropertyEditor 组件 | 属性增删改、类型选择 | > 85% |
| Schema Store | 状态管理、API 调用 | > 80% |
| 验证函数 | 名称验证、类型验证 | > 90% |

### 7.2 集成测试

| 测试场景 | 描述 |
|----------|------|
| 创建 Tag 完整流程 | 打开表单 -> 填写信息 -> 提交 -> 验证列表更新 |
| 修改 Tag 完整流程 | 点击编辑 -> 修改属性 -> 提交 -> 验证更新 |
| 删除 Tag 完整流程 | 点击删除 -> 确认 -> 验证列表更新 |
| Edge 管理完整流程 | 同上 |
| 表单验证 | 各种验证错误的提示和阻止提交 |
| 网络错误处理 | 网络异常时的错误提示和重试 |

### 7.3 端到端测试

| 测试场景 | 描述 |
|----------|------|
| Tag CRUD | 完整的 Tag 增删改查操作 |
| Edge CRUD | 完整的 Edge 增删改查操作 |
| Schema 可视化 | 可视化页面的渲染和交互 |

---

## 8. 交付物

### 8.1 代码交付物

- [ ] Tag 列表页面组件 (`pages/Schema/TagList/`)
- [ ] Edge 列表页面组件 (`pages/Schema/EdgeList/`)
- [ ] Tag 创建/编辑表单组件 (`pages/Schema/components/TagForm/`)
- [ ] Edge 创建/编辑表单组件 (`pages/Schema/components/EdgeForm/`)
- [ ] 属性编辑器组件 (`components/PropertyEditor/`)
- [ ] Schema 可视化组件 (`pages/Schema/SchemaVisualization/`)
- [ ] Schema Store 扩展 (`stores/schema.ts`)
- [ ] Schema 服务 (`services/schema.ts`)
- [ ] 类型定义 (`types/schema.ts`)
- [ ] 工具函数 (`utils/schema.ts`)

### 8.2 测试交付物

- [ ] 单元测试文件（覆盖率 > 75%）
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
| Tag 列表 | 正确显示所有 Tag，支持搜索 | 手动测试 |
| Tag 创建 | 能成功创建带属性的 Tag | 手动测试 + 自动化测试 |
| Tag 修改 | 能添加和删除属性 | 手动测试 + 自动化测试 |
| Tag 删除 | 能删除 Tag 并刷新列表 | 手动测试 + 自动化测试 |
| Edge 列表 | 正确显示所有 Edge | 手动测试 |
| Edge 创建 | 能成功创建带属性的 Edge | 手动测试 + 自动化测试 |
| Edge 修改 | 能添加和删除属性 | 手动测试 + 自动化测试 |
| Edge 删除 | 能删除 Edge 并刷新列表 | 手动测试 + 自动化测试 |
| Schema 可视化 | 正确渲染 Tag 和 Edge 关系 | 手动测试 |

### 9.2 质量验收标准

- 代码覆盖率 > 75%
- 无严重 Bug
- 无性能问题（页面加载 < 2s）
- 通过代码审查

---

## 10. 风险和假设

### 10.1 风险

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 后端 API 变更 | 高 | 及时沟通，使用接口契约 |
| Cypher 语法差异 | 中 | 确认 GraphDB 的 Cypher 支持 |
| 复杂属性类型 | 中 | 先支持基础类型，后续扩展 |

### 10.2 假设

- 后端 API 按约定提供
- GraphDB 支持标准 Cypher DDL 语句
- 用户有基本的数据库概念

---

## 11. 附录

### 11.1 术语表

| 术语 | 定义 |
|------|------|
| Tag | 标签，定义节点的类型和属性结构 |
| Edge | 边类型，定义关系的类型和属性结构 |
| Property | 属性，Tag 或 Edge 的字段定义 |
| Data Type | 数据类型，如 STRING、INT64 等 |
| Schema | 模式，数据库的结构定义 |

### 11.2 参考文档

- [GraphDB Cypher 文档](../api/cypher.md)
- [阶段 3 PRD](./prd_phase3.md)
- [阶段 5 PRD](./prd_phase5.md)
- [nebula-studio Schema 实现参考](../../ref/nebula-studio-3.10.0/app/pages/Schema/)

### 11.3 变更历史

| 版本 | 日期 | 变更内容 | 作者 |
|------|------|----------|------|
| v1.0 | 2026-03-29 | 初始版本 | - |

---

**文档结束**
