//! 表达式转换器
//!
//! 将AST表达式转换为统一表达式类型
//!
//! ## 功能说明
//!
//! 该模块提供了将抽象语法树(AST)表达式转换为内部统一表达式表示的功能。
//! 支持多种表达式类型，包括：
//! - 字面量表达式（常量值）
//! - 变量引用
//! - 二元运算表达式（算术、比较、逻辑运算）
//! - 一元运算表达式（正负号、取反等）
//! - 函数调用表达式
//! - 属性访问表达式
//! - 列表和映射表达式
//! - 条件表达式（CASE）
//! - 类型转换表达式
//!
//! ## 使用示例
//!
//! ```rust
//! use crate::query::parser::Parser;
//! use crate::query::parser::expressions::expression_converter::convert_ast_to_expression_meta;
//!
//! let query = "MATCH (n) WHERE n.age > 25 RETURN n.name";
//! let mut parser = Parser::new(query);
//! let ast_expression = parser.parse_expression().unwrap();
//! let expression_meta = convert_ast_to_expression_meta(&ast_expression).unwrap();
//! ```

use crate::core::types::expression::Expression as GraphExpression;
use crate::core::types::expression::ExpressionMeta;
use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
use crate::core::Value;
use crate::query::parser::ast::{
    BinaryExpression, BinaryOp, CaseExpression, ConstantExpression, Expression, FunctionCallExpression, LabelExpression, ListExpression,
    MapExpression, PathExpression, PropertyAccessExpression, RangeExpression, SubscriptExpression, TypeCastExpression, UnaryExpression,
    UnaryOp, VariableExpression,
};

/// 将AST表达式转换为graph表达式
///
/// 此函数是表达式转换的主入口点，负责根据AST表达式的类型
/// 分发到相应的转换函数进行处理。
///
/// # 参数
///
/// * `ast_expression` - AST表达式引用
///
/// # 返回值
///
/// 返回转换后的graph表达式，如果转换失败则返回错误信息字符串
///
/// # 支持的表达式类型
///
/// - `Constant` - 常量/字面量表达式
/// - `Variable` - 变量引用
/// - `Binary` - 二元运算表达式
/// - `Unary` - 一元运算表达式
/// - `FunctionCall` - 函数调用
/// - `PropertyAccess` - 属性访问
/// - `List` - 列表字面量
/// - `Map` - 映射字面量
/// - `Case` - 条件表达式
/// - `Subscript` - 下标访问
/// - `TypeCast` - 类型转换
/// - `Range` - 范围表达式
/// - `Path` - 路径表达式
/// - `Label` - 标签表达式
///
/// **注意**: 此函数现在为内部函数，主要由 `convert_ast_to_expression_meta` 调用。
/// 外部代码应使用 `convert_ast_to_expression_meta` 或 `parse_expression_meta_from_string`。
#[doc(hidden)]
pub fn convert_ast_to_graph_expression(ast_expression: &crate::query::parser::ast::Expression) -> Result<GraphExpression, String> {
    match ast_expression {
        Expression::Constant(expression) => convert_constant_expression(expression),
        Expression::Variable(expression) => convert_variable_expression(expression),
        Expression::Binary(expression) => convert_binary_expression(expression),
        Expression::Unary(expression) => convert_unary_expression(expression),
        Expression::FunctionCall(expression) => convert_function_call_expression(expression),
        Expression::PropertyAccess(expression) => convert_property_access_expression(expression),
        Expression::List(expression) => convert_list_expression(expression),
        Expression::Map(expression) => convert_map_expression(expression),
        Expression::Case(expression) => convert_case_expression(expression),
        Expression::Subscript(expression) => convert_subscript_expression(expression),
        Expression::TypeCast(expression) => convert_type_cast_expression(expression),
        Expression::Range(expression) => convert_range_expression(expression),
        Expression::Path(expression) => convert_path_expression(expression),
        Expression::Label(expression) => convert_label_expression(expression),
    }
}

/// 成功时返回包含转换后值的`Expression::Literal`，
/// 失败时返回错误信息字符串
///
/// # 示例
///
/// ```rust
/// use crate::query::parser::ast::ConstantExpression;
/// use crate::core::Value;
/// let expr = ConstantExpression::new(Value::Int(42), Span::default());
/// let result = convert_constant_expression(&expr);
/// assert!(matches!(result, Expression::Literal(Value::Int(42))));
/// ```
fn convert_constant_expression(expression: &ConstantExpression) -> Result<GraphExpression, String> {
    let value = match &expression.value {
        Value::Bool(b) => Value::Bool(*b),
        Value::Int(i) => Value::Int(*i),
        Value::Float(f) => Value::Float(*f),
        Value::String(s) => Value::String(s.clone()),
        Value::Null(nt) => Value::Null(nt.clone()),
        _ => return Err(format!("不支持的常量值类型: {:?}", expression.value)),
    };
    Ok(GraphExpression::Literal(value))
}

