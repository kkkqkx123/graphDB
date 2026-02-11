//! 聚合数据状态模块
//!
//! 参考 nebula-graph 的 AggData 设计，管理聚合函数的中间状态和最终结果

use crate::core::value::{NullType, Value};
use std::collections::HashSet;

/// 聚合数据状态
///
/// 与 nebula-graph 的 AggData 对应，存储聚合计算的中间状态和最终结果
#[derive(Debug, Clone)]
pub struct AggData {
    /// 计数（用于 COUNT、AVG、STD 等）
    cnt: Value,
    /// 累加和（用于 SUM、AVG 等）
    sum: Value,
    /// 平均值（用于 AVG）
    avg: Value,
    /// 方差（用于 STD）
    deviation: Value,
    /// 最终结果
    result: Value,
    /// 去重集合（用于 COLLECT_SET、COUNT DISTINCT 等）
    uniques: Option<HashSet<Value>>,
}

impl AggData {
    /// 创建新的聚合数据状态
    pub fn new() -> Self {
        Self {
            cnt: Value::Null(NullType::NaN),
            sum: Value::Null(NullType::NaN),
            avg: Value::Null(NullType::NaN),
            deviation: Value::Null(NullType::NaN),
            result: Value::Null(NullType::NaN),
            uniques: None,
        }
    }

    /// 创建带去重功能的聚合数据状态
    pub fn with_uniques() -> Self {
        Self {
            cnt: Value::Null(NullType::NaN),
            sum: Value::Null(NullType::NaN),
            avg: Value::Null(NullType::NaN),
            deviation: Value::Null(NullType::NaN),
            result: Value::Null(NullType::NaN),
            uniques: Some(HashSet::new()),
        }
    }

    /// 获取计数
    pub fn cnt(&self) -> &Value {
        &self.cnt
    }

    /// 获取可变计数
    pub fn cnt_mut(&mut self) -> &mut Value {
        &mut self.cnt
    }

    /// 设置计数
    pub fn set_cnt(&mut self, cnt: Value) {
        self.cnt = cnt;
    }

    /// 获取累加和
    pub fn sum(&self) -> &Value {
        &self.sum
    }

    /// 获取可变累加和
    pub fn sum_mut(&mut self) -> &mut Value {
        &mut self.sum
    }

    /// 设置累加和
    pub fn set_sum(&mut self, sum: Value) {
        self.sum = sum;
    }

    /// 获取平均值
    pub fn avg(&self) -> &Value {
        &self.avg
    }

    /// 获取可变平均值
    pub fn avg_mut(&mut self) -> &mut Value {
        &mut self.avg
    }

    /// 设置平均值
    pub fn set_avg(&mut self, avg: Value) {
        self.avg = avg;
    }

    /// 获取方差
    pub fn deviation(&self) -> &Value {
        &self.deviation
    }

    /// 获取可变方差
    pub fn deviation_mut(&mut self) -> &mut Value {
        &mut self.deviation
    }

    /// 设置方差
    pub fn set_deviation(&mut self, deviation: Value) {
        self.deviation = deviation;
    }

    /// 获取最终结果
    pub fn result(&self) -> &Value {
        &self.result
    }

    /// 获取可变最终结果
    pub fn result_mut(&mut self) -> &mut Value {
        &mut self.result
    }

    /// 设置最终结果
    pub fn set_result(&mut self, result: Value) {
        self.result = result;
    }

    /// 获取去重集合
    pub fn uniques(&self) -> Option<&HashSet<Value>> {
        self.uniques.as_ref()
    }

    /// 获取可变去重集合
    pub fn uniques_mut(&mut self) -> Option<&mut HashSet<Value>> {
        self.uniques.as_mut()
    }

    /// 设置去重集合
    pub fn set_uniques(&mut self, uniques: HashSet<Value>) {
        self.uniques = Some(uniques);
    }

    /// 检查是否为 BadNull
    pub fn is_bad_null(&self) -> bool {
        self.result.is_bad_null()
    }

    /// 重置状态
    pub fn reset(&mut self) {
        self.cnt = Value::Null(NullType::NaN);
        self.sum = Value::Null(NullType::NaN);
        self.avg = Value::Null(NullType::NaN);
        self.deviation = Value::Null(NullType::NaN);
        self.result = Value::Null(NullType::NaN);
        if let Some(ref mut uniques) = self.uniques {
            uniques.clear();
        }
    }

    /// 获取所有字段的可变引用（用于聚合函数内部）
    /// 
    /// 返回 (result, cnt, sum, avg, deviation) 的可变引用
    pub fn get_all_mut(&mut self) -> (&mut Value, &mut Value, &mut Value, &mut Value, &mut Value) {
        (&mut self.result, &mut self.cnt, &mut self.sum, &mut self.avg, &mut self.deviation)
    }
}

impl Default for AggData {
    fn default() -> Self {
        Self::new()
    }
}
