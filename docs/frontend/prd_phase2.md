# GraphDB 前端 PRD - 阶段 2: 查询控制台

**文档版本**: v1.0
**创建日期**: 2026-03-28
**最后更新**: 2026-03-28
**项目阶段**: Phase 2 - 查询控制台
**预计工期**: 2 周
**前置依赖**: Phase 1（基础框架和连接管理）

---

## 1. 引言（Introduction）

### 1.1 阶段目标

阶段 2 的目标是提供完整的 Cypher 查询编辑和执行环境，支持查询结果展示、历史记录和收藏功能。这是用户与 GraphDB 交互的主要界面。

### 1.2 核心价值

- **查询编辑**: 提供专业的代码编辑器，支持语法高亮和自动补全
- **查询执行**: 支持单条和批量查询执行，提供友好的错误提示
- **结果展示**: 多种格式展示查询结果，支持数据导出
- **查询管理**: 历史记录和收藏功能，提高查询效率

### 1.3 目标用户

- **数据库开发者**: 需要编写和执行 Cypher 查询的开发人员
- **数据分析师**: 需要查询和分析图数据的分析师
- **系统管理员**: 需要调试和优化查询的管理员

### 1.4 范围

**包含范围**:
- 查询编辑器（Monaco Editor）
- 查询执行功能
- 结果展示（表格、JSON）
- 查询历史记录
- 查询收藏功能
- 结果导出功能

**不包含范围**:
- 图形化结果展示（阶段 6）
- 查询优化建议
- 查询性能分析

---

## 2. 功能需求

### 2.1 查询编辑器

#### 2.1.1 功能概述

集成 Monaco Editor 作为查询编辑器，提供 Cypher 语法高亮、自动补全和快捷键支持。

#### 2.1.2 用户故事

**US-P2-EDITOR-001**: 作为数据库开发者，我想要在一个代码编辑器中编写 Cypher 查询，以便能够方便地编辑和调试查询语句。

**US-P2-EDITOR-002**: 作为数据库开发者，我想要看到 Cypher 语法的高亮显示，以便更容易识别查询语句的结构。

**US-P2-EDITOR-003**: 作为数据库开发者，我想要获得自动补全建议，以便快速输入关键字、标签、边类型和属性名。

**US-P2-EDITOR-004**: 作为数据库开发者，我想要通过快捷键（Shift+Enter）执行查询，以便提高查询效率。

#### 2.1.3 验收标准（EARS 格式）

**AC-P2-EDITOR-001**: The system shall provide a code editor with Monaco Editor for writing Cypher queries.

**AC-P2-EDITOR-002**: The system shall configure Monaco Editor to support Cypher language.

**AC-P2-EDITOR-003**: The system shall implement Cypher syntax highlighting in the editor.

**AC-P2-EDITOR-004**: The system shall provide auto-completion suggestions for Cypher keywords when typing.

**AC-P2-EDITOR-005**: The system shall provide auto-completion suggestions for Tag names when typing.

**AC-P2-EDITOR-006**: The system shall provide auto-completion suggestions for Edge types when typing.

**AC-P2-EDITOR-007**: The system shall provide auto-completion suggestions for property names when typing.

**AC-P2-EDITOR-008**: The system shall trigger auto-completion when user presses Ctrl+Space.

**AC-P2-EDITOR-009**: The system shall display auto-completion suggestions in a dropdown menu.

**AC-P2-EDITOR-010**: The system shall allow user to navigate through suggestions with arrow keys.

**AC-P2-EDITOR-011**: The system shall allow user to select a suggestion with Enter key.

**AC-P2-EDITOR-012**: The system shall close the auto-completion menu when user clicks outside.

**AC-P2-EDITOR-013**: The system shall support keyboard shortcuts (Ctrl+C, Ctrl+V, Ctrl+Z, Ctrl+Y).

**AC-P2-EDITOR-014**: The system shall execute the current query when user presses Shift+Enter.

**AC-P2-EDITOR-015**: The system shall execute the current query when user presses Ctrl+Enter.

**AC-P2-EDITOR-016**: The system shall support line numbers in the editor.

**AC-P2-EDITOR-017**: The system shall support code folding for multi-line queries.

**AC-P2-EDITOR-018**: The system shall support bracket matching for parentheses and quotes.