/// 转换类型转换表达式
///
/// 将AST中的类型转换表达式转换为graph类型转换表达式。
/// 将目标类型字符串解析为内部数据类型表示。
///
/// # 参数
///
/// * `expression` - 类型转换表达式引用
///
/// # 返回值
///
/// 成功时返回`GraphExpression::TypeCast`，包含转换后的表达式和目标类型
/// 失败时返回错误信息字符串
fn convert_type_cast_expression(expression: &TypeCastExpression) -> Result<GraphExpression, String> {
    let converted_expression = convert_ast_to_graph_expression(&expression.expression)?;
    let target_type = parse_data_type(&expression.target_type)?;
    Ok(GraphExpression::TypeCast {
        expression: Box::new(converted_expression),
        target_type,
    })
}

/// 转换范围表达式
///
/// 将AST中的范围表达式转换为graph范围表达式。
/// 范围表达式用于列表切片操作，支持可选的起始和结束索引。
///
/// # 参数
///
/// * `expression` - 范围表达式引用
///
/// # 返回值
///
/// 成功时返回`GraphExpression::Range`，包含集合和可选的起止索引
/// 失败时返回错误信息字符串
fn convert_range_expression(expression: &RangeExpression) -> Result<GraphExpression, String> {
    let collection = convert_ast_to_graph_expression(&expression.collection)?;
    let start = if let Some(ref start_expression) = expression.start {
        Some(Box::new(convert_ast_to_graph_expression(start_expression)?))
    } else {
        None
    };
    let end = if let Some(ref end_expression) = expression.end {
        Some(Box::new(convert_ast_to_graph_expression(end_expression)?))
    } else {
        None
    };
    Ok(GraphExpression::Range {
        collection: Box::new(collection),
        start,
        end,
    })
}

/// 转换路径表达式
///
/// 将AST中的路径表达式转换为graph路径表达式。
/// 路径表达式由多个节点组成，表示图中的一条路径。
///
/// # 参数
///
/// * `expression` - 路径表达式引用
///
/// # 返回值
///
/// 成功时返回`GraphExpression::Path`，包含路径上的所有节点表达式
/// 失败时返回错误信息字符串
fn convert_path_expression(expression: &PathExpression) -> Result<GraphExpression, String> {
    let elements: Result<Vec<GraphExpression>, String> = expression
        .elements
        .iter()
        .map(|elem| convert_ast_to_graph_expression(elem))
        .collect();
    Ok(GraphExpression::Path(elements?))
}

/// 转换标签表达式
///
/// 将AST中的标签表达式转换为graph标签表达式。
/// 标签用于标识节点的类型，如"Person"、"Product"等。
///
/// # 参数
///
/// * `expression` - 标签表达式引用
///
/// # 返回值
///
/// 成功时返回`GraphExpression::Label`，包含标签名称
/// 失败时返回错误信息字符串
fn convert_label_expression(expression: &LabelExpression) -> Result<GraphExpression, String> {
    Ok(GraphExpression::Label(expression.label.clone()))
}

/// 解析数据类型字符串
fn parse_data_type(type_str: &str) -> Result<crate::core::types::expression::DataType, String> {
    match type_str.to_uppercase().as_str() {
        "BOOL" | "BOOLEAN" => Ok(crate::core::types::expression::DataType::Bool),
        "INT" | "INTEGER" => Ok(crate::core::types::expression::DataType::Int),
        "FLOAT" | "DOUBLE" => Ok(crate::core::types::expression::DataType::Float),
        "STRING" | "STR" => Ok(crate::core::types::expression::DataType::String),
        "LIST" => Ok(crate::core::types::expression::DataType::List),
        "MAP" => Ok(crate::core::types::expression::DataType::Map),
        "VERTEX" => Ok(crate::core::types::expression::DataType::Vertex),
        "EDGE" => Ok(crate::core::types::expression::DataType::Edge),
        "PATH" => Ok(crate::core::types::expression::DataType::Path),
        "DATETIME" => Ok(crate::core::types::expression::DataType::DateTime),
        "DATE" => Ok(crate::core::types::expression::DataType::Date),
        "TIME" => Ok(crate::core::types::expression::DataType::Time),
        "DURATION" => Ok(crate::core::types::expression::DataType::Duration),
        _ => Err(format!("不支持的数据类型: {}", type_str)),
    }
}

