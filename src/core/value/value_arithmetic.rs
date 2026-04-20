//! Value Calculation Module
//!
//! This module provides methods for arithmetic, logical, and bitwise operations on values.
use crate::core::value::Value;

impl Value {
    /// Addition operation
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
            (FixedString { data: a, .. }, FixedString { data: b, .. }) => {
                Ok(String(format!("{}{}", a, b)))
            }
            _ => Err("Cannot perform addition on these value types".to_string()),
        }
    }

    /// Subtraction operation
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
            _ => Err("Cannot perform subtraction on these value types".to_string()),
        }
    }

    /// Multiplication operation
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
            _ => Err("Cannot perform multiplication on these value types".to_string()),
        }
    }

    /// Division operation
    pub fn div(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => {
                if *b == 0 {
                    Err("Division by zero".to_string())
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
            _ => Err("Cannot perform division on these value types".to_string()),
        }
    }

    /// Modular operation
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
            _ => Err("Modulo operation is only supported for integer types".to_string()),
        }
    }

    /// Power operation
    pub fn pow(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => {
                if *b < 0 {
                    Err("Negative exponent not supported for integer power operation".to_string())
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
            (UInt32(a), UInt32(b)) => Ok(UInt32(a.pow(*b))),
            (UInt64(a), UInt64(b)) => Ok(UInt64(a.pow(*b as u32))),
            (Float(a), Float(b)) => Ok(Float(a.powf(*b))),
            (Int(a), Float(b)) => Ok(Float((*a as f64).powf(*b))),
            (Float(a), Int(b)) => Ok(Float(a.powi(*b as i32))),
            _ => Err("Cannot perform power operation on these value types".to_string()),
        }
    }

    /// Negation operation
    pub fn neg(&self) -> Result<Value, String> {
        use Value::*;
        match self {
            Int(a) => Ok(Int(-a)),
            Int8(a) => Ok(Int8(-a)),
            Int16(a) => Ok(Int16(-a)),
            Int32(a) => Ok(Int32(-a)),
            Int64(a) => Ok(Int64(-a)),
            Float(a) => Ok(Float(-a)),
            _ => Err("Negation is only supported for numeric types".to_string()),
        }
    }

    /// Logic and Operations
    pub fn and(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Bool(a), Bool(b)) => Ok(Bool(*a && *b)),
            _ => Err("Logical AND is only supported for boolean types".to_string()),
        }
    }

    /// Logical OR operation
    pub fn or(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Bool(a), Bool(b)) => Ok(Bool(*a || *b)),
            _ => Err("Logical OR is only supported for boolean types".to_string()),
        }
    }

    /// Logical NOT operation
    pub fn not(&self) -> Result<Value, String> {
        use Value::*;
        match self {
            Bool(a) => Ok(Bool(!a)),
            _ => Err("Logical NOT is only supported for boolean types".to_string()),
        }
    }

    /// Bitwise AND operation
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
            _ => Err("Bitwise AND is only supported for integer types".to_string()),
        }
    }

    /// Bitwise OR operation
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
            _ => Err("Bitwise OR is only supported for integer types".to_string()),
        }
    }

    /// Bitwise XOR operation
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
            _ => Err("Bitwise XOR is only supported for integer types".to_string()),
        }
    }

    /// Left shift operation
    pub fn bit_shl(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => {
                if *b < 0 {
                    Err("Left shift count cannot be negative".to_string())
                } else if *b >= 64 {
                    Err("Left shift count out of range".to_string())
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
                    Ok(UInt32(a << *b))
                }
            }
            (UInt64(a), UInt64(b)) => {
                if *b >= 64 {
                    Err("左移位数超出范围".to_string())
                } else {
                    Ok(UInt64(a << *b as u32))
                }
            }
            _ => Err("Bitwise left shift is only supported for integer types".to_string()),
        }
    }

    /// Right-shift operation
    pub fn bit_shr(&self, other: &Value) -> Result<Value, String> {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => {
                if *b < 0 {
                    Err("Right shift count cannot be negative".to_string())
                } else if *b >= 64 {
                    Err("Right shift count out of range".to_string())
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
                    Ok(UInt32(a >> *b))
                }
            }
            (UInt64(a), UInt64(b)) => {
                if *b >= 64 {
                    Err("右移位数超出范围".to_string())
                } else {
                    Ok(UInt64(a >> *b as u32))
                }
            }
            _ => Err("Bitwise right shift is only supported for integer types".to_string()),
        }
    }

    /// Bitwise NOT operation
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
            _ => Err("Bitwise NOT is only supported for integer types".to_string()),
        }
    }

    /// Absolute value operation
    pub fn abs(&self) -> Result<Value, String> {
        use Value::*;
        match self {
            Int(a) => Ok(Int(a.abs())),
            Int8(a) => Ok(Int8(a.abs())),
            Int16(a) => Ok(Int16(a.abs())),
            Int32(a) => Ok(Int32(a.abs())),
            Int64(a) => Ok(Int64(a.abs())),
            UInt8(_) | UInt16(_) | UInt32(_) | UInt64(_) => Ok(self.clone()),
            Float(a) => Ok(Float(a.abs())),
            _ => Err("Absolute value is only supported for numeric types".to_string()),
        }
    }

    /// Length operation
    pub fn length(&self) -> Result<Value, String> {
        use Value::*;
        match self {
            String(s) => Ok(Int(s.len() as i64)),
            FixedString { data, .. } => Ok(Int(data.len() as i64)),
            List(l) => Ok(Int(l.values.len() as i64)),
            Map(m) => Ok(Int(m.len() as i64)),
            Set(s) => Ok(Int(s.len() as i64)),
            _ => Err("Length operation is only supported for string, list, map, or set types".to_string()),
        }
    }
}
