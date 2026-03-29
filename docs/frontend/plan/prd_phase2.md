# GraphDB 前端 PRD - 阶段 2: 查询控制台

**文档版本**: v1.0
**创建日期**: 2026-03-29
**最后更新**: 2026-03-29
**项目阶段**: Phase 2 - 查询控制台
**预计工期**: 2 周
**依赖阶段**: Phase 1 - 基础框架和连接管理

---

## 1. 引言（Introduction）

### 1.1 阶段目标

阶段 2 的目标是实现完整的 Cypher 查询控制台功能，提供查询编辑、执行、结果展示、历史记录和收藏功能。这是 GraphDB 前端的核心功能模块，为用户提供与数据库交互的主要界面。

### 1.2 核心价值

- **查询编辑**: 提供舒适的 Cypher 查询编写环境
- **即时执行**: 快速执行查询并获取结果
- **多样展示**: 支持表格和 JSON 两种结果展示方式
- **查询管理**: 保存查询历史和收藏常用查询
- **数据导出**: 支持将查询结果导出为 CSV 或 JSON 格式

### 1.3 目标用户

- **数据库开发者**: 需要编写和执行 Cypher 查询
- **数据分析师**: 需要浏览和分析图数据
- **系统管理员**: 需要监控和调试数据库

### 1.4 范围

**包含范围**:
- Cypher 查询编辑器（基于 Ant Design Input.TextArea 简化实现）
- 查询执行引擎（单条/批量）
- 结果展示组件（表格视图、JSON 视图）
- 查询历史记录管理
- 查询收藏功能
- 结果导出功能（CSV、JSON）

**不包含范围**:
- Monaco Editor 集成（后续阶段升级）
- Cypher 语法高亮（后续阶段升级）
- 自动补全功能（后续阶段升级）
- 图可视化展示（阶段 6）

---

## 2. 功能需求

### 2.1 查询编辑器

#### 2.1.1 功能概述

提供一个多行文本编辑器，用于编写 Cypher 查询语句。阶段 2 使用 Ant Design 的 Input.TextArea 组件实现基础功能，后续可升级为 Monaco Editor。

#### 2.1.2 用户故事

**US-P2-EDITOR-001**: 作为数据库开发者，我想要在一个多行文本编辑器中编写 Cypher 查询，以便能够方便地编辑和调试查询语句。

**US-P2-EDITOR-002**: 作为数据库开发者，我想要通过快捷键执行查询，以便提高查询效率。

**US-P2-EDITOR-003**: 作为数据库开发者，我想要编辑器支持 Tab 键缩进，以便格式化查询语句。

**US-P2-EDITOR-004**: 作为数据库开发者，我想要看到当前正在执行的查询，以便了解执行状态。

#### 2.1.3 验收标准（EARS 格式）

**AC-P2-EDITOR-001**: The system shall provide a multi-line text editor using Ant Design Input.TextArea component.

**AC-P2-EDITOR-002**: The system shall set the editor minimum height to 150px and maximum height to 400px.

**AC-P2-EDITOR-003**: The system shall allow users to input multiple Cypher queries separated by semicolons.

**AC-P2-EDITOR-004**: The system shall support Tab key for indentation in the editor.

**AC-P2-EDITOR-005**: When the user presses Ctrl+Enter (or Cmd+Enter on Mac), the system shall execute the current query.

**AC-P2-EDITOR-006**: When the user presses Shift+Enter, the system shall execute the current query.

**AC-P2-EDITOR-007**: The system shall provide an "Execute" button to execute the current query.

**AC-P2-EDITOR-008**: When a query is executing, the system shall display a loading indicator on the Execute button.

**AC-P2-EDITOR-009**: When a query is executing, the system shall disable the Execute button to prevent duplicate submissions.

**AC-P2-EDITOR-010**: The system shall provide a "Clear" button to clear the editor content.

**AC-P2-EDITOR-011**: The system shall display the current cursor position (line and column) in the editor status bar.

**AC-P2-EDITOR-012**: The system shall support keyboard shortcut Ctrl+/ (or Cmd+/ on Mac) to comment/uncomment the current line.

---

### 2.2 查询执行

#### 2.2.1 功能概述

实现查询执行引擎，支持单条查询和批量查询执行，处理查询进度和错误信息。

