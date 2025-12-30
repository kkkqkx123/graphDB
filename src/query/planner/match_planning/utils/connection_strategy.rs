//! 连接策略框架
//! 提供统一的连接机制，支持不同类型的连接策略

use crate::query::context::ast::base::AstContext;

use crate::query::planner::plan::utils::join_params::JoinParams;
use crate::query::planner::plan::PlanNodeFactory;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use std::collections::HashMap;
use std::collections::HashSet;

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

/// 连接策略特征
pub trait ConnectionStrategy: std::fmt::Debug + Send + Sync {
    fn connect(
        &self,
        qctx: &AstContext,
        left: &SubPlan,
        right: &SubPlan,
        params: &JoinParams,
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
        params: &JoinParams,
    ) -> Result<SubPlan, PlannerError> {
        if left.root.is_none() || right.root.is_none() {
            return Ok(if left.root.is_some() {
                left.clone()
            } else {
                right.clone()
            });
        }

        let left_root = left.root.as_ref().ok_or_else(|| {
            PlannerError::InvalidAstContext("Left plan should have a root node".to_string())
        })?;
        let right_root = right.root.as_ref().ok_or_else(|| {
            PlannerError::InvalidAstContext("Right plan should have a root node".to_string())
        })?;

        // 使用新的节点工厂创建内连接节点
        let hash_keys = params.join_keys.clone();
        let probe_keys = params.join_keys.clone(); // 简化处理，实际应该根据连接条件确定

        let join_node = PlanNodeFactory::create_inner_join(
            left_root.clone(),
            right_root.clone(),
            hash_keys,
            probe_keys,
        )?;

        Ok(SubPlan::from_single_node(join_node))
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
        params: &JoinParams,
    ) -> Result<SubPlan, PlannerError> {
        if left.root.is_none() {
            return Ok(right.clone());
        }
        if right.root.is_none() {
            return Ok(left.clone());
        }

        let left_root = left.root.as_ref().ok_or_else(|| {
            PlannerError::InvalidAstContext("Left plan should have a root node".to_string())
        })?;
        let right_root = right.root.as_ref().ok_or_else(|| {
            PlannerError::InvalidAstContext("Right plan should have a root node".to_string())
        })?;

        // 使用新的节点工厂创建左连接节点
        // 注意：这里我们暂时使用内连接节点，因为 LeftJoinNode 还没有实现
        // 在完整的实现中，应该创建一个专门的 LeftJoinNode
        let hash_keys = params.join_keys.clone();
        let probe_keys = params.join_keys.clone();

        let join_node = PlanNodeFactory::create_inner_join(
            left_root.clone(),
            right_root.clone(),
            hash_keys,
            probe_keys,
        )?;

        Ok(SubPlan::from_single_node(join_node))
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
        _params: &JoinParams,
    ) -> Result<SubPlan, PlannerError> {
        if left.root.is_none() || right.root.is_none() {
            return Ok(if left.root.is_some() {
                left.clone()
            } else {
                right.clone()
            });
        }

        let left_root = left.root.as_ref().ok_or_else(|| {
            PlannerError::InvalidAstContext("Left plan should have a root node".to_string())
        })?;
        let right_root = right.root.as_ref().ok_or_else(|| {
            PlannerError::InvalidAstContext("Right plan should have a root node".to_string())
        })?;

        // 使用新的节点工厂创建笛卡尔积节点
        // 注意：这里我们暂时使用内连接节点，因为 CartesianNode 还没有实现
        // 在完整的实现中，应该创建一个专门的 CartesianNode
        let join_node = PlanNodeFactory::create_inner_join(
            left_root.clone(),
            right_root.clone(),
            vec![],
            vec![],
        )?;

        Ok(SubPlan::from_single_node(join_node))
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
        params: &JoinParams,
    ) -> Result<SubPlan, PlannerError> {
        if left.root.is_none() {
            return Ok(right.clone());
        }

        // 使用引用避免移动值
        match (left.root.as_ref(), right.tail.as_ref()) {
            (Some(left_root), Some(right_tail)) => {
                // 设置输入变量和列名
                // 根据 copy_col_names 参数决定是否复制列名
                let copy_col_names = params
                    .as_sequential()
                    .map(|p| p.copy_col_names)
                    .unwrap_or(false);

                let mut col_names = if copy_col_names {
                    // 复制左侧计划的列名
                    left_root.col_names().to_vec()
                } else {
                    // 使用右侧计划的列名
                    right_tail.col_names().to_vec()
                };

                // 添加连接信息到列名
                col_names.push("sequential_connection".to_string());
                Ok(SubPlan::new(
                    Some(left_root.clone()),
                    Some(right_tail.clone()),
                ))
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
        _params: &JoinParams,
    ) -> Result<SubPlan, PlannerError> {
        if left.root.is_none() || right.root.is_none() {
            return Ok(if left.root.is_some() {
                left.clone()
            } else {
                right.clone()
            });
        }

        let left_root = left.root.as_ref().ok_or_else(|| {
            PlannerError::InvalidAstContext("Left plan should have a root node".to_string())
        })?;
        let right_root = right.root.as_ref().ok_or_else(|| {
            PlannerError::InvalidAstContext("Right plan should have a root node".to_string())
        })?;

        let join_node = PlanNodeFactory::create_inner_join(
            left_root.clone(),
            right_root.clone(),
            vec![],
            vec![],
        )?;

        Ok(SubPlan::from_single_node(join_node))
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
        _params: &JoinParams,
    ) -> Result<SubPlan, PlannerError> {
        if left.root.is_none() || right.root.is_none() {
            return Ok(if left.root.is_some() {
                left.clone()
            } else {
                right.clone()
            });
        }

        let left_root = left.root.as_ref().ok_or_else(|| {
            PlannerError::InvalidAstContext("Left plan should have a root node".to_string())
        })?;
        let right_root = right.root.as_ref().ok_or_else(|| {
            PlannerError::InvalidAstContext("Right plan should have a root node".to_string())
        })?;

        // 使用新的节点工厂创建卷起应用节点
        // 注意：这里我们暂时使用内连接节点，因为 RollUpApplyNode 还没有实现
        // 在完整的实现中，应该创建一个专门的 RollUpApplyNode
        let join_node = PlanNodeFactory::create_inner_join(
            left_root.clone(),
            right_root.clone(),
            vec![],
            vec![],
        )?;

        Ok(SubPlan::from_single_node(join_node))
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
        params: &JoinParams,
    ) -> Result<SubPlan, PlannerError> {
        // 根据 JoinParams 的类型确定连接类型
        let connection_type = match params.type_specific_params {
            crate::query::planner::plan::utils::join_params::TypeSpecificParams::InnerJoin(_) => {
                ConnectionType::InnerJoin
            }
            crate::query::planner::plan::utils::join_params::TypeSpecificParams::LeftJoin(_) => {
                ConnectionType::LeftJoin
            }
            crate::query::planner::plan::utils::join_params::TypeSpecificParams::RightJoin(_) => {
                ConnectionType::RightJoin
            }
            crate::query::planner::plan::utils::join_params::TypeSpecificParams::FullJoin(_) => {
                ConnectionType::FullJoin
            }
            crate::query::planner::plan::utils::join_params::TypeSpecificParams::Cartesian(_) => {
                ConnectionType::Cartesian
            }
            crate::query::planner::plan::utils::join_params::TypeSpecificParams::RollUpApply(_) => {
                ConnectionType::RollUpApply
            }
            crate::query::planner::plan::utils::join_params::TypeSpecificParams::PatternApply(
                _,
            ) => ConnectionType::PatternApply,
            crate::query::planner::plan::utils::join_params::TypeSpecificParams::Sequential(_) => {
                ConnectionType::Sequential
            }
        };

        let strategy = self.strategies.get(&connection_type).ok_or_else(|| {
            PlannerError::UnsupportedOperation(format!(
                "Unsupported connection type: {}",
                connection_type
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
        let params = JoinParams::inner_join(vec![], intersected_aliases);
        Self::new().connect(qctx, left, right, &params)
    }

    /// 左连接（静态方法，向后兼容）
    pub fn left_join(
        qctx: &AstContext,
        left: &SubPlan,
        right: &SubPlan,
        intersected_aliases: HashSet<String>,
    ) -> Result<SubPlan, PlannerError> {
        let params = JoinParams::left_join(vec![], intersected_aliases);
        Self::new().connect(qctx, left, right, &params)
    }

    /// 笛卡尔积（静态方法，向后兼容）
    pub fn cartesian_product(
        qctx: &AstContext,
        left: &SubPlan,
        right: &SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        let params = JoinParams::cartesian();
        Self::new().connect(qctx, left, right, &params)
    }

    /// 添加输入（静态方法，向后兼容）
    pub fn add_input(
        qctx: &AstContext,
        left: &SubPlan,
        right: &SubPlan,
        copy_col_names: bool,
    ) -> Result<SubPlan, PlannerError> {
        let params = JoinParams::sequential(copy_col_names);
        Self::new().connect(qctx, left, right, &params)
    }

    /// 模式应用（静态方法，向后兼容）
    pub fn pattern_apply(
        qctx: &AstContext,
        left: &SubPlan,
        right: &SubPlan,
        intersected_aliases: HashSet<String>,
    ) -> Result<SubPlan, PlannerError> {
        let params = JoinParams::pattern_apply(intersected_aliases);
        Self::new().connect(qctx, left, right, &params)
    }

    /// 卷起应用（静态方法，向后兼容）
    pub fn roll_up_apply(
        qctx: &AstContext,
        left: &SubPlan,
        right: &SubPlan,
        intersected_aliases: HashSet<String>,
    ) -> Result<SubPlan, PlannerError> {
        let params = JoinParams::roll_up_apply(intersected_aliases, vec![], vec![]);
        Self::new().connect(qctx, left, right, &params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::ast::base::AstContext;

    use std::sync::Arc;

    #[test]
    fn test_connection_type_display() {
        assert_eq!(ConnectionType::InnerJoin.to_string(), "InnerJoin");
        assert_eq!(ConnectionType::LeftJoin.to_string(), "LeftJoin");
        assert_eq!(ConnectionType::Cartesian.to_string(), "Cartesian");
        assert_eq!(ConnectionType::Sequential.to_string(), "Sequential");
    }

    #[test]
    fn test_join_params() {
        let mut aliases = HashSet::new();
        aliases.insert("a".to_string());
        aliases.insert("b".to_string());

        let params = JoinParams::inner_join(vec![], aliases.clone());
        assert_eq!(params.intersected_aliases, aliases);
        assert!(params.join_keys.is_empty());
        assert!(params.filter_condition.is_none());

        // 创建一个非空的连接键列表
        use crate::query::parser::ast::expr::{Expr, VariableExpr};
        use crate::query::parser::ast::types::Span;
        let mock_expr = Expr::Variable(VariableExpr::new("test".to_string(), Span::default()));
        let params_with_keys = params.with_join_keys(vec![mock_expr]);
        assert!(!params_with_keys.join_keys.is_empty());
    }

    #[test]
    fn test_inner_join_strategy() {
        let strategy = InnerJoinStrategy;
        assert!(strategy.can_handle(&ConnectionType::InnerJoin));
        assert!(!strategy.can_handle(&ConnectionType::LeftJoin));

        let qctx = AstContext::from_strings("test", "test");
        let left = SubPlan::new(None, None);
        let right = SubPlan::new(None, None);
        let params = JoinParams::inner_join(vec![], HashSet::new());

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

        let qctx = AstContext::from_strings("test", "test");
        let left = SubPlan::new(None, None);
        let right = SubPlan::new(None, None);
        let params = JoinParams::cartesian();

        let result = connector.connect(&qctx, &left, &right, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_static_methods() {
        let qctx = AstContext::from_strings("test", "test");
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
