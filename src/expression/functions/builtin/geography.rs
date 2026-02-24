//! 地理空间函数实现

use crate::core::error::ExpressionError;
use crate::core::value::geography::GeographyValue;
use crate::core::value::NullType;
use crate::core::Value;
use crate::define_function_enum;
use crate::define_binary_geography_fn;

define_function_enum! {
    /// 地理空间函数枚举
    pub enum GeographyFunction {
        StPoint => {
            name: "st_point",
            arity: 2,
            variadic: false,
            description: "创建地理点 (经度, 纬度)",
            handler: execute_st_point
        },
        StGeogFromText => {
            name: "st_geogfromtext",
            arity: 1,
            variadic: false,
            description: "从WKT文本创建地理对象",
            handler: execute_st_geogfromtext
        },
        StAsText => {
            name: "st_astext",
            arity: 1,
            variadic: false,
            description: "将地理对象转换为WKT文本",
            handler: execute_st_astext
        },
        StCentroid => {
            name: "st_centroid",
            arity: 1,
            variadic: false,
            description: "计算地理对象的中心点",
            handler: execute_st_centroid
        },
        StIsValid => {
            name: "st_isvalid",
            arity: 1,
            variadic: false,
            description: "检查地理对象是否有效",
            handler: execute_st_isvalid
        },
        StIntersects => {
            name: "st_intersects",
            arity: 2,
            variadic: false,
            description: "检查两个地理对象是否相交",
            handler: execute_st_intersects
        },
        StCovers => {
            name: "st_covers",
            arity: 2,
            variadic: false,
            description: "检查第一个地理对象是否覆盖第二个",
            handler: execute_st_covers
        },
        StCoveredBy => {
            name: "st_coveredby",
            arity: 2,
            variadic: false,
            description: "检查第一个地理对象是否被第二个覆盖",
            handler: execute_st_coveredby
        },
        StDWithin => {
            name: "st_dwithin",
            arity: 3,
            variadic: false,
            description: "检查两个地理对象是否在指定距离内（单位：公里）",
            handler: execute_st_dwithin
        },
        StDistance => {
            name: "st_distance",
            arity: 2,
            variadic: false,
            description: "计算两个地理对象之间的距离（单位：公里）",
            handler: execute_st_distance
        },
    }
}

fn execute_st_point(args: &[Value]) -> Result<Value, ExpressionError> {
    match (&args[0], &args[1]) {
        (Value::Float(lon), Value::Float(lat)) => {
            let geo = GeographyValue {
                longitude: *lon,
                latitude: *lat,
            };
            Ok(Value::Geography(geo))
        }
        (Value::Int(lon), Value::Int(lat)) => {
            let geo = GeographyValue {
                longitude: *lon as f64,
                latitude: *lat as f64,
            };
            Ok(Value::Geography(geo))
        }
        (Value::Float(lon), Value::Int(lat)) => {
            let geo = GeographyValue {
                longitude: *lon,
                latitude: *lat as f64,
            };
            Ok(Value::Geography(geo))
        }
        (Value::Int(lon), Value::Float(lat)) => {
            let geo = GeographyValue {
                longitude: *lon as f64,
                latitude: *lat,
            };
            Ok(Value::Geography(geo))
        }
        (Value::Null(_), _) | (_, Value::Null(_)) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("st_point函数需要数值参数")),
    }
}

fn execute_st_geogfromtext(args: &[Value]) -> Result<Value, ExpressionError> {
    use crate::core::value::geography::Geography;
    match &args[0] {
        Value::String(wkt) => match Geography::from_wkt(wkt) {
            Ok(Geography::Point(geo)) => Ok(Value::Geography(geo)),
            Ok(_) => Err(ExpressionError::type_error("st_geogfromtext目前只支持点类型")),
            Err(e) => Err(ExpressionError::type_error(&format!("解析WKT失败: {}", e))),
        },
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("st_geogfromtext函数需要字符串参数")),
    }
}

