# Space与用户管理机制改进方案

## 参考nebula-graph实现

本文档基于nebula-graph 3.8.0的Meta服务架构，针对GraphDB当前Space和用户管理存在的问题，提出改进方案。

---

## 一、当前问题总结

### 1.1 用户管理问题

| 问题 | 严重程度 | 说明 |
|------|----------|------|
| 用户数据非持久化 | **严重** | 用户仅存储在内存HashMap中，重启后丢失 |
| 密码明文存储 | **严重** | UserInfo.password为明文，存在安全风险 |
| 缺少用户列表功能 | 中等 | 没有list_users接口 |
| 权限数据不一致 | 中等 | PermissionManager和UserInfo.roles双重存储 |
| 缺少资源限制 | 低 | 没有max_queries_per_hour等限制 |

### 1.2 Space管理问题

| 问题 | 严重程度 | 说明 |
|------|----------|------|
| Space ID生成问题 | **严重** | space_id固定为0，未实现自增 |
| 类型定义重复 | 中等 | 多个SpaceInfo定义存在于不同模块 |
| 缺少Space级别权限检查 | 中等 | 切换Space时未验证用户权限 |

### 1.3 权限管理问题

| 问题 | 严重程度 | 说明 |
|------|----------|------|
| 角色模型过于简单 | 中等 | 只有Admin/User两级，缺少GOD/DBA/GUEST |
| 权限检查不完整 | 中等 | 缺少canReadSpace/canWriteSchema等细粒度检查 |
| Space ID类型不一致 | 低 | i32和i64混用 |

---

## 二、nebula-graph参考架构

### 2.1 核心数据结构对比

#### RoleType（角色类型）

**nebula-graph实现**（meta.thrift）:
```thrift
enum RoleType {
    GOD    = 0x01,  // 全局超级管理员，类似Linux root
    ADMIN  = 0x02,  // Space管理员
    DBA    = 0x03,  // 数据库管理员，可修改Schema
    USER   = 0x04,  // 普通用户，读写数据
    GUEST  = 0x05,  // 只读用户
}
```

**GraphDB当前实现**:
```rust
pub enum RoleType {
    Admin,  // 拥有所有权限
    User,   // 仅读写权限
}
```

#### UserItem（用户信息）

**nebula-graph实现**:
```thrift
struct UserItem {
    1: binary account,
    2: bool   is_lock,                  // 是否锁定
    3: i32    max_queries_per_hour,     // 每小时最大查询数
    4: i32    max_updates_per_hour,     // 每小时最大更新数
    5: i32    max_connections_per_hour, // 每小时最大连接数
    6: i32    max_user_connections,     // 最大并发连接数
}
```

**GraphDB当前实现**:
```rust
pub struct UserInfo {
    pub username: String,
    pub password: String,        // 明文存储（问题）
    pub role: String,
    pub is_locked: bool,
    pub roles: HashMap<i32, String>, // Space ID -> 角色
}
```

#### RoleItem（角色授权）

**nebula-graph实现**:
```thrift
struct RoleItem {
    1: binary               user_id,
    2: common.GraphSpaceID  space_id,
    3: RoleType             role_type,
}
```

**GraphDB当前实现**:
```rust
// 分散在两个地方：
// 1. PermissionManager.user_roles: HashMap<String, HashMap<i64, RoleType>>
// 2. UserInfo.roles: HashMap<i32, String>
```

#### SpaceDesc / SpaceItem

**nebula-graph实现**:
```thrift
struct SpaceDesc {
    1: binary                   space_name,
    2: i32                      partition_num = 0,
    3: i32                      replica_factor = 0,
    4: binary                   charset_name,
    5: binary                   collate_name,
    6: ColumnTypeDef            vid_type,
    7: list<binary>             zone_names,      // 分布式区域
    8: optional IsolationLevel  isolation_level, // 事务隔离级别
    9: optional binary          comment,
}

struct SpaceItem {
    1: common.GraphSpaceID  space_id,
    2: SpaceDesc            properties,
}
```

### 2.2 Meta服务接口对比

#### nebula-graph Meta服务接口

