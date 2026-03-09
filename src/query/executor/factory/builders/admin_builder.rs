//! 管理执行器构建器
//!
//! 负责创建管理类型的执行器（空间管理、标签管理、边管理、索引管理、用户管理）

use crate::core::error::QueryError;
use crate::query::executor::admin::{
    AlterEdgeExecutor, AlterTagExecutor, AlterUserExecutor, ChangePasswordExecutor,
    CreateEdgeExecutor, CreateEdgeIndexExecutor, CreateSpaceExecutor, CreateTagExecutor,
    CreateTagIndexExecutor, CreateUserExecutor, DescEdgeExecutor, DescEdgeIndexExecutor,
    DescSpaceExecutor, DescTagExecutor, DescTagIndexExecutor, DropEdgeExecutor,
    DropEdgeIndexExecutor, DropSpaceExecutor, DropTagExecutor, DropTagIndexExecutor,
    DropUserExecutor, RebuildEdgeIndexExecutor, RebuildTagIndexExecutor, ShowEdgeIndexesExecutor,
    ShowEdgesExecutor, ShowSpacesExecutor, ShowTagIndexesExecutor, ShowTagsExecutor,
};
use crate::query::executor::base::ExecutionContext;
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::planner::plan::core::nodes::{
    AlterEdgeNode, AlterTagNode, AlterUserNode, ChangePasswordNode, CreateEdgeIndexNode,
    CreateEdgeNode, CreateSpaceNode, CreateTagIndexNode, CreateTagNode, CreateUserNode,
    DescEdgeIndexNode, DescEdgeNode, DescSpaceNode, DescTagIndexNode, DescTagNode,
    DropEdgeIndexNode, DropEdgeNode, DropSpaceNode, DropTagIndexNode, DropTagNode, DropUserNode,
    RebuildEdgeIndexNode, RebuildTagIndexNode, ShowEdgeIndexesNode, ShowEdgesNode, ShowSpacesNode,
    ShowTagIndexesNode, ShowTagsNode,
};
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

/// 管理执行器构建器
pub struct AdminBuilder<S: StorageClient + Send + 'static> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient + Send + 'static> AdminBuilder<S> {
    /// 创建新的管理执行器构建器
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    // ========== 空间管理执行器 ==========

    /// 构建 CreateSpace 执行器
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

    /// 构建 DropSpace 执行器
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

    /// 构建 DescSpace 执行器
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

    /// 构建 ShowSpaces 执行器
    pub fn build_show_spaces(
        &self,
        _node: &ShowSpacesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = ShowSpacesExecutor::new(
            _node.id(),
            storage,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::ShowSpaces(executor))
    }

    // ========== 标签管理执行器 ==========

    /// 构建 CreateTag 执行器
    pub fn build_create_tag(
        &self,
        node: &CreateTagNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::admin::tag::create_tag::ExecutorTagInfo;
        let tag_info = ExecutorTagInfo::new(
            node.info().space_name.clone(),
            node.info().tag_name.clone(),
        )
        .with_properties(node.info().properties.clone());
        let executor = CreateTagExecutor::new(
            node.id(),
            storage,
            tag_info,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::CreateTag(executor))
    }

    /// 构建 AlterTag 执行器
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

    /// 构建 DescTag 执行器
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

    /// 构建 DropTag 执行器
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

    /// 构建 ShowTags 执行器
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

    // ========== 边类型管理执行器 ==========

    /// 构建 CreateEdge 执行器
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

    /// 构建 AlterEdge 执行器
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

    /// 构建 DescEdge 执行器
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

    /// 构建 DropEdge 执行器
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

    /// 构建 ShowEdges 执行器
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

    // ========== 标签索引管理执行器 ==========

    /// 构建 CreateTagIndex 执行器
    pub fn build_create_tag_index(
        &self,
        node: &CreateTagIndexNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::core::types::{Index, IndexType};
        let index = Index::new(
            0,
            node.info().index_name.clone(),
            0,
            node.info().target_name.clone(),
            Vec::new(),
            node.info().properties.clone(),
            IndexType::TagIndex,
            false,
        );
        let executor = CreateTagIndexExecutor::new(
            node.id(),
            storage,
            index,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::CreateTagIndex(executor))
    }

    /// 构建 DropTagIndex 执行器
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

    /// 构建 DescTagIndex 执行器
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

    /// 构建 ShowTagIndexes 执行器
    pub fn build_show_tag_indexes(
        &self,
        _node: &ShowTagIndexesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // ShowTagIndexesNode 没有 space_name 方法，使用空字符串
        let executor = ShowTagIndexesExecutor::new(
            _node.id(),
            storage,
            "".to_string(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::ShowTagIndexes(executor))
    }

    /// 构建 RebuildTagIndex 执行器
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

    // ========== 边索引管理执行器 ==========

    /// 构建 CreateEdgeIndex 执行器
    pub fn build_create_edge_index(
        &self,
        node: &CreateEdgeIndexNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::core::types::{Index, IndexType};
        let index = Index::new(
            0,
            node.info().index_name.clone(),
            0,
            node.info().target_name.clone(),
            Vec::new(),
            node.info().properties.clone(),
            IndexType::EdgeIndex,
            false,
        );
        let executor = CreateEdgeIndexExecutor::new(
            node.id(),
            storage,
            index,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::CreateEdgeIndex(executor))
    }

    /// 构建 DropEdgeIndex 执行器
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

    /// 构建 DescEdgeIndex 执行器
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

    /// 构建 ShowEdgeIndexes 执行器
    pub fn build_show_edge_indexes(
        &self,
        _node: &ShowEdgeIndexesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // ShowEdgeIndexesNode 没有 space_name 方法，使用空字符串
        let executor = ShowEdgeIndexesExecutor::new(
            _node.id(),
            storage,
            "".to_string(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::ShowEdgeIndexes(executor))
    }

    /// 构建 RebuildEdgeIndex 执行器
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

    // ========== 用户管理执行器 ==========

    /// 构建 CreateUser 执行器
    pub fn build_create_user(
        &self,
        node: &CreateUserNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::core::types::UserInfo;
        // CreateUserNode 使用 username() 和 password() 方法
        // UserInfo::new 需要 username 和 password 两个参数
        let user_info = UserInfo::new(
            node.username().to_string(),
            node.password().to_string(),
        ).map_err(|e| QueryError::ExecutionError(format!("创建用户信息失败: {}", e)))?;
        let executor = CreateUserExecutor::new(
            node.id(),
            storage,
            user_info,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::CreateUser(executor))
    }

    /// 构建 AlterUser 执行器
    pub fn build_alter_user(
        &self,
        node: &AlterUserNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::core::types::UserAlterInfo;
        // AlterUserNode 使用 username() 方法
        // AlterUserExecutor::new 需要 UserAlterInfo 对象
        let alter_info = UserAlterInfo::new(node.username().to_string());
        let executor = AlterUserExecutor::new(
            node.id(),
            storage,
            alter_info,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::AlterUser(executor))
    }

    /// 构建 DropUser 执行器
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

    /// 构建 ChangePassword 执行器
    pub fn build_change_password(
        &self,
        node: &ChangePasswordNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // ChangePasswordNode 使用 password_info() 方法获取 PasswordInfo
        let password_info = node.password_info();
        let username = password_info
            .username
            .clone()
            .unwrap_or_default();
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
}

impl<S: StorageClient + 'static> Default for AdminBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}