/// 转换变量表达式
///
/// 将AST中的变量引用转换为graph变量表达式。
/// 变量用于引用查询中的命名实体。
///
/// # 参数
///
/// * `expression` - 变量表达式引用
///
/// # 返回值
///
/// 成功时返回`GraphExpression::Variable`，包含变量名
fn convert_variable_expression(expression: &VariableExpression) -> Result<GraphExpression, String> {
    Ok(GraphExpression::Variable(expression.name.clone()))
}

/// 转换二元表达式
///
/// 将AST中的二元运算表达式转换为graph二元表达式。
/// 支持算术运算（+、-、*、/）、比较运算（==、!=、>、<、>=、<=）
/// 和逻辑运算（AND、OR、XOR）。
///
/// # 参数
///
/// * `expression` - 二元表达式引用
///
/// # 返回值
///
/// 成功时返回`GraphExpression::Binary`，包含左右操作数和运算符
/// 失败时返回错误信息字符串
fn convert_binary_expression(expression: &BinaryExpression) -> Result<GraphExpression, String> {
    let left = convert_ast_to_graph_expression(&expression.left)?;
    let right = convert_ast_to_graph_expression(&expression.right)?;
    let op = convert_binary_op(&expression.op)?;

    Ok(GraphExpression::Binary {
        left: Box::new(left),
        op,
        right: Box::new(right),
    })
}

/// 转换一元表达式
///
/// 将AST中的一元运算表达式转换为graph一元表达式。
/// 支持负号（-）、正号（+）和取反（NOT）。
///
/// # 参数
///
/// * `expression` - 一元表达式引用
///
/// # 返回值
///
/// 成功时返回`GraphExpression::Unary`，包含运算符和操作数
/// 失败时返回错误信息字符串
fn convert_unary_expression(expression: &UnaryExpression) -> Result<GraphExpression, String> {
    let operand = convert_ast_to_graph_expression(&expression.operand)?;
    let op = convert_unary_op(&expression.op)?;

    Ok(GraphExpression::Unary {
        op,
        operand: Box::new(operand),
    })
}

/// 转换函数调用表达式
///
/// 将AST中的函数调用表达式转换为graph函数表达式。
/// 识别聚合函数（COUNT、SUM、AVG、MIN、MAX等）并转换为聚合表达式，
/// 普通函数保持为函数表达式。
///
/// # 参数
///
/// * `expression` - 函数调用表达式引用
///
/// # 返回值
///
/// 成功时返回`GraphExpression::Aggregate`或`GraphExpression::Function`
/// 失败时返回错误信息字符串
fn convert_function_call_expression(expression: &FunctionCallExpression) -> Result<GraphExpression, String> {
    let args: Result<Vec<GraphExpression>, String> = expression
        .args
        .iter()
        .map(|arg| convert_ast_to_graph_expression(arg))
        .collect();

    let args = args?;

    // 检查是否为聚合函数
    let func_name = expression.name.to_uppercase();
    if is_aggregate_function(&func_name) {
        if args.len() != 1 {
            return Err(format!(
                "聚合函数 {} 需要一个参数，但提供了 {}",
                expression.name,
                args.len()
            ));
        }
        let arg = Box::new(args[0].clone());
        let aggregate_func = convert_aggregate_function(&func_name)?;

        Ok(GraphExpression::Aggregate {
            func: aggregate_func,
            arg,
            distinct: expression.distinct,
        })
    } else {
        // 普通函数调用
        Ok(GraphExpression::Function {
            name: expression.name.clone(),
            args,
        })
    }
}

/// 转换属性访问表达式
///
/// 将AST中的属性访问表达式转换为graph属性表达式。
/// 属性访问用于获取节点或边的属性值，如`n.name`、`e.weight`。
///
/// # 参数
///
/// * `expression` - 属性访问表达式引用
///
/// # 返回值
///
/// 成功时返回`GraphExpression::Property`，包含对象和属性名
/// 失败时返回错误信息字符串
fn convert_property_access_expression(expression: &PropertyAccessExpression) -> Result<GraphExpression, String> {
    let object = convert_ast_to_graph_expression(&expression.object)?;
    Ok(GraphExpression::Property {
        object: Box::new(object),
        property: expression.property.clone(),
    })
}

