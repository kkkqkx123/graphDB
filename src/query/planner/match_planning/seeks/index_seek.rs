/// 索引查找规划器
/// 根据标签索引和属性索引进行查找
/// 负责规划基于索引的查找操作，包括标签索引、属性索引和可变属性索引
use crate::core::Expression;
use crate::query::parser::ast::expr::Expr;
use crate::query::planner::match_planning::seeks::seek_strategy::SeekStrategy;

use crate::query::planner::plan::{PlanNodeFactory, SubPlan};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::path_structs::NodeInfo;

/// 索引查找元数据
/// 存储IndexScan节点执行所需的索引相关信息
#[derive(Debug, Clone)]
pub struct IndexScanMetadata {
    /// 标签ID列表
    pub label_ids: Vec<i32>,
    /// 标签名称列表
    pub label_names: Vec<String>,
    /// 索引ID列表
    pub index_ids: Vec<i32>,
    /// 是否有属性过滤
    pub has_property_filter: bool,
    /// 属性过滤表达式（如果存在）
    pub property_filter: Option<Expression>,
}

impl IndexScanMetadata {
    /// 创建新的IndexScan元数据
    pub fn new(label_ids: Vec<i32>, label_names: Vec<String>, index_ids: Vec<i32>) -> Self {
        Self {
            label_ids,
            label_names,
            index_ids,
            has_property_filter: false,
            property_filter: None,
        }
    }

    /// 设置属性过滤表达式
    pub fn set_property_filter(&mut self, filter: Expression) {
        self.has_property_filter = true;
        self.property_filter = Some(filter);
    }
}

/// 索引查找类型
#[derive(Debug, Clone)]
pub enum IndexSeekType {
    /// 标签索引查找
    Label,
    /// 属性索引查找
    Property(Vec<Expression>),
    /// 可变属性索引查找
    VariableProperty(Vec<Expression>),
}

/// 索引查找规划器
/// 负责规划基于索引的查找操作，支持标签索引、属性索引和可变属性索引
#[derive(Debug)]
pub struct IndexSeek {
    node_info: NodeInfo,
    seek_type: IndexSeekType,
}

impl IndexSeek {
    /// 创建标签索引查找规划器
    pub fn new_label(node_info: NodeInfo) -> Self {
        Self {
            node_info,
            seek_type: IndexSeekType::Label,
        }
    }

    /// 创建属性索引查找规划器
    pub fn new_property(node_info: NodeInfo, prop_exprs: Vec<Expression>) -> Self {
        Self {
            node_info,
            seek_type: IndexSeekType::Property(prop_exprs),
        }
    }

    /// 创建可变属性索引查找规划器
    pub fn new_variable_property(node_info: NodeInfo, prop_exprs: Vec<Expression>) -> Self {
        Self {
            node_info,
            seek_type: IndexSeekType::VariableProperty(prop_exprs),
        }
    }

    /// 构建索引查找计划
    pub fn build_plan(&self) -> Result<SubPlan, PlannerError> {
        // 验证基本条件
        self.validate_conditions()?;

        let space_id = 1; // TODO: 应该从上下文获取space_id
        let (label_id, _label_name) = self.get_primary_label_info()?;

        // 创建实际的索引扫描节点
        let index_scan_node = match &self.seek_type {
            IndexSeekType::Label => {
                // 标签索引查找
                PlanNodeFactory::create_index_scan(
                    space_id, label_id, label_id, // 使用标签ID作为索引ID
                    "RANGE",
                )?
            }
            IndexSeekType::Property(prop_exprs) => {
                // 属性索引查找
                let _filter_expr = self.create_property_filter_expression(prop_exprs)?;
                PlanNodeFactory::create_index_scan(
                    space_id, label_id, label_id, // 使用标签ID作为索引ID
                    "RANGE",
                )?
            }
            IndexSeekType::VariableProperty(prop_exprs) => {
                // 可变属性索引查找
                let _filter_expr = self.create_variable_property_filter_expression(prop_exprs)?;
                PlanNodeFactory::create_index_scan(
                    space_id, label_id, label_id, // 使用标签ID作为索引ID
                    "VARIABLE",
                )?
            }
        };

        // 处理节点属性过滤
        let root = if let Some(props) = &self.node_info.props {
            // 将 Expression 转换为 Expr
            let expr = self.convert_expression_to_expr(props.clone())?;
            PlanNodeFactory::create_filter(index_scan_node.clone(), expr)?
        } else {
            index_scan_node.clone()
        };

        // 处理额外的过滤条件
        let final_root = if let Some(filter) = &self.node_info.filter {
            // 将 Expression 转换为 Expr
            let expr = self.convert_expression_to_expr(filter.clone())?;
            PlanNodeFactory::create_filter(root, expr)?
        } else {
            root
        };

        Ok(SubPlan::new(Some(final_root), Some(index_scan_node)))
    }