**AC-P2-EDITOR-019**: The system shall display a mini-map for long queries.

**AC-P2-EDITOR-020**: The system shall support word wrap for long lines.

**AC-P2-EDITOR-021**: The system shall support multiple cursors for editing multiple positions.

**AC-P2-EDITOR-022**: The system shall support find and replace functionality.

**AC-P2-EDITOR-023**: The system shall preserve the current query content when navigating away from the page.

**AC-P2-EDITOR-024**: The system shall restore the saved query content when returning to the page.

---

### 2.2 查询执行

#### 2.2.1 功能概述

提供查询执行功能，支持单条查询和批量查询执行，显示查询进度和错误信息。

#### 2.2.2 用户故事

**US-P2-EXEC-001**: 作为数据库开发者，我想要执行我编写的查询，以便从数据库中获取数据。

**US-P2-EXEC-002**: 作为数据库开发者，我想要一次执行多条查询，以便批量处理数据操作。

**US-P2-EXEC-003**: 作为数据库开发者，我想要看到查询执行的进度，以便了解查询是否还在处理中。

**US-P2-EXEC-004**: 作为数据库开发者，我想要看到查询执行过程中的错误信息，以便快速定位和修复问题。

#### 2.2.3 验收标准（EARS 格式）

**AC-P2-EXEC-001**: The system shall provide an "Execute" button next to the editor.

**AC-P2-EXEC-002**: The system shall execute the current query when user clicks the "Execute" button.

**AC-P2-EXEC-003**: The system shall execute the current query when user presses Shift+Enter.

**AC-P2-EXEC-004**: The system shall support executing multiple queries separated by semicolons.

**AC-P2-EXEC-005**: The system shall parse the query text to identify individual queries.

**AC-P2-EXEC-006**: The system shall execute each query in the order they appear in the text.

**AC-P2-EXEC-007**: When a query is executing, the system shall display a loading indicator.

**AC-P2-EXEC-008**: The system shall disable the "Execute" button while a query is executing.

**AC-P2-EXEC-009**: The system shall display the execution time for each query.

**AC-P2-EXEC-010**: The system shall display the number of rows affected or returned for each query.

**AC-P2-EXEC-011**: When a query execution is successful, the system shall display a success message.

**AC-P2-EXEC-012**: When a query execution fails, the system shall display an error message with error details.

**AC-P2-EXEC-013**: The system shall display the error code and error message for failed queries.

**AC-P2-EXEC-014**: The system shall provide a "Stop" button to cancel a long-running query.

**AC-P2-EXEC-015**: When user clicks "Stop", the system shall cancel the ongoing query execution.

**AC-P2-EXEC-016**: The system shall allow user to execute queries even when not connected to a specific Space.

**AC-P2-EXEC-017**: The system shall validate the query syntax before executing.

**AC-P2-EXEC-018**: When the query syntax is invalid, the system shall display a syntax error message.

**AC-P2-EXEC-019**: The system shall support parameterized queries (e.g., `$param`).

**AC-P2-EXEC-020**: The system shall provide a way to set parameter values before execution.

---

### 2.3 结果展示

#### 2.3.1 功能概述

提供查询结果的多格式展示，包括表格形式和 JSON 格式，支持结果导出功能。

#### 2.3.2 用户故事

**US-P2-RESULT-001**: 作为数据库开发者，我想要以表格形式查看查询结果，以便快速浏览数据。

**US-P2-RESULT-002**: 作为数据库开发者，我想要以 JSON 格式查看查询结果，以便复制和集成到其他工具中。

**US-P2-RESULT-003**: 作为数据库开发者，我想要导出查询结果为 CSV 文件，以便在本地保存和分析数据。

**US-P2-RESULT-004**: 作为数据库开发者，我想要导出查询结果为 JSON 文件，以便在其他应用中使用数据。

#### 2.3.3 验收标准（EARS 格式）

**表格展示**

**AC-P2-RESULT-001**: The system shall display query results in a table format by default.

**AC-P2-RESULT-002**: The system shall show column headers based on the returned data structure.

**AC-P2-RESULT-003**: The system shall display data rows based on the query results.

**AC-P2-RESULT-004**: The system shall support horizontal scrolling for wide tables.

**AC-P2-RESULT-005**: The system shall support vertical scrolling for tables with many rows.