/// 转换列表表达式
///
/// 将AST中的列表字面量表达式转换为graph列表表达式。
/// 列表包含多个元素，每个元素都是一个表达式。
///
/// # 参数
///
/// * `expression` - 列表表达式引用
///
/// # 返回值
///
/// 成功时返回`GraphExpression::List`，包含所有元素
/// 失败时返回错误信息字符串
fn convert_list_expression(expression: &ListExpression) -> Result<GraphExpression, String> {
    let elements: Result<Vec<GraphExpression>, String> = expression
        .elements
        .iter()
        .map(|elem| convert_ast_to_graph_expression(elem))
        .collect();

    Ok(GraphExpression::List(elements?))
}

/// 转换映射表达式
///
/// 将AST中的映射字面量表达式转换为graph映射表达式。
/// 映射是由键值对组成的无序集合，键为字符串，值为表达式。
///
/// # 参数
///
/// * `expression` - 映射表达式引用
///
/// # 返回值
///
/// 成功时返回`GraphExpression::Map`，包含所有键值对
/// 失败时返回错误信息字符串
fn convert_map_expression(expression: &MapExpression) -> Result<GraphExpression, String> {
    let pairs: Result<Vec<(String, GraphExpression)>, String> = expression
        .pairs
        .iter()
        .map(|(key, value)| {
            let converted_value = convert_ast_to_graph_expression(value)?;
            Ok((key.clone(), converted_value))
        })
        .collect();

    Ok(GraphExpression::Map(pairs?))
}

/// 转换CASE表达式
///
/// 将AST中的条件表达式转换为graph CASE表达式。
/// CASE表达式类似于其他语言中的switch语句或if-else链，
/// 用于根据条件选择不同的值。
///
/// # 参数
///
/// * `expression` - CASE表达式引用
///
/// # 返回值
///
/// 成功时返回`GraphExpression::Case`，包含所有条件分支和默认值
/// 失败时返回错误信息字符串
fn convert_case_expression(expression: &CaseExpression) -> Result<GraphExpression, String> {
    let mut conditions = Vec::new();

    // 处理WHEN-THEN条件对
    for (when, then) in &expression.when_then_pairs {
        let when_expression = convert_ast_to_graph_expression(when)?;
        let then_expression = convert_ast_to_graph_expression(then)?;
        conditions.push((when_expression, then_expression));
    }

    let default = if let Some(ref default_expression) = expression.default {
        Some(Box::new(convert_ast_to_graph_expression(default_expression)?))
    } else {
        None
    };

    // 如果存在match表达式，需要特殊处理
    if let Some(ref match_expression) = expression.match_expression {
        // 对于有match表达式的CASE，需要将每个WHEN条件转换为与match表达式的比较
        let match_expression = convert_ast_to_graph_expression(match_expression)?;
        let mut new_conditions = Vec::new();

        for (when, then) in conditions {
            let condition = GraphExpression::Binary {
                left: Box::new(match_expression.clone()),
                op: BinaryOperator::Equal,
                right: Box::new(when),
            };
            new_conditions.push((condition, then));
        }

        Ok(GraphExpression::Case {
            conditions: new_conditions,
            default,
        })
    } else {
        Ok(GraphExpression::Case {
            conditions,
            default,
        })
    }
}

/// 转换下标表达式
///
/// 将AST中的下标访问表达式转换为graph下标表达式。
/// 下标表达式用于访问列表或映射的特定元素，如`list[0]`或`map[key]`。
///
/// # 参数
///
/// * `expression` - 下标表达式引用
///
/// # 返回值
///
/// 成功时返回`GraphExpression::Subscript`，包含集合和索引/键
/// 失败时返回错误信息字符串
fn convert_subscript_expression(expression: &SubscriptExpression) -> Result<GraphExpression, String> {
    let collection = convert_ast_to_graph_expression(&expression.collection)?;
    let index = convert_ast_to_graph_expression(&expression.index)?;

    Ok(GraphExpression::Subscript {
        collection: Box::new(collection),
        index: Box::new(index),
    })
}

