use super::types::Value;

impl Value {
    /// 加法运算
    pub fn add(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Int(a + b)),
            (Float(a), Float(b)) => Ok(Float(a + b)),
            (Int(a), Float(b)) => Ok(Float(*a as f64 + b)),
            (Float(a), Int(b)) => Ok(Float(a + *b as f64)),
            (String(a), String(b)) => Ok(String(format!("{}{}", a, b))),
            _ => Err("无法对这些类型的值进行加法运算".to_string()),
        }
    }

    /// 减法运算
    pub fn sub(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Int(a - b)),
            (Float(a), Float(b)) => Ok(Float(a - b)),
            (Int(a), Float(b)) => Ok(Float(*a as f64 - b)),
            (Float(a), Int(b)) => Ok(Float(a - *b as f64)),
            _ => Err("无法对这些值进行减法运算".to_string()),
        }
    }

    /// 乘法运算
    pub fn mul(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Int(a * b)),
            (Float(a), Float(b)) => Ok(Float(a * b)),
            (Int(a), Float(b)) => Ok(Float(*a as f64 * b)),
            (Float(a), Int(b)) => Ok(Float(a * *b as f64)),
            _ => Err("无法对这些类型的值进行乘法运算".to_string()),
        }
    }

    /// 除法运算
    pub fn div(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => {
                if *b == 0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(Int(a / b))
                }
            }
            (Float(a), Float(b)) => {
                if *b == 0.0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(Float(a / b))
                }
            }
            (Int(a), Float(b)) => {
                if *b == 0.0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(Float(*a as f64 / b))
                }
            }
            (Float(a), Int(b)) => {
                if *b == 0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(Float(a / *b as f64))
                }
            }
            _ => Err("无法对这些类型的值进行除法运算".to_string()),
        }
    }

    /// 取模运算
    pub fn rem(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => {
                if *b == 0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(Int(a % b))
                }
            }
            _ => Err("只能对整数类型进行取模运算".to_string()),
        }
    }

    /// 幂运算
    pub fn pow(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => {
                if *b < 0 {
                    Err("负指数不支持整数幂运算".to_string())
                } else {
                    Ok(Int(a.pow(*b as u32)))
                }
            }
            (Float(a), Float(b)) => Ok(Float(a.powf(*b))),
            (Int(a), Float(b)) => Ok(Float((*a as f64).powf(*b))),
            (Float(a), Int(b)) => Ok(Float(a.powi(*b as i32))),
            _ => Err("无法对这些类型的值进行幂运算".to_string()),
        }
    }

    /// 取负运算
    pub fn neg(&self) -> Result<Value, String> {
        use Value::*;
        match self {
            Int(a) => Ok(Int(-a)),
            Float(a) => Ok(Float(-a)),
            _ => Err("只能对数值类型进行取负运算".to_string()),
        }
    }

    /// 逻辑与运算
    pub fn and(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Bool(a), Bool(b)) => Ok(Bool(*a && *b)),
            _ => Err("只能对布尔类型进行逻辑与运算".to_string()),
        }
    }

    /// 逻辑或运算
    pub fn or(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Bool(a), Bool(b)) => Ok(Bool(*a || *b)),
            _ => Err("只能对布尔类型进行逻辑或运算".to_string()),
        }
    }

    /// 逻辑非运算
    pub fn not(&self) -> Result<Value, String> {
        use Value::*;
        match self {
            Bool(a) => Ok(Bool(!a)),
            _ => Err("只能对布尔类型进行逻辑非运算".to_string()),
        }
    }
}
