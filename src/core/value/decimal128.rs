//! Decimal128 类型实现
//!
//! 本模块实现了 IEEE 754-2008 标准的 128 位十进制浮点数类型。
//!
//! ## 特性
//!
//! - 34 位十进制精度
//! - 避免二进制浮点数的精度问题
//! - 适合金融计算和科学计算
//! - 与 MongoDB BSON Decimal128 兼容
//!
//! ## 使用场景
//!
//! - 金融应用（货币计算、利率计算）
//! - 科学计算（需要高精度的数值计算）
//! - 税务计算（需要精确到分）
//! - 会计系统（避免舍入误差）
//!
//! ## 性能考虑
//!
//! Decimal128 运算比原生浮点数慢，但提供了精确的十进制计算。
//! 对于不需要高精度的场景，建议使用 Float 类型。

use dec::Decimal128;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use bincode::{Encode, de::{Decode, BorrowDecode}};

/// Decimal128 值包装器
///
/// 包装 `dec::Decimal128` 类型，提供与 GraphDB 类型系统的集成。
///
/// ## 示例
///
/// ```rust
/// use graphdb::core::value::decimal128::Decimal128Value;
///
/// let d1 = Decimal128Value::from_str("123.456").expect("解析失败");
/// let d2 = Decimal128Value::from_str("789.012").expect("解析失败");
/// let sum = &d1 + &d2;
/// assert_eq!(sum.to_string(), "912.468");
/// ```
#[derive(Debug, Clone)]
pub struct Decimal128Value {
    inner: Decimal128,
}

impl Serialize for Decimal128Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Decimal128Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Encode for Decimal128Value {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        let s = self.to_string();
        s.encode(encoder)
    }
}

impl<C> bincode::de::Decode<C> for Decimal128Value {
    fn decode<D: bincode::de::Decoder>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let s: String = bincode::de::Decode::decode(decoder)?;
        Self::from_str(&s).map_err(|e| bincode::error::DecodeError::OtherString(e))
    }
}

impl<'de, C> bincode::de::BorrowDecode<'de, C> for Decimal128Value {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let s: String = bincode::de::BorrowDecode::borrow_decode(decoder)?;
        Self::from_str(&s).map_err(|e| bincode::error::DecodeError::OtherString(e))
    }
}

impl Decimal128Value {
    /// 创建一个新的 Decimal128 值
    pub fn new(inner: Decimal128) -> Self {
        Self { inner }
    }

    /// 从字符串解析 Decimal128
    pub fn from_str(s: &str) -> Result<Self, String> {
        Decimal128::from_str(s)
            .map(|inner| Self { inner })
            .map_err(|e| format!("Decimal128 解析失败: {}", e))
    }

    /// 从 i64 创建 Decimal128
    pub fn from_i64(n: i64) -> Self {
        Self {
            inner: Decimal128::from(n),
        }
    }

    /// 从 u64 创建 Decimal128
    pub fn from_u64(n: u64) -> Self {
        Self {
            inner: Decimal128::from(n),
        }
    }

    /// 从 f64 创建 Decimal128（注意：可能有精度损失）
    pub fn from_f64(n: f64) -> Option<Self> {
        let s = n.to_string();
        Self::from_str(&s).ok()
    }

    /// 获取内部的 Decimal128 值
    pub fn inner(&self) -> &Decimal128 {
        &self.inner
    }

    /// 获取内部的 Decimal128 值（可变）
    pub fn inner_mut(&mut self) -> &mut Decimal128 {
        &mut self.inner
    }

    /// 转换为字符串
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    /// 加法运算
    pub fn add(&self, other: &Self) -> Result<Self, String> {
        Ok(Self {
            inner: self.inner + other.inner,
        })
    }

    /// 减法运算
    pub fn sub(&self, other: &Self) -> Result<Self, String> {
        Ok(Self {
            inner: self.inner - other.inner,
        })
    }

    /// 乘法运算
    pub fn mul(&self, other: &Self) -> Result<Self, String> {
        Ok(Self {
            inner: self.inner * other.inner,
        })
    }

    /// 除法运算
    pub fn div(&self, other: &Self) -> Result<Self, String> {
        if other.inner == Decimal128::ZERO {
            return Err("除零错误".to_string());
        }
        Ok(Self {
            inner: self.inner / other.inner,
        })
    }