**AC-P2-RESULT-006**: The system shall support pagination if the result set exceeds 100 rows.

**AC-P2-RESULT-007**: The system shall provide page size options (10, 20, 50, 100 rows per page).

**AC-P2-RESULT-008**: The system shall allow user to sort the table by any column.

**AC-P2-RESULT-009**: The system shall display the sort direction indicator for the sorted column.

**AC-P2-RESULT-010**: The system shall allow user to filter the table by column values.

**AC-P2-RESULT-011**: The system shall highlight the column that is currently sorted.

**AC-P2-RESULT-012**: The system shall display the total number of rows in the result set.

**AC-P2-RESULT-013**: The system shall display the current page number and total pages.

**AC-P2-RESULT-014**: The system shall provide "Previous" and "Next" buttons for pagination.

**AC-P2-RESULT-015**: The system shall support jumping to a specific page number.

**AC-P2-RESULT-016**: The system shall display empty state when the query returns no results.

**AC-P2-RESULT-017**: The system shall display a helpful message when the result set is empty.

**AC-P2-RESULT-018**: The system shall support selecting multiple rows in the table.

**AC-P2-RESULT-019**: The system shall support copying selected rows to clipboard.

**AC-P2-RESULT-020**: The system shall display row numbers for easy reference.

**JSON 展示**

**AC-P2-RESULT-021**: The system shall provide a toggle to switch between table and JSON result views.

**AC-P2-RESULT-022**: When displaying results in JSON format, the system shall format the JSON with proper indentation.

**AC-P2-RESULT-023**: The system shall preserve the data structure and types in the JSON output.

**AC-P2-RESULT-024**: The system shall support syntax highlighting in the JSON viewer.

**AC-P2-RESULT-025**: The system shall support collapsing and expanding JSON objects and arrays.

**AC-P2-RESULT-026**: The system shall provide a "Copy to Clipboard" button for the JSON content.

**AC-P2-RESULT-027**: The system shall display the JSON size (number of characters) in the viewer.

**结果导出**

**AC-P2-RESULT-028**: The system shall provide an "Export to CSV" button for the current result set.

**AC-P2-RESULT-029**: The system shall provide an "Export to JSON" button for the current result set.

**AC-P2-RESULT-030**: When exporting to CSV, the system shall include column headers as the first row.

**AC-P2-RESULT-031**: When exporting to CSV, the system shall properly escape special characters.

**AC-P2-RESULT-032**: When exporting to CSV, the system shall use comma as the delimiter.

**AC-P2-RESULT-033**: When exporting to JSON, the system shall preserve the data structure and formatting.

**AC-P2-RESULT-034**: The system shall prompt the user to choose a file name and location when exporting.

**AC-P2-RESULT-035**: The system shall automatically add a file extension based on the export format (.csv or .json).

**AC-P2-RESULT-036**: The system shall display a success message after a successful export.

**AC-P2-RESULT-037**: The system shall handle large result sets (up to 10,000 rows) without crashing the browser.

**AC-P2-RESULT-038**: The system shall provide a warning when exporting very large result sets (> 10,000 rows).

---

### 2.4 查询管理

#### 2.4.1 功能概述

提供查询历史记录和收藏功能，帮助用户管理和重用查询语句。

#### 2.4.2 用户故事

**US-P2-HISTORY-001**: 作为数据库开发者，我想要查看我的查询历史记录，以便重新执行之前的查询。

**US-P2-HISTORY-002**: 作为数据库开发者，我想要收藏常用的查询语句，以便快速执行重复性查询。

**US-P2-HISTORY-003**: 作为数据库开发者，我想要清理查询历史记录，以便保持历史列表的整洁。

**US-P2-HISTORY-004**: 作为数据库开发者，我想要删除不需要的收藏查询，以便管理收藏列表。

#### 2.4.3 验收标准（EARS 格式）

**查询历史**

**AC-P2-HISTORY-001**: The system shall save each executed query to the query history.

**AC-P2-HISTORY-002**: The system shall store query history in localStorage.

**AC-P2-HISTORY-003**: The system shall limit the query history to the last 15 executed queries.

**AC-P2-HISTORY-004**: The system shall display a list of historical queries in a sidebar or drawer.

**AC-P2-HISTORY-005**: The system shall display the execution time for each historical query.

