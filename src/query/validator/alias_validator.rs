//! 别名验证器模块
//! 负责验证表达式中的别名引用和可用性

use crate::graph::expression::expr_type::Expression;
use crate::query::validator::structs::AliasType;
use std::collections::HashMap;

/// 别名验证器
pub struct AliasValidator;

impl AliasValidator {
    pub fn new() -> Self {
        Self
    }

    /// 验证表达式列表中的别名
    pub fn validate_aliases(
        &self,
        exprs: &[Expression],
        aliases: &HashMap<String, AliasType>,
    ) -> Result<(), String> {
        for expr in exprs {
            self.validate_expression_aliases(expr, aliases)?;
        }
        Ok(())
    }

    /// 验证单个表达式中的别名
    pub fn validate_expression_aliases(
        &self,
        expr: &Expression,
        aliases: &HashMap<String, AliasType>,
    ) -> Result<(), String> {
        // 首先检查表达式本身是否引用了一个别名
        if let Some(alias_name) = self.extract_alias_name(expr) {
            if !aliases.contains_key(&alias_name) {
                return Err(format!("未定义的变量别名: {}", alias_name));
            }
        }

        // 递归验证子表达式
        self.validate_subexpressions_aliases(expr, aliases)?;

        Ok(())
    }

    /// 从表达式中提取别名名称
    pub fn extract_alias_name(&self, expr: &Expression) -> Option<String> {
        match expr {
            Expression::Variable(name) => Some(name.clone()),
            Expression::Property(name) => Some(name.clone()),
            Expression::Label(name) => Some(name.clone()),
            // 根据实际的表达式类型，可能需要处理其他别名引用
            _ => None,
        }
    }

    /// 递归验证子表达式中的别名
    fn validate_subexpressions_aliases(
        &self,
        expr: &Expression,
        aliases: &HashMap<String, AliasType>,
    ) -> Result<(), String> {
        match expr {
            Expression::UnaryOp(_, operand) => self.validate_expression_aliases(operand, aliases),
            Expression::BinaryOp(left, _, right) => {
                self.validate_expression_aliases(left, aliases)?;
                self.validate_expression_aliases(right, aliases)
            }
            Expression::Property(_) => {
                // Property expression doesn't have sub-expressions
                Ok(())
            }
            Expression::Function(_, args) => {
                for arg in args {
                    self.validate_expression_aliases(arg, aliases)?;
                }
                Ok(())
            }
            // For constants, there are no sub-expressions
            Expression::Constant(_) => Ok(()),
            Expression::TagProperty { .. }
            | Expression::EdgeProperty { .. }
            | Expression::InputProperty(_)
            | Expression::VariableProperty { .. }
            | Expression::SourceProperty { .. }
            | Expression::DestinationProperty { .. } => {
                // These expressions don't have sub-expressions
                Ok(())
            }
            Expression::UnaryPlus(operand) => self.validate_expression_aliases(operand, aliases),
            Expression::UnaryNegate(operand) => self.validate_expression_aliases(operand, aliases),
            Expression::UnaryNot(operand) => self.validate_expression_aliases(operand, aliases),
            Expression::UnaryIncr(operand) => self.validate_expression_aliases(operand, aliases),
            Expression::UnaryDecr(operand) => self.validate_expression_aliases(operand, aliases),
            Expression::IsNull(operand) => self.validate_expression_aliases(operand, aliases),
            Expression::IsNotNull(operand) => self.validate_expression_aliases(operand, aliases),
            Expression::IsEmpty(operand) => self.validate_expression_aliases(operand, aliases),
            Expression::IsNotEmpty(operand) => self.validate_expression_aliases(operand, aliases),
            Expression::List(items) => {
                for item in items {
                    self.validate_expression_aliases(item, aliases)?;
                }
                Ok(())
            }
            Expression::Set(items) => {
                for item in items {
                    self.validate_expression_aliases(item, aliases)?;
                }
                Ok(())
            }
            Expression::Map(items) => {
                for (_, value) in items {
                    self.validate_expression_aliases(value, aliases)?;
                }
                Ok(())
            }
            Expression::TypeCasting { expr, .. } => self.validate_expression_aliases(expr, aliases),
            Expression::Case {
                conditions,
                default,
            } => {
                for (condition, value) in conditions {
                    self.validate_expression_aliases(condition, aliases)?;
                    self.validate_expression_aliases(value, aliases)?;
                }
                if let Some(default_expr) = default {
                    self.validate_expression_aliases(default_expr, aliases)?;
                }
                Ok(())
            }
            Expression::Aggregate { arg, .. } => self.validate_expression_aliases(arg, aliases),
            Expression::ListComprehension {
                generator,
                condition,
            } => {
                self.validate_expression_aliases(generator, aliases)?;
                if let Some(condition_expr) = condition {
                    self.validate_expression_aliases(condition_expr, aliases)?;
                }
                Ok(())
            }
            Expression::Predicate { list, condition } => {
                self.validate_expression_aliases(list, aliases)?;
                self.validate_expression_aliases(condition, aliases)
            }
            Expression::Reduce {
                list,
                initial,
                expr,
                ..
            } => {
                self.validate_expression_aliases(list, aliases)?;
                self.validate_expression_aliases(initial, aliases)?;
                self.validate_expression_aliases(expr, aliases)
            }
            Expression::PathBuild(items) => {
                for item in items {
                    self.validate_expression_aliases(item, aliases)?;
                }
                Ok(())
            }
            Expression::ESQuery(_) => {
                // ESQuery has no sub-expressions
                Ok(())
            }
            Expression::UUID => {
                // UUID has no sub-expressions
                Ok(())
            }
            Expression::Variable(_) => {
                // Variable has no sub-expressions
                Ok(())
            }
            Expression::Subscript { collection, index } => {
                self.validate_expression_aliases(collection, aliases)?;
                self.validate_expression_aliases(index, aliases)
            }
            Expression::SubscriptRange {
                collection,
                start,
                end,
            } => {
                self.validate_expression_aliases(collection, aliases)?;
                if let Some(start_expr) = start {
                    self.validate_expression_aliases(start_expr, aliases)?;
                }
                if let Some(end_expr) = end {
                    self.validate_expression_aliases(end_expr, aliases)?;
                }
                Ok(())
            }
            Expression::Label(_) => {
                // Label has no sub-expressions
                Ok(())
            }
            Expression::MatchPathPattern { patterns, .. } => {
                for pattern in patterns {
                    self.validate_expression_aliases(pattern, aliases)?;
                }
                Ok(())
            }
        }
    }

