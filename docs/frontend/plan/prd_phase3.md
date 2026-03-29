# GraphDB 前端 PRD - 阶段 3: Schema 管理 - Space

**文档版本**: v1.0
**创建日期**: 2026-03-29
**最后更新**: 2026-03-29
**项目阶段**: Phase 3 - Schema 管理 - Space
**预计工期**: 1 周

---

## 1. 引言（Introduction）

### 1.1 阶段目标

阶段 3 的目标是实现 Space（图空间）的完整管理功能，包括列表展示、创建、删除和详情查看。Space 是 GraphDB 中数据组织的顶层容器，用户需要先创建 Space 才能在其中定义 Tag 和 Edge 类型。

### 1.2 核心价值

- **数据组织**: 提供清晰的数据组织层次，支持多 Space 管理
- **Schema 基础**: 为 Tag 和 Edge 管理奠定基础
- **用户友好**: 提供直观的 Space 管理界面，降低使用门槛

### 1.3 目标用户

- **数据库开发者**: 需要创建和管理 Space 来组织数据
- **系统管理员**: 需要监控 Space 使用情况和统计信息

### 1.4 范围

**包含范围**:
- Space 列表展示
- Space 创建（配置 Vid Type、分区数、副本数）
- Space 删除（带确认）
- Space 详情查看
- Space 统计信息（节点数、边数）
- Space 切换功能

**不包含范围**:
- Tag 管理（阶段 4）
- Edge 管理（阶段 4）
- 索引管理（阶段 5）

---

## 2. 功能需求

### 2.1 Space 列表展示

#### 2.1.1 功能概述

展示数据库中所有可用的 Space，包括基本信息和统计信息。

#### 2.1.2 用户故事

**US-P3-LIST-001**: 作为数据库开发者，我想要查看所有可用的 Space，以便了解数据库的结构。

**US-P3-LIST-002**: 作为数据库开发者，我想要看到 Space 的基本信息（名称、Vid Type、创建时间），以便快速了解 Space 配置。

**US-P3-LIST-003**: 作为数据库开发者，我想要看到 Space 的统计信息（节点数、边数），以便监控数据规模。

#### 2.1.3 验收标准（EARS 格式）

**AC-P3-LIST-001**: The system shall display a list of all Spaces in the database.

**AC-P3-LIST-002**: The system shall display the Space name for each Space in the list.

**AC-P3-LIST-003**: The system shall display the Vid Type for each Space in the list.

**AC-P3-LIST-004**: The system shall display the partition number for each Space in the list.

**AC-P3-LIST-005**: The system shall display the replica factor for each Space in the list.

**AC-P3-LIST-006**: The system shall display the creation time for each Space in the list.

**AC-P3-LIST-007**: The system shall display the vertex count for each Space in the list.

**AC-P3-LIST-008**: The system shall display the edge count for each Space in the list.

**AC-P3-LIST-009**: The system shall provide a "Refresh" button to update the Space list.

**AC-P3-LIST-010**: The system shall display a loading indicator when fetching the Space list.

**AC-P3-LIST-011**: The system shall display an empty state message when no Spaces exist.

**AC-P3-LIST-012**: The system shall provide a "Create Space" button in the list view.

---

### 2.2 Space 创建

#### 2.2.1 功能概述

提供创建新 Space 的功能，支持配置 Vid Type、分区数和副本数。

#### 2.2.2 用户故事

**US-P3-CREATE-001**: 作为数据库开发者，我想要创建新的 Space，以便组织不同类型的数据。

**US-P3-CREATE-002**: 作为数据库开发者，我想要配置 Space 的 Vid Type，以便选择合适的顶点 ID 类型。

**US-P3-CREATE-003**: 作为数据库开发者，我想要配置 Space 的分区数和副本数，以便优化性能和可靠性。

#### 2.2.3 验收标准（EARS 格式）

**AC-P3-CREATE-001**: The system shall provide a "Create Space" button that opens a creation form.

**AC-P3-CREATE-002**: The system shall provide an input field for Space name in the creation form.

**AC-P3-CREATE-003**: The system shall validate that the Space name is not empty.

**AC-P3-CREATE-004**: The system shall validate that the Space name follows naming conventions (alphanumeric and underscores, starting with a letter).

**AC-P3-CREATE-005**: The system shall validate that the Space name is unique (not already exists).

**AC-P3-CREATE-006**: The system shall provide a dropdown for Vid Type selection.