**AC-P2-HISTORY-006**: The system shall display the query text for each historical query.

**AC-P2-HISTORY-007**: When the user selects a query from history, the system shall load it into the editor.

**AC-P2-HISTORY-008**: When the user selects a query from history, the system shall highlight the loaded query.

**AC-P2-HISTORY-009**: The system shall provide a "Clear History" button to remove all history records.

**AC-P2-HISTORY-010**: When the user clicks "Clear History", the system shall display a confirmation dialog.

**AC-P2-HISTORY-011**: When the user confirms clearing history, the system shall remove all history records from localStorage.

**AC-P2-HISTORY-012**: The system shall provide a search function to filter historical queries.

**AC-P2-HISTORY-013**: The system shall display the most recent queries at the top of the list.

**AC-P2-HISTORY-014**: The system shall display the number of queries in the history.

**查询收藏**

**AC-P2-HISTORY-015**: The system shall provide a "Save to Favorites" button for the current query.

**AC-P2-HISTORY-016**: When the user clicks "Save to Favorites", the system shall prompt for a name.

**AC-P2-HISTORY-017**: The system shall validate that the favorite name is not empty.

**AC-P2-HISTORY-018**: The system shall store favorite queries in localStorage.

**AC-P2-HISTORY-019**: The system shall display a list of saved favorite queries in a sidebar or drawer.

**AC-P2-HISTORY-020**: The system shall display the name and query text for each favorite query.

**AC-P2-HISTORY-021**: When the user clicks a favorite query, the system shall load it into the editor.

**AC-P2-HISTORY-022**: When the user clicks a favorite query, the system shall execute it immediately.

**AC-P2-HISTORY-023**: The system shall provide a "Delete" button for each favorite query.

**AC-P2-HISTORY-024**: When the user clicks "Delete", the system shall remove the favorite query from localStorage.

**AC-P2-HISTORY-025**: The system shall provide a confirmation dialog before deleting a favorite query.

**AC-P2-HISTORY-026**: The system shall provide an "Edit" button for each favorite query.

**AC-P2-HISTORY-027**: When the user clicks "Edit", the system shall allow modifying the favorite name and query text.

**AC-P2-HISTORY-028**: The system shall provide a search function to filter favorite queries.

**AC-P2-HISTORY-029**: The system shall display the creation date for each favorite query.

**AC-P2-HISTORY-030**: The system shall allow user to organize favorite queries into folders or categories.

---

## 3. 非功能需求

### 3.1 性能需求

**NFR-P2-001**: The system shall load the query console page within 2 seconds on a standard broadband connection.

**NFR-P2-002**: The system shall initialize Monaco Editor within 1 second.

**NFR-P2-003**: The system shall execute simple queries and display results within 2 seconds.

**NFR-P2-004**: The system shall support displaying up to 1000 rows in a table without significant performance degradation.

**NFR-P2-005**: The system shall provide visual feedback within 200ms for user interactions.

**NFR-P2-006**: The system shall handle large queries (up to 10,000 characters) without freezing the editor.

### 3.2 可用性需求

**NFR-P2-007**: The system shall provide clear and helpful error messages for query failures.

**NFR-P2-008**: The system shall provide keyboard shortcuts for common operations (Shift+Enter to execute).

**NFR-P2-009**: The system shall be accessible to users with visual impairments (WCAG 2.1 Level AA compliance).

**NFR-P2-010**: The system shall provide tooltips for all buttons and actions.

### 3.3 兼容性需求

**NFR-P2-011**: The system shall support the latest versions of Chrome, Firefox, Safari, and Edge.

**NFR-P2-012**: The system shall support responsive design for screen resolutions from 1366x768 to 1920x1080.

**NFR-P2-013**: The system shall support mobile devices with screen widths from 375px to 768px.

### 3.4 安全需求

**NFR-P2-014**: The system shall validate all query inputs before sending to the server.

**NFR-P2-015**: The system shall sanitize query results to prevent XSS attacks.

**NFR-P2-016**: The system shall implement proper error handling to prevent information leakage.

### 3.5 可靠性需求

**NFR-P2-017**: The system shall preserve unsaved query drafts in localStorage.

**NFR-P2-018**: The system shall handle network errors gracefully and display appropriate messages.

**NFR-P2-019**: The system shall automatically retry failed queries up to 3 times.

