//! 连接参数定义
//! 定义用于连接操作的各种参数结构

use crate::query::parser::ast::expr::Expr;
use std::collections::HashSet;

/// 连接参数
/// 存储连接操作所需的所有参数，包括连接键、过滤条件等
#[derive(Debug, Clone)]
pub struct JoinParams {
    /// 连接键列表
    pub join_keys: Vec<Expr>,
    /// 交集别名集合
    pub intersected_aliases: HashSet<String>,
    /// 过滤条件
    pub filter_condition: Option<Expr>,
    /// 连接类型特定的参数
    pub type_specific_params: TypeSpecificParams,
}

/// 连接类型特定的参数
#[derive(Debug, Clone)]
pub enum TypeSpecificParams {
    /// 内连接参数
    InnerJoin(InnerJoinParams),
    /// 左连接参数
    LeftJoin(LeftJoinParams),
    /// 右连接参数
    RightJoin(RightJoinParams),
    /// 全连接参数
    FullJoin(FullJoinParams),
    /// 笛卡尔积参数
    Cartesian(CartesianParams),
    /// RollUp应用参数
    RollUpApply(RollUpApplyParams),
    /// 模式应用参数
    PatternApply(PatternApplyParams),
    /// 顺序连接参数
    Sequential(SequentialParams),
}

/// 内连接参数
#[derive(Debug, Clone)]
pub struct InnerJoinParams {
    /// 是否使用哈希连接
    pub use_hash: bool,
    /// 连接算法选择
    pub algorithm: JoinAlgorithm,
}

/// 左连接参数
#[derive(Debug, Clone)]
pub struct LeftJoinParams {
    /// 是否使用哈希连接
    pub use_hash: bool,
    /// 连接算法选择
    pub algorithm: JoinAlgorithm,
    /// 是否保留左侧所有行
    pub preserve_left_all: bool,
}

/// 右连接参数
#[derive(Debug, Clone)]
pub struct RightJoinParams {
    /// 是否使用哈希连接
    pub use_hash: bool,
    /// 连接算法选择
    pub algorithm: JoinAlgorithm,
    /// 是否保留右侧所有行
    pub preserve_right_all: bool,
}

/// 全连接参数
#[derive(Debug, Clone)]
pub struct FullJoinParams {
    /// 连接算法选择
    pub algorithm: JoinAlgorithm,
}

/// 笛卡尔积参数
#[derive(Debug, Clone)]
pub struct CartesianParams {
    /// 是否需要优化
    pub optimize: bool,
}

/// RollUp应用参数
#[derive(Debug, Clone)]
pub struct RollUpApplyParams {
    /// 聚合表达式列表
    pub aggregate_exprs: Vec<Expr>,
    /// 分组键
    pub group_keys: Vec<Expr>,
}

/// 模式应用参数
#[derive(Debug, Clone)]
pub struct PatternApplyParams {
    /// 模式匹配条件
    pub pattern_condition: Option<Expr>,
    /// 是否需要短路评估
    pub short_circuit: bool,
}

/// 顺序连接参数
#[derive(Debug, Clone)]
pub struct SequentialParams {
    /// 是否复制列名
    pub copy_col_names: bool,
    /// 输入变量映射
    pub input_var_mapping: Vec<(String, String)>,
}

/// 连接算法枚举
#[derive(Debug, Clone, PartialEq)]
pub enum JoinAlgorithm {
    /// 嵌套循环连接
    NestedLoop,
    /// 哈希连接
    Hash,
    /// 排序合并连接
    SortMerge,
    /// 索引嵌套循环连接
    IndexNestedLoop,
}

impl JoinParams {
    /// 创建内连接参数
    pub fn inner_join(join_keys: Vec<Expr>, intersected_aliases: HashSet<String>) -> Self {
        Self {
            join_keys,
            intersected_aliases,
            filter_condition: None,
            type_specific_params: TypeSpecificParams::InnerJoin(InnerJoinParams {
                use_hash: true,
                algorithm: JoinAlgorithm::Hash,
            }),
        }
    }

    /// 创建左连接参数
    pub fn left_join(join_keys: Vec<Expr>, intersected_aliases: HashSet<String>) -> Self {
        Self {
            join_keys,
            intersected_aliases,
            filter_condition: None,
            type_specific_params: TypeSpecificParams::LeftJoin(LeftJoinParams {
                use_hash: true,
                algorithm: JoinAlgorithm::Hash,
                preserve_left_all: true,
            }),
        }
    }

    /// 创建右连接参数
    pub fn right_join(join_keys: Vec<Expr>, intersected_aliases: HashSet<String>) -> Self {
        Self {
            join_keys,
            intersected_aliases,
            filter_condition: None,
            type_specific_params: TypeSpecificParams::RightJoin(RightJoinParams {
                use_hash: true,
                algorithm: JoinAlgorithm::Hash,
                preserve_right_all: true,
            }),
        }
    }

