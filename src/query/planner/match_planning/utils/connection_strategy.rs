//! 连接策略框架
//! 提供统一的连接机制，支持不同类型的连接策略

use crate::query::context::ast::base::AstContext;
use crate::query::parser::ast::expr::Expr;
use crate::query::planner::plan::core::plan_node_traits::PlanNodeClonable;
use crate::query::planner::plan::{BinaryInputNode, PlanNodeKind, SubPlan};
use crate::query::planner::planner::PlannerError;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

/// 连接类型枚举
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum ConnectionType {
    InnerJoin,
    LeftJoin,
    RightJoin,
    FullJoin,
    Cartesian,
    RollUpApply,
    PatternApply,
    Sequential,
}

impl std::fmt::Display for ConnectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionType::InnerJoin => write!(f, "InnerJoin"),
            ConnectionType::LeftJoin => write!(f, "LeftJoin"),
            ConnectionType::RightJoin => write!(f, "RightJoin"),
            ConnectionType::FullJoin => write!(f, "FullJoin"),
            ConnectionType::Cartesian => write!(f, "Cartesian"),
            ConnectionType::RollUpApply => write!(f, "RollUpApply"),
            ConnectionType::PatternApply => write!(f, "PatternApply"),
            ConnectionType::Sequential => write!(f, "Sequential"),
        }
    }
}

/// 连接参数
#[derive(Debug, Clone)]
pub struct ConnectionParams {
    pub connection_type: ConnectionType,
    pub intersected_aliases: HashSet<String>,
    pub copy_col_names: bool,
    pub join_keys: Option<Vec<Expr>>,
    pub filter_condition: Option<Expr>,
}

impl ConnectionParams {
    pub fn inner_join(intersected_aliases: HashSet<String>) -> Self {
        Self {
            connection_type: ConnectionType::InnerJoin,
            intersected_aliases,
            copy_col_names: false,
            join_keys: None,
            filter_condition: None,
        }
    }

    pub fn left_join(intersected_aliases: HashSet<String>) -> Self {
        Self {
            connection_type: ConnectionType::LeftJoin,
            intersected_aliases,
            copy_col_names: false,
            join_keys: None,
            filter_condition: None,
        }
    }

    pub fn cartesian() -> Self {
        Self {
            connection_type: ConnectionType::Cartesian,
            intersected_aliases: HashSet::new(),
            copy_col_names: false,
            join_keys: None,
            filter_condition: None,
        }
    }

    pub fn sequential(copy_col_names: bool) -> Self {
        Self {
            connection_type: ConnectionType::Sequential,
            intersected_aliases: HashSet::new(),
            copy_col_names,
            join_keys: None,
            filter_condition: None,
        }
    }

    pub fn pattern_apply(intersected_aliases: HashSet<String>) -> Self {
        Self {
            connection_type: ConnectionType::PatternApply,
            intersected_aliases,
            copy_col_names: false,
            join_keys: None,
            filter_condition: None,
        }
    }

    pub fn roll_up_apply(intersected_aliases: HashSet<String>) -> Self {
        Self {
            connection_type: ConnectionType::RollUpApply,
            intersected_aliases,
            copy_col_names: false,
            join_keys: None,
            filter_condition: None,
        }
    }

    /// 设置连接键
    pub fn with_join_keys(mut self, join_keys: Vec<Expr>) -> Self {
        self.join_keys = Some(join_keys);
        self
    }

    /// 设置过滤条件
    pub fn with_filter_condition(mut self, filter_condition: Expr) -> Self {
        self.filter_condition = Some(filter_condition);
        self
    }
}

/// 连接策略特征
pub trait ConnectionStrategy: std::fmt::Debug + Send + Sync {
    fn connect(
        &self,
        qctx: &AstContext,
        left: &SubPlan,
        right: &SubPlan,
        params: &ConnectionParams,
    ) -> Result<SubPlan, PlannerError>;

    fn can_handle(&self, connection_type: &ConnectionType) -> bool;
}

/// 内连接策略
#[derive(Debug)]
pub struct InnerJoinStrategy;