---

## 4. 用户界面需求

### 4.1 查询控制台布局

**UI-P2-001**: The system shall display the query editor at the top of the page.

**UI-P2-002**: The system shall display the execute button next to the editor.

**UI-P2-003**: The system shall display the results section below the editor.

**UI-P2-004**: The system shall display a toggle to switch between table and JSON views.

**UI-P2-005**: The system shall display the query history and favorites in a sidebar or drawer.

**UI-P2-006**: The system shall display the execution time and row count for each query.

**UI-P2-007**: The system shall use Ant Design components for consistent styling.

**UI-P2-008**: The system shall provide a responsive layout that works on desktop and mobile.

### 4.2 查询编辑器界面

**UI-P2-009**: The system shall display the Monaco Editor with a fixed height of 300px.

**UI-P2-010**: The system shall display the editor with line numbers enabled.

**UI-P2-011**: The system shall display the editor with word wrap enabled.

**UI-P2-012**: The system shall display the editor with a light theme.

**UI-P2-013**: The system shall display the editor with a monospaced font.

**UI-P2-014**: The system shall display the editor with a status bar showing the current line and column.

### 4.3 结果展示界面

**UI-P2-015**: The system shall display the table view with a maximum height of 500px.

**UI-P2-016**: The system shall display the table with sticky column headers.

**UI-P2-017**: The system shall display the table with alternating row colors for readability.

**UI-P2-018**: The system shall display the JSON viewer with syntax highlighting.

**UI-P2-019**: The system shall display the JSON viewer with expand/collapse functionality.

**UI-P2-020**: The system shall display the export buttons in a toolbar above the results.

### 4.4 查询管理界面

**UI-P2-021**: The system shall display the history and favorites in a collapsible sidebar.

**UI-P2-022**: The system shall display the history section with a clear button.

**UI-P2-023**: The system shall display the favorites section with a clear button.

**UI-P2-024**: The system shall display search boxes for both history and favorites.

**UI-P2-025**: The system shall display icons for history and favorite items.

---

## 5. 数据需求

### 5.1 数据持久化

**DATA-P2-001**: The system shall store query history in localStorage.

**DATA-P2-002**: The system shall store favorite queries in localStorage.

**DATA-P2-003**: The system shall store query drafts in localStorage.

**DATA-P2-004**: The system shall limit query history to the last 15 queries.

**DATA-P2-005**: The system shall preserve unsaved query content when navigating away.

### 5.2 数据验证

**DATA-P2-006**: The system shall validate that the query is not empty before executing.

**DATA-P2-007**: The system shall validate the query syntax before executing.

**DATA-P2-008**: The system shall validate that the favorite name is not empty.

**DATA-P2-009**: The system shall validate that the favorite name is unique.

---

## 6. API 集成需求

### 6.1 查询 API

**API-P2-001**: The system shall call to `/api/query` endpoint with POST method to execute a Cypher query.

**API-P2-002**: The system shall send the query text in the request body.

**API-P2-003**: The system shall send the current Space name in the request body.

**API-P2-004**: The system shall send the connection token in the request header.

**API-P2-005**: The system shall handle the query results in the response body.

**API-P2-006**: The system shall handle query errors in the response body.

**API-P2-007**: The system shall call to `/api/batch-query` endpoint with POST method to execute multiple queries.

**API-P2-008**: The system shall send an array of queries in the request body.

**API-P2-009**: The system shall handle multiple query results in the response body.

**API-P2-010**: The system shall handle query execution time in the response body.

### 6.2 Schema API（用于自动补全）

**API-P2-011**: The system shall call to `/api/schema/tags` endpoint with GET method to fetch Tag names for auto-completion.

**API-P2-012**: The system shall call to `/api/schema/edges` endpoint with GET method to fetch Edge names for auto-completion.

**API-P2-013**: The system shall call to `/api/schema/tags/{name}` endpoint with GET method to fetch Tag properties for auto-completion.

**API-P2-014**: The system shall call to `/api/schema/edges/{name}` endpoint with GET method to fetch Edge properties for auto-completion.

---

## 7. 测试需求

### 7.1 单元测试

**TEST-P2-001**: The system shall have unit tests for the query editor component.

**TEST-P2-002**: The system shall have unit tests for the query execution logic.