#### 2.2.2 用户故事

**US-P2-EXEC-001**: 作为数据库开发者，我想要执行单条 Cypher 查询，以便获取查询结果。

**US-P2-EXEC-002**: 作为数据库开发者，我想要批量执行多条 Cypher 查询，以便一次性执行多个操作。

**US-P2-EXEC-003**: 作为数据库开发者，我想要看到查询执行进度，以便了解执行状态。

**US-P2-EXEC-004**: 作为数据库开发者，我想要看到详细的错误信息，以便快速定位和修复问题。

#### 2.2.3 验收标准（EARS 格式）

**AC-P2-EXEC-001**: The system shall parse the editor content and identify individual queries separated by semicolons.

**AC-P2-EXEC-002**: When executing a single query, the system shall call the `/api/query/execute` endpoint with POST method.

**AC-P2-EXEC-003**: When executing multiple queries, the system shall call the `/api/query/execute` endpoint for each query sequentially.

**AC-P2-EXEC-004**: The system shall send the query string and session ID in the request body.

**AC-P2-EXEC-005**: The system shall set a timeout of 30 seconds for query execution.

**AC-P2-EXEC-006**: When query execution starts, the system shall display a progress indicator.

**AC-P2-EXEC-007**: When query execution completes successfully, the system shall display the execution time in milliseconds.

**AC-P2-EXEC-008**: When query execution fails, the system shall display an error message with error code and description.

**AC-P2-EXEC-009**: When query execution fails due to syntax error, the system shall display the error position if available.

**AC-P2-EXEC-010**: When query execution fails due to connection error, the system shall display a connection error message.

**AC-P2-EXEC-011**: When executing multiple queries, the system shall display results for each query separately.

**AC-P2-EXEC-012**: When executing multiple queries, if one query fails, the system shall stop execution and display the error.

**AC-P2-EXEC-013**: The system shall limit the maximum result set to 1000 rows per query.

**AC-P2-EXEC-014**: When the result set exceeds the limit, the system shall display a warning message and truncate the results.

---

### 2.3 结果展示

#### 2.3.1 功能概述

提供多种方式展示查询结果，包括表格视图和 JSON 视图，支持结果排序和筛选。

#### 2.3.2 用户故事

**US-P2-RESULT-001**: 作为数据库开发者，我想要以表格形式查看查询结果，以便快速浏览数据。

**US-P2-RESULT-002**: 作为数据库开发者，我想要以 JSON 格式查看查询结果，以便复制和集成到其他工具中。

**US-P2-RESULT-003**: 作为数据库开发者，我想要对表格结果进行排序，以便分析数据。

**US-P2-RESULT-004**: 作为数据库开发者，我想要查看查询执行时间和返回行数，以便评估查询性能。

#### 2.3.3 验收标准（EARS 格式）

**AC-P2-RESULT-001**: The system shall display query results in a table format by default.

**AC-P2-RESULT-002**: The system shall automatically generate column headers based on the returned data structure.

**AC-P2-RESULT-003**: The system shall support horizontal scrolling for tables with many columns.

**AC-P2-RESULT-004**: The system shall support vertical scrolling for tables with many rows.

**AC-P2-RESULT-005**: The system shall display a maximum of 100 rows per page in the table view.

**AC-P2-RESULT-006**: The system shall provide pagination controls for navigating through result pages.

**AC-P2-RESULT-007**: The system shall allow users to click on column headers to sort the data in ascending or descending order.

**AC-P2-RESULT-008**: The system shall provide a toggle button to switch between Table view and JSON view.

**AC-P2-RESULT-009**: When displaying results in JSON format, the system shall format the JSON with proper indentation.

**AC-P2-RESULT-010**: When displaying results in JSON format, the system shall provide syntax highlighting.

**AC-P2-RESULT-011**: The system shall display the total number of rows returned by the query.

**AC-P2-RESULT-012**: The system shall display the query execution time in milliseconds.

**AC-P2-RESULT-013**: When no results are returned, the system shall display an empty state message.

**AC-P2-RESULT-014**: When the query returns an error, the system shall display the error message in the result area.

**AC-P2-RESULT-015**: The system shall handle large result sets efficiently without blocking the UI.

---

### 2.4 查询历史

#### 2.4.1 功能概述