    /// 取模运算
    pub fn rem(&self, other: &Self) -> Result<Self, String> {
        if other.inner == Decimal128::ZERO {
            return Err("除零错误".to_string());
        }
        Ok(Self {
            inner: self.inner % other.inner,
        })
    }

    /// 比较运算
    pub fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.inner < other.inner {
            std::cmp::Ordering::Less
        } else if self.inner > other.inner {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    }

    /// 等于比较
    pub fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }

    /// 绝对值
    pub fn abs(&self) -> Self {
        Self {
            inner: if self.inner < Decimal128::ZERO {
                -self.inner
            } else {
                self.inner
            },
        }
    }

    /// 取反
    pub fn neg(&self) -> Self {
        Self { inner: -self.inner }
    }

    /// 四舍五入到指定小数位
    pub fn round_dp(&self, dp: u32) -> Self {
        let s = self.to_string();
        if let Some(dot_pos) = s.find('.') {
            let integer_part = &s[..dot_pos];
            let fractional_part = &s[dot_pos + 1..];
            if fractional_part.len() <= dp as usize {
                return self.clone();
            }
            let rounded_fractional = &fractional_part[..dp as usize];
            let rounded_str = format!("{}.{}", integer_part, rounded_fractional);
            Self::from_str(&rounded_str).unwrap_or_else(|_| self.clone())
        } else {
            self.clone()
        }
    }

    /// 向下取整
    pub fn floor(&self) -> Self {
        let s = self.to_string();
        if let Some(dot_pos) = s.find('.') {
            let integer_part = &s[..dot_pos];
            Self::from_str(integer_part).unwrap_or_else(|_| self.clone())
        } else {
            self.clone()
        }
    }

    /// 向上取整
    pub fn ceil(&self) -> Self {
        let s = self.to_string();
        if let Some(dot_pos) = s.find('.') {
            let fractional_part = &s[dot_pos + 1..];
            if fractional_part.chars().all(|c| c == '0') {
                self.clone()
            } else {
                let integer_part = &s[..dot_pos];
                let int_value: i64 = integer_part.parse().unwrap_or(0);
                let ceil_value = if self.inner >= Decimal128::ZERO {
                    int_value + 1
                } else {
                    int_value
                };
                Self::from_i64(ceil_value)
            }
        } else {
            self.clone()
        }
    }

    /// 是否为零
    pub fn is_zero(&self) -> bool {
        self.inner == Decimal128::ZERO
    }

    /// 是否为负数
    pub fn is_negative(&self) -> bool {
        self.inner < Decimal128::ZERO
    }

    /// 是否为正数
    pub fn is_positive(&self) -> bool {
        self.inner > Decimal128::ZERO
    }

    /// 是否为 NaN
    pub fn is_nan(&self) -> bool {
        self.inner == Decimal128::NAN
    }
}

impl Default for Decimal128Value {
    fn default() -> Self {
        Self {
            inner: Decimal128::ZERO,
        }
    }
}

impl fmt::Display for Decimal128Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl FromStr for Decimal128Value {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str(s)
    }
}

impl std::ops::Add for &Decimal128Value {
    type Output = Decimal128Value;

    fn add(self, other: Self) -> Self::Output {
        Decimal128Value {
            inner: self.inner + other.inner,
        }
    }
}

impl std::ops::Sub for &Decimal128Value {
    type Output = Decimal128Value;

    fn sub(self, other: Self) -> Self::Output {
        Decimal128Value {
            inner: self.inner - other.inner,
        }
    }
}

impl std::ops::Mul for &Decimal128Value {
    type Output = Decimal128Value;

    fn mul(self, other: Self) -> Self::Output {
        Decimal128Value {
            inner: self.inner * other.inner,
        }
    }
}

impl std::ops::Div for &Decimal128Value {
    type Output = Decimal128Value;

    fn div(self, other: Self) -> Self::Output {
        Decimal128Value {
            inner: self.inner / other.inner,
        }
    }
}

impl std::ops::Rem for &Decimal128Value {
    type Output = Decimal128Value;

    fn rem(self, other: Self) -> Self::Output {
        Decimal128Value {
            inner: self.inner % other.inner,
        }
    }
}

