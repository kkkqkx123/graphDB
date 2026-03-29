# GraphDB 前端阶段1执行方案

**文档版本**: v1.0  
**创建日期**: 2026-03-29  
**对应PRD**: [prd_phase1.md](../prd_phase1.md)

---

## 1. 执行概览

### 1.1 目标

搭建前端项目基础架构，实现数据库连接管理功能。

### 1.2 工期

预计 1 周

### 1.3 交付物

- 可运行的前端项目
- 登录/连接页面
- 主应用布局
- 连接状态管理

---

## 2. 执行步骤

### 步骤1: 项目初始化

**任务**: 创建 Vite + React + TypeScript 项目

**操作**:
1. 使用 Vite 创建项目模板
2. 安装核心依赖（React、React Router、Ant Design、Zustand、Axios）
3. 安装开发依赖（ESLint、Prettier、TypeScript）
4. 配置路径别名
5. 配置 ESLint 和 Prettier

**验收**:
- 项目可以正常启动
- 无 ESLint 错误

---

### 步骤2: 目录结构搭建

**任务**: 按架构设计创建目录

**目录结构**:
```
src/
├── components/
│   ├── common/           # Button, Icon, EmptyTableTip, Avatar, ErrorBoundary
│   └── layout/           # Header, Sidebar, MainLayout
├── pages/
│   ├── Login/            # 登录页面
│   └── MainPage/         # 主布局页面
├── stores/
│   └── connection.ts     # 连接状态管理
├── services/
│   └── connection.ts     # 连接API服务
├── utils/
│   ├── http.ts           # Axios封装
│   ├── function.ts       # 工具函数
│   └── storage.ts        # localStorage封装
├── config/
│   ├── routes.tsx        # 路由配置
│   └── theme.ts          # 主题配置
├── App.tsx
└── main.tsx
```

**验收**:
- 所有目录创建完成
- 空文件已建立

---

### 步骤3: 工具函数实现

**任务**: 实现基础工具函数

**文件清单**:

| 文件 | 功能 | 参考来源 |
|------|------|----------|
| `utils/http.ts` | Axios实例、拦截器 | nebula-studio |
| `utils/function.ts` | 通用工具函数 | nebula-studio |
| `utils/storage.ts` | localStorage封装 | 新建 |

**验收**:
- HTTP请求正常发送
- localStorage读写正常

---

### 步骤4: 状态管理实现

**任务**: 实现连接状态管理

**Store设计**:
- `isConnected`: 连接状态
- `host`, `port`, `username`: 连接信息
- `sessionId`: 会话令牌
- `connect()`: 连接方法
- `disconnect()`: 断开方法

**持久化**:
- 使用 Zustand persist 中间件
- 记住密码时保存到 localStorage

**验收**:
- Store状态可正常读写
- 刷新页面后连接信息保留

---

### 步骤5: 登录页面实现

**任务**: 实现登录/连接页面

**页面元素**:
- Logo展示
- 连接表单（host、port、username、password）
- "记住我"复选框
- 连接按钮
- 错误提示区域

**表单验证**:
- host: 必填
- port: 1-65535
- username: 必填
- password: 必填

**交互**:
- 提交后调用 `/api/connect`
- 成功: 保存sessionId，跳转到主页面
- 失败: 显示错误信息

**验收**:
- 表单验证正常
- 连接成功可跳转
- 连接失败显示错误

---

### 步骤6: 主布局实现

**任务**: 实现主应用布局

**布局组成**:
- Header: Logo、连接状态、断开按钮
- Sidebar: 导航菜单（Console、Schema）
- Content: 主内容区域

**Header功能**:
- 显示连接状态（绿色/红色）
- 显示host和username
- 断开连接按钮

**Sidebar功能**:
- 导航链接
- 当前页面高亮

**验收**:
- 布局显示正常
- 导航可切换
- 断开连接可返回登录页

---

### 步骤7: 路由配置

**任务**: 配置React Router

**路由结构**:
- `/login`: 登录页面
- `/`: 主布局（嵌套路由）
  - `/console`: 查询控制台（占位）
  - `/schema`: Schema管理（占位）

**路由守卫**:
- 未连接时自动跳转到登录页
- 已连接时访问登录页跳转到主页面

**验收**:
- 路由切换正常
- 路由守卫生效

---

### 步骤8: 健康检查

**任务**: 实现连接健康检查

**功能**:
- 定时调用 `/api/health`
- 连接断开时显示警告
- 连接断开时自动跳转登录页

**验收**:
- 健康检查定时执行
- 连接断开时正确提示

---

## 3. API接口清单

| 接口 | 方法 | 用途 |
|------|------|------|
| `/api/connect` | POST | 建立连接 |
| `/api/disconnect` | POST | 断开连接 |
| `/api/health` | GET | 健康检查 |

**请求头**:
- `X-Session-ID`: 会话ID（除connect外）

---

## 4. 组件清单

### 公共组件

| 组件 | 用途 | 优先级 |
|------|------|--------|
| Button | 按钮 | P1 |
| Icon | 图标 | P1 |
| EmptyTableTip | 空数据提示 | P2 |
| Avatar | 用户头像 | P2 |
| ErrorBoundary | 错误边界 | P1 |

### 布局组件

| 组件 | 用途 | 优先级 |
|------|------|--------|
| Header | 页面头部 | P0 |
| Sidebar | 侧边导航 | P0 |
| MainLayout | 主布局 | P0 |

### 页面

| 页面 | 用途 | 优先级 |
|------|------|--------|
| Login | 登录/连接 | P0 |
| MainPage | 主框架 | P0 |

---

## 5. 依赖清单

### 生产依赖

```
react
react-dom
react-router-dom
antd
zustand
axios
```

### 开发依赖

```
typescript
vite
@vitejs/plugin-react
eslint
prettier
@typescript-eslint/*
```

---

## 6. 配置清单

### 需要创建的配置文件

| 文件 | 用途 |
|------|------|
| `vite.config.ts` | Vite配置 |
| `tsconfig.json` | TypeScript配置 |
| `.eslintrc.js` | ESLint配置 |
| `.prettierrc` | Prettier配置 |
| `.env.development` | 开发环境变量 |
| `.env.production` | 生产环境变量 |

---

## 7. 验收检查表

### 功能验收

- [ ] 项目可正常启动
- [ ] 登录页面可访问
- [ ] 表单验证正常
- [ ] 连接成功可跳转
- [ ] 连接失败显示错误
- [ ] 主布局显示正常
- [ ] 导航可切换
- [ ] 断开连接可返回登录页
- [ ] 健康检查定时执行

### 代码质量

- [ ] 无 ESLint 错误
- [ ] TypeScript 类型完整
- [ ] 代码格式统一

---

## 8. 风险与应对

| 风险 | 应对 |
|------|------|
| 后端API未就绪 | 使用Mock数据开发 |
| 跨域问题 | 配置Vite代理 |

---

## 9. 参考文档

- [技术栈设计](../architecture/tech_stack.md)
- [目录结构设计](../architecture/directory_structure.md)
- [组件复用分析](../component_reuse_analysis.md)
- [Web API文档](../../api/web/web_api_overview.md)

---

**文档结束**
