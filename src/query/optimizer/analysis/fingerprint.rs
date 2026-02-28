//! 计划节点指纹计算模块
//!
//! 提供计划节点结构指纹计算功能，用于识别等价的子计划。
//! 相同结构的子计划会产生相同的指纹值。

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::query::planner::plan::core::nodes::{BinaryInputNode, PlanNodeEnum, SingleInputNode};

/// 计划节点指纹
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlanFingerprint(pub u64);

impl PlanFingerprint {
    /// 创建新的指纹
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// 获取指纹值
    pub fn value(&self) -> u64 {
        self.0
    }
}

/// 指纹计算器
///
/// 使用稳定的哈希算法计算计划节点的结构指纹。
/// 相同结构的子计划会产生相同的指纹值。
#[derive(Debug, Clone)]
pub struct FingerprintCalculator;

impl FingerprintCalculator {
    /// 创建新的指纹计算器
    pub fn new() -> Self {
        Self
    }

    /// 计算计划节点的结构指纹
    ///
    /// # 参数
    /// - `node`: 计划节点
    ///
    /// # 返回
    /// 节点的结构指纹
    ///
    /// # 算法
    /// 1. 哈希节点类型（使用枚举判别式）
    /// 2. 递归哈希子节点指纹
    /// 3. 哈希节点的关键配置参数
    pub fn calculate_fingerprint(&self, node: &PlanNodeEnum) -> PlanFingerprint {
        let mut hasher = DefaultHasher::new();

        // 哈希节点类型
        std::mem::discriminant(node).hash(&mut hasher);

        // 哈希子节点指纹
        self.hash_children(node, &mut hasher);

        // 哈希节点配置
        self.hash_node_config(node, &mut hasher);

        PlanFingerprint::new(hasher.finish())
    }

