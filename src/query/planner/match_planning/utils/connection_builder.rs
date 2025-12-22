//! 连接构建器
//! 提供现代化的流式API用于构建连接操作

use crate::query::context::ast::base::AstContext;
use crate::query::parser::ast::expr::Expr;
use crate::query::planner::plan::utils::join_params::{JoinAlgorithm, JoinParams};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use std::collections::HashSet;

/// 连接构建器
/// 提供流式API用于构建各种类型的连接操作
#[derive(Debug, Clone)]
pub struct ConnectionBuilder {
    join_params: JoinParams,
}

impl ConnectionBuilder {
    /// 创建一个新的连接构建器
    pub fn new() -> Self {
        Self {
            join_params: JoinParams::cartesian(),
        }
    }

    /// 设置为内连接
    pub fn inner_join(mut self) -> Self {
        let aliases = self.join_params.intersected_aliases.clone();
        self.join_params = JoinParams::inner_join(self.join_params.join_keys.clone(), aliases);
        self
    }

    /// 设置为左连接
    pub fn left_join(mut self) -> Self {
        let aliases = self.join_params.intersected_aliases.clone();
        self.join_params = JoinParams::left_join(self.join_params.join_keys.clone(), aliases);
        self
    }

    /// 设置为右连接
    pub fn right_join(mut self) -> Self {
        let aliases = self.join_params.intersected_aliases.clone();
        self.join_params = JoinParams::right_join(self.join_params.join_keys.clone(), aliases);
        self
    }

    /// 设置为全连接
    pub fn full_join(mut self) -> Self {
        let aliases = self.join_params.intersected_aliases.clone();
        self.join_params = JoinParams::full_join(self.join_params.join_keys.clone(), aliases);
        self
    }

    /// 设置为笛卡尔积
    pub fn cartesian(mut self) -> Self {
        self.join_params = JoinParams::cartesian();
        self
    }

    /// 设置为顺序连接
    pub fn sequential(mut self, copy_col_names: bool) -> Self {
        self.join_params = JoinParams::sequential(copy_col_names);
        self
    }

    /// 设置为模式应用
    pub fn pattern_apply(mut self) -> Self {
        let aliases = self.join_params.intersected_aliases.clone();
        self.join_params = JoinParams::pattern_apply(aliases);
        self
    }

    /// 设置为RollUp应用
    pub fn roll_up_apply(mut self) -> Self {
        let aliases = self.join_params.intersected_aliases.clone();
        self.join_params = JoinParams::roll_up_apply(
            aliases,
            self.join_params
                .as_roll_up_apply()
                .map(|p| p.aggregate_exprs.clone())
                .unwrap_or_default(),
            self.join_params
                .as_roll_up_apply()
                .map(|p| p.group_keys.clone())
                .unwrap_or_default(),
        );
        self
    }

    /// 添加连接键
    pub fn with_join_keys(mut self, join_keys: Vec<Expr>) -> Self {
        self.join_params = self.join_params.with_join_keys(join_keys);
        self
    }

    /// 添加单个连接键
    pub fn add_join_key(mut self, join_key: Expr) -> Self {
        let mut keys = self.join_params.join_keys.clone();
        keys.push(join_key);
        self.join_params = self.join_params.with_join_keys(keys);
        self
    }

    /// 设置过滤条件
    pub fn with_filter_condition(mut self, filter_condition: Expr) -> Self {
        self.join_params = self.join_params.with_filter_condition(filter_condition);
        self
    }

    /// 添加交集别名
    pub fn with_intersected_aliases(mut self, aliases: HashSet<String>) -> Self {
        self.join_params = self.join_params.with_intersected_aliases(aliases);
        self
    }

    /// 添加单个交集别名
    pub fn add_intersected_alias(mut self, alias: String) -> Self {
        let mut aliases = self.join_params.intersected_aliases.clone();
        aliases.insert(alias);
        self.join_params = self.join_params.with_intersected_aliases(aliases);
        self
    }