| 操作类型 | 接口名称 | 说明 |
|----------|----------|------|
| **Space管理** | | |
| | createSpace | 创建Space |
| | dropSpace | 删除Space |
| | getSpace | 获取Space信息 |
| | listSpaces | 列出所有Space |
| | alterSpace | 修改Space配置 |
| **用户管理** | | |
| | createUser | 创建用户 |
| | dropUser | 删除用户 |
| | alterUser | 修改用户 |
| | listUsers | 列出所有用户 |
| | changePassword | 修改密码 |
| **角色管理** | | |
| | grantRole | 授予角色 |
| | revokeRole | 撤销角色 |
| | listRoles | 列出Space的角色 |
| | getUserRoles | 获取用户的所有角色 |

#### GraphDB当前接口

| 操作类型 | 接口名称 | 状态 |
|----------|----------|------|
| Space管理 | create_space | 已实现 |
| | drop_space | 已实现 |
| | get_space | 已实现 |
| | list_spaces | 已实现 |
| | alter_space | 部分实现 |
| 用户管理 | create_user | 已实现（内存） |
| | drop_user | 已实现（内存） |
| | alter_user | 已实现（内存） |
| | **list_users** | **缺失** |
| | change_password | 已实现 |
| 角色管理 | grant_role | 已实现（内存） |
| | revoke_role | 已实现（内存） |
| | **list_roles** | **缺失** |

---

## 三、改进方案设计

### 3.1 整体架构调整

```
┌─────────────────────────────────────────────────────────────┐
│                      API Service Layer                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ GraphService│  │AuthService  │  │  AdminService       │  │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │
└─────────┼────────────────┼────────────────────┼─────────────┘
          │                │                    │
          ▼                ▼                    ▼
┌─────────────────────────────────────────────────────────────┐
│                      Meta Service Layer                      │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              MetaService (统一入口)                  │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐            │    │
│  │  │SpaceManager│ │UserManager│ │RoleManager│            │    │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘            │    │
│  └───────┼────────────┼────────────┼──────────────────┘    │
└──────────┼────────────┼────────────┼─────────────────────────┘
           │            │            │
           ▼            ▼            ▼
┌─────────────────────────────────────────────────────────────┐
│                      Storage Layer                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ SPACES_TABLE │  │  USERS_TABLE │  │ ROLES_TABLE  │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 数据模型改进

#### 3.2.1 新的RoleType定义

```rust
// src/api/service/permission_manager.rs

/// 角色类型 - 参考nebula-graph实现
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub enum RoleType {
    /// 全局超级管理员，拥有所有权限（类似Linux root）
    God = 0x01,
    /// Space管理员，可以管理Space内的Schema和用户
    Admin = 0x02,
    /// 数据库管理员，可以修改Schema
    Dba = 0x03,
    /// 普通用户，可以读写数据
    User = 0x04,
    /// 只读用户，只能读取数据
    Guest = 0x05,
}

impl RoleType {
    /// 检查角色是否拥有指定权限
    pub fn has_permission(&self, permission: Permission) -> bool {
        match self {
            RoleType::God => true,
            RoleType::Admin => matches!(
                permission,
                Permission::Read | Permission::Write | Permission::Delete |
                Permission::Schema | Permission::Admin
            ),
            RoleType::Dba => matches!(
                permission,
                Permission::Read | Permission::Write | Permission::Delete |
                Permission::Schema
            ),
            RoleType::User => matches!(
                permission,
                Permission::Read | Permission::Write | Permission::Delete
            ),
            RoleType::Guest => matches!(permission, Permission::Read),
        }
    }

    /// 检查是否可以授予目标角色
    pub fn can_grant(&self, target_role: RoleType) -> bool {
        match self {
            RoleType::God => true,
            RoleType::Admin => matches!(target_role, RoleType::Dba | RoleType::User | RoleType::Guest),
            RoleType::Dba => matches!(target_role, RoleType::User | RoleType::Guest),
            _ => false,
        }
    }
}
```

#### 3.2.2 新的UserInfo定义

```rust
// src/core/types/metadata.rs

/// 用户信息 - 持久化到存储层
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct UserInfo {
    /// 用户名（唯一标识）
    pub username: String,
    /// 密码哈希（bcrypt加密）
    pub password_hash: String,
    /// 是否锁定
    pub is_locked: bool,
    /// 每小时最大查询数（0表示无限制）
    pub max_queries_per_hour: i32,
    /// 每小时最大更新数（0表示无限制）
    pub max_updates_per_hour: i32,
    /// 每小时最大连接数（0表示无限制）
    pub max_connections_per_hour: i32,
    /// 最大并发连接数（0表示无限制）
    pub max_user_connections: i32,
    /// 创建时间
    pub created_at: i64,
    /// 最后登录时间
    pub last_login_at: Option<i64>,
    /// 密码最后修改时间
    pub password_changed_at: i64,
}

