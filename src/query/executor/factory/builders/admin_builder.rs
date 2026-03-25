//! Management Executor Builder
//!
//! Responsible for creating and managing various types of executors (space management, tag management, edge management, index management, user management).

use crate::core::error::QueryError;
use crate::core::types::index::IndexConfig;
use crate::core::types::IndexField;
use crate::core::RoleType;
use crate::core::Value;
use crate::query::executor::admin::query_management::show_stats::ShowStatsType as ExecutorShowStatsType;
use crate::query::executor::admin::space::alter_space::SpaceAlterOption as ExecutorSpaceAlterOption;
use crate::query::executor::admin::{
    AlterEdgeExecutor, AlterSpaceExecutor, AlterTagExecutor, AlterUserExecutor,
    ChangePasswordExecutor, ClearSpaceExecutor, CreateEdgeExecutor, CreateEdgeIndexExecutor,
    CreateSpaceExecutor, CreateTagExecutor, CreateTagIndexExecutor, CreateUserExecutor,
    DescEdgeExecutor, DescEdgeIndexExecutor, DescSpaceExecutor, DescTagExecutor,
    DescTagIndexExecutor, DropEdgeExecutor, DropEdgeIndexExecutor, DropSpaceExecutor,
    DropTagExecutor, DropTagIndexExecutor, DropUserExecutor, GrantRoleExecutor,
    RebuildEdgeIndexExecutor, RebuildTagIndexExecutor, RevokeRoleExecutor, ShowEdgeIndexesExecutor,
    ShowEdgesExecutor, ShowSpacesExecutor, ShowStatsExecutor, ShowTagIndexesExecutor,
    ShowTagsExecutor, SwitchSpaceExecutor,
};
use crate::query::executor::base::ExecutionContext;
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::planning::plan::core::nodes::{
    AlterEdgeNode, AlterSpaceNode, AlterTagNode, AlterUserNode, ChangePasswordNode, ClearSpaceNode,
    CreateEdgeIndexNode, CreateEdgeNode, CreateSpaceNode, CreateTagIndexNode, CreateTagNode,
    CreateUserNode, DescEdgeIndexNode, DescEdgeNode, DescSpaceNode, DescTagIndexNode, DescTagNode,
    DropEdgeIndexNode, DropEdgeNode, DropSpaceNode, DropTagIndexNode, DropTagNode, DropUserNode,
    GrantRoleNode, RebuildEdgeIndexNode, RebuildTagIndexNode, RevokeRoleNode, ShowEdgeIndexesNode,
    ShowEdgesNode, ShowSpacesNode, ShowStatsNode, ShowTagIndexesNode, ShowTagsNode,
    SwitchSpaceNode,
};
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

/// Management Executor Builder
pub struct AdminBuilder<S: StorageClient + Send + 'static> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient + Send + 'static> AdminBuilder<S> {
    /// Create a new management executor builder.
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    // Space Management Executor

    /// Building the CreateSpace executor
    pub fn build_create_space(
        &self,
        node: &CreateSpaceNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::admin::space::create_space::ExecutorSpaceInfo;
        let space_info = ExecutorSpaceInfo::new(node.info().space_name.clone())
            .with_vid_type(node.info().vid_type.clone());
        let executor = CreateSpaceExecutor::new(
            node.id(),
            storage,
            space_info,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::CreateSpace(executor))
    }