**AC-P3-CREATE-007**: The system shall provide the following Vid Type options: INT64, FIXED_STRING(32).

**AC-P3-CREATE-008**: The system shall provide an input field for partition number.

**AC-P3-CREATE-009**: The system shall provide a default value of 100 for partition number.

**AC-P3-CREATE-010**: The system shall validate that partition number is a positive integer.

**AC-P3-CREATE-011**: The system shall provide an input field for replica factor.

**AC-P3-CREATE-012**: The system shall provide a default value of 1 for replica factor.

**AC-P3-CREATE-013**: The system shall validate that replica factor is a positive integer.

**AC-P3-CREATE-014**: The system shall provide a "Create" button to submit the form.

**AC-P3-CREATE-015**: The system shall provide a "Cancel" button to close the form.

**AC-P3-CREATE-016**: When the user submits valid Space creation data, the system shall execute the CREATE SPACE command.

**AC-P3-CREATE-017**: When Space creation is successful, the system shall display a success message.

**AC-P3-CREATE-018**: When Space creation is successful, the system shall refresh the Space list.

**AC-P3-CREATE-019**: When Space creation is successful, the system shall close the creation form.

**AC-P3-CREATE-020**: When Space creation fails, the system shall display an error message with details.

**AC-P3-CREATE-021**: The system shall display a loading indicator during Space creation.

---

### 2.3 Space 删除

#### 2.3.1 功能概述

提供删除 Space 的功能，包含确认对话框以防止误操作。

#### 2.3.2 用户故事

**US-P3-DELETE-001**: 作为数据库开发者，我想要删除不需要的 Space，以便释放存储空间。

**US-P3-DELETE-002**: 作为数据库开发者，我想要在删除前看到确认提示，以避免误删除数据。

#### 2.3.3 验收标准（EARS 格式）

**AC-P3-DELETE-001**: The system shall provide a "Delete" button for each Space in the list.

**AC-P3-DELETE-002**: When the user clicks the "Delete" button, the system shall display a confirmation dialog.

**AC-P3-DELETE-003**: The confirmation dialog shall display the Space name to be deleted.

**AC-P3-DELETE-004**: The confirmation dialog shall display a warning message about data loss.

**AC-P3-DELETE-005**: The confirmation dialog shall provide a "Confirm" button to proceed with deletion.

**AC-P3-DELETE-006**: The confirmation dialog shall provide a "Cancel" button to abort deletion.

**AC-P3-DELETE-007**: When the user confirms deletion, the system shall execute the DROP SPACE command.

**AC-P3-DELETE-008**: When Space deletion is successful, the system shall display a success message.

**AC-P3-DELETE-009**: When Space deletion is successful, the system shall refresh the Space list.

**AC-P3-DELETE-010**: When Space deletion fails, the system shall display an error message with details.

**AC-P3-DELETE-011**: The system shall display a loading indicator during Space deletion.

---

### 2.4 Space 详情查看

#### 2.4.1 功能概述

提供查看 Space 详细信息的功能，包括配置参数和统计信息。

#### 2.4.2 用户故事

**US-P3-DETAIL-001**: 作为数据库开发者，我想要查看 Space 的详细信息，包括 Vid Type 和副本数，以便了解配置。

**US-P3-DETAIL-002**: 作为数据库开发者，我想要查看 Space 的统计信息，包括节点和边的数量，以便监控数据规模。

#### 2.4.3 验收标准（EARS 格式）

**AC-P3-DETAIL-001**: The system shall provide a "View Details" option for each Space in the list.

**AC-P3-DETAIL-002**: When the user clicks "View Details", the system shall display a details panel or modal.

**AC-P3-DETAIL-003**: The details panel shall display the Space name.

**AC-P3-DETAIL-004**: The details panel shall display the Vid Type.

**AC-P3-DETAIL-005**: The details panel shall display the partition number.

**AC-P3-DETAIL-006**: The details panel shall display the replica factor.

**AC-P3-DETAIL-007**: The details panel shall display the creation time.

**AC-P3-DETAIL-008**: The details panel shall display the vertex count.

**AC-P3-DETAIL-009**: The details panel shall display the edge count.

**AC-P3-DETAIL-010**: The details panel shall provide a "Close" button.

**AC-P3-DETAIL-011**: The details panel shall provide a "Refresh Statistics" button to update the statistics.

---

### 2.5 Space 切换

#### 2.5.1 功能概述

提供在当前 Space 之间切换的功能，影响后续的 Schema 操作和数据查询。

