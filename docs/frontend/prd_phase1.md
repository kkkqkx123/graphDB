# GraphDB 前端 PRD - 阶段 1: 基础框架和连接管理

**文档版本**: v1.0
**创建日期**: 2026-03-28
**最后更新**: 2026-03-28
**项目阶段**: Phase 1 - 基础框架和连接管理
**预计工期**: 1 周

---

## 1. 引言（Introduction）

### 1.1 阶段目标

阶段 1 的目标是搭建前端项目的基础架构，并实现数据库连接管理功能。这是整个前端项目的基础，为后续所有功能模块的开发奠定技术基础。

### 1.2 核心价值

- **技术基础**: 建立稳定、可扩展的前端项目架构
- **连接管理**: 提供安全、可靠的数据库连接功能
- **用户体验**: 创建直观、易用的连接界面

### 1.3 目标用户

- **数据库开发者**: 需要连接到 GraphDB 进行开发和测试
- **系统管理员**: 需要管理数据库连接和配置

### 1.4 范围

**包含范围**:
- 项目初始化和配置
- 数据库连接界面
- 连接状态管理
- 连接信息持久化
- 基础布局和导航

**不包含范围**:
- 查询功能（阶段 2）
- Schema 管理（阶段 3-5）
- 数据可视化（阶段 6）

---

## 2. 功能需求

### 2.1 项目初始化

#### 2.1.1 功能概述

搭建 React + TypeScript + Vite 项目，集成必要的依赖和配置，建立项目的基础架构。

#### 2.1.2 用户故事

**US-P1-INIT-001**: 作为开发团队，我想要一个标准化的前端项目结构，以便团队协作开发。

**US-P1-INIT-002**: 作为开发团队，我想要集成 Ant Design 组件库，以便快速构建用户界面。

**US-P1-INIT-003**: 作为开发团队，我想要配置 React Router，以便实现页面导航。

**US-P1-INIT-004**: 作为开发团队，我想要配置全局样式和主题，以便保持界面一致性。

#### 2.1.3 验收标准（EARS 格式）

**AC-P1-INIT-001**: The system shall provide a React 18 + TypeScript project created with Vite.

**AC-P1-INIT-002**: The system shall install and configure Ant Design 5.x as the UI component library.

**AC-P1-INIT-003**: The system shall configure React Router v6 for client-side routing.

**AC-P1-INIT-004**: The system shall configure Axios for HTTP requests.

**AC-P1-INIT-005**: The system shall configure Zustand for state management.

**AC-P1-INIT-006**: The system shall create a project structure with separate directories for components, pages, stores, and utils.

**AC-P1-INIT-007**: The system shall configure ESLint and Prettier for code quality.

**AC-P1-INIT-008**: The system shall configure TypeScript strict mode for type safety.

**AC-P1-INIT-009**: The system shall create a global CSS file with theme variables.

**AC-P1-INIT-010**: The system shall configure environment variables for API endpoints.

---

### 2.2 连接管理

#### 2.2.1 功能概述

连接管理模块提供用户与 GraphDB 数据库建立连接的功能，包括连接配置、认证、状态显示和断开连接等操作。

#### 2.2.2 用户故事

**US-P1-CONN-001**: 作为数据库开发者，我想要配置数据库连接信息，以便能够连接到我的 GraphDB 实例。

**US-P1-CONN-002**: 作为数据库开发者，我想要看到当前的连接状态，以便了解数据库是否可用。

**US-P1-CONN-003**: 作为数据库开发者，我想要断开当前的数据库连接，以便释放资源或切换到其他数据库。

**US-P1-CONN-004**: 作为数据库开发者，我想要保存我的连接配置，以便下次打开应用时自动填充连接信息。

#### 2.2.3 验收标准（EARS 格式）

**AC-P1-CONN-001**: The system shall provide a login page with a connection form.

**AC-P1-CONN-002**: The system shall provide input fields for host address, port, username, and password.

**AC-P1-CONN-003**: The system shall provide default values for host (localhost) and port (7001).

**AC-P1-CONN-004**: The system shall validate that host address is not empty.

**AC-P1-CONN-005**: The system shall validate that port is a valid number between 1 and 65535.

**AC-P1-CONN-006**: The system shall validate that username is not empty.

**AC-P1-CONN-007**: The system shall validate that password is not empty.

**AC-P1-CONN-008**: The system shall mask the password field during input.

**AC-P1-CONN-009**: The system shall provide a "Remember me" checkbox to control credential persistence.

**AC-P1-CONN-010**: The system shall provide a "Connect" button to submit the connection form.

**AC-P1-CONN-011**: When the user submits valid connection credentials, the system shall call the `/api/connect` endpoint.