保存用户执行的查询历史，支持查看、重新执行和清理历史记录。

#### 2.4.2 用户故事

**US-P2-HISTORY-001**: 作为数据库开发者，我想要查看我的查询历史记录，以便重新执行之前的查询。

**US-P2-HISTORY-002**: 作为数据库开发者，我想要从历史记录中加载查询到编辑器，以便修改和重新执行。

**US-P2-HISTORY-003**: 作为数据库开发者，我想要清理查询历史记录，以便保护隐私或释放存储空间。

**US-P2-HISTORY-004**: 作为数据库开发者，我想要看到每条历史记录的执行时间，以便找到最近执行的查询。

#### 2.4.3 验收标准（EARS 格式）

**AC-P2-HISTORY-001**: The system shall save each executed query to the query history.

**AC-P2-HISTORY-002**: The system shall store query history in localStorage with a maximum of 50 entries.

**AC-P2-HISTORY-003**: The system shall display a "History" button to open the history panel.

**AC-P2-HISTORY-004**: When the user clicks the "History" button, the system shall display a side panel with the query history list.

**AC-P2-HISTORY-005**: The system shall display each history entry with the query text (truncated to 100 characters), execution time, and timestamp.

**AC-P2-HISTORY-006**: When the user clicks on a history entry, the system shall load the query into the editor.

**AC-P2-HISTORY-007**: When the user clicks on a history entry, the system shall close the history panel.

**AC-P2-HISTORY-008**: The system shall provide a "Clear History" button to remove all history records.

**AC-P2-HISTORY-009**: When the user clicks "Clear History", the system shall display a confirmation dialog.

**AC-P2-HISTORY-010**: When the user confirms clearing history, the system shall remove all history records from localStorage.

**AC-P2-HISTORY-011**: The system shall display history entries in reverse chronological order (newest first).

**AC-P2-HISTORY-012**: When the history exceeds 50 entries, the system shall remove the oldest entries automatically.

**AC-P2-HISTORY-013**: The system shall persist query history across browser sessions.

---

### 2.5 查询收藏

#### 2.5.1 功能概述

允许用户收藏常用查询，方便快速访问和执行。

#### 2.5.2 用户故事

**US-P2-FAVORITE-001**: 作为数据库开发者，我想要收藏常用的查询语句，以便快速执行重复性查询。

**US-P2-FAVORITE-002**: 作为数据库开发者，我想要为收藏的查询添加自定义名称，以便识别不同的查询。

**US-P2-FAVORITE-003**: 作为数据库开发者，我想要删除不再需要的收藏查询，以便管理收藏列表。

**US-P2-FAVORITE-004**: 作为数据库开发者，我想要从收藏中直接执行查询，而无需先加载到编辑器。

#### 2.5.3 验收标准（EARS 格式）

**AC-P2-FAVORITE-001**: The system shall provide a "Save to Favorites" button in the editor toolbar.

**AC-P2-FAVORITE-002**: When the user clicks "Save to Favorites", the system shall display a dialog to enter the query name.

**AC-P2-FAVORITE-003**: The system shall validate that the query name is not empty and unique.

**AC-P2-FAVORITE-004**: When the user submits a valid query name, the system shall save the current query to favorites in localStorage.

**AC-P2-FAVORITE-005**: The system shall store favorites with name, query text, and creation timestamp.

**AC-P2-FAVORITE-006**: The system shall display a "Favorites" button to open the favorites panel.

**AC-P2-FAVORITE-007**: When the user clicks the "Favorites" button, the system shall display a side panel with the favorites list.

**AC-P2-FAVORITE-008**: The system shall display each favorite entry with the query name and creation date.

**AC-P2-FAVORITE-009**: When the user clicks on a favorite entry, the system shall load the query into the editor.

**AC-P2-FAVORITE-010**: The system shall provide a "Execute" button for each favorite entry to execute the query directly.

**AC-P2-FAVORITE-011**: The system shall provide a "Delete" button for each favorite entry to remove it from favorites.

**AC-P2-FAVORITE-012**: When the user clicks "Delete" on a favorite entry, the system shall display a confirmation dialog.

**AC-P2-FAVORITE-013**: When the user confirms deletion, the system shall remove the favorite from localStorage.

**AC-P2-FAVORITE-014**: The system shall persist favorites across browser sessions.