impl UserInfo {
    pub fn new(username: String, password: String) -> Result<Self, StorageError> {
        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| StorageError::DbError(format!("密码加密失败: {}", e)))?;
        
        let now = chrono::Utc::now().timestamp_millis();
        
        Ok(Self {
            username,
            password_hash,
            is_locked: false,
            max_queries_per_hour: 0,
            max_updates_per_hour: 0,
            max_connections_per_hour: 0,
            max_user_connections: 0,
            created_at: now,
            last_login_at: None,
            password_changed_at: now,
        })
    }

    /// 验证密码
    pub fn verify_password(&self, password: &str) -> bool {
        bcrypt::verify(password, &self.password_hash).unwrap_or(false)
    }

    /// 修改密码
    pub fn change_password(&mut self, new_password: String) -> Result<(), StorageError> {
        self.password_hash = bcrypt::hash(new_password, bcrypt::DEFAULT_COST)
            .map_err(|e| StorageError::DbError(format!("密码加密失败: {}", e)))?;
        self.password_changed_at = chrono::Utc::now().timestamp_millis();
        Ok(())
    }
}
```

#### 3.2.3 新的RoleItem定义

```rust
// src/core/types/metadata.rs

/// 角色授权信息 - 表示用户在某个Space的角色
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct RoleItem {
    /// 用户名
    pub username: String,
    /// Space ID
    pub space_id: i32,
    /// 角色类型
    pub role_type: RoleType,
    /// 授权时间
    pub granted_at: i64,
    /// 授权者（谁授予的该角色）
    pub granted_by: Option<String>,
}

impl RoleItem {
    pub fn new(username: String, space_id: i32, role_type: RoleType) -> Self {
        Self {
            username,
            space_id,
            role_type,
            granted_at: chrono::Utc::now().timestamp_millis(),
            granted_by: None,
        }
    }

    pub fn with_granted_by(mut self, granted_by: String) -> Self {
        self.granted_by = Some(granted_by);
        self
    }
}
```

#### 3.2.4 新的SpaceInfo定义

```rust
// src/core/types/metadata.rs

/// Space描述信息
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct SpaceDesc {
    pub space_name: String,
    pub partition_num: i32,
    pub replica_factor: i32,
    pub charset_name: String,
    pub collate_name: String,
    pub vid_type: DataType,
    pub comment: Option<String>,
}

/// Space完整信息（包含ID）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct SpaceItem {
    pub space_id: i32,
    pub properties: SpaceDesc,
    pub created_at: i64,
    pub created_by: Option<String>,
}

/// 保留原有的SpaceInfo用于兼容性，逐步迁移到SpaceItem
pub type SpaceInfo = SpaceItem;
```

### 3.3 存储层改进

#### 3.3.1 新增Redb表定义

```rust
// src/storage/redb_types.rs

use redb::TableDefinition;

/// 用户表: username -> UserInfo
pub const USERS_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("users");

/// 角色授权表: (username, space_id) -> RoleItem
pub const ROLES_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("roles");

/// Space ID计数器: "space_id_counter" -> i32
pub const SPACE_ID_COUNTER_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("space_id_counter");

/// 用户登录尝试记录: username -> (failed_attempts, last_attempt_time)
pub const USER_LOGIN_ATTEMPTS_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("user_login_attempts");
```

#### 3.3.2 新的UserManager trait

```rust
// src/storage/metadata/user_manager.rs

use crate::core::types::metadata::{UserInfo, RoleItem, UserAlterInfo};
use crate::api::service::permission_manager::RoleType;
use crate::core::StorageError;

/// 用户管理接口
pub trait UserManager: Send + Sync + std::fmt::Debug {
    /// 创建用户
    fn create_user(&self, user: &UserInfo) -> Result<bool, StorageError>;
    
    /// 删除用户
    fn drop_user(&self, username: &str) -> Result<bool, StorageError>;
    
    /// 修改用户信息
    fn alter_user(&self, info: &UserAlterInfo) -> Result<bool, StorageError>;
    