    /// 设置连接算法（仅适用于支持的连接类型）
    pub fn with_algorithm(mut self, algorithm: JoinAlgorithm) -> Self {
        // 根据当前连接类型更新算法
        match &self.join_params.type_specific_params {
            crate::query::planner::plan::utils::join_params::TypeSpecificParams::InnerJoin(_) => {
                let aliases = self.join_params.intersected_aliases.clone();
                let keys = self.join_params.join_keys.clone();
                self.join_params = JoinParams::inner_join(keys, aliases);
                if let Some(inner_params) = self.join_params.as_inner_join_mut() {
                    inner_params.algorithm = algorithm;
                }
            }
            crate::query::planner::plan::utils::join_params::TypeSpecificParams::LeftJoin(_) => {
                let aliases = self.join_params.intersected_aliases.clone();
                let keys = self.join_params.join_keys.clone();
                self.join_params = JoinParams::left_join(keys, aliases);
                if let Some(left_params) = self.join_params.as_left_join_mut() {
                    left_params.algorithm = algorithm;
                }
            }
            _ => {} // 其他连接类型不支持算法选择
        }
        self
    }

    /// 执行连接操作
    pub fn connect(
        &self,
        qctx: &AstContext,
        left: &SubPlan,
        right: &SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        let connector = super::connection_strategy::UnifiedConnector::new();
        connector.connect(qctx, left, right, &self.join_params)
    }

    /// 获取构建的连接参数
    pub fn build(&self) -> &JoinParams {
        &self.join_params
    }

    /// 消费构建器并返回连接参数
    pub fn into_params(self) -> JoinParams {
        self.join_params
    }
}

impl Default for ConnectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 便捷函数：创建内连接
pub fn inner_join() -> ConnectionBuilder {
    ConnectionBuilder::new().inner_join()
}

/// 便捷函数：创建左连接
pub fn left_join() -> ConnectionBuilder {
    ConnectionBuilder::new().left_join()
}

/// 便捷函数：创建右连接
pub fn right_join() -> ConnectionBuilder {
    ConnectionBuilder::new().right_join()
}

/// 便捷函数：创建全连接
pub fn full_join() -> ConnectionBuilder {
    ConnectionBuilder::new().full_join()
}

/// 便捷函数：创建笛卡尔积
pub fn cartesian() -> ConnectionBuilder {
    ConnectionBuilder::new().cartesian()
}

/// 便捷函数：创建顺序连接
pub fn sequential(copy_col_names: bool) -> ConnectionBuilder {
    ConnectionBuilder::new().sequential(copy_col_names)
}

/// 便捷函数：创建模式应用
pub fn pattern_apply() -> ConnectionBuilder {
    ConnectionBuilder::new().pattern_apply()
}

/// 便捷函数：创建RollUp应用
pub fn roll_up_apply() -> ConnectionBuilder {
    ConnectionBuilder::new().roll_up_apply()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::ast::base::AstContext;
    use crate::query::parser::ast::expr::{Expr, VariableExpr};
    use crate::query::parser::ast::types::Span;
    use std::collections::HashSet;

    #[test]
    fn test_connection_builder_fluent_api() {
        let mut aliases = HashSet::new();
        aliases.insert("a".to_string());
        aliases.insert("b".to_string());

        let join_key = Expr::Variable(VariableExpr::new("key".to_string(), Span::default()));

        let builder = ConnectionBuilder::new()
            .inner_join()
            .with_join_keys(vec![join_key.clone()])
            .with_intersected_aliases(aliases.clone())
            .with_algorithm(JoinAlgorithm::Hash);

        let params = builder.build();
        assert!(!params.join_keys.is_empty());
        assert_eq!(params.intersected_aliases, aliases);
        assert!(params.as_inner_join().is_some());
        assert_eq!(
            params
                .as_inner_join()
                .expect("params should be inner_join")
                .algorithm,
            JoinAlgorithm::Hash
        );
    }

    #[test]
    fn test_convenience_functions() {
        let builder = inner_join()
            .add_join_key(Expr::Variable(VariableExpr::new(
                "test".to_string(),
                Span::default(),
            )))
            .add_intersected_alias("alias".to_string());

        let params = builder.build();
        assert!(!params.join_keys.is_empty());
        assert!(params.intersected_aliases.contains("alias"));
        assert!(params.as_inner_join().is_some());
    }

    #[test]
    fn test_sequential_connection() {
        let builder = sequential(true);
        let params = builder.build();
        assert!(params.as_sequential().is_some());
        assert!(
            params
                .as_sequential()
                .expect("params should be sequential")
                .copy_col_names
        );
    }
}