    /// 验证查找条件
    fn validate_conditions(&self) -> Result<(), PlannerError> {
        match &self.seek_type {
            IndexSeekType::Label => {
                if self.node_info.labels.is_empty() {
                    return Err(PlannerError::InvalidAstContext(
                        "节点必须有标签才能使用标签索引查找".to_string(),
                    ));
                }
                if self.node_info.tids.is_empty() {
                    return Err(PlannerError::InvalidAstContext(
                        "节点标签ID列表不能为空".to_string(),
                    ));
                }
            }
            IndexSeekType::Property(prop_exprs) => {
                if prop_exprs.is_empty() {
                    return Err(PlannerError::UnsupportedOperation(
                        "No property expressions for index seek".to_string(),
                    ));
                }
            }
            IndexSeekType::VariableProperty(prop_exprs) => {
                if prop_exprs.is_empty() {
                    return Err(PlannerError::UnsupportedOperation(
                        "No property expressions for variable index seek".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// 获取主要标签信息
    fn get_primary_label_info(&self) -> Result<(i32, String), PlannerError> {
        match &self.seek_type {
            IndexSeekType::Label => {
                if self.node_info.labels.is_empty() || self.node_info.tids.is_empty() {
                    return Err(PlannerError::InvalidAstContext(
                        "无效的节点信息：标签或标签ID为空".to_string(),
                    ));
                }
                let label_id = self.node_info.tids[0];
                let label_name = self.node_info.labels[0].clone();
                Ok((label_id, label_name))
            }
            IndexSeekType::Property(_) | IndexSeekType::VariableProperty(_) => {
                // 对于属性索引查找，如果有标签信息则使用，否则使用默认值
                if !self.node_info.labels.is_empty() && !self.node_info.tids.is_empty() {
                    let label_id = self.node_info.tids[0];
                    let label_name = self.node_info.labels[0].clone();
                    Ok((label_id, label_name))
                } else {
                    // 使用默认标签信息
                    Ok((0, "default".to_string()))
                }
            }
        }
    }

    /// 检查是否可以使用索引查找
    pub fn match_node(&self) -> bool {
        match &self.seek_type {
            IndexSeekType::Label => {
                !self.node_info.labels.is_empty() && !self.node_info.tids.is_empty()
            }
            IndexSeekType::Property(prop_exprs) => !prop_exprs.is_empty(),
            IndexSeekType::VariableProperty(prop_exprs) => {
                !prop_exprs.is_empty()
                    && prop_exprs
                        .iter()
                        .any(|expr| matches!(expr, Expression::Label(_) | Expression::Variable(_)))
            }
        }
    }

    /// 获取索引扫描元数据
    pub fn get_index_metadata(&self) -> Result<IndexScanMetadata, PlannerError> {
        let (label_id, label_name) = self.get_primary_label_info()?;
        let index_id = label_id;

        let mut metadata = IndexScanMetadata::new(vec![label_id], vec![label_name], vec![index_id]);

        // 如果有属性过滤表达式，记录在元数据中
        if let Some(props) = &self.node_info.props {
            metadata.set_property_filter(props.clone());
        }

        Ok(metadata)
    }

    /// 获取查找类型
    pub fn seek_type(&self) -> &IndexSeekType {
        &self.seek_type
    }

    /// 获取节点信息
    pub fn node_info(&self) -> &NodeInfo {
        &self.node_info
    }

    /// 创建属性过滤表达式
    fn create_property_filter_expression(
        &self,
        prop_exprs: &[Expression],
    ) -> Result<Expression, PlannerError> {
        if prop_exprs.is_empty() {
            return Err(PlannerError::InvalidAstContext(
                "属性表达式列表不能为空".to_string(),
            ));
        }

        // 将多个属性表达式组合为AND条件
        let mut filter_expr = prop_exprs[0].clone();
        for expr in &prop_exprs[1..] {
            filter_expr = Expression::Binary {
                left: Box::new(filter_expr),
                op: crate::core::BinaryOperator::And,
                right: Box::new(expr.clone()),
            };
        }

        Ok(filter_expr)
    }

    /// 创建可变属性索引过滤表达式
    fn create_variable_property_filter_expression(
        &self,
        prop_exprs: &[Expression],
    ) -> Result<Expression, PlannerError> {
        // 验证至少有一个有效的变量表达式
        let valid_exprs: Vec<_> = prop_exprs
            .iter()
            .filter(|expr| matches!(expr, Expression::Variable(_) | Expression::Label(_)))
            .collect();

        if valid_exprs.is_empty() {
            return Err(PlannerError::InvalidAstContext(
                "没有有效的变量表达式".to_string(),
            ));
        }

        // 创建参数化查询表达式
        // 对于可变属性索引，我们需要创建一个参数表达式
        let param_expr = Expression::Variable("__index_param".to_string());

        // 如果有多个表达式，创建AND条件
        let mut filter_expr = param_expr;
        for expr in &valid_exprs[1..] {
            filter_expr = Expression::Binary {
                left: Box::new(filter_expr),
                op: crate::core::BinaryOperator::And,
                right: Box::new((*expr).clone()),
            };
        }

        Ok(filter_expr)
    }

    /// 将 Expression 转换为 Expr
    fn convert_expression_to_expr(&self, expr: Expression) -> Result<Expr, PlannerError> {
        // 这里需要实现从 Expression 到 Expr 的转换
        // 由于这是一个复杂的转换，我们暂时使用一个简单的实现
        // 在实际项目中，需要实现完整的转换逻辑

        // 对于简单的情况，我们可以创建一个变量表达式
        match expr {
            Expression::Variable(name) => Ok(Expr::Variable(
                crate::query::parser::ast::expr::VariableExpr::new(
                    name,
                    crate::query::parser::ast::Span::default(),
                ),
            )),
            Expression::Label(name) => Ok(Expr::Variable(
                crate::query::parser::ast::expr::VariableExpr::new(
                    name,
                    crate::query::parser::ast::Span::default(),
                ),
            )),
            _ => {
                // 对于复杂的表达式，暂时返回错误
                Err(PlannerError::InvalidAstContext(
                    "复杂的表达式转换尚未实现".to_string(),
                ))
            }
        }
    }

    /// 验证属性表达式
    pub fn validate_property_expressions(
        &self,
        expressions: &[Expression],
    ) -> Result<(), PlannerError> {
        if expressions.is_empty() {
            return Err(PlannerError::InvalidAstContext(
                "属性表达式列表不能为空".to_string(),
            ));
        }

        // 验证每个表达式
        for expr in expressions {
            match expr {
                Expression::Binary { op, .. } => {
                    // 只允许比较操作符
                    match op {
                        crate::core::BinaryOperator::Equal
                        | crate::core::BinaryOperator::NotEqual
                        | crate::core::BinaryOperator::LessThan
                        | crate::core::BinaryOperator::LessThanOrEqual
                        | crate::core::BinaryOperator::GreaterThan
                        | crate::core::BinaryOperator::GreaterThanOrEqual => {
                            // 这些是有效的比较操作符
                        }
                        _ => {
                            return Err(PlannerError::InvalidAstContext(format!(
                                "不支持的操作符 {:?} 用于属性索引查找",
                                op
                            )));
                        }
                    }
                }
                Expression::Variable(_) | Expression::Label(_) => {
                    // 变量和标签表达式是有效的
                }
                Expression::Literal(_) => {
                    // 字面量是有效的
                }
                _ => {
                    return Err(PlannerError::InvalidAstContext(format!(
                        "不支持的表达式类型 {:?} 用于属性索引查找",
                        expr
                    )));
                }
            }
        }

        Ok(())
    }
}

impl SeekStrategy for IndexSeek {
    fn build_plan(&self) -> Result<SubPlan, PlannerError> {
        self.build_plan()
    }

    fn match_node(&self) -> bool {
        self.match_node()
    }

    fn name(&self) -> &'static str {
        match &self.seek_type {
            IndexSeekType::Label => "LabelIndexSeek",
            IndexSeekType::Property(_) => "PropIndexSeek",
            IndexSeekType::VariableProperty(_) => "VariablePropIndexSeek",
        }
    }

    fn estimate_cost(&self) -> f64 {
        match &self.seek_type {
            IndexSeekType::Label => 50.0,
            IndexSeekType::Property(prop_exprs) => {
                // 属性索引查找的成本与属性数量成正比
                prop_exprs.len() as f64 * 10.0
            }
            IndexSeekType::VariableProperty(prop_exprs) => {
                // 可变属性索引查找的成本更高
                prop_exprs.len() as f64 * 20.0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;

    fn create_test_node_info(labels: Vec<&str>, tids: Vec<i32>) -> NodeInfo {
        NodeInfo {
            alias: "n".to_string(),
            labels: labels.into_iter().map(|s| s.to_string()).collect(),
            props: None,
            anonymous: false,
            filter: None,
            tids,
            label_props: vec![None],
        }
    }

    #[test]
    fn test_index_seek_new_label() {
        let node_info = create_test_node_info(vec!["Person"], vec![1]);
        let seeker = IndexSeek::new_label(node_info);

        match seeker.seek_type() {
            IndexSeekType::Label => {
                // 预期的类型
            }
            _ => panic!("Expected Label seek type"),
        }
    }

    #[test]
    fn test_index_seek_new_property() {
        let node_info = create_test_node_info(vec![], vec![]);
        let prop_exprs = vec![Expression::Variable("x".to_string())];
        let seeker = IndexSeek::new_property(node_info, prop_exprs.clone());

        match seeker.seek_type() {
            IndexSeekType::Property(stored_exprs) => {
                assert_eq!(stored_exprs, &prop_exprs);
            }
            _ => panic!("Expected Property seek type"),
        }
    }

    #[test]
    fn test_index_seek_new_variable_property() {
        let node_info = create_test_node_info(vec![], vec![]);
        let prop_exprs = vec![Expression::Variable("x".to_string())];
        let seeker = IndexSeek::new_variable_property(node_info, prop_exprs.clone());

        match seeker.seek_type() {
            IndexSeekType::VariableProperty(stored_exprs) => {
                assert_eq!(stored_exprs, &prop_exprs);
            }
            _ => panic!("Expected VariableProperty seek type"),
        }
    }

    #[test]
    fn test_match_node_label() {
        let node_info = create_test_node_info(vec!["Person"], vec![1]);
        let seeker = IndexSeek::new_label(node_info);
        assert!(seeker.match_node());

        let empty_node_info = create_test_node_info(vec![], vec![1]);
        let empty_seeker = IndexSeek::new_label(empty_node_info);
        assert!(!empty_seeker.match_node());
    }

    #[test]
    fn test_match_node_property() {
        let node_info = create_test_node_info(vec![], vec![]);
        let prop_exprs = vec![Expression::Variable("x".to_string())];
        let seeker = IndexSeek::new_property(node_info.clone(), prop_exprs);
        assert!(seeker.match_node());

        let empty_seeker = IndexSeek::new_property(node_info, vec![]);
        assert!(!empty_seeker.match_node());
    }

    #[test]
    fn test_match_node_variable_property() {
        let node_info = create_test_node_info(vec![], vec![]);
        let valid_exprs = vec![Expression::Variable("x".to_string())];
        let seeker = IndexSeek::new_variable_property(node_info.clone(), valid_exprs);
        assert!(seeker.match_node());

        let invalid_exprs = vec![Expression::Literal(crate::core::Value::String(
            "test".to_string(),
        ))];
        let invalid_seeker = IndexSeek::new_variable_property(node_info, invalid_exprs);
        assert!(!invalid_seeker.match_node());
    }

    #[test]
    fn test_seek_strategy_name() {
        let node_info = create_test_node_info(vec!["Person"], vec![1]);
        let label_seeker = IndexSeek::new_label(node_info);
        assert_eq!(label_seeker.name(), "LabelIndexSeek");

        let node_info2 = create_test_node_info(vec![], vec![]);
        let prop_seeker = IndexSeek::new_property(
            node_info2.clone(),
            vec![Expression::Variable("x".to_string())],
        );
        assert_eq!(prop_seeker.name(), "PropIndexSeek");

        let var_prop_seeker = IndexSeek::new_variable_property(
            node_info2,
            vec![Expression::Variable("x".to_string())],
        );
        assert_eq!(var_prop_seeker.name(), "VariablePropIndexSeek");
    }

    #[test]
    fn test_estimate_cost() {
        let node_info = create_test_node_info(vec!["Person"], vec![1]);
        let label_seeker = IndexSeek::new_label(node_info);
        assert_eq!(label_seeker.estimate_cost(), 50.0);

        let node_info2 = create_test_node_info(vec![], vec![]);
        let single_prop_seeker = IndexSeek::new_property(
            node_info2.clone(),
            vec![Expression::Variable("x".to_string())],
        );
        assert_eq!(single_prop_seeker.estimate_cost(), 10.0);

        let multi_prop_seeker = IndexSeek::new_property(
            node_info2.clone(),
            vec![
                Expression::Variable("x".to_string()),
                Expression::Variable("y".to_string()),
            ],
        );
        assert_eq!(multi_prop_seeker.estimate_cost(), 20.0);

        let var_prop_seeker = IndexSeek::new_variable_property(
            node_info2,
            vec![Expression::Variable("x".to_string())],
        );
        assert_eq!(var_prop_seeker.estimate_cost(), 20.0);
    }

    #[test]
    fn test_build_plan_label() {
        let node_info = create_test_node_info(vec!["Person"], vec![1]);
        let seeker = IndexSeek::new_label(node_info);
        let result = seeker.build_plan();
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_plan_property() {
        let node_info = create_test_node_info(vec![], vec![]);
        let seeker =
            IndexSeek::new_property(node_info, vec![Expression::Variable("x".to_string())]);
        let result = seeker.build_plan();
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_plan_variable_property() {
        let node_info = create_test_node_info(vec![], vec![]);
        let seeker = IndexSeek::new_variable_property(
            node_info,
            vec![Expression::Variable("x".to_string())],
        );
        let result = seeker.build_plan();
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_plan_label_without_labels() {
        let node_info = create_test_node_info(vec![], vec![1]);
        let seeker = IndexSeek::new_label(node_info);
        let result = seeker.build_plan();
        assert!(result.is_err());
    }

    #[test]
    fn test_build_plan_property_without_exprs() {
        let node_info = create_test_node_info(vec![], vec![]);
        let seeker = IndexSeek::new_property(node_info, vec![]);
        let result = seeker.build_plan();
        assert!(result.is_err());
    }

    #[test]
    fn test_get_index_metadata() {
        let node_info = create_test_node_info(vec!["Person"], vec![1]);
        let seeker = IndexSeek::new_label(node_info);
        let result = seeker.get_index_metadata();
        assert!(result.is_ok());

        let metadata = result.expect("Failed to get index metadata");
        assert_eq!(metadata.label_ids, vec![1]);
        assert_eq!(metadata.label_names, vec!["Person".to_string()]);
        assert_eq!(metadata.index_ids, vec![1]);
    }

    #[test]
    fn test_create_property_filter_expression() {
        let node_info = create_test_node_info(vec![], vec![]);
        let seeker = IndexSeek::new_property(node_info, vec![]);

        // 空表达式列表应该返回错误
        let result = seeker.create_property_filter_expression(&[]);
        assert!(result.is_err());

        // 单个表达式
        let node_info = create_test_node_info(vec![], vec![]);
        let seeker =
            IndexSeek::new_property(node_info, vec![Expression::Variable("x".to_string())]);
        let result =
            seeker.create_property_filter_expression(&[Expression::Variable("x".to_string())]);
        assert!(result.is_ok());

        // 多个表达式
        let node_info = create_test_node_info(vec![], vec![]);
        let seeker = IndexSeek::new_property(
            node_info,
            vec![
                Expression::Variable("x".to_string()),
                Expression::Variable("y".to_string()),
            ],
        );
        let result = seeker.create_property_filter_expression(&[
            Expression::Variable("x".to_string()),
            Expression::Variable("y".to_string()),
        ]);
        assert!(result.is_ok());

        // 验证结果是AND表达式
        let expr = result.expect("Failed to create property filter expression");
        match expr {
            Expression::Binary { op, .. } => {
                assert_eq!(op, crate::core::BinaryOperator::And);
            }
            _ => panic!("Expected Binary expression with AND operator"),
        }
    }

    #[test]
    fn test_create_variable_property_filter_expression() {
        let node_info = create_test_node_info(vec![], vec![]);
        let seeker = IndexSeek::new_variable_property(node_info, vec![]);

        // 空表达式列表应该返回错误
        let result = seeker.create_variable_property_filter_expression(&[]);
        assert!(result.is_err());

        // 无效表达式列表
        let invalid_exprs = vec![Expression::Literal(crate::core::Value::String(
            "test".to_string(),
        ))];
        let result = seeker.create_variable_property_filter_expression(&invalid_exprs);
        assert!(result.is_err());

        // 有效的变量表达式
        let valid_exprs = vec![Expression::Variable("x".to_string())];
        let result = seeker.create_variable_property_filter_expression(&valid_exprs);
        assert!(result.is_ok());

        // 验证结果是参数表达式
        let expr = result.expect("Failed to create variable property filter expression");
        match expr {
            Expression::Variable(name) => {
                assert_eq!(name, "__index_param");
            }
            _ => panic!("Expected Variable expression"),
        }
    }

    #[test]
    fn test_validate_property_expressions() {
        let node_info = create_test_node_info(vec![], vec![]);
        let seeker = IndexSeek::new_property(node_info, vec![]);

        // 空表达式列表
        let result = seeker.validate_property_expressions(&[]);
        assert!(result.is_err());

        // 有效的表达式
        let valid_exprs = vec![Expression::Binary {
            left: Box::new(Expression::Variable("x".to_string())),
            op: crate::core::BinaryOperator::Equal,
            right: Box::new(Expression::Literal(crate::core::Value::String(
                "test".to_string(),
            ))),
        }];
        let result = seeker.validate_property_expressions(&valid_exprs);
        assert!(result.is_ok());

        // 无效的二元操作符
        let invalid_exprs = vec![Expression::Binary {
            left: Box::new(Expression::Variable("x".to_string())),
            op: crate::core::BinaryOperator::Add,
            right: Box::new(Expression::Literal(crate::core::Value::Int(1))),
        }];
        let result = seeker.validate_property_expressions(&invalid_exprs);
        assert!(result.is_err());

        // 不支持的表达式类型
        let unsupported_exprs = vec![Expression::List(vec![])];
        let result = seeker.validate_property_expressions(&unsupported_exprs);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_plan_with_actual_nodes() {
        // 测试标签索引查找
        let node_info = create_test_node_info(vec!["Person"], vec![1]);
        let seeker = IndexSeek::new_label(node_info);
        let result = seeker.build_plan();
        assert!(result.is_ok());

        // 测试属性索引查找
        let node_info = create_test_node_info(vec!["Person"], vec![1]);
        let prop_exprs = vec![Expression::Binary {
            left: Box::new(Expression::Variable("x".to_string())),
            op: crate::core::BinaryOperator::Equal,
            right: Box::new(Expression::Literal(crate::core::Value::String(
                "test".to_string(),
            ))),
        }];
        let seeker = IndexSeek::new_property(node_info, prop_exprs);
        let result = seeker.build_plan();
        assert!(result.is_ok());

        // 测试可变属性索引查找
        let node_info = create_test_node_info(vec!["Person"], vec![1]);
        let var_prop_exprs = vec![Expression::Variable("param".to_string())];
        let seeker = IndexSeek::new_variable_property(node_info, var_prop_exprs);
        let result = seeker.build_plan();
        assert!(result.is_ok());
    }
}