    /// 获取用户信息
    fn get_user(&self, username: &str) -> Result<Option<UserInfo>, StorageError>;
    
    /// 列出所有用户
    fn list_users(&self) -> Result<Vec<UserInfo>, StorageError>;
    
    /// 授予角色
    fn grant_role(&self, role_item: &RoleItem) -> Result<bool, StorageError>;
    
    /// 撤销角色
    fn revoke_role(&self, username: &str, space_id: i32) -> Result<bool, StorageError>;
    
    /// 获取用户在指定Space的角色
    fn get_role(&self, username: &str, space_id: i32) -> Result<Option<RoleType>, StorageError>;
    
    /// 获取用户的所有角色
    fn get_user_roles(&self, username: &str) -> Result<Vec<RoleItem>, StorageError>;
    
    /// 列出指定Space的所有角色
    fn list_roles(&self, space_id: i32) -> Result<Vec<RoleItem>, StorageError>;
    
    /// 修改密码
    fn change_password(&self, username: &str, old_password: &str, new_password: &str) -> Result<bool, StorageError>;
    
    /// 验证用户登录
    fn authenticate(&self, username: &str, password: &str) -> Result<bool, StorageError>;
    
    /// 锁定用户
    fn lock_user(&self, username: &str) -> Result<bool, StorageError>;
    
    /// 解锁用户
    fn unlock_user(&self, username: &str) -> Result<bool, StorageError>;
}
```

#### 3.3.3 新的RedbUserManager实现

```rust
// src/storage/metadata/redb_user_manager.rs

use super::UserManager;
use crate::core::types::metadata::{UserInfo, RoleItem, UserAlterInfo};
use crate::api::service::permission_manager::RoleType;
use crate::core::StorageError;
use crate::storage::redb_types::{USERS_TABLE, ROLES_TABLE, USER_LOGIN_ATTEMPTS_TABLE, ByteKey};
use crate::storage::serializer::{user_to_bytes, user_from_bytes, role_to_bytes, role_from_bytes};
use redb::Database;
use std::sync::Arc;

pub struct RedbUserManager {
    db: Arc<Database>,
}

impl RedbUserManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