**AC-P2-FAVORITE-015**: The system shall allow a maximum of 30 favorite queries.

---

### 2.6 结果导出

#### 2.6.1 功能概述

支持将查询结果导出为 CSV 或 JSON 格式文件，方便离线分析和分享。

#### 2.6.2 用户故事

**US-P2-EXPORT-001**: 作为数据库开发者，我想要导出查询结果为 CSV 文件，以便在 Excel 中分析数据。

**US-P2-EXPORT-002**: 作为数据库开发者，我想要导出查询结果为 JSON 文件，以便在其他程序中使用数据。

**US-P2-EXPORT-003**: 作为数据库开发者，我想要导出时包含列头信息，以便了解数据的含义。

#### 2.6.3 验收标准（EARS 格式）

**AC-P2-EXPORT-001**: The system shall provide an "Export" dropdown button in the result toolbar.

**AC-P2-EXPORT-002**: The system shall provide "Export to CSV" option in the dropdown menu.

**AC-P2-EXPORT-003**: The system shall provide "Export to JSON" option in the dropdown menu.

**AC-P2-EXPORT-004**: When the user selects "Export to CSV", the system shall generate a CSV file from the current result set.

**AC-P2-EXPORT-005**: When exporting to CSV, the system shall include column headers as the first row.

**AC-P2-EXPORT-006**: When exporting to CSV, the system shall use comma as the delimiter.

**AC-P2-EXPORT-007**: When exporting to CSV, the system shall wrap fields containing commas in double quotes.

**AC-P2-EXPORT-008**: When the user selects "Export to JSON", the system shall generate a JSON file from the current result set.

**AC-P2-EXPORT-009**: When exporting to JSON, the system shall preserve the data structure and formatting.

**AC-P2-EXPORT-010**: The system shall use the query execution timestamp as part of the exported file name.

**AC-P2-EXPORT-011**: The system shall trigger the browser download dialog for the exported file.

**AC-P2-EXPORT-012**: When there are no results to export, the system shall disable the Export button.

---

## 3. 非功能需求

### 3.1 性能需求

**NFR-P2-001**: The system shall display query results within 1 second for result sets up to 100 rows.

**NFR-P2-002**: The system shall handle result sets up to 1000 rows without UI freezing.

**NFR-P2-003**: The system shall execute queries with a maximum timeout of 30 seconds.

**NFR-P2-004**: The system shall render the table view with virtualization for large result sets.

**NFR-P2-005**: The system shall load the query history panel within 500ms.

### 3.2 可用性需求

**NFR-P2-006**: The system shall provide keyboard shortcuts for all common operations (execute, clear, save favorite).

**NFR-P2-007**: The system shall display loading indicators for operations that take longer than 500ms.

**NFR-P2-008**: The system shall provide clear error messages that explain how to fix the problem.

**NFR-P2-009**: The system shall support copy-paste operations in the editor and result areas.

### 3.3 兼容性需求

**NFR-P2-010**: The system shall support the latest versions of Chrome, Firefox, Safari, and Edge.

**NFR-P2-011**: The system shall support responsive design for screen resolutions from 1366x768 to 1920x1080.

**NFR-P2-012**: The system shall handle different data types in results (string, number, boolean, date, null).

### 3.4 安全需求

**NFR-P2-013**: The system shall validate all user inputs before sending to the server.

**NFR-P2-014**: The system shall prevent SQL injection attacks through parameterized queries.

**NFR-P2-015**: The system shall not expose sensitive information in error messages.

### 3.5 可靠性需求

**NFR-P2-016**: The system shall handle network errors gracefully and allow retry.

**NFR-P2-017**: The system shall preserve editor content when a query execution fails.

**NFR-P2-018**: The system shall auto-save editor content to localStorage every 30 seconds.

---

## 4. 用户界面需求

### 4.1 控制台页面布局

**UI-P2-001**: The system shall display the Console page with a two-panel layout: editor on top, results on bottom.

**UI-P2-002**: The system shall allow users to resize the editor and result panels by dragging the divider.

**UI-P2-003**: The system shall display the editor panel with a toolbar containing action buttons.

**UI-P2-004**: The system shall display the result panel with a toolbar containing view toggle and export buttons.

### 4.2 编辑器区域