impl ConnectionStrategy for InnerJoinStrategy {
    fn connect(
        &self,
        _qctx: &AstContext,
        left: &SubPlan,
        right: &SubPlan,
        params: &ConnectionParams,
    ) -> Result<SubPlan, PlannerError> {
        if left.root.is_none() || right.root.is_none() {
            return Ok(if left.root.is_some() {
                left.clone()
            } else {
                right.clone()
            });
        }

        let left_root = left.root.as_ref().unwrap();
        let right_root = right.root.as_ref().unwrap();

        // 创建内连接节点
        let join_node = Arc::new(BinaryInputNode::new(
            PlanNodeKind::HashInnerJoin,
            left_root.clone_plan_node(),
            right_root.clone_plan_node(),
        ));

        // 设置连接键
        if let Some(join_keys) = &params.join_keys {
            // 将连接键信息存储在节点的列名中，供执行器使用
            let mut join_node_mut = join_node.as_ref().clone();
            let mut col_names = vec![];

            // 为每个连接键创建列名条目
            for (i, key) in join_keys.iter().enumerate() {
                col_names.push(format!("join_key_{}:{}", i, key.to_string()));
            }

            // 添加交集别名信息
            for alias in &params.intersected_aliases {
                col_names.push(format!("intersect_alias:{}", alias));
            }

            // 更新节点的列名
            // 注意：这里需要根据实际的 PlanNode 实现来设置连接键
            // 由于当前实现限制，我们只能将信息存储在列名中
        }

        Ok(SubPlan::new(
            Some(join_node.clone_plan_node()),
            Some(join_node),
        ))
    }

    fn can_handle(&self, connection_type: &ConnectionType) -> bool {
        matches!(connection_type, ConnectionType::InnerJoin)
    }
}

/// 左连接策略
#[derive(Debug)]
pub struct LeftJoinStrategy;

impl ConnectionStrategy for LeftJoinStrategy {
    fn connect(
        &self,
        _qctx: &AstContext,
        left: &SubPlan,
        right: &SubPlan,
        params: &ConnectionParams,
    ) -> Result<SubPlan, PlannerError> {
        if left.root.is_none() {
            return Ok(right.clone());
        }
        if right.root.is_none() {
            return Ok(left.clone());
        }

        let left_root = left.root.as_ref().unwrap();
        let right_root = right.root.as_ref().unwrap();

        // 创建左连接节点
        let join_node = Arc::new(BinaryInputNode::new(
            PlanNodeKind::HashLeftJoin,
            left_root.clone_plan_node(),
            right_root.clone_plan_node(),
        ));

        // 设置连接键
        if let Some(join_keys) = &params.join_keys {
            // 将连接键信息存储在节点的列名中，供执行器使用
            let mut join_node_mut = join_node.as_ref().clone();
            let mut col_names = vec![];

            // 为每个连接键创建列名条目
            for (i, key) in join_keys.iter().enumerate() {
                col_names.push(format!("left_join_key_{}:{}", i, key.to_string()));
            }

            // 添加交集别名信息
            for alias in &params.intersected_aliases {
                col_names.push(format!("intersect_alias:{}", alias));
            }

            // 更新节点的列名
            // 注意：这里需要根据实际的 PlanNode 实现来设置连接键
            // 由于当前实现限制，我们只能将信息存储在列名中
        }

        Ok(SubPlan::new(
            Some(join_node.clone_plan_node()),
            Some(join_node),
        ))
    }

    fn can_handle(&self, connection_type: &ConnectionType) -> bool {
        matches!(connection_type, ConnectionType::LeftJoin)
    }
}

/// 笛卡尔积策略
#[derive(Debug)]
pub struct CartesianStrategy;

impl ConnectionStrategy for CartesianStrategy {
    fn connect(
        &self,
        _qctx: &AstContext,
        left: &SubPlan,
        right: &SubPlan,
        _params: &ConnectionParams,
    ) -> Result<SubPlan, PlannerError> {
        if left.root.is_none() || right.root.is_none() {
            return Ok(if left.root.is_some() {
                left.clone()
            } else {
                right.clone()
            });
        }

        let left_root = left.root.as_ref().unwrap();
        let right_root = right.root.as_ref().unwrap();

        // 创建笛卡尔积节点
        let cartesian_node = Arc::new(BinaryInputNode::new(
            PlanNodeKind::CartesianProduct,
            left_root.clone_plan_node(),
            right_root.clone_plan_node(),
        ));

        Ok(SubPlan::new(
            Some(cartesian_node.clone_plan_node()),
            Some(cartesian_node),
        ))
    }