    /// 创建全连接参数
    pub fn full_join(join_keys: Vec<Expr>, intersected_aliases: HashSet<String>) -> Self {
        Self {
            join_keys,
            intersected_aliases,
            filter_condition: None,
            type_specific_params: TypeSpecificParams::FullJoin(FullJoinParams {
                algorithm: JoinAlgorithm::Hash,
            }),
        }
    }

    /// 创建笛卡尔积参数
    pub fn cartesian() -> Self {
        Self {
            join_keys: Vec::new(),
            intersected_aliases: HashSet::new(),
            filter_condition: None,
            type_specific_params: TypeSpecificParams::Cartesian(CartesianParams { optimize: true }),
        }
    }

    /// 创建RollUp应用参数
    pub fn roll_up_apply(
        intersected_aliases: HashSet<String>,
        aggregate_exprs: Vec<Expr>,
        group_keys: Vec<Expr>,
    ) -> Self {
        Self {
            join_keys: Vec::new(),
            intersected_aliases,
            filter_condition: None,
            type_specific_params: TypeSpecificParams::RollUpApply(RollUpApplyParams {
                aggregate_exprs,
                group_keys,
            }),
        }
    }

    /// 创建模式应用参数
    pub fn pattern_apply(intersected_aliases: HashSet<String>) -> Self {
        Self {
            join_keys: Vec::new(),
            intersected_aliases,
            filter_condition: None,
            type_specific_params: TypeSpecificParams::PatternApply(PatternApplyParams {
                pattern_condition: None,
                short_circuit: true,
            }),
        }
    }

    /// 创建顺序连接参数
    pub fn sequential(copy_col_names: bool) -> Self {
        Self {
            join_keys: Vec::new(),
            intersected_aliases: HashSet::new(),
            filter_condition: None,
            type_specific_params: TypeSpecificParams::Sequential(SequentialParams {
                copy_col_names,
                input_var_mapping: Vec::new(),
            }),
        }
    }

    /// 设置连接键
    pub fn with_join_keys(mut self, join_keys: Vec<Expr>) -> Self {
        self.join_keys = join_keys;
        self
    }

    /// 设置过滤条件
    pub fn with_filter_condition(mut self, filter_condition: Expr) -> Self {
        self.filter_condition = Some(filter_condition);
        self
    }

    /// 设置交集别名
    pub fn with_intersected_aliases(mut self, intersected_aliases: HashSet<String>) -> Self {
        self.intersected_aliases = intersected_aliases;
        self
    }

    /// 获取内连接参数
    pub fn as_inner_join(&self) -> Option<&InnerJoinParams> {
        match &self.type_specific_params {
            TypeSpecificParams::InnerJoin(params) => Some(params),
            _ => None,
        }
    }

    /// 获取左连接参数
    pub fn as_left_join(&self) -> Option<&LeftJoinParams> {
        match &self.type_specific_params {
            TypeSpecificParams::LeftJoin(params) => Some(params),
            _ => None,
        }
    }

    /// 获取右连接参数
    pub fn as_right_join(&self) -> Option<&RightJoinParams> {
        match &self.type_specific_params {
            TypeSpecificParams::RightJoin(params) => Some(params),
            _ => None,
        }
    }

    /// 获取全连接参数
    pub fn as_full_join(&self) -> Option<&FullJoinParams> {
        match &self.type_specific_params {
            TypeSpecificParams::FullJoin(params) => Some(params),
            _ => None,
        }
    }

    /// 获取笛卡尔积参数
    pub fn as_cartesian(&self) -> Option<&CartesianParams> {
        match &self.type_specific_params {
            TypeSpecificParams::Cartesian(params) => Some(params),
            _ => None,
        }
    }

    /// 获取RollUp应用参数
    pub fn as_roll_up_apply(&self) -> Option<&RollUpApplyParams> {
        match &self.type_specific_params {
            TypeSpecificParams::RollUpApply(params) => Some(params),
            _ => None,
        }
    }

    /// 获取模式应用参数
    pub fn as_pattern_apply(&self) -> Option<&PatternApplyParams> {
        match &self.type_specific_params {
            TypeSpecificParams::PatternApply(params) => Some(params),
            _ => None,
        }
    }

    /// 获取顺序连接参数
    pub fn as_sequential(&self) -> Option<&SequentialParams> {
        match &self.type_specific_params {
            TypeSpecificParams::Sequential(params) => Some(params),
            _ => None,
        }
    }

    /// 获取内连接参数的可变引用
    pub fn as_inner_join_mut(&mut self) -> Option<&mut InnerJoinParams> {
        match &mut self.type_specific_params {
            TypeSpecificParams::InnerJoin(params) => Some(params),
            _ => None,
        }
    }

    /// 获取左连接参数的可变引用
    pub fn as_left_join_mut(&mut self) -> Option<&mut LeftJoinParams> {
        match &mut self.type_specific_params {
            TypeSpecificParams::LeftJoin(params) => Some(params),
            _ => None,
        }
    }