#### 2.5.2 用户故事

**US-P3-SWITCH-001**: 作为数据库开发者，我想要切换当前使用的 Space，以便在不同 Space 之间工作。

**US-P3-SWITCH-002**: 作为数据库开发者，我想要看到当前选中的 Space，以便确认工作上下文。

#### 2.5.3 验收标准（EARS 格式）

**AC-P3-SWITCH-001**: The system shall display the current Space name in the application header or sidebar.

**AC-P3-SWITCH-002**: The system shall provide a Space selector dropdown in the header or sidebar.

**AC-P3-SWITCH-003**: The Space selector shall list all available Spaces.

**AC-P3-SWITCH-004**: The Space selector shall highlight the currently selected Space.

**AC-P3-SWITCH-005**: When the user selects a different Space, the system shall switch to that Space.

**AC-P3-SWITCH-006**: When switching Spaces, the system shall refresh the Schema data for the new Space.

**AC-P3-SWITCH-007**: When switching Spaces, the system shall display a success message.

**AC-P3-SWITCH-008**: The system shall persist the selected Space in the state management store.

**AC-P3-SWITCH-009**: The system shall display a breadcrumb navigation showing the current Space.

---

## 3. 非功能需求

### 3.1 性能需求

**NFR-P3-001**: The system shall load the Space list within 1 second on a standard broadband connection.

**NFR-P3-002**: The system shall complete Space creation within 2 seconds after form submission.

**NFR-P3-003**: The system shall complete Space deletion within 2 seconds after confirmation.

**NFR-P3-004**: The system shall display Space statistics updates within 1 second.

### 3.2 可用性需求

**NFR-P3-005**: The system shall provide clear and helpful error messages for Space operations.

**NFR-P3-006**: The system shall provide inline validation feedback for form fields.

**NFR-P3-007**: The system shall be accessible to users with visual impairments (WCAG 2.1 Level AA compliance).

### 3.3 兼容性需求

**NFR-P3-008**: The system shall support the latest versions of Chrome, Firefox, Safari, and Edge.

**NFR-P3-009**: The system shall support responsive design for screen resolutions from 1366x768 to 1920x1080.

### 3.4 安全需求

**NFR-P3-010**: The system shall validate all user inputs on the client side before sending to the server.

**NFR-P3-011**: The system shall require confirmation before deleting a Space.

**NFR-P3-012**: The system shall not allow deletion of system Spaces.

### 3.5 可靠性需求

**NFR-P3-013**: The system shall handle network errors gracefully and display appropriate messages.

**NFR-P3-014**: The system shall automatically refresh the Space list after create/delete operations.

---

## 4. 用户界面需求

### 4.1 Space 列表页面

**UI-P3-001**: The system shall display a page title "Space Management" at the top of the page.

**UI-P3-002**: The system shall display a "Create Space" button in the top-right corner.

**UI-P3-003**: The system shall display the Space list in a table format.

**UI-P3-004**: The table shall have the following columns: Name, Vid Type, Partitions, Replicas, Vertices, Edges, Created At, Actions.

**UI-P3-005**: The Actions column shall contain "View Details" and "Delete" buttons.

**UI-P3-006**: The system shall display a loading spinner when fetching data.

**UI-P3-007**: The system shall display an empty state with a message when no Spaces exist.

**UI-P3-008**: The system shall use Ant Design Table component for consistent styling.

### 4.2 Space 创建表单

**UI-P3-009**: The system shall display the creation form in a modal dialog.

**UI-P3-010**: The system shall display a form title "Create New Space".

**UI-P3-011**: The system shall provide input fields with labels: Space Name, Vid Type, Partition Number, Replica Factor.

**UI-P3-012**: The system shall provide placeholder text for each input field.

**UI-P3-013**: The system shall provide help text explaining Vid Type options.

**UI-P3-014**: The system shall provide "Create" and "Cancel" buttons at the bottom of the form.

**UI-P3-015**: The system shall highlight required fields with an asterisk.

**UI-P3-016**: The system shall display validation errors below the corresponding fields.

### 4.3 Space 删除确认

**UI-P3-017**: The system shall display a confirmation modal when deleting a Space.

**UI-P3-018**: The confirmation modal shall display a warning icon.

**UI-P3-019**: The confirmation modal shall display the message: "Are you sure you want to delete Space '{name}'? This action cannot be undone and all data in this Space will be lost."

**UI-P3-020**: The confirmation modal shall provide "Delete" and "Cancel" buttons.