**UI-P2-005**: The system shall display the query editor with a minimum height of 150px.

**UI-P2-006**: The system shall display line numbers in the editor gutter.

**UI-P2-007**: The system shall highlight the current line in the editor.

**UI-P2-008**: The system shall display the editor toolbar with Execute, Clear, Save Favorite, History, and Favorites buttons.

**UI-P2-009**: The system shall display the current cursor position (line:column) in the editor status bar.

**UI-P2-010**: The system shall display a character count in the editor status bar.

### 4.3 结果区域

**UI-P2-011**: The system shall display the result panel with tabs for Table view and JSON view.

**UI-P2-012**: The system shall display the result toolbar with view toggle buttons and Export dropdown.

**UI-P2-013**: The system shall display the execution time and row count in the result status bar.

**UI-P2-014**: The system shall display an empty state illustration when no results are available.

**UI-P2-015**: The system shall display error messages in a highlighted box with an error icon.

### 4.4 历史面板

**UI-P2-016**: The system shall display the history panel as a slide-out drawer from the right side.

**UI-P2-017**: The system shall display each history entry with query preview, execution time, and timestamp.

**UI-P2-018**: The system shall provide hover effects on history entries.

**UI-P2-019**: The system shall display a "Clear History" button at the bottom of the history panel.

### 4.5 收藏面板

**UI-P2-020**: The system shall display the favorites panel as a slide-out drawer from the right side.

**UI-P2-021**: The system shall display each favorite entry with name, query preview, and action buttons.

**UI-P2-022**: The system shall provide an "Add Favorite" button at the top of the favorites panel.

**UI-P2-023**: The system shall display a modal dialog for adding/editing favorites.

---

## 5. 数据需求

### 5.1 数据持久化

**DATA-P2-001**: The system shall store query history in localStorage with key `graphdb_query_history`.

**DATA-P2-002**: The system shall store favorite queries in localStorage with key `graphdb_query_favorites`.

**DATA-P2-003**: The system shall store editor draft content in localStorage with key `graphdb_editor_draft`.

**DATA-P2-004**: The system shall store user preferences (default view, page size) in localStorage.

### 5.2 数据结构

**DATA-P2-005**: The query history entry shall contain: id, query text, execution time, timestamp, row count.

**DATA-P2-006**: The favorite entry shall contain: id, name, query text, creation timestamp.

**DATA-P2-007**: The system shall generate unique IDs using timestamp and random string.

### 5.3 数据验证

**DATA-P2-008**: The system shall validate that query text is not empty before execution.

**DATA-P2-009**: The system shall validate that favorite name is not empty and unique.

**DATA-P2-010**: The system shall limit query text length to maximum 10000 characters.

---

## 6. API 集成需求

### 6.1 查询 API

**API-P2-001**: The system shall call the `/api/query/execute` endpoint with POST method to execute a query.

**API-P2-002**: The system shall send the following parameters in the request body: query (string), sessionId (string).

**API-P2-003**: The system shall handle the following response format:
```json
{
  "success": true,
  "data": {
    "columns": ["column1", "column2"],
    "rows": [["value1", "value2"], ["value3", "value4"]],
    "rowCount": 2,
    "executionTime": 150
  }
}
```

**API-P2-004**: The system shall handle error responses with the following format:
```json
{
  "success": false,
  "error": {
    "code": "SYNTAX_ERROR",
    "message": "Invalid syntax at line 1, column 10",
    "position": {"line": 1, "column": 10}
  }
}
```

**API-P2-005**: The system shall include the session ID in the request header as `X-Session-ID`.

**API-P2-006**: The system shall handle HTTP 401 errors by redirecting to the login page.

**API-P2-007**: The system shall handle HTTP 500 errors by displaying a generic error message.

---

## 7. 测试需求

### 7.1 单元测试

**TEST-P2-001**: The system shall have unit tests for the query parser (splitting multiple queries).

**TEST-P2-002**: The system shall have unit tests for the result formatter (CSV and JSON export).

**TEST-P2-003**: The system shall have unit tests for the history management functions.

**TEST-P2-004**: The system shall have unit tests for the favorites management functions.

**TEST-P2-005**: The system shall have unit tests for the localStorage utility functions.

### 7.2 集成测试

**TEST-P2-006**: The system shall have integration tests for the complete query execution flow (editor → API → results).