impl UserManager for RedbUserManager {
    fn create_user(&self, user: &UserInfo) -> Result<bool, StorageError> {
        let key = user.username.as_bytes();
        let user_bytes = user_to_bytes(user)?;

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        {
            let mut table = write_txn.open_table(USERS_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            // 检查用户是否已存在
            if table.get(ByteKey(key.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
                .is_some() {
                return Ok(false);
            }

            table.insert(ByteKey(key.to_vec()), ByteKey(user_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        Ok(true)
    }

    fn get_user(&self, username: &str) -> Result<Option<UserInfo>, StorageError> {
        let key = username.as_bytes();
        
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(USERS_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        match table.get(ByteKey(key.to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))? {
            Some(value) => {
                let user = user_from_bytes(&value.value().0)?;
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    fn list_users(&self) -> Result<Vec<UserInfo>, StorageError> {
        let read_txn = self.db.begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn.open_table(USERS_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        let mut users = Vec::new();
        for item in table.iter()
            .map_err(|e| StorageError::DbError(e.to_string()))? {
            let (_, value) = item.map_err(|e| StorageError::DbError(e.to_string()))?;
            let user = user_from_bytes(&value.value().0)?;
            users.push(user);
        }
        
        Ok(users)
    }

    fn grant_role(&self, role_item: &RoleItem) -> Result<bool, StorageError> {
        let key = format!("{}:{}", role_item.username, role_item.space_id);
        let role_bytes = role_to_bytes(role_item)?;

        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        {
            let mut table = write_txn.open_table(ROLES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            table.insert(ByteKey(key.as_bytes().to_vec()), ByteKey(role_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        Ok(true)
    }

    fn authenticate(&self, username: &str, password: &str) -> Result<bool, StorageError> {
        match self.get_user(username)? {
            Some(user) => {
                if user.is_locked {
                    return Err(StorageError::DbError("用户已被锁定".to_string()));
                }
                Ok(user.verify_password(password))
            }
            None => Ok(false),
        }
    }

    // ... 其他方法实现
}
```

### 3.4 Space ID生成改进

```rust
// src/storage/metadata/redb_schema_manager.rs

impl RedbSchemaManager {
    /// 生成下一个Space ID
    fn next_space_id(&self) -> Result<i32, StorageError> {
        let write_txn = self.db.begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        let next_id = {
            let mut table = write_txn.open_table(SPACE_ID_COUNTER_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            
            const COUNTER_KEY: &[u8] = b"space_id_counter";
            
            let current = match table.get(ByteKey(COUNTER_KEY.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))? {
                Some(value) => {
                    let bytes = value.value().0;
                    i32::from_le_bytes(bytes.try_into().unwrap_or([0; 4]))
                }
                None => 0,
            };
            
            let next = current + 1;
            table.insert(ByteKey(COUNTER_KEY.to_vec()), ByteKey(next.to_le_bytes().to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            
            next
        };
        
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        Ok(next_id)
    }

    /// 创建Space（带自增ID）
    fn create_space_with_id(&self, desc: &SpaceDesc, created_by: Option<String>) -> Result<SpaceItem, StorageError> {
        let space_id = self.next_space_id()?;
        
        let space_item = SpaceItem {
            space_id,
            properties: desc.clone(),
            created_at: chrono::Utc::now().timestamp_millis(),
            created_by,
        };
        
        // 存储Space...
        
        Ok(space_item)
    }
}
```

### 3.5 PermissionManager改进

```rust
// src/api/service/permission_manager.rs

use crate::core::types::metadata::RoleType;
use crate::storage::metadata::UserManager;
use std::sync::Arc;

pub struct PermissionManager {
    user_manager: Arc<dyn UserManager>,
}

impl PermissionManager {
    pub fn new(user_manager: Arc<dyn UserManager>) -> Self {
        Self { user_manager }
    }

    /// 检查是否可以读取Space（参考nebula-graph）
    pub fn can_read_space(&self, username: &str, space_id: i32) -> Result<(), PermissionError> {
        // GOD角色可以读取任何Space
        if self.is_god(username)? {
            return Ok(());
        }
        
        // 检查用户在Space的角色
        match self.user_manager.get_role(username, space_id)? {
            Some(_) => Ok(()), // 只要有角色就可以读取
            None => Err(PermissionError::NoPermission {
                user: username.to_string(),
                space: space_id,
                action: "read space".to_string(),
            }),
        }
    }

    /// 检查是否可以写入Space（创建Space）
    pub fn can_write_space(&self, username: &str) -> Result<(), PermissionError> {
        // 只有GOD可以创建Space
        if self.is_god(username)? {
            Ok(())
        } else {
            Err(PermissionError::NoPermission {
                user: username.to_string(),
                space: 0,
                action: "create space".to_string(),
            })
        }
    }

    /// 检查是否可以写入Schema
    pub fn can_write_schema(&self, username: &str, space_id: i32) -> Result<(), PermissionError> {
        match self.user_manager.get_role(username, space_id)? {
            Some(role) => {
                if role.has_permission(Permission::Schema) {
                    Ok(())
                } else {
                    Err(PermissionError::NoPermission {
                        user: username.to_string(),
                        space: space_id,
                        action: "write schema".to_string(),
                    })
                }
            }
            None => Err(PermissionError::NoPermission {
                user: username.to_string(),
                space: space_id,
                action: "write schema".to_string(),
            }),
        }
    }

    /// 检查是否可以写入角色
    pub fn can_write_role(
        &self,
        username: &str,
        target_role: RoleType,
        space_id: i32,
        target_user: &str,
    ) -> Result<(), PermissionError> {
        // 不能修改自己的角色
        if username == target_user {
            return Err(PermissionError::CannotModifyOwnRole);
        }

        // 获取当前用户的角色
        let user_role = self.user_manager.get_role(username, space_id)?;
        
        match user_role {
            Some(role) => {
                // 检查是否可以授予目标角色
                if role.can_grant(target_role) {
                    Ok(())
                } else {
                    Err(PermissionError::CannotGrantRole {
                        from: role,
                        to: target_role,
                    })
                }
            }
            None => Err(PermissionError::NoPermission {
                user: username.to_string(),
                space: space_id,
                action: "grant role".to_string(),
            }),
        }
    }

    /// 检查是否是GOD角色
    fn is_god(&self, username: &str) -> Result<bool, PermissionError> {
        // 获取用户在任意Space的角色，检查是否有GOD
        let roles = self.user_manager.get_user_roles(username)?;
        Ok(roles.iter().any(|r| r.role_type == RoleType::God))
    }
}

#[derive(Debug, Clone)]
pub enum PermissionError {
    NoPermission { user: String, space: i32, action: String },
    CannotModifyOwnRole,
    CannotGrantRole { from: RoleType, to: RoleType },
    UserNotFound(String),
    SpaceNotFound(i32),
}

impl std::fmt::Display for PermissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionError::NoPermission { user, space, action } => {
                write!(f, "用户 {} 在Space {} 没有 {} 权限", user, space, action)
            }
            PermissionError::CannotModifyOwnRole => {
                write!(f, "不能修改自己的角色")
            }
            PermissionError::CannotGrantRole { from, to } => {
                write!(f, "角色 {:?} 不能授予角色 {:?}", from, to)
            }
            PermissionError::UserNotFound(user) => {
                write!(f, "用户 {} 不存在", user)
            }
            PermissionError::SpaceNotFound(space) => {
                write!(f, "Space {} 不存在", space)
            }
        }
    }
}

impl std::error::Error for PermissionError {}
```

---

## 四、执行计划

### 4.1 第一阶段：数据模型重构

1. **修改RoleType定义**
   - 添加God、Dba、Guest角色
   - 实现has_permission和can_grant方法

2. **重构UserInfo**
   - 添加password_hash字段（替换明文password）
   - 添加资源限制字段
   - 添加时间戳字段

3. **新增RoleItem结构**
   - 表示用户在Space的角色授权

4. **统一SpaceInfo**
   - 删除重复定义
   - 添加SpaceDesc和SpaceItem

### 4.2 第二阶段：存储层实现

1. **新增Redb表**
   - USERS_TABLE
   - ROLES_TABLE
   - SPACE_ID_COUNTER_TABLE
   - USER_LOGIN_ATTEMPTS_TABLE

2. **实现UserManager trait**
   - RedbUserManager实现所有用户管理方法
   - 实现持久化存储

3. **修改SchemaManager**
   - 实现Space ID自增
   - 修改create_space接口

4. **添加序列化支持**
   - 实现user_to_bytes/user_from_bytes
   - 实现role_to_bytes/role_from_bytes

### 4.3 第三阶段：权限管理重构

1. **重构PermissionManager**
   - 依赖UserManager而不是内部HashMap
   - 实现can_read_space、can_write_space等方法
   - 添加PermissionError类型

2. **集成到执行器**
   - 修改CreateSpaceExecutor添加权限检查
   - 修改SwitchSpaceExecutor添加权限检查
   - 修改GrantRoleExecutor添加权限检查

### 4.4 第四阶段：API层适配

1. **修改StorageClient trait**
   - 添加list_users、list_roles方法
   - 修改create_user、grant_role等方法签名

2. **实现新的执行器**
   - ShowUsersExecutor
   - ShowRolesExecutor

3. **集成认证器**
   - 修改PasswordAuthenticator使用UserManager
   - 添加密码哈希验证

### 4.5 第五阶段：测试与验证

1. **单元测试**
   - UserManager各方法测试
   - PermissionManager权限检查测试
   - Space ID生成测试

2. **集成测试**
   - 用户CRUD完整流程测试
   - 角色授权流程测试
   - 权限检查流程测试

3. **数据迁移**
   - 如果有存量数据，编写迁移脚本

---

## 五、依赖添加

需要在Cargo.toml中添加以下依赖：

```toml
[dependencies]
# 密码哈希
bcrypt = "0.15"

# 序列化（如果尚未添加）
bincode = { version = "2.0", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
```

---

## 六、风险与注意事项

1. **密码迁移**：现有明文密码需要重新设置或编写迁移脚本进行哈希
2. **API兼容性**：StorageClient trait的修改会影响所有实现，需要同步修改MockStorage
3. **数据一致性**：确保UserManager和PermissionManager的数据一致性
4. **性能考虑**：权限检查会增加查询延迟，需要考虑缓存策略

---

## 七、参考文档

- nebula-graph meta.thrift: `nebula-3.8.0/src/interface/meta.thrift`
- nebula-graph PermissionManager: `nebula-3.8.0/src/graph/service/PermissionManager.h`
- nebula-graph MetaServiceHandler: `nebula-3.8.0/src/meta/MetaServiceHandler.h`