/// 转换二元操作符
///
/// 将AST中的二元操作符转换为内部graph操作符表示。
/// 不同的操作符类型映射到相应的BinaryOperator枚举值。
///
/// # 参数
///
/// * `op` - AST二元操作符引用
///
/// # 返回值
///
/// 成功时返回对应的BinaryOperator枚举值
/// 对于不支持的操作符（如XOR），返回错误信息字符串
fn convert_binary_op(op: &BinaryOp) -> Result<BinaryOperator, String> {
    match op {
        // 算术操作符
        BinaryOp::Add => Ok(BinaryOperator::Add),
        BinaryOp::Subtract => Ok(BinaryOperator::Subtract),
        BinaryOp::Multiply => Ok(BinaryOperator::Multiply),
        BinaryOp::Divide => Ok(BinaryOperator::Divide),
        BinaryOp::Modulo => Ok(BinaryOperator::Modulo),
        BinaryOp::Exponent => Ok(BinaryOperator::Exponent),

        // 逻辑操作符
        BinaryOp::And => Ok(BinaryOperator::And),
        BinaryOp::Or => Ok(BinaryOperator::Or),
        BinaryOp::Xor => Err("XOR操作符在graph表达式中不支持".to_string()),

        // 关系操作符
        BinaryOp::Equal => Ok(BinaryOperator::Equal),
        BinaryOp::NotEqual => Ok(BinaryOperator::NotEqual),
        BinaryOp::LessThan => Ok(BinaryOperator::LessThan),
        BinaryOp::LessThanOrEqual => Ok(BinaryOperator::LessThanOrEqual),
        BinaryOp::GreaterThan => Ok(BinaryOperator::GreaterThan),
        BinaryOp::GreaterThanOrEqual => Ok(BinaryOperator::GreaterThanOrEqual),

        // 字符串操作符
        BinaryOp::Like => Ok(BinaryOperator::Like), // Like
        BinaryOp::In => Ok(BinaryOperator::In),
        BinaryOp::NotIn => Ok(BinaryOperator::NotIn),
        BinaryOp::Contains => Ok(BinaryOperator::Contains),
        BinaryOp::StartsWith => Ok(BinaryOperator::StartsWith),
        BinaryOp::EndsWith => Ok(BinaryOperator::EndsWith),

        // 其他操作符
        BinaryOp::StringConcat => Ok(BinaryOperator::StringConcat),
        BinaryOp::Subscript => Ok(BinaryOperator::Subscript),
        BinaryOp::Attribute => Ok(BinaryOperator::Attribute),
        BinaryOp::Union => Ok(BinaryOperator::Union),
        BinaryOp::Intersect => Ok(BinaryOperator::Intersect),
        BinaryOp::Except => Ok(BinaryOperator::Except),
    }
}

/// 转换一元操作符
///
/// 将AST中的一元操作符转换为内部graph一元操作符表示。
/// 支持的操作符包括：NOT、+、-、IS NULL、IS NOT NULL、IS EMPTY、IS NOT EMPTY。
///
/// # 参数
///
/// * `op` - AST一元操作符引用
///
/// # 返回值
///
/// 成功时返回对应的UnaryOperator枚举值
fn convert_unary_op(op: &UnaryOp) -> Result<UnaryOperator, String> {
    match op {
        UnaryOp::Not => Ok(UnaryOperator::Not),
        UnaryOp::Plus => Ok(UnaryOperator::Plus),
        UnaryOp::Minus => Ok(UnaryOperator::Minus),
        UnaryOp::IsNull => Ok(UnaryOperator::IsNull),
        UnaryOp::IsNotNull => Ok(UnaryOperator::IsNotNull),
        UnaryOp::IsEmpty => Ok(UnaryOperator::IsEmpty),
        UnaryOp::IsNotEmpty => Ok(UnaryOperator::IsNotEmpty),
    }
}

/// 转换聚合函数
///
/// 将函数名转换为对应的聚合函数枚举值。
/// 支持的聚合函数包括：COUNT、SUM、AVG、MIN、MAX、COLLECT、DISTINCT、PERCENTILE。
///
/// # 参数
///
/// * `func_name` - 函数名字符串
///
/// # 返回值
///
/// 成功时返回对应的AggregateFunction枚举值
/// 对于不支持的函数名，返回错误信息字符串
fn convert_aggregate_function(func_name: &str) -> Result<AggregateFunction, String> {
    match func_name {
        "COUNT" => Ok(AggregateFunction::Count(None)),
        "SUM" => Ok(AggregateFunction::Sum("".to_string())),
        "AVG" => Ok(AggregateFunction::Avg("".to_string())),
        "MIN" => Ok(AggregateFunction::Min("".to_string())),
        "MAX" => Ok(AggregateFunction::Max("".to_string())),
        "COLLECT" => Ok(AggregateFunction::Collect("".to_string())),
        "DISTINCT" => Ok(AggregateFunction::Distinct("".to_string())),
        "PERCENTILE" => Ok(AggregateFunction::Percentile("".to_string(), 50.0)), // 默认50%
        _ => Err(format!("不支持的聚合函数: {}", func_name)),
    }
}

