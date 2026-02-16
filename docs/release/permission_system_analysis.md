# GraphDB 权限系统分析文档

## 1. 概述

GraphDB 的权限系统是一个基于角色的访问控制（RBAC）实现，参考了 Nebula-Graph 的设计，采用5级角色模型，支持细粒度的权限控制。

## 2. 涉及模块

权限系统涉及以下核心模块：

### 2.1 核心权限模块

| 模块路径 | 说明 |
|---------|------|
| `src/api/service/permission_manager.rs` | 权限管理器核心实现 |
| `src/api/service/permission_checker.rs` | 权限检查器，统一权限检查入口 |
| `src/api/service/authenticator.rs` | 认证器，处理用户登录认证 |
| `src/api/session/client_session.rs` | 客户端会话，存储用户角色信息 |
| `src/config/mod.rs` | 认证配置定义 |

### 2.2 服务层集成

| 模块路径 | 说明 |
|---------|------|
| `src/api/service/graph_service.rs` | 图服务，集成权限检查到查询执行流程 |
| `src/api/service/mod.rs` | 服务模块导出 |
| `src/api/session/mod.rs` | 会话模块导出 |

## 3. 核心数据结构

### 3.1 权限类型 (Permission)

```rust
pub enum Permission {
    Read,    // 读取权限
    Write,   // 写入权限
    Delete,  // 删除权限
    Schema,  // Schema操作权限
    Admin,   // 管理权限
}
```

### 3.2 角色类型 (RoleType)

采用5级权限模型：

```rust
pub enum RoleType {
    God = 0x01,    // 全局超级管理员
    Admin = 0x02,  // Space管理员
    Dba = 0x03,    // 数据库管理员
    User = 0x04,   // 普通用户
    Guest = 0x05,  // 只读用户
}
```

### 3.3 角色权限矩阵

| 角色 | Read | Write | Delete | Schema | Admin |
|------|------|-------|--------|--------|-------|
| God | ✓ | ✓ | ✓ | ✓ | ✓ |
| Admin | ✓ | ✓ | ✓ | ✓ | ✓ |
| Dba | ✓ | ✓ | ✓ | ✓ | ✗ |
| User | ✓ | ✓ | ✓ | ✗ | ✗ |
| Guest | ✓ | ✗ | ✗ | ✗ | ✗ |

### 3.4 角色授予权限矩阵

| 角色 | 可授予角色 |
|------|-----------|
| God | God, Admin, Dba, User, Guest |
| Admin | Dba, User, Guest |
| Dba | User, Guest |
| User | 无 |
| Guest | 无 |

## 4. 核心功能

### 4.1 权限管理 (PermissionManager)

**位置**: `src/api/service/permission_manager.rs`

**主要功能**:

1. **角色管理**
   - `grant_role()`: 授予用户角色
   - `revoke_role()`: 撤销用户角色
   - `get_role()`: 获取用户在指定Space的角色
   - `list_user_roles()`: 列出用户的所有角色
   - `list_space_users()`: 列出Space中的所有用户及其角色

2. **权限检查**
   - `check_permission()`: 检查用户是否拥有指定权限
   - `can_read_space()`: 检查是否可以读取Space
   - `can_write_space()`: 检查是否可以写入Space（仅God）
   - `can_write_schema()`: 检查是否可以写入Schema
   - `can_write_role()`: 检查是否可以授予角色

3. **角色判断**
   - `is_god()`: 判断是否为God角色
   - `is_admin()`: 判断是否为Admin角色

### 4.2 权限检查器 (PermissionChecker)

**位置**: `src/api/service/permission_checker.rs`

**主要功能**:

1. **操作类型定义**
   ```rust
   pub enum OperationType {
       ReadSpace,      // USE, DESCRIBE SPACE
       WriteSpace,     // CREATE SPACE, DROP SPACE
       ReadSchema,     // DESCRIBE TAG, DESCRIBE EDGE
       WriteSchema,    // CREATE TAG, ALTER TAG
       ReadData,       // GO, MATCH, FETCH
       WriteData,      // INSERT, UPDATE, DELETE
       ReadUser,       // DESCRIBE USER
       WriteUser,      // CREATE USER, DROP USER
       WriteRole,      // GRANT, REVOKE
       Show,           // SHOW SPACES, SHOW USERS
       ChangePassword, // CHANGE PASSWORD
   }
   ```

2. **统一权限检查入口**
   - `check_permission()`: 统一的权限检查方法
   - 支持授权开关控制（`enable_authorize`）
   - 根据操作类型分发到具体的权限检查逻辑

3. **便捷方法**
   - `can_read_space()` / `can_write_space()`
   - `can_read_schema()` / `can_write_schema()`
   - `can_read_data()` / `can_write_data()`
   - `can_read_user()` / `can_write_user()`
   - `can_write_role()`

### 4.3 认证器 (Authenticator)

**位置**: `src/api/service/authenticator.rs`

**主要功能**:

1. **认证接口**
   ```rust
   pub trait Authenticator: Send + Sync {
       fn authenticate(&self, username: &str, password: &str) -> Result<()>;
   }
   ```

