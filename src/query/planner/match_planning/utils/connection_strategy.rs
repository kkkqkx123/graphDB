//! 连接策略框架
//! 提供统一的连接机制，支持不同类型的连接策略

use crate::query::context::ast::base::AstContext;
use crate::query::parser::ast::expr::Expr;
use crate::query::planner::match_planning::utils::join_params::{JoinAlgorithm, JoinParams};
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
/// 保持向后兼容的包装器，内部使用新的 JoinParams
#[derive(Debug, Clone)]
pub struct ConnectionParams {
    pub connection_type: ConnectionType,
    pub join_params: JoinParams,
}

impl ConnectionParams {
    pub fn inner_join(intersected_aliases: HashSet<String>) -> Self {
        Self {
            connection_type: ConnectionType::InnerJoin,
            join_params: JoinParams::inner_join(Vec::new(), intersected_aliases),
        }
    }

    pub fn left_join(intersected_aliases: HashSet<String>) -> Self {
        Self {
            connection_type: ConnectionType::LeftJoin,
            join_params: JoinParams::left_join(Vec::new(), intersected_aliases),
        }
    }

    pub fn cartesian() -> Self {
        Self {
            connection_type: ConnectionType::Cartesian,
            join_params: JoinParams::cartesian(),
        }
    }

    pub fn sequential(copy_col_names: bool) -> Self {
        Self {
            connection_type: ConnectionType::Sequential,
            join_params: JoinParams::sequential(copy_col_names),
        }
    }

    pub fn pattern_apply(intersected_aliases: HashSet<String>) -> Self {
        Self {
            connection_type: ConnectionType::PatternApply,
            join_params: JoinParams::pattern_apply(intersected_aliases),
        }
    }

    pub fn roll_up_apply(intersected_aliases: HashSet<String>) -> Self {
        Self {
            connection_type: ConnectionType::RollUpApply,
            join_params: JoinParams::roll_up_apply(intersected_aliases, Vec::new(), Vec::new()),
        }
    }

    /// 设置连接键
    pub fn with_join_keys(mut self, join_keys: Vec<Expr>) -> Self {
        self.join_params = self.join_params.with_join_keys(join_keys);
        self
    }

    /// 设置过滤条件
    pub fn with_filter_condition(mut self, filter_condition: Expr) -> Self {
        self.join_params = self.join_params.with_filter_condition(filter_condition);
        self
    }

    /// 获取交集别名
    pub fn intersected_aliases(&self) -> &HashSet<String> {
        &self.join_params.intersected_aliases
    }

    /// 获取连接键
    pub fn join_keys(&self) -> &Vec<Expr> {
        &self.join_params.join_keys
    }

    /// 获取过滤条件
    pub fn filter_condition(&self) -> &Option<Expr> {
        &self.join_params.filter_condition
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
        let mut join_node =
            crate::query::planner::plan::operations::join_ops::HashInnerJoin::new(0);
        join_node.deps.push(left_root.clone_plan_node());
        join_node.deps.push(right_root.clone_plan_node());

        // 设置连接参数
        join_node.join_params = Some(params.join_params.clone());

        Ok(SubPlan::new(
            Some(Arc::new(join_node.clone())),
            Some(Arc::new(join_node)),
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
        let mut join_node = crate::query::planner::plan::operations::join_ops::HashLeftJoin::new(0);
        join_node.deps.push(left_root.clone_plan_node());
        join_node.deps.push(right_root.clone_plan_node());

        // 设置连接参数
        join_node.join_params = Some(params.join_params.clone());

        Ok(SubPlan::new(
            Some(Arc::new(join_node.clone())),
            Some(Arc::new(join_node)),
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

        // 创建笛卡尔积节点
        let mut cartesian_node =
            crate::query::planner::plan::operations::join_ops::CrossJoin::new(0);
        cartesian_node.deps.push(left_root.clone_plan_node());
        cartesian_node.deps.push(right_root.clone_plan_node());

        // 设置连接参数
        cartesian_node.join_params = Some(params.join_params.clone());

        Ok(SubPlan::new(
            Some(Arc::new(cartesian_node.clone())),
            Some(Arc::new(cartesian_node)),
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
                let copy_col_names = params
                    .join_params
                    .as_sequential()
                    .map(|p| p.copy_col_names)
                    .unwrap_or(false);

                let mut col_names = if copy_col_names {
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
        let mut pattern_apply_node =
            crate::query::planner::plan::operations::data_processing_ops::PatternApply::new(
                0, "pattern", "apply",
            );
        pattern_apply_node.deps.push(left_root.clone_plan_node());
        pattern_apply_node.deps.push(right_root.clone_plan_node());

        // 设置连接参数
        pattern_apply_node.join_params = Some(params.join_params.clone());

        Ok(SubPlan::new(
            Some(Arc::new(pattern_apply_node.clone())),
            Some(Arc::new(pattern_apply_node)),
        ))
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
        let mut roll_up_apply_node =
            crate::query::planner::plan::operations::data_processing_ops::RollUpApply::new(
                0,
                Vec::new(),
                Vec::new(),
            );
        roll_up_apply_node.deps.push(left_root.clone_plan_node());
        roll_up_apply_node.deps.push(right_root.clone_plan_node());

        // 设置连接参数
        roll_up_apply_node.join_params = Some(params.join_params.clone());

        Ok(SubPlan::new(
            Some(Arc::new(roll_up_apply_node.clone())),
            Some(Arc::new(roll_up_apply_node)),
        ))
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
        assert_eq!(params.intersected_aliases(), &aliases);
        assert!(params.join_keys().is_empty());
        assert!(params.filter_condition().is_none());

        // 创建一个非空的连接键列表
        use crate::query::parser::ast::expr::{Expr, VariableExpr};
        use crate::query::parser::ast::types::Span;
        let mock_expr = Expr::Variable(VariableExpr::new("test".to_string(), Span::default()));
        let params_with_keys = params.with_join_keys(vec![mock_expr]);
        assert!(!params_with_keys.join_keys().is_empty());
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