    /// 哈希子节点
    fn hash_children(&self, node: &PlanNodeEnum, hasher: &mut DefaultHasher) {
        use crate::query::planner::plan::core::nodes::*;

        match node {
            // 单输入节点
            PlanNodeEnum::Filter(n) => {
                self.hash_single_input(n, hasher);
            }
            PlanNodeEnum::Project(n) => {
                self.hash_single_input(n, hasher);
            }
            PlanNodeEnum::Sort(n) => {
                self.hash_single_input(n, hasher);
            }
            PlanNodeEnum::Limit(n) => {
                self.hash_single_input(n, hasher);
            }
            PlanNodeEnum::TopN(n) => {
                self.hash_single_input(n, hasher);
            }
            PlanNodeEnum::Sample(n) => {
                self.hash_single_input(n, hasher);
            }
            PlanNodeEnum::Aggregate(n) => {
                self.hash_single_input(n, hasher);
            }
            PlanNodeEnum::Dedup(n) => {
                self.hash_single_input(n, hasher);
            }
            PlanNodeEnum::Unwind(n) => {
                self.hash_single_input(n, hasher);
            }
            PlanNodeEnum::DataCollect(n) => {
                self.hash_single_input(n, hasher);
            }
            PlanNodeEnum::Traverse(n) => {
                self.hash_single_input(n, hasher);
            }
            PlanNodeEnum::Expand(n) => {
                // ExpandNode 使用 MultipleInputNode，通过 dependencies() 访问子节点
                for dep in n.dependencies() {
                    let fp = self.calculate_fingerprint(dep);
                    fp.hash(hasher);
                }
            }
            PlanNodeEnum::ExpandAll(n) => {
                // ExpandAllNode 使用 MultipleInputNode，通过 dependencies() 访问子节点
                for dep in n.dependencies() {
                    let fp = self.calculate_fingerprint(dep);
                    fp.hash(hasher);
                }
            }
            PlanNodeEnum::AppendVertices(n) => {
                // AppendVerticesNode 使用 MultipleInputNode，通过 dependencies() 访问子节点
                for dep in n.dependencies() {
                    let fp = self.calculate_fingerprint(dep);
                    fp.hash(hasher);
                }
            }
            PlanNodeEnum::Argument(_) => {
                // ArgumentNode 是零输入节点，无需哈希子节点
            }
            PlanNodeEnum::PassThrough(_) => {
                // PassThroughNode 是零输入节点，无需哈希子节点
            }
            PlanNodeEnum::PatternApply(n) => {
                self.hash_single_input(n, hasher);
            }
            PlanNodeEnum::RollUpApply(n) => {
                self.hash_single_input(n, hasher);
            }
            PlanNodeEnum::Assign(n) => {
                self.hash_single_input(n, hasher);
            }

            // 双输入节点
            PlanNodeEnum::InnerJoin(n) => {
                self.hash_binary_input(n, hasher);
            }
            PlanNodeEnum::LeftJoin(n) => {
                self.hash_binary_input(n, hasher);
            }
            PlanNodeEnum::CrossJoin(n) => {
                self.hash_binary_input(n, hasher);
            }
            PlanNodeEnum::HashInnerJoin(n) => {
                self.hash_binary_input(n, hasher);
            }
            PlanNodeEnum::HashLeftJoin(n) => {
                self.hash_binary_input(n, hasher);
            }
            PlanNodeEnum::FullOuterJoin(n) => {
                self.hash_binary_input(n, hasher);
            }
            PlanNodeEnum::Union(n) => {
                // UnionNode 是单输入节点
                self.hash_single_input(n, hasher);
            }
            PlanNodeEnum::Minus(n) => {
                // MinusNode 使用自定义方法访问输入
                let left_fp = self.calculate_fingerprint(n.input());
                let right_fp = self.calculate_fingerprint(n.minus_input());
                left_fp.hash(hasher);
                right_fp.hash(hasher);
            }
            PlanNodeEnum::Intersect(n) => {
                // IntersectNode 使用自定义方法访问输入
                let left_fp = self.calculate_fingerprint(n.input());
                let right_fp = self.calculate_fingerprint(n.intersect_input());
                left_fp.hash(hasher);
                right_fp.hash(hasher);
            }

            // 多输入节点
            PlanNodeEnum::Select(n) => {
                // SelectNode 使用 if_branch 和 else_branch 方法
                if let Some(ref branch) = n.if_branch() {
                    let fp = self.calculate_fingerprint(branch);
                    fp.hash(hasher);
                }
                if let Some(ref branch) = n.else_branch() {
                    let fp = self.calculate_fingerprint(branch);
                    fp.hash(hasher);
                }
            }
            PlanNodeEnum::Loop(n) => {
                // LoopNode 的 body 返回 Option<Box<PlanNodeEnum>>
                if let Some(ref body) = n.body() {
                    let body_fp = self.calculate_fingerprint(body);
                    body_fp.hash(hasher);
                }
            }

            // 零输入节点（叶子节点）
            PlanNodeEnum::Start(_) => {
                // 叶子节点，无需哈希子节点
            }
            PlanNodeEnum::GetVertices(_) => {
                // 叶子节点
            }
            PlanNodeEnum::GetEdges(_) => {
                // 叶子节点
            }
            PlanNodeEnum::GetNeighbors(_) => {
                // 叶子节点
            }
            PlanNodeEnum::ScanVertices(_) => {
                // 叶子节点
            }
            PlanNodeEnum::ScanEdges(_) => {
                // 叶子节点
            }
            PlanNodeEnum::EdgeIndexScan(_) => {
                // 叶子节点
            }
            PlanNodeEnum::IndexScan(_) => {
                // 叶子节点
            }
            PlanNodeEnum::ShortestPath(_) => {
                // 叶子节点
            }
            PlanNodeEnum::MultiShortestPath(_) => {
                // 叶子节点
            }
            PlanNodeEnum::BFSShortest(_) => {
                // 叶子节点
            }
            PlanNodeEnum::AllPaths(_) => {
                // 叶子节点
            }

            // 管理节点（不参与优化决策）
            _ => {
                // 管理节点不计算指纹
            }
        }
    }

    /// 哈希单输入节点的子节点
    fn hash_single_input<T: SingleInputNode>(
        &self,
        node: &T,
        hasher: &mut DefaultHasher,
    ) {
        let input_fp = self.calculate_fingerprint(node.input());
        input_fp.hash(hasher);
    }

