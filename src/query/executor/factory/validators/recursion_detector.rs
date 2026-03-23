//! 递归检测器
//!
//! 负责检测执行计划中的循环引用，防止无限递归

use crate::core::error::DBError;
use std::collections::HashSet;

/// 递归检测器
#[derive(Debug, Clone)]
pub struct RecursionDetector {
    /// 访问栈，用于检测循环
    visit_stack: Vec<(i64, &'static str)>,
    /// 已访问的节点集合
    visited: HashSet<i64>,
    /// 最大递归深度
    max_depth: usize,
}

impl RecursionDetector {
    /// 创建新的递归检测器
    pub fn new(max_depth: usize) -> Self {
        Self {
            visit_stack: Vec::new(),
            visited: HashSet::new(),
            max_depth,
        }
    }

    /// 重置检测器状态
    pub fn reset(&mut self) {
        self.visit_stack.clear();
        self.visited.clear();
    }

    /// 验证执行器是否会导致递归
    pub fn validate_executor(
        &mut self,
        node_id: i64,
        node_name: &'static str,
    ) -> Result<(), DBError> {
        // 检查是否超过最大深度
        if self.visit_stack.len() >= self.max_depth {
            return Err(DBError::Internal(format!(
                "执行计划深度超过最大限制 {}: 当前节点 {}({})",
                self.max_depth, node_name, node_id
            )));
        }

        // 检查循环引用
        if self.visit_stack.iter().any(|(id, _)| *id == node_id) {
            let cycle_path: Vec<String> = self
                .visit_stack
                .iter()
                .map(|(id, name)| format!("{}({})", name, id))
                .collect();
            return Err(DBError::Internal(format!(
                "检测到执行计划循环引用: {} -> {}({})",
                cycle_path.join(" -> "),
                node_name,
                node_id
            )));
        }

        // 将当前节点压入栈
        self.visit_stack.push((node_id, node_name));
        self.visited.insert(node_id);

        Ok(())
    }

    /// 离开当前节点（出栈）
    pub fn leave_executor(&mut self) {
        self.visit_stack.pop();
    }

    /// 获取当前深度
    pub fn current_depth(&self) -> usize {
        self.visit_stack.len()
    }

    /// 是否已访问过该节点
    pub fn is_visited(&self, node_id: i64) -> bool {
        self.visited.contains(&node_id)
    }
}

impl Default for RecursionDetector {
    fn default() -> Self {
        Self::new(100)
    }
}
