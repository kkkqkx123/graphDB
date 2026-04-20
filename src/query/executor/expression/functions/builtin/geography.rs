//! Implementation of geospatial functions

use crate::core::error::ExpressionError;
use crate::core::value::geography::GeographyValue;
use crate::core::value::NullType;
use crate::core::Value;

define_function_enum! {
    /// Enumeration of geospatial functions
    pub enum GeographyFunction {
        StPoint => {
            name: "st_point",
            arity: 2,
            variadic: false,
            description: "Create Geographic Points (Longitude, Latitude)",
            handler: execute_st_point
        },
        StGeogFromText => {
            name: "st_geogfromtext",
            arity: 1,
            variadic: false,
            description: "Creating geographic objects from WKT text",
            handler: execute_st_geogfromtext
        },
        StAsText => {
            name: "st_astext",
            arity: 1,
            variadic: false,
            description: "Convert geographic objects to WKT text",
            handler: execute_st_astext
        },
        StCentroid => {
            name: "st_centroid",
            arity: 1,
            variadic: false,
            description: "Calculate the center point of a geographic object",
            handler: execute_st_centroid
        },
        StIsValid => {
            name: "st_isvalid",
            arity: 1,
            variadic: false,
            description: "Checking the validity of geographic objects",
            handler: execute_st_isvalid
        },
        StIntersects => {
            name: "st_intersects",
            arity: 2,
            variadic: false,
            description: "Check if two geographic objects intersect",
            handler: execute_st_intersects
        },
        StCovers => {
            name: "st_covers",
            arity: 2,
            variadic: false,
            description: "Check if the first geographic object overrides the second",
            handler: execute_st_covers
        },
        StCoveredBy => {
            name: "st_coveredby",
            arity: 2,
            variadic: false,
            description: "Check if the first geographic object is overwritten by the second",
            handler: execute_st_coveredby
        },
        StDWithin => {
            name: "st_dwithin",
            arity: 3,
            variadic: false,
            description: "Check that two geographic objects are within the specified distance (in kilometers)",
            handler: execute_st_dwithin
        },
        StDistance => {
            name: "st_distance",
            arity: 2,
            variadic: false,
            description: "Calculation of the distance between two geographical objects (in kilometers)",
            handler: execute_st_distance
        },
    }
}

fn execute_st_point(args: &[Value]) -> Result<Value, ExpressionError> {
    match (&args[0], &args[1]) {
        (Value::Float(lon), Value::Float(lat)) => {
            let geo = GeographyValue {
                longitude: *lon as f64,
                latitude: *lat as f64,
            };
            Ok(Value::Geography(geo))
        }
        (Value::Double(lon), Value::Double(lat)) => {
            let geo = GeographyValue {
                longitude: *lon,
                latitude: *lat,
            };
            Ok(Value::Geography(geo))
        }
        (Value::SmallInt(lon), Value::SmallInt(lat)) => {
            let geo = GeographyValue {
                longitude: *lon as f64,
                latitude: *lat as f64,
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
        (Value::BigInt(lon), Value::BigInt(lat)) => {
            let geo = GeographyValue {
                longitude: *lon as f64,
                latitude: *lat as f64,
            };
            Ok(Value::Geography(geo))
        }
        (Value::Float(lon), Value::Double(lat)) => {
            let geo = GeographyValue {
                longitude: *lon as f64,
                latitude: *lat,
            };
            Ok(Value::Geography(geo))
        }
        (Value::Double(lon), Value::Float(lat)) => {
            let geo = GeographyValue {
                longitude: *lon,
                latitude: *lat as f64,
            };
            Ok(Value::Geography(geo))
        }
        (Value::Null(_), _) | (_, Value::Null(_)) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("The st_point function takes numeric arguments")),
    }
}

fn execute_st_geogfromtext(args: &[Value]) -> Result<Value, ExpressionError> {
    use crate::core::value::geography::Geography;
    match &args[0] {
        Value::String(wkt) => match Geography::from_wkt(wkt) {
            Ok(Geography::Point(geo)) => Ok(Value::Geography(geo)),
            Err(e) => Err(ExpressionError::type_error(format!(
                "Failed to parse WKT: {}",
                e
            ))),
        },
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error(
            "The st_geogfromtext function takes string arguments",
        )),
    }
}

fn execute_st_astext(args: &[Value]) -> Result<Value, ExpressionError> {
    match &args[0] {
        Value::Geography(geo) => {
            let wkt = format!("POINT({} {})", geo.longitude, geo.latitude);
            Ok(Value::String(wkt))
        }
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("The st_astext function requires the geographic type")),
    }
}

fn execute_st_centroid(args: &[Value]) -> Result<Value, ExpressionError> {
    match &args[0] {
        Value::Geography(geo) => Ok(Value::Geography(geo.clone())),
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("The st_centroid function requires the geography type")),
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
        _ => Err(ExpressionError::type_error("The st_isvalid function requires the geography type")),
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
    |geo1: &GeographyValue, geo2: &GeographyValue| { Ok(Value::Double(geo1.distance(geo2))) },
    "st_distance"
);

fn execute_st_dwithin(args: &[Value]) -> Result<Value, ExpressionError> {
    match (&args[0], &args[1], &args[2]) {
        (Value::Geography(geo1), Value::Geography(geo2), Value::Float(distance)) => {
            let actual_distance = geo1.distance(geo2);
            Ok(Value::Bool(actual_distance <= *distance as f64))
        }
        (Value::Geography(geo1), Value::Geography(geo2), Value::Double(distance)) => {
            let actual_distance = geo1.distance(geo2);
            Ok(Value::Bool(actual_distance <= *distance))
        }
        (Value::Geography(geo1), Value::Geography(geo2), Value::Int(distance)) => {
            let actual_distance = geo1.distance(geo2);
            Ok(Value::Bool(actual_distance <= *distance as f64))
        }
        (Value::Geography(geo1), Value::Geography(geo2), Value::BigInt(distance)) => {
            let actual_distance = geo1.distance(geo2);
            Ok(Value::Bool(actual_distance <= *distance as f64))
        }
        (Value::Null(_), _, _) | (_, Value::Null(_), _) | (_, _, Value::Null(_)) => {
            Ok(Value::Null(NullType::Null))
        }
        _ => Err(ExpressionError::type_error(
            "The st_dwithin function requires geotype and numeric distance parameters",
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
            .expect("Implementation should not fail");
        assert!(matches!(result, Value::Geography(_)));
    }

    #[test]
    fn test_st_isvalid() {
        let func = GeographyFunction::StIsValid;
        let geo = GeographyValue {
            longitude: 116.4074,
            latitude: 39.9042,
        };
        let result = func
            .execute(&[Value::Geography(geo)])
            .expect("Implementation should not fail");
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
        let result = func
            .execute(&[Value::Geography(geo1), Value::Geography(geo2)])
            .expect("Implementation should not fail");
        assert!(matches!(result, Value::Float(_)));
    }

    #[test]
    fn test_null_handling() {
        let func = GeographyFunction::StIsValid;
        let result = func
            .execute(&[Value::Null(NullType::Null)])
            .expect("Implementation should not fail");
        assert_eq!(result, Value::Null(NullType::Null));
    }
}
