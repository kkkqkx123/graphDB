//! 地理空间函数实现
//!
//! 提供地理空间操作函数，包括创建点、线、多边形，以及空间查询和计算

use crate::core::error::ExpressionError;
use crate::core::value::geography::{Geography, GeographyValue};
use crate::core::value::NullType;
use crate::core::Value;
use crate::expression::functions::registry::FunctionRegistry;
use crate::expression::functions::signature::FunctionSignature;
use crate::expression::functions::signature::ValueType;

/// 注册所有地理空间函数
pub fn register_all(registry: &mut FunctionRegistry) {
    register_st_point(registry);
    register_st_geogfromtext(registry);
    register_st_astext(registry);
    register_st_centroid(registry);
    register_st_isvalid(registry);
    register_st_intersects(registry);
    register_st_covers(registry);
    register_st_coveredby(registry);
    register_st_dwithin(registry);
    register_st_distance(registry);
}

fn register_st_point(registry: &mut FunctionRegistry) {
    registry.register(
        "st_point",
        FunctionSignature::new(
            "st_point",
            vec![ValueType::Float, ValueType::Float],
            ValueType::Geography,
            2,
            2,
            true,
            "创建地理点 (经度, 纬度)",
        ),
        |args| {
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
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(NullType::Null))
                }
                _ => Err(ExpressionError::type_error("st_point函数需要数值参数")),
            }
        },
    );
}

fn register_st_geogfromtext(registry: &mut FunctionRegistry) {
    registry.register(
        "st_geogfromtext",
        FunctionSignature::new(
            "st_geogfromtext",
            vec![ValueType::String],
            ValueType::Geography,
            1,
            1,
            true,
            "从WKT文本创建地理对象",
        ),
        |args| {
            match &args[0] {
                Value::String(wkt) => {
                    match Geography::from_wkt(wkt) {
                        Ok(Geography::Point(geo)) => Ok(Value::Geography(geo)),
                        Ok(_) => Err(ExpressionError::type_error("st_geogfromtext目前只支持点类型")),
                        Err(e) => Err(ExpressionError::type_error(&format!("解析WKT失败: {}", e))),
                    }
                }
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("st_geogfromtext函数需要字符串参数")),
            }
        },
    );
}

fn register_st_astext(registry: &mut FunctionRegistry) {
    registry.register(
        "st_astext",
        FunctionSignature::new(
            "st_astext",
            vec![ValueType::Geography],
            ValueType::String,
            1,
            1,
            true,
            "将地理对象转换为WKT文本",
        ),
        |args| {
            match &args[0] {
                Value::Geography(geo) => {
                    let wkt = format!("POINT({} {})", geo.longitude, geo.latitude);
                    Ok(Value::String(wkt))
                }
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("st_astext函数需要地理类型")),
            }
        },
    );
}

fn register_st_centroid(registry: &mut FunctionRegistry) {
    registry.register(
        "st_centroid",
        FunctionSignature::new(
            "st_centroid",
            vec![ValueType::Geography],
            ValueType::Geography,
            1,
            1,
            true,
            "计算地理对象的中心点",
        ),
        |args| {
            match &args[0] {
                Value::Geography(geo) => {
                    Ok(Value::Geography(geo.clone()))
                }
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err(ExpressionError::type_error("st_centroid函数需要地理类型")),
            }
        },
    );
}

fn register_st_isvalid(registry: &mut FunctionRegistry) {
    registry.register(
        "st_isvalid",
        FunctionSignature::new(
            "st_isvalid",
            vec![ValueType::Geography],
            ValueType::Bool,
            1,
            1,
            true,
            "检查地理对象是否有效",
        ),
        |args| {
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
        },
    );
}

fn register_st_intersects(registry: &mut FunctionRegistry) {
    registry.register(
        "st_intersects",
        FunctionSignature::new(
            "st_intersects",
            vec![ValueType::Geography, ValueType::Geography],
            ValueType::Bool,
            2,
            2,
            true,
            "检查两个地理对象是否相交",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::Geography(geo1), Value::Geography(geo2)) => {
                    let distance = geo1.distance(geo2);
                    Ok(Value::Bool(distance < 0.001))
                }
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(NullType::Null))
                }
                _ => Err(ExpressionError::type_error("st_intersects函数需要地理类型参数")),
            }
        },
    );
}