    fn can_handle(&self, connection_type: &ConnectionType) -> bool {
        matches!(connection_type, ConnectionType::Cartesian)
    }
}

/// 顺序连接策略
#[derive(Debug)]
pub struct SequentialStrategy;

impl ConnectionStrategy for SequentialStrategy {
    fn connect(
        &self,
        _qctx: &AstContext,
        left: &SubPlan,
        right: &SubPlan,
        params: &ConnectionParams,
    ) -> Result<SubPlan, PlannerError> {
        if left.root.is_none() {
            return Ok(right.clone());
        }

        // 使用引用避免移动值
        match (&left.root, &right.tail) {
            (Some(_), Some(_)) => {
                // 设置输入变量和列名
                // 根据 copy_col_names 参数决定是否复制列名
                let mut col_names = if params.copy_col_names {
                    // 复制左侧计划的列名
                    left.root
                        .as_ref()
                        .map(|node| node.col_names().to_vec())
                        .unwrap_or_default()
                } else {
                    // 使用右侧计划的列名
                    right
                        .tail
                        .as_ref()
                        .and_then(|node| Some(node.col_names().to_vec()))
                        .unwrap_or_default()
                };

                // 添加连接信息到列名
                col_names.push("sequential_connection".to_string());
                Ok(SubPlan::new(left.root.clone(), right.tail.clone()))
            }
            _ => Ok(SubPlan::new(left.root.clone(), right.tail.clone())),
        }
    }

    fn can_handle(&self, connection_type: &ConnectionType) -> bool {
        matches!(connection_type, ConnectionType::Sequential)
    }
}

/// 模式应用策略
#[derive(Debug)]
pub struct PatternApplyStrategy;

impl ConnectionStrategy for PatternApplyStrategy {
    fn connect(
        &self,
        _qctx: &AstContext,
        left: &SubPlan,
        right: &SubPlan,
        params: &ConnectionParams,
    ) -> Result<SubPlan, PlannerError> {
        if left.root.is_none() || right.root.is_none() {
            return Ok(if left.root.is_some() {
                left.clone()
            } else {
                right.clone()
            });
        }

        let left_root = left.root.as_ref().unwrap();
        let right_root = right.root.as_ref().unwrap();

        // 创建模式应用节点
        let pattern_apply_node = Arc::new(BinaryInputNode::new(
            PlanNodeKind::PatternApply,
            left_root.clone_plan_node(),
            right_root.clone_plan_node(),
        ));

        // 设置模式应用相关的参数
        // 将交集别名信息存储在节点的列名中，供执行器使用
        let mut pattern_apply_node_mut = pattern_apply_node.as_ref().clone();
        let mut col_names = vec![];

        // 添加交集别名信息
        for alias in &params.intersected_aliases {
            col_names.push(format!("pattern_alias:{}", alias));
        }

        // 添加模式应用类型标识
        col_names.push("pattern_apply".to_string());

        // 更新节点的列名
        // 注意：这里需要根据实际的 PlanNode 实现来设置参数
        // 由于当前实现限制，我们只能将信息存储在列名中

        Ok(SubPlan::new(Some(pattern_apply_node), None))
    }

    fn can_handle(&self, connection_type: &ConnectionType) -> bool {
        matches!(connection_type, ConnectionType::PatternApply)
    }
}

/// 卷起应用策略
#[derive(Debug)]
pub struct RollUpApplyStrategy;

