//! 节点代价估算结果
//!
//! 定义节点代价估算的数据结构，包含：
//! - 节点自身代价（不包含子节点）
//! - 累计代价（包含所有子节点）
//! - 估算的输出行数

/// 节点代价和行数估算结果
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeCostEstimate {
    /// 节点自身代价（不包含子节点）
    pub node_cost: f64,
    /// 累计代价（包含所有子节点）
    pub total_cost: f64,
    /// 估算的输出行数
    pub output_rows: u64,
}

impl NodeCostEstimate {
    /// 创建新的估算结果
    pub fn new(node_cost: f64, total_cost: f64, output_rows: u64) -> Self {
        Self {
            node_cost,
            total_cost,
            output_rows,
        }
    }

    /// 创建叶子节点的估算结果（无子节点）
    pub fn leaf(node_cost: f64, output_rows: u64) -> Self {
        Self {
            node_cost,
            total_cost: node_cost,
            output_rows,
        }
    }

    /// 创建零代价估算结果
    pub fn zero() -> Self {
        Self {
            node_cost: 0.0,
            total_cost: 0.0,
            output_rows: 0,
        }
    }

    /// 合并多个子节点的估算结果
    pub fn combine_children(children: &[Self], node_cost: f64, output_rows: u64) -> Self {
        let child_total_cost: f64 = children.iter().map(|e| e.total_cost).sum();
        Self {
            node_cost,
            total_cost: node_cost + child_total_cost,
            output_rows,
        }
    }

    /// 获取代价比率（节点代价/累计代价）
    pub fn cost_ratio(&self) -> f64 {
        if self.total_cost == 0.0 {
            0.0
        } else {
            self.node_cost / self.total_cost
        }
    }

    /// 检查估算结果是否有效
    pub fn is_valid(&self) -> bool {
        self.node_cost >= 0.0 && self.total_cost >= 0.0
    }
}

impl Default for NodeCostEstimate {
    fn default() -> Self {
        Self::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_cost_estimate_new() {
        let estimate = NodeCostEstimate::new(10.0, 100.0, 50);
        assert_eq!(estimate.node_cost, 10.0);
        assert_eq!(estimate.total_cost, 100.0);
        assert_eq!(estimate.output_rows, 50);
    }

    #[test]
    fn test_node_cost_estimate_leaf() {
        let estimate = NodeCostEstimate::leaf(10.0, 50);
        assert_eq!(estimate.node_cost, 10.0);
        assert_eq!(estimate.total_cost, 10.0);
        assert_eq!(estimate.output_rows, 50);
    }

    #[test]
    fn test_node_cost_estimate_zero() {
        let estimate = NodeCostEstimate::zero();
        assert_eq!(estimate.node_cost, 0.0);
        assert_eq!(estimate.total_cost, 0.0);
        assert_eq!(estimate.output_rows, 0);
    }

    #[test]
    fn test_combine_children() {
        let child1 = NodeCostEstimate::leaf(10.0, 100);
        let child2 = NodeCostEstimate::leaf(20.0, 200);
        let combined = NodeCostEstimate::combine_children(&[child1, child2], 5.0, 50);
        
        assert_eq!(combined.node_cost, 5.0);
        assert_eq!(combined.total_cost, 35.0); // 5 + 10 + 20
        assert_eq!(combined.output_rows, 50);
    }

    #[test]
    fn test_cost_ratio() {
        let estimate = NodeCostEstimate::new(10.0, 100.0, 50);
        assert_eq!(estimate.cost_ratio(), 0.1);

        let zero = NodeCostEstimate::zero();
        assert_eq!(zero.cost_ratio(), 0.0);
    }

    #[test]
    fn test_is_valid() {
        let valid = NodeCostEstimate::new(10.0, 100.0, 50);
        assert!(valid.is_valid());

        let invalid = NodeCostEstimate::new(-1.0, 100.0, 50);
        assert!(!invalid.is_valid());
    }
}