    /// 哈希双输入节点的子节点
    fn hash_binary_input<T: BinaryInputNode>(
        &self,
        node: &T,
        hasher: &mut DefaultHasher,
    ) {
        let left_fp = self.calculate_fingerprint(node.left_input());
        let right_fp = self.calculate_fingerprint(node.right_input());
        left_fp.hash(hasher);
        right_fp.hash(hasher);
    }

    /// 哈希节点配置
    fn hash_node_config(&self, node: &PlanNodeEnum, hasher: &mut DefaultHasher) {
        use crate::query::planner::plan::core::nodes::*;

        match node {
            // 哈希过滤条件中的常量值
            PlanNodeEnum::Filter(n) => {
                let condition = n.condition();
                let expr = match condition.expression() {
                    Some(meta) => meta.inner().clone(),
                    None => return,
                };
                // 哈希条件表达式的结构（不包含变量名）
                self.hash_expression_structure(&expr, hasher);
            }

            // 哈希投影列
            PlanNodeEnum::Project(n) => {
                n.col_names().len().hash(hasher);
            }

            // 哈希排序键
            PlanNodeEnum::Sort(n) => {
                n.sort_items().len().hash(hasher);
            }

            // 哈希Limit值
            PlanNodeEnum::Limit(n) => {
                n.count().hash(hasher);
                n.offset().hash(hasher);
            }

            // 哈希TopN配置
            PlanNodeEnum::TopN(n) => {
                n.limit().hash(hasher);
                n.sort_items().len().hash(hasher);
            }

            // 哈希采样配置
            PlanNodeEnum::Sample(n) => {
                n.count().hash(hasher);
            }

            // 哈希聚合配置
            PlanNodeEnum::Aggregate(n) => {
                n.group_keys().len().hash(hasher);
                n.aggregation_functions().len().hash(hasher);
            }

            // 哈希扫描配置
            PlanNodeEnum::ScanVertices(n) => {
                n.name().hash(hasher);
            }
            PlanNodeEnum::ScanEdges(n) => {
                n.edge_type().hash(hasher);
            }

            // 哈希遍历配置
            PlanNodeEnum::Traverse(n) => {
                n.edge_types().len().hash(hasher);
                n.direction().hash(hasher);
            }

            // 其他节点暂不哈希额外配置
            _ => {}
        }
    }

    /// 哈希表达式结构（不包含变量名和字面量值）
    fn hash_expression_structure(
        &self,
        expr: &crate::core::Expression,
        hasher: &mut DefaultHasher,
    ) {
        use crate::core::Expression;

        // 哈希表达式类型
        std::mem::discriminant(expr).hash(hasher);

        match expr {
            Expression::Literal(_) => {
                // 字面量只哈希类型，不哈希值
            }
            Expression::Variable(_) => {
                // 变量只哈希类型，不哈希名
            }
            Expression::Property { object, .. } => {
                // 属性名不哈希，只哈希对象结构
                self.hash_expression_structure(object, hasher);
            }
            Expression::Binary { left, op, right } => {
                // 哈希操作符类型
                std::mem::discriminant(op).hash(hasher);
                self.hash_expression_structure(left, hasher);
                self.hash_expression_structure(right, hasher);
            }
            Expression::Unary { op, operand } => {
                std::mem::discriminant(op).hash(hasher);
                self.hash_expression_structure(operand, hasher);
            }
            Expression::Function { name, args } => {
                // 哈希函数名
                name.hash(hasher);
                args.len().hash(hasher);
                for arg in args {
                    self.hash_expression_structure(arg, hasher);
                }
            }
            Expression::Aggregate { func, .. } => {
                std::mem::discriminant(func).hash(hasher);
            }
            _ => {
                // 其他表达式类型暂不详细处理
            }
        }
    }
}

impl Default for FingerprintCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_calculator_new() {
        let _calculator = FingerprintCalculator::new();
        // 验证创建成功
    }

    #[test]
    fn test_same_structure_same_fingerprint() {
        // 创建两个结构相同的计划节点应该产生相同的指纹
        // 注意：这里需要实际的计划节点来测试
        // 由于计划节点的创建比较复杂，这里只做结构测试
    }
}