2. **密码认证器 (PasswordAuthenticator)**
   - 支持自定义用户验证回调
   - 支持登录失败次数限制
   - 支持账户锁定机制
   - 支持配置默认用户名/密码

3. **认证器工厂 (AuthenticatorFactory)**
   - 创建密码认证器
   - 创建默认密码认证器

### 4.4 会话角色管理 (ClientSession)

**位置**: `src/api/session/client_session.rs`

**主要功能**:

1. **角色存储**
   - 使用 `HashMap<i64, RoleType>` 存储用户在不同Space的角色
   - 支持多Space角色管理

2. **角色检查方法**
   - `is_god()`: 检查是否为God角色
   - `is_admin()`: 检查是否为Admin角色
   - `role_with_space()`: 获取指定Space的角色
   - `set_role()`: 设置角色

## 5. 配置项

**位置**: `src/config/mod.rs`

```rust
pub struct AuthConfig {
    pub enable_authorize: bool,              // 是否启用授权
    pub failed_login_attempts: u32,          // 登录失败次数限制
    pub session_idle_timeout_secs: u64,      // 会话空闲超时时间
    pub force_change_default_password: bool, // 是否强制修改默认密码
    pub default_username: String,            // 默认用户名
    pub default_password: String,            // 默认密码
}
```

## 6. 权限检查流程

### 6.1 查询执行时的权限检查

```
GraphService::execute_with_permission()
    ↓
检查是否启用授权 (enable_authorize)
    ↓
提取操作类型和权限
    ↓
PermissionManager::check_permission()
    ↓
判断是否为God角色（拥有所有权限）
    ↓
根据Space ID查找用户角色
    ↓
RoleType::has_permission() 检查权限
    ↓
返回结果（成功或权限不足）
```

### 6.2 角色授予流程

```
PermissionManager::can_write_role()
    ↓
检查是否修改自己的角色（禁止）
    ↓
检查是否为God角色（可以授予任何角色）
    ↓
获取当前用户在目标Space的角色
    ↓
RoleType::can_grant() 检查是否可以授予目标角色
    ↓
返回结果
```

## 7. 特殊处理

### 7.1 God角色

- 使用特殊的Space ID: `GOD_SPACE_ID = -1` 表示全局角色
- 不绑定特定Space，拥有所有Space的访问权限
- 可以执行所有操作，包括创建/删除Space
- 可以授予任何角色

### 7.2 默认用户

- 系统初始化时自动创建 `root` 用户作为God角色
- 使用配置中的默认用户名和密码

### 7.3 授权开关

- 通过 `enable_authorize` 配置项控制
- 禁用授权时，所有权限检查直接返回成功
- 适用于单用户模式或开发测试环境

## 8. 与 Nebula-Graph 的对比

| 特性 | GraphDB | Nebula-Graph |
|------|---------|--------------|
| 角色模型 | 5级（God/Admin/Dba/User/Guest） | 5级（GOD/ADMIN/DBA/USER/GUEST） |
| 权限粒度 | Space级别 | Space级别 |
| 认证方式 | 密码认证 | 密码认证 + LDAP/Cloud |
| 角色存储 | 内存（HashMap） | Meta服务持久化 |
| 会话管理 | 本地内存 | 分布式会话管理 |
| 权限缓存 | 无实时缓存 | ClientSession本地缓存 |

## 9. 测试覆盖

权限系统包含完整的单元测试：

- `permission_manager.rs`: 测试角色权限、角色授予、权限检查
- `permission_checker.rs`: 测试操作类型权限检查、便捷方法
- `authenticator.rs`: 测试认证成功/失败、登录限制

## 10. 使用示例

### 10.1 检查用户权限

```rust
let checker = PermissionChecker::new(permission_manager, auth_config);

// 检查Space读取权限
checker.can_read_space(&session, space_id)?;

// 检查数据写入权限
checker.can_write_data(&session, space_id)?;

// 检查角色授予权限
checker.can_write_role(&session, space_id, RoleType::User, "target_user")?;
```

### 10.2 授予角色

```rust
let pm = PermissionManager::new();

// 授予User角色
pm.grant_role("user1", space_id, RoleType::User)?;

// 授予Admin角色
pm.grant_role("admin1", space_id, RoleType::Admin)?;
```

### 10.3 用户认证

```rust
let auth = PasswordAuthenticator::new_default(config);

// 认证用户
match auth.authenticate("username", "password") {
    Ok(()) => println!("认证成功"),
    Err(e) => println!("认证失败: {}", e),
}
```

## 11. 总结

GraphDB 的权限系统实现了完整的RBAC模型，具有以下特点：

1. **5级角色模型**：清晰的权限层次结构
2. **Space级别隔离**：支持多租户场景
3. **细粒度权限控制**：支持读、写、删、Schema、Admin五种权限
4. **角色继承关系**：高角色可以授予低角色
5. **God超级管理员**：全局管理权限
6. **可配置授权**：支持禁用授权用于开发测试
7. **登录保护**：支持失败次数限制和账户锁定
