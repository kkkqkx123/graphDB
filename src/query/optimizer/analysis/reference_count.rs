//! 引用计数分析模块
//!
//! 识别执行计划中被多次引用的子计划节点，为物化策略选择提供数据支持。

use std::collections::HashMap;

use crate::query::planner::plan::core::nodes::{
    plan_node_traits::SingleInputNode,
    PlanNodeEnum,
};

use super::fingerprint::FingerprintCalculator;

/// 子计划唯一标识
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubplanId(pub u64);

impl SubplanId {
    /// 创建新的子计划ID
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// 获取ID值
    pub fn value(&self) -> u64 {
        self.0
    }
}

/// 子计划引用信息
#[derive(Debug, Clone)]
pub struct SubplanReferenceInfo {
    /// 子计划的唯一标识
    pub subplan_id: SubplanId,
    /// 子计划根节点ID
    pub root_node_id: i64,
    /// 被引用次数
    pub reference_count: usize,
    /// 引用位置（父节点ID列表）
    pub reference_locations: Vec<i64>,
    /// 子计划包含的节点数量
    pub node_count: usize,
}

impl SubplanReferenceInfo {
    /// 创建新的子计划引用信息
    pub fn new(subplan_id: SubplanId, root_node_id: i64) -> Self {
        Self {
            subplan_id,
            root_node_id,
            reference_count: 0,
            reference_locations: Vec::new(),
            node_count: 0,
        }
    }

    /// 增加引用计数
    pub fn add_reference(&mut self, location: i64) {
        self.reference_count += 1;
        if !self.reference_locations.contains(&location) {
            self.reference_locations.push(location);
        }
    }
}

/// 引用计数分析结果
#[derive(Debug, Clone)]
pub struct ReferenceCountAnalysis {
    /// 所有被多次引用的子计划（引用次数 >= 2）
    pub repeated_subplans: Vec<SubplanReferenceInfo>,
    /// 节点ID到引用信息的映射
    pub node_reference_map: HashMap<i64, SubplanReferenceInfo>,
}

impl ReferenceCountAnalysis {
    /// 创建空的分析结果
    pub fn new() -> Self {
        Self {
            repeated_subplans: Vec::new(),
            node_reference_map: HashMap::new(),
        }
    }

    /// 获取指定节点的引用信息
    pub fn get_node_info(&self, node_id: i64) -> Option<&SubplanReferenceInfo> {
        self.node_reference_map.get(&node_id)
    }

    /// 检查子计划是否被多次引用
    pub fn is_repeated(&self, node_id: i64) -> bool {
        self.node_reference_map
            .get(&node_id)
            .map(|info| info.reference_count >= 2)
            .unwrap_or(false)
    }

    /// 获取被多次引用的子计划数量
    pub fn repeated_count(&self) -> usize {
        self.repeated_subplans.len()
    }
}

impl Default for ReferenceCountAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

/// 分析上下文
struct AnalysisContext {
    /// 指纹到引用信息的映射
    fingerprint_map: HashMap<u64, SubplanReferenceInfo>,
    /// 节点ID到指纹的映射
    node_fingerprint_map: HashMap<i64, u64>,
    /// 节点计数映射（用于估算子计划大小）
    node_count_map: HashMap<i64, usize>,
}

impl AnalysisContext {
    /// 创建新的分析上下文
    fn new() -> Self {
        Self {
            fingerprint_map: HashMap::new(),
            node_fingerprint_map: HashMap::new(),
            node_count_map: HashMap::new(),
        }
    }

    /// 记录引用
    fn record_reference(&mut self, fingerprint: u64, node_id: i64, parent_id: Option<i64>) {
        // 记录节点到指纹的映射
        self.node_fingerprint_map.insert(node_id, fingerprint);

        // 获取或创建引用信息
        let info = self
            .fingerprint_map
            .entry(fingerprint)
            .or_insert_with(|| SubplanReferenceInfo::new(SubplanId::new(fingerprint), node_id));

        // 增加引用计数
        if let Some(parent) = parent_id {
            info.add_reference(parent);
        } else {
            // 根节点，引用计数至少为1
            info.reference_count = info.reference_count.max(1);
        }
    }

    /// 记录节点数量
    fn record_node_count(&mut self, node_id: i64, count: usize) {
        self.node_count_map.insert(node_id, count);
    }

    /// 转换为分析结果
    fn into_analysis_result(self) -> ReferenceCountAnalysis {
        let mut repeated_subplans = Vec::new();
        let mut node_reference_map = HashMap::new();

        for (fingerprint, mut info) in self.fingerprint_map {
            // 只保留被多次引用的子计划
            if info.reference_count >= 2 {
                // 更新节点数量
                if let Some(&count) = self.node_count_map.get(&info.root_node_id) {
                    info.node_count = count;
                }

                // 为所有具有相同指纹的节点添加引用信息
                for (node_id, fp) in &self.node_fingerprint_map {
                    if *fp == fingerprint {
                        node_reference_map.insert(*node_id, info.clone());
                    }
                }

                repeated_subplans.push(info);
            }
        }

        ReferenceCountAnalysis {
            repeated_subplans,
            node_reference_map,
        }
    }
}