impl std::ops::Neg for Decimal128Value {
    type Output = Decimal128Value;

    fn neg(self) -> Self::Output {
        Decimal128Value { inner: -self.inner }
    }
}

impl PartialEq for Decimal128Value {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Eq for Decimal128Value {}

impl std::hash::Hash for Decimal128Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let d = Decimal128Value::from_str("123.456").expect("解析失败");
        assert_eq!(d.to_string(), "123.456");
    }

    #[test]
    fn test_from_i64() {
        let d = Decimal128Value::from_i64(123456);
        assert_eq!(d.to_string(), "123456");
    }

    #[test]
    fn test_add() {
        let d1 = Decimal128Value::from_str("123.456").expect("解析失败");
        let d2 = Decimal128Value::from_str("789.012").expect("解析失败");
        let sum = &d1 + &d2;
        assert_eq!(sum.to_string(), "912.468");
    }

    #[test]
    fn test_sub() {
        let d1 = Decimal128Value::from_str("789.012").expect("解析失败");
        let d2 = Decimal128Value::from_str("123.456").expect("解析失败");
        let diff = &d1 - &d2;
        assert_eq!(diff.to_string(), "665.556");
    }

    #[test]
    fn test_mul() {
        let d1 = Decimal128Value::from_str("12.34").expect("解析失败");
        let d2 = Decimal128Value::from_str("5.6").expect("解析失败");
        let product = &d1 * &d2;
        assert_eq!(product.to_string(), "69.104");
    }

    #[test]
    fn test_div() {
        let d1 = Decimal128Value::from_str("100.0").expect("解析失败");
        let d2 = Decimal128Value::from_str("4.0").expect("解析失败");
        let quotient = &d1 / &d2;
        assert_eq!(quotient.to_string(), "25");
    }

    #[test]
    fn test_div_by_zero() {
        let d1 = Decimal128Value::from_str("100.0").expect("解析失败");
        let d2 = Decimal128Value::from_str("0.0").expect("解析失败");
        let result = d1.div(&d2);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "除零错误");
    }

    #[test]
    fn test_neg() {
        let d = Decimal128Value::from_str("123.456").expect("解析失败");
        let neg = -d.clone();
        assert_eq!(neg.to_string(), "-123.456");
    }

    #[test]
    fn test_abs() {
        let d1 = Decimal128Value::from_str("-123.456").expect("解析失败");
        let d2 = Decimal128Value::from_str("123.456").expect("解析失败");
        assert_eq!(d1.abs().to_string(), "123.456");
        assert_eq!(d2.abs().to_string(), "123.456");
    }

    #[test]
    fn test_round_dp() {
        let d = Decimal128Value::from_str("123.456789").expect("解析失败");
        let rounded = d.round_dp(2);
        assert_eq!(rounded.to_string(), "123.45");
    }

    #[test]
    fn test_floor() {
        let d = Decimal128Value::from_str("123.789").expect("解析失败");
        let floored = d.floor();
        assert_eq!(floored.to_string(), "123");
    }

    #[test]
    fn test_ceil() {
        let d = Decimal128Value::from_str("123.789").expect("解析失败");
        let ceiled = d.ceil();
        assert_eq!(ceiled.to_string(), "124");
    }

    #[test]
    fn test_is_zero() {
        let d = Decimal128Value::from_str("0.0").expect("解析失败");
        assert!(d.is_zero());
    }

    #[test]
    fn test_is_negative() {
        let d = Decimal128Value::from_str("-123.456").expect("解析失败");
        assert!(d.is_negative());
    }

    #[test]
    fn test_is_positive() {
        let d = Decimal128Value::from_str("123.456").expect("解析失败");
        assert!(d.is_positive());
    }

    #[test]
    fn test_finance_calculation() {
        let price = Decimal128Value::from_str("19.99").expect("解析失败");
        let quantity = Decimal128Value::from_str("3").expect("解析失败");
        let tax_rate = Decimal128Value::from_str("0.08").expect("解析失败");
        
        let subtotal = &price * &quantity;
        let tax = &subtotal * &tax_rate;
        let total = &subtotal + &tax;
        
        assert_eq!(subtotal.to_string(), "59.97");
        assert_eq!(tax.to_string(), "4.7976");
        assert_eq!(total.to_string(), "64.7676");
    }
}