**AC-P1-CONN-012**: When the connection is successful, the system shall store the connection token in memory.

**AC-P1-CONN-013**: When the connection is successful, the system shall display a success message.

**AC-P1-CONN-014**: When the connection is successful, the system shall navigate to the main dashboard.

**AC-P1-CONN-015**: When the connection fails, the system shall display an error message with the error details.

**AC-P1-CONN-016**: When the connection fails, the system shall keep the user on the login page.

**AC-P1-CONN-017**: When the "Remember me" option is checked, the system shall store the connection credentials in localStorage.

**AC-P1-CONN-018**: When the "Remember me" option is unchecked, the system shall not store the connection credentials.

**AC-P1-CONN-019**: The system shall store the connection credentials securely (password should be hashed or encrypted if possible).

**AC-P1-CONN-020**: When the application loads, the system shall check for stored credentials in localStorage.

**AC-P1-CONN-021**: When stored credentials are found, the system shall pre-fill the connection form.

**AC-P1-CONN-022**: The system shall provide a "Disconnect" button in the application header.

**AC-P1-CONN-023**: When the user clicks the "Disconnect" button, the system shall call the `/api/disconnect` endpoint.

**AC-P1-CONN-024**: When the disconnection is successful, the system shall clear the connection token from memory.

**AC-P1-CONN-025**: When the disconnection is successful, the system shall navigate to the login page.

**AC-P1-CONN-026**: The system shall display the current connection status in the application header.

**AC-P1-CONN-027**: When the user is connected, the system shall display "Connected" status in green.

**AC-P1-CONN-028**: When the user is disconnected, the system shall display "Disconnected" status in red.

**AC-P1-CONN-029**: When the user is connected, the system shall display the connected host and username in the header.

**AC-P1-CONN-030**: The system shall periodically check the connection health using the `/api/health` endpoint.

**AC-P1-CONN-031**: When the connection health check fails, the system shall display a warning message.

**AC-P1-CONN-032**: When the connection is lost unexpectedly, the system shall automatically navigate to the login page.

---

### 2.3 基础布局

#### 2.3.1 功能概述

创建应用的基础布局，包括头部、侧边栏和主内容区域，为后续功能提供统一的界面框架。

#### 2.3.2 用户故事

**US-P1-LAYOUT-001**: 作为应用用户，我想要一个清晰的导航结构，以便快速访问不同的功能模块。

**US-P1-LAYOUT-002**: 作为应用用户，我想要看到当前连接状态，以便了解数据库是否可用。

**US-P1-LAYOUT-003**: 作为应用用户，我想要一个退出按钮，以便安全地断开连接。

#### 2.3.3 验收标准（EARS 格式）

**AC-P1-LAYOUT-001**: The system shall provide a header bar at the top of the application.

**AC-P1-LAYOUT-002**: The system shall display the GraphDB logo in the header.

**AC-P1-LAYOUT-003**: The system shall display the current connection status in the header.

**AC-P1-LAYOUT-004**: The system shall display the connected host and username when connected.

**AC-P1-LAYOUT-005**: The system shall provide a "Disconnect" button in the header.

**AC-P1-LAYOUT-006**: The system shall provide a sidebar navigation on the left side of the application.

**AC-P1-LAYOUT-007**: The system shall display navigation links for Console and Schema in the sidebar.

**AC-P1-LAYOUT-008**: The system shall highlight the active navigation link based on the current page.

**AC-P1-LAYOUT-009**: The system shall display a main content area on the right side of the application.

**AC-P1-LAYOUT-010**: The system shall provide a responsive layout that works on desktop (1366x768+) and mobile (375px+).

**AC-P1-LAYOUT-011**: The system shall use Ant Design components for consistent styling.

**AC-P1-LAYOUT-012**: The system shall apply a consistent color scheme throughout the application.

**AC-P1-LAYOUT-013**: The system shall provide smooth transitions between pages.

---

## 3. 非功能需求

### 3.1 性能需求

**NFR-P1-001**: The system shall load the initial login page within 2 seconds on a standard broadband connection.

**NFR-P1-002**: The system shall establish a database connection within 1 second after the user submits the form.

**NFR-P1-003**: The system shall display connection status updates within 200ms of state changes.

### 3.2 可用性需求

**NFR-P1-004**: The system shall provide clear and helpful error messages for connection failures.

**NFR-P1-005**: The system shall provide keyboard shortcuts (Enter to submit form).

**NFR-P1-006**: The system shall be accessible to users with visual impairments (WCAG 2.1 Level AA compliance).

### 3.3 兼容性需求

**NFR-P1-007**: The system shall support the latest versions of Chrome, Firefox, Safari, and Edge.