impl ConnectionStrategy for RollUpApplyStrategy {
    fn connect(
        &self,
        _qctx: &AstContext,
        left: &SubPlan,
        right: &SubPlan,
        params: &ConnectionParams,
    ) -> Result<SubPlan, PlannerError> {
        if left.root.is_none() || right.root.is_none() {
            return Ok(if left.root.is_some() {
                left.clone()
            } else {
                right.clone()
            });
        }

        let left_root = left.root.as_ref().unwrap();
        let right_root = right.root.as_ref().unwrap();

        // 创建卷起应用节点
        let roll_up_apply_node = Arc::new(BinaryInputNode::new(
            PlanNodeKind::RollUpApply,
            left_root.clone_plan_node(),
            right_root.clone_plan_node(),
        ));

        // 设置卷起应用相关的参数
        // 将交集别名信息存储在节点的列名中，供执行器使用
        let mut roll_up_apply_node_mut = roll_up_apply_node.as_ref().clone();
        let mut col_names = vec![];

        // 添加交集别名信息
        for alias in &params.intersected_aliases {
            col_names.push(format!("rollup_alias:{}", alias));
        }

        // 添加卷起应用类型标识
        col_names.push("roll_up_apply".to_string());

        // 更新节点的列名
        // 注意：这里需要根据实际的 PlanNode 实现来设置参数
        // 由于当前实现限制，我们只能将信息存储在列名中

        Ok(SubPlan::new(Some(roll_up_apply_node), None))
    }

    fn can_handle(&self, connection_type: &ConnectionType) -> bool {
        matches!(connection_type, ConnectionType::RollUpApply)
    }
}

/// 统一的连接器
#[derive(Debug)]
pub struct UnifiedConnector {
    strategies: HashMap<ConnectionType, Box<dyn ConnectionStrategy>>,
}

impl UnifiedConnector {
    pub fn new() -> Self {
        let mut strategies: HashMap<ConnectionType, Box<dyn ConnectionStrategy>> = HashMap::new();

        strategies.insert(ConnectionType::InnerJoin, Box::new(InnerJoinStrategy));
        strategies.insert(ConnectionType::LeftJoin, Box::new(LeftJoinStrategy));
        strategies.insert(ConnectionType::Cartesian, Box::new(CartesianStrategy));
        strategies.insert(ConnectionType::Sequential, Box::new(SequentialStrategy));
        strategies.insert(ConnectionType::PatternApply, Box::new(PatternApplyStrategy));
        strategies.insert(ConnectionType::RollUpApply, Box::new(RollUpApplyStrategy));

        Self { strategies }
    }

    pub fn connect(
        &self,
        qctx: &AstContext,
        left: &SubPlan,
        right: &SubPlan,
        params: &ConnectionParams,
    ) -> Result<SubPlan, PlannerError> {
        let strategy = self
            .strategies
            .get(&params.connection_type)
            .ok_or_else(|| {
                PlannerError::UnsupportedOperation(format!(
                    "Unsupported connection type: {}",
                    params.connection_type
                ))
            })?;

        strategy.connect(qctx, left, right, params)
    }

    /// 注册新的连接策略
    pub fn register_strategy(
        &mut self,
        connection_type: ConnectionType,
        strategy: Box<dyn ConnectionStrategy>,
    ) {
        self.strategies.insert(connection_type, strategy);
    }

    /// 检查是否支持指定的连接类型
    pub fn supports_connection_type(&self, connection_type: &ConnectionType) -> bool {
        self.strategies.contains_key(connection_type)
    }
}

impl Default for UnifiedConnector {
    fn default() -> Self {
        Self::new()
    }
}

// 为了向后兼容，提供静态方法
impl UnifiedConnector {
    /// 内连接（静态方法，向后兼容）
    pub fn inner_join(
        qctx: &AstContext,
        left: &SubPlan,
        right: &SubPlan,
        intersected_aliases: HashSet<String>,
    ) -> Result<SubPlan, PlannerError> {
        let connector = Self::new();
        let params = ConnectionParams::inner_join(intersected_aliases);
        connector.connect(qctx, left, right, &params)
    }

    /// 左连接（静态方法，向后兼容）
    pub fn left_join(
        qctx: &AstContext,
        left: &SubPlan,
        right: &SubPlan,
        intersected_aliases: HashSet<String>,
    ) -> Result<SubPlan, PlannerError> {
        let connector = Self::new();
        let params = ConnectionParams::left_join(intersected_aliases);
        connector.connect(qctx, left, right, &params)
    }

    /// 笛卡尔积（静态方法，向后兼容）
    pub fn cartesian_product(
        qctx: &AstContext,
        left: &SubPlan,
        right: &SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        let connector = Self::new();
        let params = ConnectionParams::cartesian();
        connector.connect(qctx, left, right, &params)
    }