/// 检查是否为聚合函数
///
/// 判断给定的函数名是否为聚合函数。
/// 聚合函数在查询处理中有特殊的语义（通常与GROUP BY一起使用）。
///
/// # 参数
///
/// * `func_name` - 函数名字符串
///
/// # 返回值
///
/// 如果是聚合函数返回true，否则返回false
fn is_aggregate_function(func_name: &str) -> bool {
    matches!(
        func_name,
        "COUNT" | "SUM" | "AVG" | "MIN" | "MAX" | "COLLECT" | "DISTINCT" | "PERCENTILE"
    )
}

/// 将AST表达式转换为富表达式（包含位置信息）
///
/// 此函数是新的推荐入口点，返回包含Span信息的ExpressionMeta。
/// Span信息用于错误定位和调试。
///
/// # 参数
///
/// * `ast_expression` - AST表达式引用
///
/// # 返回值
///
/// 返回包含位置信息的ExpressionMeta，如果转换失败则返回错误信息字符串
pub fn convert_ast_to_expression_meta(ast_expression: &crate::query::parser::ast::Expression) -> Result<ExpressionMeta, String> {
    let span = ast_expression.span();
    let core_expression = convert_ast_to_graph_expression(ast_expression)?;
    Ok(ExpressionMeta::with_span(core_expression, span))
}

/// 从字符串解析表达式并返回富表达式
///
/// 解析给定的表达式字符串，返回包含Span信息的ExpressionMeta。
/// Span信息反映表达式在源字符串中的位置范围。
///
/// # 参数
///
/// * `condition` - 包含表达式的字符串
///
/// # 返回值
///
/// 成功时返回包含位置信息的ExpressionMeta
/// 失败时返回错误信息字符串
///
/// # 示例
///
/// ```rust
/// let result = parse_expression_meta_from_string("n.age > 25");
/// assert!(result.is_ok());
/// let meta = result.unwrap();
/// assert!(meta.span().is_some());
/// ```
pub fn parse_expression_meta_from_string(condition: &str) -> Result<ExpressionMeta, String> {
    let mut parser = crate::query::parser::Parser::new(condition);
    let core_expression = parser
        .parse_expression()
        .map_err(|e| format!("语法分析错误: {:?}", e))?;
    // 现在 Parser 直接返回 Core Expression，不需要转换
    Ok(ExpressionMeta::new(core_expression))
}

