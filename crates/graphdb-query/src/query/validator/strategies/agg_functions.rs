//! Metadata definition for aggregate functions
//! Define the supported aggregate functions and their constraints.

/// Aggregation function metadata
#[derive(Debug, Clone)]
pub struct AggFunctionMeta {
    /// Function name
    pub name: &'static str,
    /// Should the parameters be required to be of a numeric type (SUM, AVG, STD, etc.)?
    pub require_numeric: bool,
    /// Are wildcard attributes (*.) allowed to be used as parameters?
    pub allow_wildcard: bool,
}

impl AggFunctionMeta {
    /// Retrieve metadata for aggregate functions based on the function name
    ///
    /// # Arguments
    /// * `name` – The name of the aggregate function (case-insensitive).
    ///
    /// # Returns
    /// Return “Some” if the function is valid; otherwise, return “None”.
    pub fn get(name: &str) -> Option<Self> {
        match name.to_uppercase().as_str() {
            // The COUNT function accepts any number of parameters and also supports the use of wildcards.
            "COUNT" => Some(AggFunctionMeta {
                name: "COUNT",
                require_numeric: false,
                allow_wildcard: true,
            }),
            // The SUM function requires numeric parameters; wildcards are not allowed.
            "SUM" => Some(AggFunctionMeta {
                name: "SUM",
                require_numeric: true,
                allow_wildcard: false,
            }),
            // AVG requires numerical parameters; wildcards are not allowed.
            "AVG" => Some(AggFunctionMeta {
                name: "AVG",
                require_numeric: true,
                allow_wildcard: false,
            }),
            // MAX can compare various types; the use of wildcards is not allowed.
            "MAX" => Some(AggFunctionMeta {
                name: "MAX",
                require_numeric: false,
                allow_wildcard: false,
            }),
            // MIN can compare various types; wildcards are not allowed.
            "MIN" => Some(AggFunctionMeta {
                name: "MIN",
                require_numeric: false,
                allow_wildcard: false,
            }),
            // The STD requires a numerical parameter; wildcards are not allowed.
            "STD" => Some(AggFunctionMeta {
                name: "STD",
                require_numeric: true,
                allow_wildcard: false,
            }),
            // The BIT_AND function requires integer parameters and does not allow the use of wildcards.
            "BIT_AND" => Some(AggFunctionMeta {
                name: "BIT_AND",
                require_numeric: true,
                allow_wildcard: false,
            }),
            // The BIT_OR function requires integer parameters; wildcards are not allowed.
            "BIT_OR" => Some(AggFunctionMeta {
                name: "BIT_OR",
                require_numeric: true,
                allow_wildcard: false,
            }),
            // The BIT_XOR function requires integer parameters; wildcards are not allowed.
            "BIT_XOR" => Some(AggFunctionMeta {
                name: "BIT_XOR",
                require_numeric: true,
                allow_wildcard: false,
            }),
            // The `COLLECT` function allows any number of parameters, but wildcards are not permitted.
            "COLLECT" => Some(AggFunctionMeta {
                name: "COLLECT",
                require_numeric: false,
                allow_wildcard: false,
            }),
            // The `COLLECT_SET` function allows any number of parameters, but wildcards are not permitted.
            "COLLECT_SET" => Some(AggFunctionMeta {
                name: "COLLECT_SET",
                require_numeric: false,
                allow_wildcard: false,
            }),
            _ => None,
        }
    }

    /// Get a list of all the names of the supported aggregate functions.
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

    /// Check whether the function name is valid.
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