    /// Building the DropSpace executor
    pub fn build_drop_space(
        &self,
        node: &DropSpaceNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = DropSpaceExecutor::new(
            node.id(),
            storage,
            node.space_name().to_string(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::DropSpace(executor))
    }

    /// Building the DescSpace executor
    pub fn build_desc_space(
        &self,
        node: &DescSpaceNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = DescSpaceExecutor::new(
            node.id(),
            storage,
            node.space_name().to_string(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::DescSpace(executor))
    }

    /// Building the ShowSpaces executor
    pub fn build_show_spaces(
        &self,
        _node: &ShowSpacesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor =
            ShowSpacesExecutor::new(_node.id(), storage, context.expression_context().clone());
        Ok(ExecutorEnum::ShowSpaces(executor))
    }

    // Tag Management Executor

    /// Building the CreateTag executor
    pub fn build_create_tag(
        &self,
        node: &CreateTagNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::admin::tag::create_tag::ExecutorTagInfo;
        let tag_info =
            ExecutorTagInfo::new(node.info().space_name.clone(), node.info().tag_name.clone())
                .with_properties(node.info().properties.clone());
        let executor = CreateTagExecutor::new(
            node.id(),
            storage,
            tag_info,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::CreateTag(executor))
    }

    /// Building the AlterTag executor
    pub fn build_alter_tag(
        &self,
        node: &AlterTagNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::admin::tag::alter_tag::{AlterTagInfo, AlterTagItem};
        let mut alter_info =
            AlterTagInfo::new(node.info().space_name.clone(), node.info().tag_name.clone());
        for prop in node.info().additions.iter() {
            let item = AlterTagItem::add_property(prop.clone());
            alter_info = alter_info.with_items(vec![item]);
        }
        for prop_name in node.info().deletions.iter() {
            let item = AlterTagItem::drop_property(prop_name.clone());
            alter_info = alter_info.with_items(vec![item]);
        }
        let executor = AlterTagExecutor::new(
            node.id(),
            storage,
            alter_info,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::AlterTag(executor))
    }

    /// Building the DescTag executor
    pub fn build_desc_tag(
        &self,
        node: &DescTagNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = DescTagExecutor::new(
            node.id(),
            storage,
            node.space_name().to_string(),
            node.tag_name().to_string(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::DescTag(executor))
    }

    /// Building the DropTag executor
    pub fn build_drop_tag(
        &self,
        node: &DropTagNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = DropTagExecutor::new(
            node.id(),
            storage,
            node.space_name().to_string(),
            node.tag_name().to_string(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::DropTag(executor))
    }

    /// Constructing the ShowTags executor
    pub fn build_show_tags(
        &self,
        _node: &ShowTagsNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = ShowTagsExecutor::new(
            _node.id(),
            storage,
            "".to_string(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::ShowTags(executor))
    }

    //  Ellison Type Management Executor ============

    /// Building the CreateEdge executor
    pub fn build_create_edge(
        &self,
        node: &CreateEdgeNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::admin::edge::create_edge::ExecutorEdgeInfo;
        let edge_info = ExecutorEdgeInfo {
            space_name: node.info().space_name.clone(),
            edge_name: node.info().edge_name.clone(),
            properties: node.info().properties.clone(),
            comment: None,
        };
        let executor = CreateEdgeExecutor::new(
            node.id(),
            storage,
            edge_info,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::CreateEdge(executor))
    }

    /// Building the AlterEdge executor
    pub fn build_alter_edge(
        &self,
        node: &AlterEdgeNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::admin::edge::alter_edge::{AlterEdgeInfo, AlterEdgeItem};
        let mut alter_info = AlterEdgeInfo::new(
            node.info().space_name.clone(),
            node.info().edge_name.clone(),
        );
        for prop in node.info().additions.iter() {
            let item = AlterEdgeItem::add_property(prop.clone());
            alter_info = alter_info.with_items(vec![item]);
        }
        for prop_name in node.info().deletions.iter() {
            let item = AlterEdgeItem::drop_property(prop_name.clone());
            alter_info = alter_info.with_items(vec![item]);
        }
        let executor = AlterEdgeExecutor::new(
            node.id(),
            storage,
            alter_info,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::AlterEdge(executor))
    }

    /// Building the DescEdge executor
    pub fn build_desc_edge(
        &self,
        node: &DescEdgeNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = DescEdgeExecutor::new(
            node.id(),
            storage,
            node.space_name().to_string(),
            node.edge_name().to_string(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::DescEdge(executor))
    }

    /// Building the DropEdge executor
    pub fn build_drop_edge(
        &self,
        node: &DropEdgeNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = DropEdgeExecutor::new(
            node.id(),
            storage,
            node.space_name().to_string(),
            node.edge_name().to_string(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::DropEdge(executor))
    }

    /// Constructing the ShowEdges executor
    pub fn build_show_edges(
        &self,
        _node: &ShowEdgesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = ShowEdgesExecutor::new(
            _node.id(),
            storage,
            "".to_string(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::ShowEdges(executor))
    }

    // Tag Index Management Executor

    /// Construct the CreateTagIndex executor.
    pub fn build_create_tag_index(
        &self,
        node: &CreateTagIndexNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::core::types::{Index, IndexType};
        let fields = node
            .info()
            .properties
            .iter()
            .map(|prop| IndexField::new(prop.clone(), Value::String("string".to_string()), false))
            .collect();
        let index = Index::new(IndexConfig {
            id: 0,
            name: node.info().index_name.clone(),
            space_id: 0,
            schema_name: node.info().target_name.clone(),
            fields,
            properties: node.info().properties.clone(),
            index_type: IndexType::TagIndex,
            is_unique: false,
        });
        let executor = CreateTagIndexExecutor::new(
            node.id(),
            storage,
            index,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::CreateTagIndex(executor))
    }

    /// Building the DropTagIndex executor
    pub fn build_drop_tag_index(
        &self,
        node: &DropTagIndexNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = DropTagIndexExecutor::new(
            node.id(),
            storage,
            node.space_name().to_string(),
            node.index_name().to_string(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::DropTagIndex(executor))
    }

    /// Constructing the DescTagIndex executor
    pub fn build_desc_tag_index(
        &self,
        node: &DescTagIndexNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = DescTagIndexExecutor::new(
            node.id(),
            storage,
            node.space_name().to_string(),
            node.index_name().to_string(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::DescTagIndex(executor))
    }

    /// Constructing the ShowTagIndexes executor
    pub fn build_show_tag_indexes(
        &self,
        _node: &ShowTagIndexesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // The ShowTagIndexesNode class does not have a method named “space_name”; therefore, an empty string is used in its place.
        let executor = ShowTagIndexesExecutor::new(
            _node.id(),
            storage,
            "".to_string(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::ShowTagIndexes(executor))
    }

    /// Constructing the RebuildTagIndex executor
    pub fn build_rebuild_tag_index(
        &self,
        node: &RebuildTagIndexNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = RebuildTagIndexExecutor::new(
            node.id(),
            storage,
            node.space_name().to_string(),
            node.index_name().to_string(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::RebuildTagIndex(executor))
    }

    // ========== Side Index Management Executor ----------

    /// Build the CreateEdgeIndex executor.
    pub fn build_create_edge_index(
        &self,
        node: &CreateEdgeIndexNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::core::types::{Index, IndexType};
        let fields = node
            .info()
            .properties
            .iter()
            .map(|prop| IndexField::new(prop.clone(), Value::String("string".to_string()), false))
            .collect();
        let index = Index::new(IndexConfig {
            id: 0,
            name: node.info().index_name.clone(),
            space_id: 0,
            schema_name: node.info().target_name.clone(),
            fields,
            properties: node.info().properties.clone(),
            index_type: IndexType::EdgeIndex,
            is_unique: false,
        });
        let executor = CreateEdgeIndexExecutor::new(
            node.id(),
            storage,
            index,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::CreateEdgeIndex(executor))
    }

    /// Constructing the DropEdgeIndex executor
    pub fn build_drop_edge_index(
        &self,
        node: &DropEdgeIndexNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = DropEdgeIndexExecutor::new(
            node.id(),
            storage,
            node.space_name().to_string(),
            node.index_name().to_string(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::DropEdgeIndex(executor))
    }

    /// Constructing the DescEdgeIndex executor
    pub fn build_desc_edge_index(
        &self,
        node: &DescEdgeIndexNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = DescEdgeIndexExecutor::new(
            node.id(),
            storage,
            node.space_name().to_string(),
            node.index_name().to_string(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::DescEdgeIndex(executor))
    }

    /// Constructing the ShowEdgeIndexes executor
    pub fn build_show_edge_indexes(
        &self,
        _node: &ShowEdgeIndexesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // The `ShowEdgeIndexesNode` class does not have a `space_name` method; therefore, an empty string is used in its place.
        let executor = ShowEdgeIndexesExecutor::new(
            _node.id(),
            storage,
            "".to_string(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::ShowEdgeIndexes(executor))
    }

    /// Constructing the RebuildEdgeIndex executor
    pub fn build_rebuild_edge_index(
        &self,
        node: &RebuildEdgeIndexNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = RebuildEdgeIndexExecutor::new(
            node.id(),
            storage,
            node.space_name().to_string(),
            node.index_name().to_string(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::RebuildEdgeIndex(executor))
    }

    // >User Management Executor==========

    /// Constructing the CreateUser executor
    pub fn build_create_user(
        &self,
        node: &CreateUserNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::core::types::UserInfo;
        // CreateUserNode 使用 username() 和 password() 方法
        // The `UserInfo::new` method requires two parameters: `username` and `password`.
        let user_info = UserInfo::new(node.username().to_string(), node.password().to_string())
            .map_err(|e| QueryError::ExecutionError(format!("创建用户信息失败: {}", e)))?;
        let executor = CreateUserExecutor::new(
            node.id(),
            storage,
            user_info,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::CreateUser(executor))
    }

    /// Constructing the AlterUser executor
    pub fn build_alter_user(
        &self,
        node: &AlterUserNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::core::types::UserAlterInfo;
        // AlterUserNode 使用 username() 方法
        // The `AlterUserExecutor::new` method requires a `UserAlterInfo` object.
        let alter_info = UserAlterInfo::new(node.username().to_string());
        let executor = AlterUserExecutor::new(
            node.id(),
            storage,
            alter_info,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::AlterUser(executor))
    }

    /// Building the DropUser executor
    pub fn build_drop_user(
        &self,
        node: &DropUserNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // DropUserNode 使用 username() 方法
        let executor = DropUserExecutor::new(
            node.id(),
            storage,
            node.username().to_string(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::DropUser(executor))
    }

    /// Constructing the ChangePassword executor
    pub fn build_change_password(
        &self,
        node: &ChangePasswordNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // ChangePasswordNode 使用 password_info() 方法获取 PasswordInfo
        let password_info = node.password_info();
        let username = password_info.username.clone().unwrap_or_default();
        let executor = ChangePasswordExecutor::new(
            node.id(),
            storage,
            Some(username),
            password_info.old_password.clone(),
            password_info.new_password.clone(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::ChangePassword(executor))
    }

    /// Building the GrantRole executor
    pub fn build_grant_role(
        &self,
        node: &GrantRoleNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let role = match node.role() {
            "admin" => RoleType::Admin,
            "dba" => RoleType::Dba,
            "user" => RoleType::User,
            "guest" => RoleType::Guest,
            _ => RoleType::User,
        };
        let executor = GrantRoleExecutor::new(
            node.id(),
            storage,
            node.username().to_string(),
            node.space_name().to_string(),
            role,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::GrantRole(executor))
    }

    /// Building the RevokeRole executor
    pub fn build_revoke_role(
        &self,
        node: &RevokeRoleNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = RevokeRoleExecutor::new(
            node.id(),
            storage,
            node.username().to_string(),
            node.space_name().to_string(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::RevokeRole(executor))
    }

    /// Building the SwitchSpace executor
    pub fn build_switch_space(
        &self,
        node: &SwitchSpaceNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = SwitchSpaceExecutor::new(
            node.id(),
            storage,
            node.space_name().to_string(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::SwitchSpace(executor))
    }

    /// Building the AlterSpace executor
    pub fn build_alter_space(
        &self,
        node: &AlterSpaceNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let options: Vec<ExecutorSpaceAlterOption> = node
            .options()
            .iter()
            .map(|opt| match opt {
                crate::query::planning::plan::core::nodes::SpaceAlterOption::Comment(c) => {
                    ExecutorSpaceAlterOption::Comment(c.clone())
                }
            })
            .collect();
        let executor = AlterSpaceExecutor::new(
            node.id(),
            storage,
            node.space_name().to_string(),
            options,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::AlterSpace(executor))
    }

    /// Building the ClearSpace executor
    pub fn build_clear_space(
        &self,
        node: &ClearSpaceNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = ClearSpaceExecutor::new(
            node.id(),
            storage,
            node.space_name().to_string(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::ClearSpace(executor))
    }

    /// Building the ShowStats executor
    pub fn build_show_stats(
        &self,
        node: &ShowStatsNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let stats_type = match node.stats_type() {
            crate::query::planning::plan::core::nodes::ShowStatsType::Storage => {
                ExecutorShowStatsType::Storage
            }
            crate::query::planning::plan::core::nodes::ShowStatsType::Space => {
                ExecutorShowStatsType::Space
            }
        };
        let executor = ShowStatsExecutor::new(
            node.id(),
            storage,
            stats_type,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::ShowStats(executor))
    }
}

impl<S: StorageClient + 'static> Default for AdminBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}