fn register_st_covers(registry: &mut FunctionRegistry) {
    registry.register(
        "st_covers",
        FunctionSignature::new(
            "st_covers",
            vec![ValueType::Geography, ValueType::Geography],
            ValueType::Bool,
            2,
            2,
            true,
            "检查第一个地理对象是否覆盖第二个",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::Geography(geo1), Value::Geography(geo2)) => {
                    let distance = geo1.distance(geo2);
                    Ok(Value::Bool(distance < 0.001))
                }
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(NullType::Null))
                }
                _ => Err(ExpressionError::type_error("st_covers函数需要地理类型参数")),
            }
        },
    );
}

fn register_st_coveredby(registry: &mut FunctionRegistry) {
    registry.register(
        "st_coveredby",
        FunctionSignature::new(
            "st_coveredby",
            vec![ValueType::Geography, ValueType::Geography],
            ValueType::Bool,
            2,
            2,
            true,
            "检查第一个地理对象是否被第二个覆盖",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::Geography(geo1), Value::Geography(geo2)) => {
                    let distance = geo1.distance(geo2);
                    Ok(Value::Bool(distance < 0.001))
                }
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(NullType::Null))
                }
                _ => Err(ExpressionError::type_error("st_coveredby函数需要地理类型参数")),
            }
        },
    );
}

fn register_st_dwithin(registry: &mut FunctionRegistry) {
    registry.register(
        "st_dwithin",
        FunctionSignature::new(
            "st_dwithin",
            vec![ValueType::Geography, ValueType::Geography, ValueType::Float],
            ValueType::Bool,
            3,
            3,
            true,
            "检查两个地理对象是否在指定距离内（单位：公里）",
        ),
        |args| {
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
                _ => Err(ExpressionError::type_error("st_dwithin函数需要地理类型和数值距离参数")),
            }
        },
    );
}

fn register_st_distance(registry: &mut FunctionRegistry) {
    registry.register(
        "st_distance",
        FunctionSignature::new(
            "st_distance",
            vec![ValueType::Geography, ValueType::Geography],
            ValueType::Float,
            2,
            2,
            true,
            "计算两个地理对象之间的距离（单位：公里）",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::Geography(geo1), Value::Geography(geo2)) => {
                    let distance = geo1.distance(geo2);
                    Ok(Value::Float(distance))
                }
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(NullType::Null))
                }
                _ => Err(ExpressionError::type_error("st_distance函数需要地理类型参数")),
            }
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_registry() -> FunctionRegistry {
        let mut registry = FunctionRegistry::new();
        register_all(&mut registry);
        registry
    }

    #[test]
    fn test_st_point() {
        let registry = create_test_registry();
        let result = registry
            .execute("st_point", &[Value::Float(116.4074), Value::Float(39.9042)])
            .expect("st_point函数执行应该成功");
        assert!(matches!(result, Value::Geography(_)));
    }

    #[test]
    fn test_st_distance() {
        let registry = create_test_registry();
        let beijing = Value::Geography(GeographyValue {
            longitude: 116.4074,
            latitude: 39.9042,
        });
        let shanghai = Value::Geography(GeographyValue {
            longitude: 121.4737,
            latitude: 31.2304,
        });
        let result = registry
            .execute("st_distance", &[beijing, shanghai])
            .expect("st_distance函数执行应该成功");
        if let Value::Float(distance) = result {
            assert!(distance > 1000.0 && distance < 1100.0);
        } else {
            panic!("st_distance函数应该返回浮点数");
        }
    }

    #[test]
    fn test_st_isvalid() {
        let registry = create_test_registry();
        let valid_point = Value::Geography(GeographyValue {
            longitude: 116.4074,
            latitude: 39.9042,
        });
        let result = registry
            .execute("st_isvalid", &[valid_point])
            .expect("st_isvalid函数执行应该成功");
        assert_eq!(result, Value::Bool(true));

        let invalid_point = Value::Geography(GeographyValue {
            longitude: 200.0,
            latitude: 39.9042,
        });
        let result = registry
            .execute("st_isvalid", &[invalid_point])
            .expect("st_isvalid函数执行应该成功");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_st_dwithin() {
        let registry = create_test_registry();
        let point1 = Value::Geography(GeographyValue {
            longitude: 116.4074,
            latitude: 39.9042,
        });
        let point2 = Value::Geography(GeographyValue {
            longitude: 116.4075,
            latitude: 39.9043,
        });
        let result = registry
            .execute("st_dwithin", &[point1, point2, Value::Float(1.0)])
            .expect("st_dwithin函数执行应该成功");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_st_astext() {
        let registry = create_test_registry();
        let point = Value::Geography(GeographyValue {
            longitude: 116.4074,
            latitude: 39.9042,
        });
        let result = registry
            .execute("st_astext", &[point])
            .expect("st_astext函数执行应该成功");
        assert_eq!(result, Value::String("POINT(116.4074 39.9042)".to_string()));
    }
}