fn execute_st_astext(args: &[Value]) -> Result<Value, ExpressionError> {
    match &args[0] {
        Value::Geography(geo) => {
            let wkt = format!("POINT({} {})", geo.longitude, geo.latitude);
            Ok(Value::String(wkt))
        }
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("st_astext函数需要地理类型")),
    }
}

fn execute_st_centroid(args: &[Value]) -> Result<Value, ExpressionError> {
    match &args[0] {
        Value::Geography(geo) => Ok(Value::Geography(geo.clone())),
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("st_centroid函数需要地理类型")),
    }
}

fn execute_st_isvalid(args: &[Value]) -> Result<Value, ExpressionError> {
    match &args[0] {
        Value::Geography(geo) => {
            let is_valid = geo.latitude >= -90.0
                && geo.latitude <= 90.0
                && geo.longitude >= -180.0
                && geo.longitude <= 180.0;
            Ok(Value::Bool(is_valid))
        }
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("st_isvalid函数需要地理类型")),
    }
}

define_binary_geography_fn!(
    execute_st_intersects,
    |geo1: &GeographyValue, geo2: &GeographyValue| {
        let distance = geo1.distance(geo2);
        Ok(Value::Bool(distance < 0.001))
    },
    "st_intersects"
);

define_binary_geography_fn!(
    execute_st_covers,
    |geo1: &GeographyValue, geo2: &GeographyValue| {
        let distance = geo1.distance(geo2);
        Ok(Value::Bool(distance < 0.001))
    },
    "st_covers"
);

define_binary_geography_fn!(
    execute_st_coveredby,
    |geo1: &GeographyValue, geo2: &GeographyValue| {
        let distance = geo1.distance(geo2);
        Ok(Value::Bool(distance < 0.001))
    },
    "st_coveredby"
);

define_binary_geography_fn!(
    execute_st_distance,
    |geo1: &GeographyValue, geo2: &GeographyValue| {
        Ok(Value::Float(geo1.distance(geo2)))
    },
    "st_distance"
);

fn execute_st_dwithin(args: &[Value]) -> Result<Value, ExpressionError> {
    match (&args[0], &args[1], &args[2]) {
        (Value::Geography(geo1), Value::Geography(geo2), Value::Float(distance)) => {
            let actual_distance = geo1.distance(geo2);
            Ok(Value::Bool(actual_distance <= *distance))
        }
        (Value::Geography(geo1), Value::Geography(geo2), Value::Int(distance)) => {
            let actual_distance = geo1.distance(geo2);
            Ok(Value::Bool(actual_distance <= *distance as f64))
        }
        (Value::Null(_), _, _) | (_, Value::Null(_), _) | (_, _, Value::Null(_)) => {
            Ok(Value::Null(NullType::Null))
        }
        _ => Err(ExpressionError::type_error(
            "st_dwithin函数需要地理类型和数值距离参数",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_st_point() {
        let func = GeographyFunction::StPoint;
        let result = func
            .execute(&[Value::Float(116.4074), Value::Float(39.9042)])
            .unwrap();
        assert!(matches!(result, Value::Geography(_)));
    }

    #[test]
    fn test_st_isvalid() {
        let func = GeographyFunction::StIsValid;
        let geo = GeographyValue {
            longitude: 116.4074,
            latitude: 39.9042,
        };
        let result = func.execute(&[Value::Geography(geo)]).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_st_distance() {
        let func = GeographyFunction::StDistance;
        let geo1 = GeographyValue {
            longitude: 116.4074,
            latitude: 39.9042,
        };
        let geo2 = GeographyValue {
            longitude: 121.4737,
            latitude: 31.2304,
        };
        let result = func.execute(&[Value::Geography(geo1), Value::Geography(geo2)]).unwrap();
        assert!(matches!(result, Value::Float(_)));
    }

    #[test]
    fn test_null_handling() {
        let func = GeographyFunction::StIsValid;
        let result = func.execute(&[Value::Null(NullType::Null)]).unwrap();
        assert_eq!(result, Value::Null(NullType::Null));
    }
}
