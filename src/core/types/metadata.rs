//! 元数据类型定义（遗留类型）
//!
//! 此模块保留尚未迁移的类型，逐步迁移到各自的原子模块

use crate::core::Value;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

// 从原子模块重新导出基础类型
pub use crate::core::types::space::{generate_space_id, reset_space_id_counter, SpaceInfo};
pub use crate::core::types::property::PropertyDef;
pub use crate::core::types::tag::TagInfo;
pub use crate::core::types::edge::EdgeTypeInfo;
pub use crate::core::types::metadata_version::{MetadataVersion, SchemaVersion, SchemaHistory};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub enum SchemaChangeType {
    AddProperty,
    DropProperty,
    ModifyProperty,
    AddIndex,
    DropIndex,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct SchemaChange {
    pub change_type: SchemaChangeType,
    pub target: String,
    pub property: Option<PropertyDef>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct InsertVertexInfo {
    pub space_id: u64,
    pub vertex_id: Value,
    pub tag_name: String,
    pub props: Vec<(String, Value)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct InsertEdgeInfo {
    pub space_id: u64,
    pub src_vertex_id: Value,
    pub dst_vertex_id: Value,
    pub edge_name: String,
    pub rank: i64,
    pub props: Vec<(String, Value)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct UpdateTarget {
    pub space_name: String,
    pub label: String,
    pub id: Value,
    pub prop: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub enum UpdateOp {
    Set,
    Add,
    Subtract,
    Append,
    Remove,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct UpdateInfo {
    pub update_target: UpdateTarget,
    pub update_op: UpdateOp,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct PasswordInfo {
    pub username: Option<String>,
    pub old_password: String,
    pub new_password: String,
}

/// 用户信息 - 参考nebula-graph UserItem实现
/// 包含密码哈希和资源限制
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct UserInfo {
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
    /// 创建新用户（使用明文密码，内部自动哈希）
    pub fn new(username: String, password: String) -> Result<Self, crate::core::StorageError> {
        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| crate::core::StorageError::DbError(format!("密码加密失败: {}", e)))?;
        
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
    pub fn change_password(&mut self, new_password: String) -> Result<(), crate::core::StorageError> {
        self.password_hash = bcrypt::hash(new_password, bcrypt::DEFAULT_COST)
            .map_err(|e| crate::core::StorageError::DbError(format!("密码加密失败: {}", e)))?;
        self.password_changed_at = chrono::Utc::now().timestamp_millis();
        Ok(())
    }

    pub fn with_locked(mut self, is_locked: bool) -> Self {
        self.is_locked = is_locked;
        self
    }

    pub fn with_max_queries_per_hour(mut self, limit: i32) -> Self {
        self.max_queries_per_hour = limit;
        self
    }

    pub fn with_max_updates_per_hour(mut self, limit: i32) -> Self {
        self.max_updates_per_hour = limit;
        self
    }

    pub fn with_max_connections_per_hour(mut self, limit: i32) -> Self {
        self.max_connections_per_hour = limit;
        self
    }

    pub fn with_max_user_connections(mut self, limit: i32) -> Self {
        self.max_user_connections = limit;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct UserAlterInfo {
    pub username: String,
    /// 新的锁定状态
    pub is_locked: Option<bool>,
    /// 新的每小时最大查询数
    pub max_queries_per_hour: Option<i32>,
    /// 新的每小时最大更新数
    pub max_updates_per_hour: Option<i32>,
    /// 新的每小时最大连接数
    pub max_connections_per_hour: Option<i32>,
    /// 新的最大并发连接数
    pub max_user_connections: Option<i32>,
}

impl UserAlterInfo {
    pub fn new(username: String) -> Self {
        Self {
            username,
            is_locked: None,
            max_queries_per_hour: None,
            max_updates_per_hour: None,
            max_connections_per_hour: None,
            max_user_connections: None,
        }
    }

    pub fn with_locked(mut self, is_locked: bool) -> Self {
        self.is_locked = Some(is_locked);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct ClusterInfo {
    pub cluster_id: i32,
    pub nodes: Vec<String>,
    pub total_space: i64,
    pub used_space: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct CharsetInfo {
    pub charset: String,
    pub collation: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct SchemaExportConfig {
    pub space_id: Option<u64>,
    pub format: ExportFormat,
    pub include_comments: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub enum ExportFormat {
    JSON,
    YAML,
    Rust,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct SchemaImportResult {
    pub success: bool,
    pub space_name: String,
    pub imported_items: i32,
    pub imported_tags: Vec<String>,
    pub imported_edge_types: Vec<String>,
    pub skipped_items: Vec<String>,
    pub errors: Vec<String>,
}

impl Default for SchemaImportResult {
    fn default() -> Self {
        Self {
            success: false,
            space_name: String::new(),
            imported_items: 0,
            imported_tags: Vec::new(),
            imported_edge_types: Vec::new(),
            skipped_items: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl SchemaImportResult {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaFieldChange {
    pub field_name: String,
    pub change_type: FieldChangeType,
    pub old_value: Option<PropertyDef>,
    pub new_value: Option<PropertyDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldChangeType {
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaAlterOperation {
    pub space_name: String,
    pub target_type: AlterTargetType,
    pub target_name: String,
    pub field_changes: Vec<SchemaFieldChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlterTargetType {
    Tag,
    EdgeType,
}

impl SchemaAlterOperation {
    pub fn new_add_tag_field(
        space_name: String,
        tag_name: String,
        field: PropertyDef,
    ) -> Self {
        let field_name = field.name.clone();
        Self {
            space_name,
            target_type: AlterTargetType::Tag,
            target_name: tag_name,
            field_changes: vec![SchemaFieldChange {
                field_name,
                change_type: FieldChangeType::Added,
                old_value: None,
                new_value: Some(field),
            }],
        }
    }

    pub fn new_remove_tag_field(
        space_name: String,
        tag_name: String,
        field_name: String,
    ) -> Self {
        Self {
            space_name,
            target_type: AlterTargetType::Tag,
            target_name: tag_name,
            field_changes: vec![SchemaFieldChange {
                field_name,
                change_type: FieldChangeType::Removed,
                old_value: None,
                new_value: None,
            }],
        }
    }

    pub fn new_modify_tag_field(
        space_name: String,
        tag_name: String,
        old_field: PropertyDef,
        new_field: PropertyDef,
    ) -> Self {
        let field_name = old_field.name.clone();
        Self {
            space_name,
            target_type: AlterTargetType::Tag,
            target_name: tag_name,
            field_changes: vec![SchemaFieldChange {
                field_name,
                change_type: FieldChangeType::Modified,
                old_value: Some(old_field),
                new_value: Some(new_field),
            }],
        }
    }
}