/// 从字符串解析表达式
///
/// **已废弃**：请使用 `parse_expression_meta_from_string` 替代。
/// 此函数保留用于向后兼容。
///
/// # 参数
///
/// * `condition` - 包含表达式的字符串
///
/// # 返回值
///
/// 成功时返回转换后的graph表达式
/// 失败时返回错误信息字符串
///
/// # 示例
///
/// ```rust
/// let result = parse_expression_meta_from_string("n.age > 25");
/// assert!(result.is_ok());
/// ```

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;
    use crate::query::parser::ast::{
        BinaryExpression, BinaryOp, ConstantExpression, Expression, LabelExpression,
        Span, TypeCastExpression, UnaryExpression, UnaryOp, VariableExpression,
    };

    #[test]
    fn test_convert_constant_expression() {
        let ast_expression = Expression::Constant(ConstantExpression::new(Value::Int(42), Span::default()));
        let result = convert_ast_to_graph_expression(&ast_expression)
            .expect("Expected successful conversion of constant expression");

        assert!(matches!(result, GraphExpression::Literal(Value::Int(42))),
            "Expected Literal(Int(42)), got {:?}", result);
        if let GraphExpression::Literal(Value::Int(value)) = result {
            assert_eq!(value, 42);
        }
    }

    #[test]
    fn test_convert_variable_expression() {
        let ast_expression = Expression::Variable(VariableExpression::new("test_var".to_string(), Span::default()));
        let result = convert_ast_to_graph_expression(&ast_expression)
            .expect("Expected successful conversion of variable expression");

        assert!(matches!(result, GraphExpression::Variable(ref name) if name == "test_var"),
            "Expected Variable(\"test_var\"), got {:?}", result);
        if let GraphExpression::Variable(ref name) = result {
            assert_eq!(*name, "test_var");
        }
    }

    #[test]
    fn test_convert_type_cast_expression() {
        let inner_expression = Expression::Constant(ConstantExpression::new(Value::Int(42), Span::default()));
        let ast_expression = Expression::TypeCast(TypeCastExpression::new(
            inner_expression,
            "FLOAT".to_string(),
            Span::default(),
        ));
        let result = convert_ast_to_graph_expression(&ast_expression)
            .expect("Expected successful conversion of type cast expression");

        assert!(matches!(result, GraphExpression::TypeCast { expression: _, ref target_type }),
            "Expected TypeCast, got {:?}", result);
        if let GraphExpression::TypeCast { expression, ref target_type } = result {
            assert_eq!(*expression, GraphExpression::Literal(Value::Int(42)));
            assert_eq!(*target_type, crate::core::types::expression::DataType::Float);
        }
    }

    #[test]
    fn test_convert_label_expression() {
        let ast_expression = Expression::Label(LabelExpression::new("Person".to_string(), Span::default()));
        let result = convert_ast_to_graph_expression(&ast_expression)
            .expect("Expected successful conversion of label expression");

        assert!(matches!(result, GraphExpression::Label(ref label) if label == "Person"),
            "Expected Label, got {:?}", result);
        if let GraphExpression::Label(ref label) = result {
            assert_eq!(*label, "Person");
        }
    }

    #[test]
    fn test_convert_binary_expression() {
        let left = Expression::Constant(ConstantExpression::new(Value::Int(5), Span::default()));
        let right = Expression::Constant(ConstantExpression::new(Value::Int(3), Span::default()));
        let ast_expression = Expression::Binary(BinaryExpression::new(left, BinaryOp::Add, right, Span::default()));

        let result = convert_ast_to_graph_expression(&ast_expression)
            .expect("Expected successful conversion of binary expression");

        assert!(matches!(result, GraphExpression::Binary { left: _, op: BinaryOperator::Add, right: _ }),
            "Expected Binary expression, got {:?}", result);
        if let GraphExpression::Binary { left, op, right } = result {
            assert_eq!(*left, GraphExpression::Literal(Value::Int(5)));
            assert_eq!(op, BinaryOperator::Add);
            assert_eq!(*right, GraphExpression::Literal(Value::Int(3)));
        }
    }

    #[test]
    fn test_convert_unary_expression() {
        let operand = Expression::Constant(ConstantExpression::new(Value::Bool(true), Span::default()));
        let ast_expression = Expression::Unary(UnaryExpression::new(UnaryOp::Not, operand, Span::default()));

        let result = convert_ast_to_graph_expression(&ast_expression)
            .expect("Expected successful conversion of unary expression");

        assert!(matches!(result, GraphExpression::Unary { op: UnaryOperator::Not, operand: _ }),
            "Expected Unary expression, got {:?}", result);
        if let GraphExpression::Unary { op, operand } = result {
            assert_eq!(op, UnaryOperator::Not);
            assert_eq!(*operand, GraphExpression::Literal(Value::Bool(true)));
        }
    }

    #[test]
    fn test_convert_unsupported_operator() {
        let left = Expression::Constant(ConstantExpression::new(Value::Int(5), Span::default()));
        let right = Expression::Constant(ConstantExpression::new(Value::Int(3), Span::default()));
        let ast_expression = Expression::Binary(BinaryExpression::new(left, BinaryOp::Xor, right, Span::default()));

        let result = convert_ast_to_graph_expression(&ast_expression);
        assert!(result.is_err());
        assert!(result
            .expect_err("Expected error for unsupported operator")
            .contains("XOR操作符在graph表达式中不支持"));
    }

    #[test]
    fn test_parse_expression_meta_from_string() {
        let result = parse_expression_meta_from_string("5 + 3");
        assert!(result.is_ok());

        let meta = result.expect("Expected successful parsing of expression from string");
        assert!(matches!(meta.inner(), crate::core::types::expression::Expression::Binary { .. }));
    }

    #[test]
    fn test_convert_list_expression() {
        use crate::query::parser::ast::ListExpression;

        let elements = vec![
            Expression::Constant(ConstantExpression::new(Value::Int(1), Span::default())),
            Expression::Constant(ConstantExpression::new(Value::Int(2), Span::default())),
            Expression::Constant(ConstantExpression::new(Value::Int(3), Span::default())),
        ];
        let ast_expression = Expression::List(ListExpression::new(elements, Span::default()));

        let result = convert_ast_to_graph_expression(&ast_expression)
            .expect("Expected successful conversion of list expression");

        assert!(matches!(result, GraphExpression::List(ref items) if items.len() == 3),
            "Expected List with 3 elements, got {:?}", result);
        if let GraphExpression::List(ref items) = result {
            assert_eq!(items.len(), 3);
        }
    }

    #[test]
    fn test_convert_map_expression() {
        use crate::query::parser::ast::MapExpression;

        let mut pairs = Vec::new();
        pairs.push((
            "name".to_string(),
            Expression::Constant(ConstantExpression::new(Value::String("test".to_string()), Span::default())),
        ));
        pairs.push((
            "age".to_string(),
            Expression::Constant(ConstantExpression::new(Value::Int(25), Span::default())),
        ));
        let ast_expression = Expression::Map(MapExpression::new(pairs, Span::default()));

        let result = convert_ast_to_graph_expression(&ast_expression)
            .expect("Expected successful conversion of map expression");

        assert!(matches!(result, GraphExpression::Map(ref pairs) if pairs.len() == 2),
            "Expected Map with 2 pairs, got {:?}", result);
    }

    #[test]
    fn test_convert_property_access_expression() {
        use crate::query::parser::ast::PropertyAccessExpression;

        let variable = Box::new(Expression::Variable(VariableExpression::new("person".to_string(), Span::default())));
        let ast_expression = Expression::PropertyAccess(PropertyAccessExpression::new(
            *variable,
            "name".to_string(),
            Span::default(),
        ));

        let result = convert_ast_to_graph_expression(&ast_expression)
            .expect("Expected successful conversion of property access expression");

        assert!(matches!(result, GraphExpression::Property { object: _, property: _ }),
            "Expected Property access, got {:?}", result);
    }

    #[test]
    fn test_convert_function_call_expression() {
        use crate::query::parser::ast::FunctionCallExpression;

        let args = vec![
            Expression::Constant(ConstantExpression::new(Value::Int(1), Span::default())),
        ];
        let ast_expression = Expression::FunctionCall(FunctionCallExpression::new(
            "SUM".to_string(),
            args,
            false,
            Span::default(),
        ));

        let result = convert_ast_to_graph_expression(&ast_expression)
            .expect("Expected successful conversion of function call expression");

        assert!(matches!(result, GraphExpression::Aggregate { .. }),
            "Expected Aggregate function, got {:?}", result);
        if let GraphExpression::Aggregate { func, arg, distinct } = result {
            assert!(matches!(func, crate::core::types::operators::AggregateFunction::Sum(_)));
        }
    }

    #[test]
    fn test_convert_nested_expression() {
        let inner = Expression::Constant(ConstantExpression::new(Value::Int(10), Span::default()));
        let outer = Expression::Binary(BinaryExpression::new(
            inner,
            BinaryOp::Multiply,
            Expression::Constant(ConstantExpression::new(Value::Int(2), Span::default())),
            Span::default(),
        ));
        let ast_expression = Expression::Binary(BinaryExpression::new(
            Expression::Constant(ConstantExpression::new(Value::Int(5), Span::default())),
            BinaryOp::Add,
            outer,
            Span::default(),
        ));

        let result = convert_ast_to_graph_expression(&ast_expression)
            .expect("Expected successful conversion of nested expression");

        assert!(matches!(result, GraphExpression::Binary { left: _, op: BinaryOperator::Add, right: _ }),
            "Expected nested Binary expression, got {:?}", result);
    }

    #[test]
    fn test_convert_multiple_types() {
        let int_expr = Expression::Constant(ConstantExpression::new(Value::Int(100), Span::default()));
        let float_expr = Expression::Constant(ConstantExpression::new(Value::Float(3.14), Span::default()));
        let string_expr = Expression::Constant(ConstantExpression::new(Value::String("hello".to_string()), Span::default()));
        let bool_expr = Expression::Constant(ConstantExpression::new(Value::Bool(false), Span::default()));

        let int_result = convert_ast_to_graph_expression(&int_expr);
        let float_result = convert_ast_to_graph_expression(&float_expr);
        let string_result = convert_ast_to_graph_expression(&string_expr);
        let bool_result = convert_ast_to_graph_expression(&bool_expr);

        assert!(int_result.is_ok());
        assert!(float_result.is_ok());
        assert!(string_result.is_ok());
        assert!(bool_result.is_ok());

        assert!(matches!(int_result.unwrap(), GraphExpression::Literal(Value::Int(100))));
        assert!(matches!(float_result.unwrap(), GraphExpression::Literal(Value::Float(_))));
        assert!(matches!(string_result.unwrap(), GraphExpression::Literal(Value::String(_))));
        assert!(matches!(bool_result.unwrap(), GraphExpression::Literal(Value::Bool(false))));
    }
}