**TEST-P2-007**: The system shall have integration tests for the history save and load flow.

**TEST-P2-008**: The system shall have integration tests for the favorites save and load flow.

**TEST-P2-009**: The system shall have integration tests for the export functionality.

### 7.3 端到端测试

**TEST-P2-010**: The system shall have E2E tests for the complete user journey from writing a query to viewing results.

**TEST-P2-011**: The system shall have E2E tests for the query history functionality.

**TEST-P2-012**: The system shall have E2E tests for the favorites functionality.

**TEST-P2-013**: The system shall have E2E tests for the export functionality.

---

## 8. 交付物

### 8.1 代码交付

**DEL-P2-001**: Query editor component with Input.TextArea implementation.

**DEL-P2-002**: Query execution service with API integration.

**DEL-P2-003**: Result display components (Table view, JSON view).

**DEL-P2-004**: Query history management module.

**DEL-P2-005**: Query favorites management module.

**DEL-P2-006**: Result export functionality (CSV, JSON).

**DEL-P2-007**: Console page integrating all components.

### 8.2 文档交付

**DEL-P2-008**: Component documentation for query editor and result display.

**DEL-P2-009**: API integration documentation for query endpoints.

**DEL-P2-010**: User guide for the console functionality.

### 8.3 测试交付

**DEL-P2-011**: Unit test suite with all test cases passing.

**DEL-P2-012**: Integration test suite with all test cases passing.

**DEL-P2-013**: E2E test suite with all test cases passing.

**DEL-P2-014**: Test execution report with coverage metrics (minimum 75% coverage).

---

## 9. 验收标准

### 9.1 功能验收

**ACCEPT-P2-001**: All user stories in Section 2 are implemented and meet their acceptance criteria.

**ACCEPT-P2-002**: All non-functional requirements in Section 3 are met.

**ACCEPT-P2-003**: All user interface requirements in Section 4 are implemented.

**ACCEPT-P2-004**: All data requirements in Section 5 are implemented.

**ACCEPT-P2-005**: All API integration requirements in Section 6 are working correctly.

### 9.2 质量验收

**ACCEPT-P2-006**: The application passes all unit tests with at least 75% code coverage.

**ACCEPT-P2-007**: The application passes all integration tests.

**ACCEPT-P2-008**: The application passes all E2E tests.

**ACCEPT-P2-009**: The application has no critical or high-severity bugs.

**ACCEPT-P2-010**: The application has been tested on all supported browsers.

### 9.3 性能验收

**ACCEPT-P2-011**: The query execution completes within 1 second for simple queries.

**ACCEPT-P2-012**: The result display renders within 1 second for 100 rows.

**ACCEPT-P2-013**: The editor remains responsive with queries up to 10000 characters.

---

## 10. 风险和假设

### 10.1 风险

**RISK-P2-001**: The backend query API may not support all Cypher features.

**RISK-P2-002**: Large result sets may cause performance issues in the browser.

**RISK-P2-003**: The simplified TextArea editor may not provide the best user experience compared to Monaco Editor.

### 10.2 假设

**ASSUME-P2-001**: The GraphDB backend provides a RESTful API for query execution (`/api/query/execute`).

**ASSUME-P2-002**: The backend supports Cypher query language.

**ASSUME-P2-003**: The backend returns results in a structured format (columns and rows).

**ASSUME-P2-004**: The development team has access to a GraphDB instance with sample data for testing.

---

## 11. 附录

### 11.1 术语表

| 术语 | 定义 |
|------|------|
| Cypher | Neo4j 开发的图数据库查询语言 |
| CSV | Comma-Separated Values，逗号分隔值文件格式 |
| JSON | JavaScript Object Notation，轻量级数据交换格式 |
| Monaco Editor | Microsoft 开发的代码编辑器组件 |

### 11.2 参考文档

- [Cypher Query Language Reference](https://neo4j.com/docs/cypher-manual/)
- [Ant Design Input.TextArea Documentation](https://ant.design/components/input)
- [GraphDB Backend API Documentation](../../api/server/README.md)

### 11.3 变更历史

| 版本 | 日期 | 变更内容 | 作者 |
|------|------|----------|------|
| v1.0 | 2026-03-29 | 初始版本 | - |

---

**文档结束**