    /// 获取右连接参数的可变引用
    pub fn as_right_join_mut(&mut self) -> Option<&mut RightJoinParams> {
        match &mut self.type_specific_params {
            TypeSpecificParams::RightJoin(params) => Some(params),
            _ => None,
        }
    }

    /// 获取全连接参数的可变引用
    pub fn as_full_join_mut(&mut self) -> Option<&mut FullJoinParams> {
        match &mut self.type_specific_params {
            TypeSpecificParams::FullJoin(params) => Some(params),
            _ => None,
        }
    }

    /// 获取笛卡尔积参数的可变引用
    pub fn as_cartesian_mut(&mut self) -> Option<&mut CartesianParams> {
        match &mut self.type_specific_params {
            TypeSpecificParams::Cartesian(params) => Some(params),
            _ => None,
        }
    }

    /// 获取RollUp应用参数的可变引用
    pub fn as_roll_up_apply_mut(&mut self) -> Option<&mut RollUpApplyParams> {
        match &mut self.type_specific_params {
            TypeSpecificParams::RollUpApply(params) => Some(params),
            _ => None,
        }
    }

    /// 获取模式应用参数的可变引用
    pub fn as_pattern_apply_mut(&mut self) -> Option<&mut PatternApplyParams> {
        match &mut self.type_specific_params {
            TypeSpecificParams::PatternApply(params) => Some(params),
            _ => None,
        }
    }

    /// 获取顺序连接参数的可变引用
    pub fn as_sequential_mut(&mut self) -> Option<&mut SequentialParams> {
        match &mut self.type_specific_params {
            TypeSpecificParams::Sequential(params) => Some(params),
            _ => None,
        }
    }

    /// 获取左输入变量名
    pub fn left_input_var(&self) -> &str {
        // 从连接键中推断左输入变量名
        // 这是一个简化的实现，实际可能需要更复杂的逻辑
        if let Some(expr) = self.join_keys.first() {
            match expr {
                Expr::Variable(var_expr) => &var_expr.name,
                _ => "left",
            }
        } else {
            "left"
        }
    }

    /// 获取右输入变量名
    pub fn right_input_var(&self) -> &str {
        // 从交集别名中推断右输入变量名
        // 这是一个简化的实现，实际可能需要更复杂的逻辑
        if let Some(alias) = self.intersected_aliases.iter().next() {
            alias
        } else {
            "right"
        }
    }

    /// 获取左键列表
    pub fn left_keys(&self) -> Vec<String> {
        // 将 Expr 转换为字符串
        self.join_keys
            .iter()
            .map(|expr| match expr {
                Expr::Variable(var_expr) => var_expr.name.clone(),
                _ => format!("{:?}", expr),
            })
            .collect()
    }

    /// 获取右键列表
    pub fn right_keys(&self) -> Vec<String> {
        // 使用交集别名作为右键
        self.intersected_aliases.iter().cloned().collect()
    }

    /// 获取输出列列表
    pub fn output_columns(&self) -> Vec<String> {
        // 根据连接类型生成输出列
        match &self.type_specific_params {
            TypeSpecificParams::InnerJoin(_) => {
                let mut cols = self.left_keys();
                cols.extend(self.right_keys());
                cols
            }
            TypeSpecificParams::LeftJoin(_) => {
                let mut cols = self.left_keys();
                cols.extend(self.right_keys());
                cols
            }
            TypeSpecificParams::Cartesian(_) => {
                vec!["id".to_string(), "name".to_string()]
            }
            _ => {
                vec![]
            }
        }
    }

    /// 获取输入变量列表（用于笛卡尔积）
    pub fn input_vars(&self) -> Vec<String> {
        match &self.type_specific_params {
            TypeSpecificParams::Cartesian(_) => {
                vec!["left".to_string(), "right".to_string()]
            }
            _ => {
                vec![
                    self.left_input_var().to_string(),
                    self.right_input_var().to_string(),
                ]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inner_join_params() {
        let join_keys = vec![];
        let mut aliases = HashSet::new();
        aliases.insert("test".to_string());

        let params = JoinParams::inner_join(join_keys.clone(), aliases.clone());
        assert_eq!(params.join_keys, join_keys);
        assert_eq!(params.intersected_aliases, aliases);
        assert!(params.filter_condition.is_none());
        assert!(params.as_inner_join().is_some());
        assert!(params.as_left_join().is_none());
    }

    #[test]
    fn test_with_methods() {
        use crate::query::parser::ast::expr::{Expr, VariableExpr};
        use crate::query::parser::ast::types::Span;

        let mock_expr = Expr::Variable(VariableExpr::new("test".to_string(), Span::default()));
        let params = JoinParams::cartesian()
            .with_join_keys(vec![mock_expr])
            .with_intersected_aliases({
                let mut set = HashSet::new();
                set.insert("test".to_string());
                set
            });

        assert!(!params.join_keys.is_empty());
        assert!(!params.intersected_aliases.is_empty());
    }
}