    /// 检查别名类型是否匹配使用方式
    pub fn check_alias(
        &self,
        ref_expr: &Expression,
        aliases_available: &HashMap<String, AliasType>,
    ) -> Result<(), String> {
        // 提取表达式中的别名名称
        if let Some(alias_name) = self.extract_alias_name(ref_expr) {
            if !aliases_available.contains_key(&alias_name) {
                return Err(format!("未定义的别名: {}", alias_name));
            }

            // 进一步验证别名类型是否匹配使用方式
            match ref_expr {
                Expression::SourceProperty { .. } | Expression::DestinationProperty { .. } => {
                    // 源/目标属性应指向节点类型的别名
                    if let Some(alias_type) = aliases_available.get(&alias_name) {
                        if alias_type == &AliasType::Edge || alias_type == &AliasType::Path {
                            return Err(format!(
                                "要获取边/路径的源/目标顶点ID，请使用 src/dst/endNode({})",
                                alias_name
                            ));
                        } else if alias_type != &AliasType::Node {
                            return Err(format!("别名 `{}` 没有边属性 src/dst", alias_name));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// 结合别名
    pub fn combine_aliases(
        &self,
        cur_aliases: &mut HashMap<String, AliasType>,
        last_aliases: &HashMap<String, AliasType>,
    ) -> Result<(), String> {
        for (name, alias_type) in last_aliases {
            if let Some(existing_type) = cur_aliases.get(name) {
                // 检查类型是否冲突
                if existing_type != alias_type {
                    return Err(format!("`{}': 别名类型冲突", name));
                }
            } else {
                // 插入新的别名
                cur_aliases.insert(name.clone(), alias_type.clone());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::expression::expr_type::Expression;
    use std::collections::HashMap;

    #[test]
    fn test_alias_validator_creation() {
        let validator = AliasValidator::new();
        // 验证器创建成功
        assert!(true); // 占位测试
    }

    #[test]
    fn test_validate_aliases() {
        let validator = AliasValidator::new();

        // 创建一个别名映射
        let mut aliases = HashMap::new();
        aliases.insert("n".to_string(), AliasType::Node);
        aliases.insert("e".to_string(), AliasType::Edge);

        // 测试有效的别名引用
        let expr = Expression::Variable("n".to_string());
        assert!(validator.validate_aliases(&[expr], &aliases).is_ok());

        // 测试无效的别名引用
        let invalid_expr = Expression::Variable("invalid".to_string());
        assert!(validator
            .validate_aliases(&[invalid_expr], &aliases)
            .is_err());
    }

    #[test]
    fn test_extract_alias_name() {
        let validator = AliasValidator::new();

        // 测试从变量表达式中提取别名
        let var_expr = Expression::Variable("test_var".to_string());
        assert_eq!(
            validator.extract_alias_name(&var_expr),
            Some("test_var".to_string())
        );

        // 测试从常量表达式中提取别名（应该返回None）
        let const_expr = Expression::Constant(crate::core::Value::Int(42));
        assert_eq!(validator.extract_alias_name(&const_expr), None);
    }

    #[test]
    fn test_check_alias() {
        let validator = AliasValidator::new();

        let mut aliases = HashMap::new();
        aliases.insert("node_alias".to_string(), AliasType::Node);
        aliases.insert("edge_alias".to_string(), AliasType::Edge);

        // 测试有效的别名检查
        let valid_expr = Expression::Variable("node_alias".to_string());
        assert!(validator.check_alias(&valid_expr, &aliases).is_ok());

        // 测试无效的别名检查
        let invalid_expr = Expression::Variable("nonexistent".to_string());
        assert!(validator.check_alias(&invalid_expr, &aliases).is_err());
    }

    #[test]
    fn test_combine_aliases() {
        let validator = AliasValidator::new();

        let mut cur_aliases = HashMap::new();
        cur_aliases.insert("a".to_string(), AliasType::Node);

        let mut last_aliases = HashMap::new();
        last_aliases.insert("b".to_string(), AliasType::Edge);
        last_aliases.insert("c".to_string(), AliasType::Path);

        // 组合别名
        assert!(validator
            .combine_aliases(&mut cur_aliases, &last_aliases)
            .is_ok());
        assert_eq!(cur_aliases.len(), 3);
        assert!(cur_aliases.contains_key("a"));
        assert!(cur_aliases.contains_key("b"));
        assert!(cur_aliases.contains_key("c"));
    }

    #[test]
    fn test_combine_aliases_conflict() {
        let validator = AliasValidator::new();

        let mut cur_aliases = HashMap::new();
        cur_aliases.insert("a".to_string(), AliasType::Node);

        let mut last_aliases = HashMap::new();
        last_aliases.insert("a".to_string(), AliasType::Edge); // 冲突的别名

        // 组合别名应该失败
        assert!(validator
            .combine_aliases(&mut cur_aliases, &last_aliases)
            .is_err());
    }
}