**NFR-P1-008**: The system shall support responsive design for screen resolutions from 1366x768 to 1920x1080.

**NFR-P1-009**: The system shall support mobile devices with screen widths from 375px to 768px.

### 3.4 安全需求

**NFR-P1-010**: The system shall transmit passwords over HTTPS when in production.

**NFR-P1-011**: The system shall not store passwords in plain text in localStorage.

**NFR-P1-012**: The system shall implement proper authentication with the GraphDB backend.

**NFR-P1-013**: The system shall validate all user inputs on the client side before sending to the server.

### 3.5 可靠性需求

**NFR-P1-014**: The system shall handle network errors gracefully and display appropriate messages.

**NFR-P1-015**: The system shall automatically reconnect to the database if the connection is lost.

**NFR-P1-016**: The system shall preserve unsaved connection drafts in localStorage.

---

## 4. 用户界面需求

### 4.1 登录页面

**UI-P1-001**: The system shall display a centered login form on the login page.

**UI-P1-002**: The system shall display the GraphDB logo above the login form.

**UI-P1-003**: The system shall provide a title "Connect to GraphDB" on the login page.

**UI-P1-004**: The system shall provide input fields for host, port, username, and password.

**UI-P1-005**: The system shall display labels for each input field.

**UI-P1-006**: The system shall provide placeholder text for each input field.

**UI-P1-007**: The system shall mask the password field with asterisks.

**UI-P1-008**: The system shall provide a "Remember me" checkbox.

**UI-P1-009**: The system shall provide a "Connect" button.

**UI-P1-010**: The system shall display error messages below the form when connection fails.

**UI-P1-011**: The system shall use Ant Design Form component for consistent styling.

### 4.2 应用头部

**UI-P1-012**: The system shall display a header bar at the top of the application.

**UI-P1-013**: The system shall display the GraphDB logo in the header.

**UI-P1-014**: The system shall display the current connection status in the header.

**UI-P1-015**: The system shall display "Connected" in green when the user is connected.

**UI-P1-016**: The system shall display "Disconnected" in red when the user is disconnected.

**UI-P1-017**: The system shall display the connected host and username when connected.

**UI-P1-018**: The system shall provide a "Disconnect" button in the header.

**UI-P1-019**: The system shall use Ant Design Layout component for consistent styling.

### 4.3 侧边栏导航

**UI-P1-020**: The system shall display a sidebar on the left side of the application.

**UI-P1-021**: The system shall display navigation links for Console and Schema.

**UI-P1-022**: The system shall highlight the active navigation link based on the current page.

**UI-P1-023**: The system shall use Ant Design Menu component for consistent styling.

**UI-P1-024**: The system shall display icons for each navigation link.

### 4.4 主内容区域

**UI-P1-025**: The system shall display a main content area on the right side of the application.

**UI-P1-026**: The system shall use Ant Design Content component for consistent styling.

**UI-P1-027**: The system shall provide a breadcrumb navigation for the current page.

---

## 5. 数据需求

### 5.1 数据持久化

**DATA-P1-001**: The system shall store connection credentials in localStorage when "Remember me" is checked.

**DATA-P1-002**: The system shall store the connection token in memory (Zustand store).

**DATA-P1-003**: The system shall clear the connection token from memory when the user disconnects.

**DATA-P1-004**: The system shall clear the connection credentials from localStorage when the user unchecks "Remember me".

### 5.2 数据验证

**DATA-P1-005**: The system shall validate that the host field is not empty.

**DATA-P1-006**: The system shall validate that the port field is a valid number.

**DATA-P1-007**: The system shall validate that the username field is not empty.

**DATA-P1-008**: The system shall validate that the password field is not empty.

**DATA-P1-009**: The system shall validate that the port is between 1 and 65535.

---

## 6. API 集成需求

### 6.1 连接 API

**API-P1-001**: The system shall call the `/api/connect` endpoint with POST method to establish a database connection.

**API-P1-002**: The system shall send the connection credentials (host, port, username, password) in the request body.

**API-P1-003**: The system shall store the returned connection token in the state management store.

**API-P1-004**: The system shall call the `/api/disconnect` endpoint with POST method to terminate a database connection.

**API-P1-005**: The system shall send the connection token in the request header or body.

**API-P1-006**: The system shall clear the connection token from the state management store after disconnection.

**API-P1-007**: The system shall call the `/api/health` endpoint with GET method to check connection status.

**API-P1-008**: The system shall send the connection token in the request header for health checks.

**API-P1-009**: The system shall handle connection errors (401, 403, 500) appropriately.

---

## 7. 测试需求

### 7.1 单元测试

**TEST-P1-001**: The system shall have unit tests for the connection form validation logic.