**UI-P3-021**: The "Delete" button shall be styled as a danger button (red).

### 4.4 Space 详情面板

**UI-P3-022**: The system shall display Space details in a modal dialog or drawer.

**UI-P3-023**: The system shall organize details into sections: Basic Info, Configuration, Statistics.

**UI-P3-024**: The Basic Info section shall display: Name, Created At.

**UI-P3-025**: The Configuration section shall display: Vid Type, Partition Number, Replica Factor.

**UI-P3-026**: The Statistics section shall display: Vertex Count, Edge Count.

**UI-P3-027**: The system shall provide a "Refresh Statistics" button in the Statistics section.

**UI-P3-028**: The system shall provide a "Close" button at the bottom of the panel.

### 4.5 Space 选择器

**UI-P3-029**: The system shall display the current Space name in the header or sidebar.

**UI-P3-030**: The system shall provide a dropdown selector for switching Spaces.

**UI-P3-031**: The dropdown shall display all available Spaces.

**UI-P3-032**: The current Space shall be highlighted in the dropdown.

---

## 5. 数据需求

### 5.1 数据模型

**Space 数据结构**:
```typescript
interface Space {
  name: string;
  vidType: 'INT64' | 'FIXED_STRING(32)';
  partitionNum: number;
  replicaFactor: number;
  createdAt: string;
  vertexCount: number;
  edgeCount: number;
}
```

**Space 创建参数**:
```typescript
interface CreateSpaceParams {
  name: string;
  vidType: 'INT64' | 'FIXED_STRING(32)';
  partitionNum: number;
  replicaFactor: number;
}
```

### 5.2 数据持久化

**DATA-P3-001**: The system shall store the currently selected Space in the Zustand store.

**DATA-P3-002**: The system shall persist the selected Space in localStorage.

**DATA-P3-003**: The system shall load the selected Space from localStorage on application startup.

### 5.3 数据验证

**DATA-P3-004**: The system shall validate that Space name is not empty.

**DATA-P3-005**: The system shall validate that Space name matches the pattern: `^[a-zA-Z][a-zA-Z0-9_]*$`.

**DATA-P3-006**: The system shall validate that Space name length is between 1 and 64 characters.

**DATA-P3-007**: The system shall validate that partition number is a positive integer.

**DATA-P3-008**: The system shall validate that replica factor is a positive integer.

---

## 6. API 集成需求

### 6.1 Space API

**API-P3-001**: The system shall call the `/api/schema/spaces` endpoint with GET method to fetch the list of Spaces.

**API-P3-002**: The system shall send the session ID in the request header.

**API-P3-003**: The system shall call the `/api/schema/spaces` endpoint with POST method to create a new Space.

**API-P3-004**: The system shall send the Space creation parameters in the request body.

**API-P3-005**: The system shall call the `/api/schema/spaces/{name}` endpoint with DELETE method to delete a Space.

**API-P3-006**: The system shall send the session ID in the request header for delete operations.

**API-P3-007**: The system shall call the `/api/schema/spaces/{name}/stats` endpoint with GET method to fetch Space statistics.

**API-P3-008**: The system shall call the `/api/query/execute` endpoint with POST method to execute Cypher commands for Space operations.

**API-P3-009**: The system shall handle API errors (400, 401, 403, 404, 500) appropriately.

### 6.2 Cypher 命令

**创建 Space**:
```cypher
CREATE SPACE IF NOT EXISTS {name} (vid_type = {vidType}, partition_num = {partitionNum}, replica_factor = {replicaFactor})
```

**删除 Space**:
```cypher
DROP SPACE IF EXISTS {name}
```

**列出所有 Spaces**:
```cypher
SHOW SPACES
```

**描述 Space**:
```cypher
DESCRIBE SPACE {name}
```

**使用 Space**:
```cypher
USE {name}
```

**获取统计信息**:
```cypher
SHOW STATS
```

---

## 7. 测试需求

### 7.1 单元测试

**TEST-P3-001**: The system shall have unit tests for Space form validation logic.

**TEST-P3-002**: The system shall have unit tests for the Space API service functions.

**TEST-P3-003**: The system shall have unit tests for the Space state management store.

**TEST-P3-004**: The system shall have unit tests for the Space list component.

**TEST-P3-005**: The system shall have unit tests for the Space creation form component.

### 7.2 集成测试

**TEST-P3-006**: The system shall have integration tests for the complete Space creation flow.

