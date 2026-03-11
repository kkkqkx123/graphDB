//! 值运算模块
//!
//! 此模块提供值的算术、逻辑和位运算方法。
use super::types::Value;

impl Value {
    /// 加法运算
    pub fn add(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Int(a + b)),
            (Int8(a), Int8(b)) => Ok(Int8(a.wrapping_add(*b))),
            (Int16(a), Int16(b)) => Ok(Int16(a.wrapping_add(*b))),
            (Int32(a), Int32(b)) => Ok(Int32(a.wrapping_add(*b))),
            (Int64(a), Int64(b)) => Ok(Int64(a.wrapping_add(*b))),
            (UInt8(a), UInt8(b)) => Ok(UInt8(a.wrapping_add(*b))),
            (UInt16(a), UInt16(b)) => Ok(UInt16(a.wrapping_add(*b))),
            (UInt32(a), UInt32(b)) => Ok(UInt32(a.wrapping_add(*b))),
            (UInt64(a), UInt64(b)) => Ok(UInt64(a.wrapping_add(*b))),
            (Float(a), Float(b)) => Ok(Float(a + b)),
            (Int(a), Float(b)) => Ok(Float(*a as f64 + b)),
            (Float(a), Int(b)) => Ok(Float(a + *b as f64)),
            (String(a), String(b)) => Ok(String(format!("{}{}", a, b))),
            (String(a), FixedString { data: b, .. }) => Ok(String(format!("{}{}", a, b))),
            (FixedString { data: a, .. }, String(b)) => Ok(String(format!("{}{}", a, b))),
            (FixedString { data: a, .. }, FixedString { data: b, .. }) => Ok(String(format!("{}{}", a, b))),
            _ => Err("无法对这些类型的值进行加法运算".to_string()),
        }
    }

    /// 减法运算
    pub fn sub(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Int(a - b)),
            (Int8(a), Int8(b)) => Ok(Int8(a.wrapping_sub(*b))),
            (Int16(a), Int16(b)) => Ok(Int16(a.wrapping_sub(*b))),
            (Int32(a), Int32(b)) => Ok(Int32(a.wrapping_sub(*b))),
            (Int64(a), Int64(b)) => Ok(Int64(a.wrapping_sub(*b))),
            (UInt8(a), UInt8(b)) => Ok(UInt8(a.wrapping_sub(*b))),
            (UInt16(a), UInt16(b)) => Ok(UInt16(a.wrapping_sub(*b))),
            (UInt32(a), UInt32(b)) => Ok(UInt32(a.wrapping_sub(*b))),
            (UInt64(a), UInt64(b)) => Ok(UInt64(a.wrapping_sub(*b))),
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
            (Int8(a), Int8(b)) => Ok(Int8(a.wrapping_mul(*b))),
            (Int16(a), Int16(b)) => Ok(Int16(a.wrapping_mul(*b))),
            (Int32(a), Int32(b)) => Ok(Int32(a.wrapping_mul(*b))),
            (Int64(a), Int64(b)) => Ok(Int64(a.wrapping_mul(*b))),
            (UInt8(a), UInt8(b)) => Ok(UInt8(a.wrapping_mul(*b))),
            (UInt16(a), UInt16(b)) => Ok(UInt16(a.wrapping_mul(*b))),
            (UInt32(a), UInt32(b)) => Ok(UInt32(a.wrapping_mul(*b))),
            (UInt64(a), UInt64(b)) => Ok(UInt64(a.wrapping_mul(*b))),
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
            (Int8(a), Int8(b)) => {
                if *b == 0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(Int8(a / b))
                }
            }
            (Int16(a), Int16(b)) => {
                if *b == 0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(Int16(a / b))
                }
            }
            (Int32(a), Int32(b)) => {
                if *b == 0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(Int32(a / b))
                }
            }
            (Int64(a), Int64(b)) => {
                if *b == 0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(Int64(a / b))
                }
            }
            (UInt8(a), UInt8(b)) => {
                if *b == 0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(UInt8(a / b))
                }
            }
            (UInt16(a), UInt16(b)) => {
                if *b == 0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(UInt16(a / b))
                }
            }
            (UInt32(a), UInt32(b)) => {
                if *b == 0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(UInt32(a / b))
                }
            }
            (UInt64(a), UInt64(b)) => {
                if *b == 0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(UInt64(a / b))
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
            (Int8(a), Int8(b)) => {
                if *b == 0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(Int8(a % b))
                }
            }
            (Int16(a), Int16(b)) => {
                if *b == 0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(Int16(a % b))
                }
            }
            (Int32(a), Int32(b)) => {
                if *b == 0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(Int32(a % b))
                }
            }
            (Int64(a), Int64(b)) => {
                if *b == 0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(Int64(a % b))
                }
            }
            (UInt8(a), UInt8(b)) => {
                if *b == 0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(UInt8(a % b))
                }
            }
            (UInt16(a), UInt16(b)) => {
                if *b == 0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(UInt16(a % b))
                }
            }
            (UInt32(a), UInt32(b)) => {
                if *b == 0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(UInt32(a % b))
                }
            }
            (UInt64(a), UInt64(b)) => {
                if *b == 0 {
                    Err("除零错误".to_string())
                } else {
                    Ok(UInt64(a % b))
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
            (Int8(a), Int8(b)) => {
                if *b < 0 {
                    Err("负指数不支持整数幂运算".to_string())
                } else {
                    Ok(Int8(a.pow(*b as u32)))
                }
            }
            (Int16(a), Int16(b)) => {
                if *b < 0 {
                    Err("负指数不支持整数幂运算".to_string())
                } else {
                    Ok(Int16(a.pow(*b as u32)))
                }
            }
            (Int32(a), Int32(b)) => {
                if *b < 0 {
                    Err("负指数不支持整数幂运算".to_string())
                } else {
                    Ok(Int32(a.pow(*b as u32)))
                }
            }
            (Int64(a), Int64(b)) => {
                if *b < 0 {
                    Err("负指数不支持整数幂运算".to_string())
                } else {
                    Ok(Int64(a.pow(*b as u32)))
                }
            }
            (UInt8(a), UInt8(b)) => Ok(UInt8(a.pow(*b as u32))),
            (UInt16(a), UInt16(b)) => Ok(UInt16(a.pow(*b as u32))),
            (UInt32(a), UInt32(b)) => Ok(UInt32(a.pow(*b as u32))),
            (UInt64(a), UInt64(b)) => Ok(UInt64(a.pow(*b as u32))),
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
            Int8(a) => Ok(Int8(-a)),
            Int16(a) => Ok(Int16(-a)),
            Int32(a) => Ok(Int32(-a)),
            Int64(a) => Ok(Int64(-a)),
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

    /// 位与运算
    pub fn bit_and(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Int(a & b)),
            (Int8(a), Int8(b)) => Ok(Int8(a & b)),
            (Int16(a), Int16(b)) => Ok(Int16(a & b)),
            (Int32(a), Int32(b)) => Ok(Int32(a & b)),
            (Int64(a), Int64(b)) => Ok(Int64(a & b)),
            (UInt8(a), UInt8(b)) => Ok(UInt8(a & b)),
            (UInt16(a), UInt16(b)) => Ok(UInt16(a & b)),
            (UInt32(a), UInt32(b)) => Ok(UInt32(a & b)),
            (UInt64(a), UInt64(b)) => Ok(UInt64(a & b)),
            _ => Err("只能对整数类型进行位与运算".to_string()),
        }
    }

    /// 位或运算
    pub fn bit_or(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Int(a | b)),
            (Int8(a), Int8(b)) => Ok(Int8(a | b)),
            (Int16(a), Int16(b)) => Ok(Int16(a | b)),
            (Int32(a), Int32(b)) => Ok(Int32(a | b)),
            (Int64(a), Int64(b)) => Ok(Int64(a | b)),
            (UInt8(a), UInt8(b)) => Ok(UInt8(a | b)),
            (UInt16(a), UInt16(b)) => Ok(UInt16(a | b)),
            (UInt32(a), UInt32(b)) => Ok(UInt32(a | b)),
            (UInt64(a), UInt64(b)) => Ok(UInt64(a | b)),
            _ => Err("只能对整数类型进行位或运算".to_string()),
        }
    }

    /// 位异或运算
    pub fn bit_xor(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => Ok(Int(a ^ b)),
            (Int8(a), Int8(b)) => Ok(Int8(a ^ b)),
            (Int16(a), Int16(b)) => Ok(Int16(a ^ b)),
            (Int32(a), Int32(b)) => Ok(Int32(a ^ b)),
            (Int64(a), Int64(b)) => Ok(Int64(a ^ b)),
            (UInt8(a), UInt8(b)) => Ok(UInt8(a ^ b)),
            (UInt16(a), UInt16(b)) => Ok(UInt16(a ^ b)),
            (UInt32(a), UInt32(b)) => Ok(UInt32(a ^ b)),
            (UInt64(a), UInt64(b)) => Ok(UInt64(a ^ b)),
            _ => Err("只能对整数类型进行位异或运算".to_string()),
        }
    }

    /// 位左移运算
    pub fn bit_shl(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => {
                if *b < 0 {
                    Err("左移位数不能为负数".to_string())
                } else if *b >= 64 {
                    Err("左移位数超出范围".to_string())
                } else {
                    Ok(Int(a << *b as u32))
                }
            }
            (Int8(a), Int8(b)) => {
                if *b < 0 {
                    Err("左移位数不能为负数".to_string())
                } else if *b >= 8 {
                    Err("左移位数超出范围".to_string())
                } else {
                    Ok(Int8(a << *b as u32))
                }
            }
            (Int16(a), Int16(b)) => {
                if *b < 0 {
                    Err("左移位数不能为负数".to_string())
                } else if *b >= 16 {
                    Err("左移位数超出范围".to_string())
                } else {
                    Ok(Int16(a << *b as u32))
                }
            }
            (Int32(a), Int32(b)) => {
                if *b < 0 {
                    Err("左移位数不能为负数".to_string())
                } else if *b >= 32 {
                    Err("左移位数超出范围".to_string())
                } else {
                    Ok(Int32(a << *b as u32))
                }
            }
            (Int64(a), Int64(b)) => {
                if *b < 0 {
                    Err("左移位数不能为负数".to_string())
                } else if *b >= 64 {
                    Err("左移位数超出范围".to_string())
                } else {
                    Ok(Int64(a << *b as u32))
                }
            }
            (UInt8(a), UInt8(b)) => {
                if *b >= 8 {
                    Err("左移位数超出范围".to_string())
                } else {
                    Ok(UInt8(a << *b as u32))
                }
            }
            (UInt16(a), UInt16(b)) => {
                if *b >= 16 {
                    Err("左移位数超出范围".to_string())
                } else {
                    Ok(UInt16(a << *b as u32))
                }
            }
            (UInt32(a), UInt32(b)) => {
                if *b >= 32 {
                    Err("左移位数超出范围".to_string())
                } else {
                    Ok(UInt32(a << *b as u32))
                }
            }
            (UInt64(a), UInt64(b)) => {
                if *b >= 64 {
                    Err("左移位数超出范围".to_string())
                } else {
                    Ok(UInt64(a << *b as u32))
                }
            }
            _ => Err("只能对整数类型进行位左移运算".to_string()),
        }
    }

    /// 位右移运算
    pub fn bit_shr(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => {
                if *b < 0 {
                    Err("右移位数不能为负数".to_string())
                } else if *b >= 64 {
                    Err("右移位数超出范围".to_string())
                } else {
                    Ok(Int(a >> *b as u32))
                }
            }
            (Int8(a), Int8(b)) => {
                if *b < 0 {
                    Err("右移位数不能为负数".to_string())
                } else if *b >= 8 {
                    Err("右移位数超出范围".to_string())
                } else {
                    Ok(Int8(a >> *b as u32))
                }
            }
            (Int16(a), Int16(b)) => {
                if *b < 0 {
                    Err("右移位数不能为负数".to_string())
                } else if *b >= 16 {
                    Err("右移位数超出范围".to_string())
                } else {
                    Ok(Int16(a >> *b as u32))
                }
            }
            (Int32(a), Int32(b)) => {
                if *b < 0 {
                    Err("右移位数不能为负数".to_string())
                } else if *b >= 32 {
                    Err("右移位数超出范围".to_string())
                } else {
                    Ok(Int32(a >> *b as u32))
                }
            }
            (Int64(a), Int64(b)) => {
                if *b < 0 {
                    Err("右移位数不能为负数".to_string())
                } else if *b >= 64 {
                    Err("右移位数超出范围".to_string())
                } else {
                    Ok(Int64(a >> *b as u32))
                }
            }
            (UInt8(a), UInt8(b)) => {
                if *b >= 8 {
                    Err("右移位数超出范围".to_string())
                } else {
                    Ok(UInt8(a >> *b as u32))
                }
            }
            (UInt16(a), UInt16(b)) => {
                if *b >= 16 {
                    Err("右移位数超出范围".to_string())
                } else {
                    Ok(UInt16(a >> *b as u32))
                }
            }
            (UInt32(a), UInt32(b)) => {
                if *b >= 32 {
                    Err("右移位数超出范围".to_string())
                } else {
                    Ok(UInt32(a >> *b as u32))
                }
            }
            (UInt64(a), UInt64(b)) => {
                if *b >= 64 {
                    Err("右移位数超出范围".to_string())
                } else {
                    Ok(UInt64(a >> *b as u32))
                }
            }
            _ => Err("只能对整数类型进行位右移运算".to_string()),
        }
    }

    /// 位取反运算
    pub fn bit_not(&self) -> Result<Value, String> {
        use Value::*;
        match self {
            Int(a) => Ok(Int(!a)),
            Int8(a) => Ok(Int8(!a)),
            Int16(a) => Ok(Int16(!a)),
            Int32(a) => Ok(Int32(!a)),
            Int64(a) => Ok(Int64(!a)),
            UInt8(a) => Ok(UInt8(!a)),
            UInt16(a) => Ok(UInt16(!a)),
            UInt32(a) => Ok(UInt32(!a)),
            UInt64(a) => Ok(UInt64(!a)),
            _ => Err("只能对整数类型进行位取反运算".to_string()),
        }
    }
}