**TEST-P2-003**: The system shall have unit tests for the result display components.

**TEST-P2-004**: The system shall have unit tests for the history management logic.

**TEST-P2-005**: The system shall have unit tests for the favorite management logic.

**TEST-P2-006**: The system shall have unit tests for the export functionality.

### 7.2 集成测试

**TEST-P2-007**: The system shall have integration tests for the complete query execution flow.

**TEST-P2-008**: The system shall have integration tests for the batch query execution flow.

**TEST-P2-009**: The system shall have integration tests for the history and favorites flow.

### 7.3 端到端测试

**TEST-P2-010**: The system shall have E2E tests for the complete user journey from writing a query to viewing results.

**TEST-P2-011**: The system shall have E2E tests for the history and favorites workflow.

**TEST-P2-012**: The system shall have E2E tests for the export functionality.

---

## 8. 交付物

### 8.1 代码交付

**DEL-P2-001**: Query editor component with Monaco Editor integration.

**DEL-P2-002**: Query execution component with loading states.

**DEL-P2-003**: Result display components (table view, JSON view).

**DEL-P2-004**: History management component.

**DEL-P2-005**: Favorites management component.

**DEL-P2-006**: Export functionality (CSV, JSON).

**DEL-P2-007**: Auto-completion service for Cypher keywords, Tags, and Edges.

### 8.2 文档交付

**DEL-P2-008**: Query console user guide.

**DEL-P2-009**: Cypher query language reference.

**DEL-P2-010**: Component documentation for all query console components.

### 8.3 测试交付

**DEL-P2-011**: Unit test suite with all test cases passing.

**DEL-P2-012**: Integration test suite with all test cases passing.

**DEL-P2-013**: E2E test suite with all test cases passing.

**DEL-P2-014**: Test execution report with coverage metrics.

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

**ACCEPT-P2-010**: The application has been tested on all supported browsers (Chrome, Firefox, Safari, Edge).

### 9.3 性能验收

**ACCEPT-P2-011**: The query console page loads within 2 seconds.

**ACCEPT-P2-012**: Monaco Editor initializes within 1 second.

**ACCEPT-P2-013**: Simple queries execute and display results within 2 seconds.

**ACCEPT-P2-014**: The application remains responsive when displaying 1000 rows of data.

**ACCEPT-P2-015**: The application handles large queries (up to 10,000 characters) without freezing.

---

## 10. 风险和假设

### 10.1 风险

**RISK-P2-001**: The Cypher syntax support in Monaco Editor may require custom language configuration.

**RISK-P2-002**: Large result sets may cause performance issues in the browser.

**RISK-P2-003**: Auto-completion may not work correctly if the Schema data is not up-to-date.

**RISK-P2-004**: Query execution may time out for complex queries on large datasets.

### 10.2 假设

**ASSUME-P2-001**: The GraphDB backend provides RESTful APIs for query execution (`/api/query`, `/api/batch-query`).

**ASSUME-P2-002**: The GraphDB backend supports Cypher query language.

**ASSUME-P2-003**: The GraphDB backend returns query results in a structured format.

**ASSUME-P2-004**: The GraphDB backend provides Schema API endpoints for auto-completion.

---

## 11. 附录

### 11.1 术语表

| 术语 | 定义 |
|------|------|
| Cypher | 图查询语言，用于查询和操作图数据 |
| Monaco Editor | 基于 VS Code 的代码编辑器，提供语法高亮和自动补全 |
| 查询历史 | 用户执行过的查询记录 |
| 查询收藏 | 用户保存的常用查询 |
| 结果集 | 查询返回的数据集合 |
| 分页 | 将大数据集分成多个小页面显示 |

### 11.2 参考文档

- [GraphDB 后端 API 文档](../../api/server/README.md)
- [GraphDB 前端阶段规划](./phase_planning.md)
- [Monaco Editor 文档](https://microsoft.github.io/monaco-editor/)
- [Cypher 查询语言参考](https://neo4j.com/docs/cypher-cheat-sheet/)
- [Ant Design 文档](https://ant.design/)

### 11.3 变更历史

| 版本 | 日期 | 作者 | 变更描述 |
|------|------|------|----------|
| v1.0 | 2026-03-28 | 初始版本，定义阶段 2 需求 |

---

**文档结束**