**TEST-P3-007**: The system shall have integration tests for the Space deletion flow.

**TEST-P3-008**: The system shall have integration tests for the Space switching flow.

### 7.3 端到端测试

**TEST-P3-009**: The system shall have E2E tests for the complete Space management user journey.

**TEST-P3-010**: The system shall have E2E tests for the Space creation validation scenarios.

---

## 8. 交付物

### 8.1 代码交付

**DEL-P3-001**: Space list page component (`pages/Schema/SpaceList/`).

**DEL-P3-002**: Space creation form component (`pages/Schema/components/SpaceCreateForm/`).

**DEL-P3-003**: Space details modal component (`pages/Schema/components/SpaceDetailModal/`).

**DEL-P3-004**: Space selector component (`components/business/SpaceSelector/`).

**DEL-P3-005**: Schema state management store (`stores/schema.ts`).

**DEL-P3-006**: Space API service functions (`services/schema.ts`).

**DEL-P3-007**: Cypher query generation utilities (`utils/gql.ts` updates).

### 8.2 文档交付

**DEL-P3-008**: Space management feature documentation.

**DEL-P3-009**: API integration documentation for Space endpoints.

**DEL-P3-010**: Component documentation for Space-related components.

### 8.3 测试交付

**DEL-P3-011**: Unit test suite with all test cases passing.

**DEL-P3-012**: Integration test suite with all test cases passing.

**DEL-P3-013**: E2E test suite with all test cases passing.

**DEL-P3-014**: Test execution report with coverage metrics (target > 70%).

---

## 9. 验收标准

### 9.1 功能验收

**ACCEPT-P3-001**: All user stories in Section 2 are implemented and meet their acceptance criteria.

**ACCEPT-P3-002**: All non-functional requirements in Section 3 are met.

**ACCEPT-P3-003**: All user interface requirements in Section 4 are implemented.

**ACCEPT-P3-004**: All data requirements in Section 5 are implemented.

**ACCEPT-P3-005**: All API integration requirements in Section 6 are working correctly.

### 9.2 质量验收

**ACCEPT-P3-006**: The application passes all unit tests with at least 70% code coverage.

**ACCEPT-P3-007**: The application passes all integration tests.

**ACCEPT-P3-008**: The application passes all E2E tests.

**ACCEPT-P3-009**: The application has no critical or high-severity bugs.

**ACCEPT-P3-010**: The application has been tested on all supported browsers (Chrome, Firefox, Safari, Edge).

### 9.3 性能验收

**ACCEPT-P3-011**: The Space list loads within 1 second.

**ACCEPT-P3-012**: Space creation completes within 2 seconds.

**ACCEPT-P3-013**: Space deletion completes within 2 seconds.

---

## 10. 风险和假设

### 10.1 风险

**RISK-P3-001**: The GraphDB backend may not support all Space management APIs.

**RISK-P3-002**: Space creation may take longer than expected for large partition numbers.

**RISK-P3-003**: Space deletion may fail if there are existing data or dependencies.

### 10.2 假设

**ASSUME-P3-001**: The GraphDB backend provides RESTful APIs for Space operations.

**ASSUME-P3-002**: The GraphDB backend supports Cypher commands for Space management.

**ASSUME-P3-003**: The development team has access to a GraphDB instance with admin privileges.

**ASSUME-P3-004**: Space statistics can be fetched via API or Cypher queries.

---

## 11. 附录

### 11.1 术语表

| 术语 | 定义 |
|------|------|
| Space | 图空间，GraphDB 中数据组织的顶层容器 |
| Vid Type | 顶点 ID 类型，支持 INT64 或 FIXED_STRING |
| Partition | 分区，用于数据分布和并行处理 |
| Replica Factor | 副本因子，用于数据冗余和可靠性 |
| Schema | 数据库结构定义，包括 Space、Tag、Edge 等 |

### 11.2 参考文档

- [GraphDB 后端 API 文档](../../api/server/README.md)
- [GraphDB 前端阶段规划](../phase_planning.md)
- [GraphDB 前端技术栈](../architecture/tech_stack.md)
- [GraphDB 前端目录结构](../architecture/directory_structure.md)
- [阶段1 PRD](../prd_phase1.md)
- [阶段2 PRD](./prd_phase2.md)

### 11.3 变更历史

| 版本 | 日期 | 作者 | 变更描述 |
|------|------|------|----------|
| v1.0 | 2026-03-29 | - | 初始版本，定义阶段 3 需求 |

---

**文档结束**