**TEST-P1-002**: The system shall have unit tests for the API service functions (connect, disconnect, health).

**TEST-P1-003**: The system shall have unit tests for the state management store (connection state).

**TEST-P1-004**: The system shall have unit tests for the localStorage utility functions.

### 7.2 集成测试

**TEST-P1-005**: The system shall have integration tests for the complete connection flow (form submit → API call → state update → navigation).

**TEST-P1-006**: The system shall have integration tests for the disconnection flow (button click → API call → state update → navigation).

**TEST-P1-007**: The system shall have integration tests for the connection health check flow.

### 7.3 端到端测试

**TEST-P1-008**: The system shall have E2E tests for the complete user journey from opening the application to successful connection.

**TEST-P1-009**: The system shall have E2E tests for the connection failure scenario.

**TEST-P1-010**: The system shall have E2E tests for the disconnection scenario.

---

## 8. 交付物

### 8.1 代码交付

**DEL-P1-001**: Complete React + TypeScript + Vite project source code.

**DEL-P1-002**: Connection management components (login form, header, sidebar).

**DEL-P1-003**: State management store for connection state.

**DEL-P1-004**: API service functions for connection operations.

**DEL-P1-005**: Utility functions for localStorage operations.

**DEL-P1-006**: Configuration files (package.json, vite.config.ts, tsconfig.json).

### 8.2 文档交付

**DEL-P1-007**: Project setup and development guide.

**DEL-P1-008**: API integration documentation for connection endpoints.

**DEL-P1-009**: Component documentation for connection management.

### 8.3 测试交付

**DEL-P1-010**: Unit test suite with all test cases passing.

**DEL-P1-011**: Integration test suite with all test cases passing.

**DEL-P1-012**: E2E test suite with all test cases passing.

**DEL-P1-013**: Test execution report with coverage metrics.

---

## 9. 验收标准

### 9.1 功能验收

**ACCEPT-P1-001**: All user stories in Section 2 are implemented and meet their acceptance criteria.

**ACCEPT-P1-002**: All non-functional requirements in Section 3 are met.

**ACCEPT-P1-003**: All user interface requirements in Section 4 are implemented.

**ACCEPT-P1-004**: All data requirements in Section 5 are implemented.

**ACCEPT-P1-005**: All API integration requirements in Section 6 are working correctly.

### 9.2 质量验收

**ACCEPT-P1-006**: The application passes all unit tests with at least 70% code coverage.

**ACCEPT-P1-007**: The application passes all integration tests.

**ACCEPT-P1-008**: The application passes all E2E tests.

**ACCEPT-P1-009**: The application has no critical or high-severity bugs.

**ACCEPT-P1-010**: The application has been tested on all supported browsers (Chrome, Firefox, Safari, Edge).

### 9.3 性能验收

**ACCEPT-P1-011**: The initial login page load time is less than 2 seconds.

**ACCEPT-P1-012**: The database connection is established within 1 second after form submission.

**ACCEPT-P1-013**: The application remains responsive on all supported screen resolutions.

---

## 10. 风险和假设

### 10.1 风险

**RISK-P1-001**: The GraphDB backend API may not be fully implemented by the time frontend development starts.

**RISK-P1-002**: The connection authentication mechanism may differ from the expected design.

**RISK-P1-003**: Network connectivity issues may cause connection failures that are hard to debug.

### 10.2 假设

**ASSUME-P1-001**: The GraphDB backend provides RESTful APIs for connection operations (`/api/connect`, `/api/disconnect`, `/api/health`).

**ASSUME-P1-002**: The GraphDB backend supports token-based authentication.

**ASSUME-P1-003**: The development team has access to a GraphDB instance for testing.

**ASSUME-P1-004**: The GraphDB backend returns appropriate error codes for connection failures.

---

## 11. 附录

### 11.1 术语表

| 术语 | 定义 |
|------|------|
| Host | 数据库服务器地址 |
| Port | 数据库服务器端口 |
| Token | 认证令牌，用于标识已连接的会话 |
| localStorage | 浏览器本地存储，用于持久化数据 |
| Zustand | 轻量级状态管理库 |

### 11.2 参考文档

- [GraphDB 后端 API 文档](../../api/server/README.md)
- [GraphDB 前端阶段规划](./phase_planning.md)
- [React 官方文档](https://react.dev/)
- [Ant Design 文档](https://ant.design/)
- [Vite 文档](https://vitejs.dev/)

### 11.3 变更历史

| 版本 | 日期 | 作者 | 变更描述 |
|------|------|------|----------|
| v1.0 | 2026-03-28 | 初始版本，定义阶段 1 需求 |

---

**文档结束**