/// 引用计数分析器
///
/// 分析执行计划，识别被多次引用的子计划节点。
#[derive(Debug, Clone)]
pub struct ReferenceCountAnalyzer {
    /// 指纹计算器
    fingerprint_calculator: FingerprintCalculator,
}

impl ReferenceCountAnalyzer {
    /// 创建新的引用计数分析器
    pub fn new() -> Self {
        Self {
            fingerprint_calculator: FingerprintCalculator::new(),
        }
    }

    /// 分析计划的引用计数
    ///
    /// # 参数
    /// - `plan`: 要分析的执行计划根节点
    ///
    /// # 返回
    /// 引用计数分析结果
    ///
    /// # 算法
    /// 1. 后序遍历计划树
    /// 2. 为每个节点计算结构指纹
    /// 3. 统计每个指纹的出现次数
    /// 4. 返回被多次引用的子计划信息
    pub fn analyze(&self, plan: &PlanNodeEnum) -> ReferenceCountAnalysis {
        let mut context = AnalysisContext::new();
        self.analyze_recursive(plan, &mut context, None);
        context.into_analysis_result()
    }

    /// 递归分析计划树
    ///
    /// # 参数
    /// - `node`: 当前节点
    /// - `context`: 分析上下文
    /// - `parent_id`: 父节点ID（用于记录引用位置）
    ///
    /// # 返回
    /// 节点数量
    fn analyze_recursive(
        &self,
        node: &PlanNodeEnum,
        context: &mut AnalysisContext,
        parent_id: Option<i64>,
    ) -> usize {
        // 计算当前节点指纹
        let fingerprint = self.fingerprint_calculator.calculate_fingerprint(node);
        let node_id = node.id();

        // 记录引用
        context.record_reference(fingerprint.value(), node_id, parent_id);

        // 递归分析子节点并计算节点数量
        let child_count = match node {
            // 单输入节点
            PlanNodeEnum::Filter(n) => {
                let count = self.analyze_recursive(n.input(), context, Some(node_id));
                1 + count
            }
            PlanNodeEnum::Project(n) => {
                let count = self.analyze_recursive(n.input(), context, Some(node_id));
                1 + count
            }
            PlanNodeEnum::Sort(n) => {
                let count = self.analyze_recursive(n.input(), context, Some(node_id));
                1 + count
            }
            PlanNodeEnum::Limit(n) => {
                let count = self.analyze_recursive(n.input(), context, Some(node_id));
                1 + count
            }
            PlanNodeEnum::TopN(n) => {
                let count = self.analyze_recursive(n.input(), context, Some(node_id));
                1 + count
            }
            PlanNodeEnum::Sample(n) => {
                let count = self.analyze_recursive(n.input(), context, Some(node_id));
                1 + count
            }
            PlanNodeEnum::Aggregate(n) => {
                let count = self.analyze_recursive(n.input(), context, Some(node_id));
                1 + count
            }
            PlanNodeEnum::Dedup(n) => {
                let count = self.analyze_recursive(n.input(), context, Some(node_id));
                1 + count
            }
            PlanNodeEnum::Unwind(n) => {
                let count = self.analyze_recursive(n.input(), context, Some(node_id));
                1 + count
            }
            PlanNodeEnum::DataCollect(n) => {
                let count = self.analyze_recursive(n.input(), context, Some(node_id));
                1 + count
            }
            PlanNodeEnum::Traverse(n) => {
                // TraverseNode 使用 SingleInputNode trait，input() 方法在 input 为 None 时会 panic
                // 这里我们直接访问 deps 字段来遍历子节点
                let mut total = 1;
                for dep in n.dependencies() {
                    let count = self.analyze_recursive(dep, context, Some(node_id));
                    total += count;
                }
                total
            }
            PlanNodeEnum::Expand(n) => {
                // ExpandNode 使用 MultipleInputNode，通过 dependencies() 访问子节点
                let mut total = 1;
                for dep in n.dependencies() {
                    let count = self.analyze_recursive(dep, context, Some(node_id));
                    total += count;
                }
                total
            }
            PlanNodeEnum::ExpandAll(n) => {
                // ExpandAllNode 使用 MultipleInputNode，通过 dependencies() 访问子节点
                let mut total = 1;
                for dep in n.dependencies() {
                    let count = self.analyze_recursive(dep, context, Some(node_id));
                    total += count;
                }
                total
            }
            PlanNodeEnum::AppendVertices(n) => {
                // AppendVerticesNode 使用 MultipleInputNode，通过 dependencies() 访问子节点
                let mut total = 1;
                for dep in n.dependencies() {
                    let count = self.analyze_recursive(dep, context, Some(node_id));
                    total += count;
                }
                total
            }
            // ArgumentNode 和 PassThroughNode 是零输入节点
            PlanNodeEnum::Argument(_) => 1,
            PlanNodeEnum::PassThrough(_) => 1,
            PlanNodeEnum::PatternApply(n) => {
                let count = self.analyze_recursive(n.input(), context, Some(node_id));
                1 + count
            }
            PlanNodeEnum::RollUpApply(n) => {
                let count = self.analyze_recursive(n.input(), context, Some(node_id));
                1 + count
            }
            PlanNodeEnum::Assign(n) => {
                let count = self.analyze_recursive(n.input(), context, Some(node_id));
                1 + count
            }
            PlanNodeEnum::Minus(n) => {
                let left_count = self.analyze_recursive(n.input(), context, Some(node_id));
                let right_count = self.analyze_recursive(n.minus_input(), context, Some(node_id));
                1 + left_count + right_count
            }
            PlanNodeEnum::Intersect(n) => {
                let left_count = self.analyze_recursive(n.input(), context, Some(node_id));
                let right_count = self.analyze_recursive(n.intersect_input(), context, Some(node_id));
                1 + left_count + right_count
            }

            // 双输入节点
            PlanNodeEnum::InnerJoin(n) => {
                let left_count = self.analyze_recursive(n.left_input(), context, Some(node_id));
                let right_count = self.analyze_recursive(n.right_input(), context, Some(node_id));
                1 + left_count + right_count
            }
            PlanNodeEnum::LeftJoin(n) => {
                let left_count = self.analyze_recursive(n.left_input(), context, Some(node_id));
                let right_count = self.analyze_recursive(n.right_input(), context, Some(node_id));
                1 + left_count + right_count
            }
            PlanNodeEnum::CrossJoin(n) => {
                let left_count = self.analyze_recursive(n.left_input(), context, Some(node_id));
                let right_count = self.analyze_recursive(n.right_input(), context, Some(node_id));
                1 + left_count + right_count
            }
            PlanNodeEnum::HashInnerJoin(n) => {
                let left_count = self.analyze_recursive(n.left_input(), context, Some(node_id));
                let right_count = self.analyze_recursive(n.right_input(), context, Some(node_id));
                1 + left_count + right_count
            }
            PlanNodeEnum::HashLeftJoin(n) => {
                let left_count = self.analyze_recursive(n.left_input(), context, Some(node_id));
                let right_count = self.analyze_recursive(n.right_input(), context, Some(node_id));
                1 + left_count + right_count
            }
            PlanNodeEnum::FullOuterJoin(n) => {
                let left_count = self.analyze_recursive(n.left_input(), context, Some(node_id));
                let right_count = self.analyze_recursive(n.right_input(), context, Some(node_id));
                1 + left_count + right_count
            }
            PlanNodeEnum::Union(n) => {
                let count = self.analyze_recursive(n.input(), context, Some(node_id));
                1 + count
            }

            // 多输入节点
            PlanNodeEnum::Select(n) => {
                let mut total = 1; // 当前节点
                if let Some(ref branch) = n.if_branch() {
                    let count = self.analyze_recursive(branch, context, Some(node_id));
                    total += count;
                }
                if let Some(ref branch) = n.else_branch() {
                    let count = self.analyze_recursive(branch, context, Some(node_id));
                    total += count;
                }
                total
            }
            PlanNodeEnum::Loop(n) => {
                let mut total = 1; // 当前节点
                if let Some(ref body) = n.body() {
                    let count = self.analyze_recursive(body, context, Some(node_id));
                    total += count;
                }
                total
            }

            // 零输入节点（叶子节点）
            _ => 1, // 只有当前节点
        };

        // 记录节点数量
        context.record_node_count(node_id, child_count);

        child_count
    }

}

impl Default for ReferenceCountAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_count_analyzer_new() {
        let _analyzer = ReferenceCountAnalyzer::new();
        // 验证创建成功
    }

    #[test]
    fn test_subplan_id() {
        let id = SubplanId::new(12345);
        assert_eq!(id.value(), 12345);
    }

    #[test]
    fn test_subplan_reference_info() {
        let mut info = SubplanReferenceInfo::new(SubplanId::new(1), 100);
        assert_eq!(info.reference_count, 0);

        info.add_reference(200);
        assert_eq!(info.reference_count, 1);
        assert!(info.reference_locations.contains(&200));

        info.add_reference(200); // 重复添加
        assert_eq!(info.reference_count, 2);
        assert_eq!(info.reference_locations.len(), 1); // 位置不重复
    }

    #[test]
    fn test_reference_count_analysis() {
        let analysis = ReferenceCountAnalysis::new();
        assert_eq!(analysis.repeated_count(), 0);
        assert!(!analysis.is_repeated(1));
    }
}
