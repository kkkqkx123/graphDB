//! 聚合函数元数据定义
//! 定义支持的聚合函数及其约束条件

/// 聚合函数元数据
#[derive(Debug, Clone)]
pub struct AggFunctionMeta {
    /// 函数名
    pub name: &'static str,
    /// 是否要求参数为数值类型（SUM、AVG、STD等）
    pub require_numeric: bool,
    /// 是否允许通配符属性 (*.) 作为参数
    pub allow_wildcard: bool,
}

impl AggFunctionMeta {
    /// 根据函数名获取聚合函数元数据
    ///
    /// # Arguments
    /// * `name` - 聚合函数名（大小写不敏感）
    ///
    /// # Returns
    /// 如果函数有效则返回Some，否则返回None
    pub fn get(name: &str) -> Option<Self> {
        match name.to_uppercase().as_str() {
            // COUNT 允许任意参数，且允许通配符
            "COUNT" => Some(AggFunctionMeta {
                name: "COUNT",
                require_numeric: false,
                allow_wildcard: true,
            }),
            // SUM 要求数值参数，不允许通配符
            "SUM" => Some(AggFunctionMeta {
                name: "SUM",
                require_numeric: true,
                allow_wildcard: false,
            }),
            // AVG 要求数值参数，不允许通配符
            "AVG" => Some(AggFunctionMeta {
                name: "AVG",
                require_numeric: true,
                allow_wildcard: false,
            }),
            // MAX 可以比较各种类型，不允许通配符
            "MAX" => Some(AggFunctionMeta {
                name: "MAX",
                require_numeric: false,
                allow_wildcard: false,
            }),
            // MIN 可以比较各种类型，不允许通配符
            "MIN" => Some(AggFunctionMeta {
                name: "MIN",
                require_numeric: false,
                allow_wildcard: false,
            }),
            // STD 要求数值参数，不允许通配符
            "STD" => Some(AggFunctionMeta {
                name: "STD",
                require_numeric: true,
                allow_wildcard: false,
            }),
            // BIT_AND 要求整数参数，不允许通配符
            "BIT_AND" => Some(AggFunctionMeta {
                name: "BIT_AND",
                require_numeric: true,
                allow_wildcard: false,
            }),
            // BIT_OR 要求整数参数，不允许通配符
            "BIT_OR" => Some(AggFunctionMeta {
                name: "BIT_OR",
                require_numeric: true,
                allow_wildcard: false,
            }),
            // BIT_XOR 要求整数参数，不允许通配符
            "BIT_XOR" => Some(AggFunctionMeta {
                name: "BIT_XOR",
                require_numeric: true,
                allow_wildcard: false,
            }),
            // COLLECT 允许任意参数，不允许通配符
            "COLLECT" => Some(AggFunctionMeta {
                name: "COLLECT",
                require_numeric: false,
                allow_wildcard: false,
            }),
            // COLLECT_SET 允许任意参数，不允许通配符
            "COLLECT_SET" => Some(AggFunctionMeta {
                name: "COLLECT_SET",
                require_numeric: false,
                allow_wildcard: false,
            }),
            _ => None,
        }
    }

    /// 获取所有支持的聚合函数名列表
    pub fn all_functions() -> Vec<&'static str> {
        vec![
            "COUNT",
            "SUM",
            "AVG",
            "MAX",
            "MIN",
            "STD",
            "BIT_AND",
            "BIT_OR",
            "BIT_XOR",
            "COLLECT",
            "COLLECT_SET",
        ]
    }

    /// 检查函数名是否有效
    pub fn is_valid(name: &str) -> bool {
        Self::get(name).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agg_function_meta_get_count() {
        let meta = AggFunctionMeta::get("COUNT").expect("COUNT function should exist in test");
        assert_eq!(meta.name, "COUNT");
        assert!(!meta.require_numeric);
        assert!(meta.allow_wildcard);
    }

    #[test]
    fn test_agg_function_meta_get_sum() {
        let meta = AggFunctionMeta::get("SUM").expect("SUM function should exist in test");
        assert_eq!(meta.name, "SUM");
        assert!(meta.require_numeric);
        assert!(!meta.allow_wildcard);
    }

    #[test]
    fn test_agg_function_meta_get_case_insensitive() {
        let meta_upper = AggFunctionMeta::get("COUNT");
        let meta_lower = AggFunctionMeta::get("count");
        let meta_mixed = AggFunctionMeta::get("CoUnT");

        assert!(meta_upper.is_some());
        assert!(meta_lower.is_some());
        assert!(meta_mixed.is_some());
    }

    #[test]
    fn test_agg_function_meta_get_invalid() {
        let meta = AggFunctionMeta::get("INVALID_FUNC");
        assert!(meta.is_none());
    }

    #[test]
    fn test_all_functions_count() {
        let funcs = AggFunctionMeta::all_functions();
        assert_eq!(funcs.len(), 11);
    }

    #[test]
    fn test_is_valid() {
        assert!(AggFunctionMeta::is_valid("COUNT"));
        assert!(AggFunctionMeta::is_valid("sum"));
        assert!(AggFunctionMeta::is_valid("COLLECT_SET"));
        assert!(!AggFunctionMeta::is_valid("UNKNOWN"));
    }
}