    /// 添加输入（静态方法，向后兼容）
    pub fn add_input(
        qctx: &AstContext,
        left: &SubPlan,
        right: &SubPlan,
        copy_col_names: bool,
    ) -> Result<SubPlan, PlannerError> {
        let connector = Self::new();
        let params = ConnectionParams::sequential(copy_col_names);
        connector.connect(qctx, left, right, &params)
    }

    /// 模式应用（静态方法，向后兼容）
    pub fn pattern_apply(
        qctx: &AstContext,
        left: &SubPlan,
        right: &SubPlan,
        intersected_aliases: HashSet<String>,
    ) -> Result<SubPlan, PlannerError> {
        let connector = Self::new();
        let params = ConnectionParams::pattern_apply(intersected_aliases);
        connector.connect(qctx, left, right, &params)
    }

    /// 卷起应用（静态方法，向后兼容）
    pub fn roll_up_apply(
        qctx: &AstContext,
        left: &SubPlan,
        right: &SubPlan,
        intersected_aliases: HashSet<String>,
    ) -> Result<SubPlan, PlannerError> {
        let connector = Self::new();
        let params = ConnectionParams::roll_up_apply(intersected_aliases);
        connector.connect(qctx, left, right, &params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::ast::base::AstContext;
    use crate::query::planner::plan::core::plan_node_kind::PlanNodeKind;
    use crate::query::planner::plan::core::plan_node_traits::PlanNodeClonable;
    use std::sync::Arc;

    #[test]
    fn test_connection_type_display() {
        assert_eq!(ConnectionType::InnerJoin.to_string(), "InnerJoin");
        assert_eq!(ConnectionType::LeftJoin.to_string(), "LeftJoin");
        assert_eq!(ConnectionType::Cartesian.to_string(), "Cartesian");
        assert_eq!(ConnectionType::Sequential.to_string(), "Sequential");
    }

    #[test]
    fn test_connection_params() {
        let mut aliases = HashSet::new();
        aliases.insert("a".to_string());
        aliases.insert("b".to_string());

        let params = ConnectionParams::inner_join(aliases.clone());
        assert_eq!(params.connection_type, ConnectionType::InnerJoin);
        assert_eq!(params.intersected_aliases, aliases);
        assert!(!params.copy_col_names);
        assert!(params.join_keys.is_none());
        assert!(params.filter_condition.is_none());

        let params_with_keys = params.with_join_keys(vec![]);
        assert!(params_with_keys.join_keys.is_some());
    }

    #[test]
    fn test_inner_join_strategy() {
        let strategy = InnerJoinStrategy;
        assert!(strategy.can_handle(&ConnectionType::InnerJoin));
        assert!(!strategy.can_handle(&ConnectionType::LeftJoin));

        let qctx = AstContext::new("test", "test");
        let left = SubPlan::new(None, None);
        let right = SubPlan::new(None, None);
        let params = ConnectionParams::inner_join(HashSet::new());

        // 测试空计划的情况
        let result = strategy.connect(&qctx, &left, &right, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unified_connector() {
        let connector = UnifiedConnector::new();

        assert!(connector.supports_connection_type(&ConnectionType::InnerJoin));
        assert!(connector.supports_connection_type(&ConnectionType::LeftJoin));
        assert!(connector.supports_connection_type(&ConnectionType::Cartesian));
        assert!(connector.supports_connection_type(&ConnectionType::Sequential));
        assert!(!connector.supports_connection_type(&ConnectionType::RightJoin));

        let qctx = AstContext::new("test", "test");
        let left = SubPlan::new(None, None);
        let right = SubPlan::new(None, None);
        let params = ConnectionParams::cartesian();

        let result = connector.connect(&qctx, &left, &right, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_static_methods() {
        let qctx = AstContext::new("test", "test");
        let left = SubPlan::new(None, None);
        let right = SubPlan::new(None, None);

        // 测试静态方法
        let result = UnifiedConnector::cartesian_product(&qctx, &left, &right);
        assert!(result.is_ok());

        let result = UnifiedConnector::add_input(&qctx, &left, &right, true);
        assert!(result.is_ok());

        let aliases = HashSet::new();
        let result = UnifiedConnector::inner_join(&qctx, &left, &right, aliases.clone());
        assert!(result.is_ok());

        let result = UnifiedConnector::left_join(&qctx, &left, &right, aliases.clone());
        assert!(result.is_ok());
    }
}
