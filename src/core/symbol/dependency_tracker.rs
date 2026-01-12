//! 依赖关系跟踪器模块 - 管理变量读写依赖关系

use crate::core::PlanNodeRef;
use std::collections::{HashMap, HashSet};

/// 依赖关系类型
#[derive(Debug, Clone, PartialEq)]
pub enum DependencyType {
    Read,
    Write,
    ReadWrite,
}

/// 依赖关系
#[derive(Debug, Clone)]
pub struct Dependency {
    pub node: PlanNodeRef,
    pub dep_type: DependencyType,
    pub timestamp: std::time::SystemTime,
}

impl Dependency {
    pub fn new(node: PlanNodeRef, dep_type: DependencyType) -> Self {
        Self {
            node,
            dep_type,
            timestamp: std::time::SystemTime::now(),
        }
    }
}

/// 变量依赖信息
#[derive(Debug)]
pub struct VariableDependencies {
    pub variable_name: String,
    pub readers: HashSet<PlanNodeRef>,
    pub writers: HashSet<PlanNodeRef>,
    pub dependencies: Vec<Dependency>,
    pub user_count: std::sync::atomic::AtomicU64,
}

impl Clone for VariableDependencies {
    fn clone(&self) -> Self {
        Self {
            variable_name: self.variable_name.clone(),
            readers: self.readers.clone(),
            writers: self.writers.clone(),
            dependencies: self.dependencies.clone(),
            user_count: std::sync::atomic::AtomicU64::new(
                self.user_count.load(std::sync::atomic::Ordering::Relaxed),
            ),
        }
    }
}

impl VariableDependencies {
    pub fn new(variable_name: String) -> Self {
        Self {
            variable_name,
            readers: HashSet::new(),
            writers: HashSet::new(),
            dependencies: Vec::new(),
            user_count: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// 添加读取依赖
    pub fn add_reader(&mut self, node: PlanNodeRef) {
        self.readers.insert(node.clone());
        self.dependencies
            .push(Dependency::new(node, DependencyType::Read));
        self.increment_user_count();
    }

    /// 添加写入依赖
    pub fn add_writer(&mut self, node: PlanNodeRef) {
        self.writers.insert(node.clone());
        self.dependencies
            .push(Dependency::new(node, DependencyType::Write));
        self.increment_user_count();
    }

    /// 添加读写依赖
    pub fn add_reader_writer(&mut self, node: PlanNodeRef) {
        self.readers.insert(node.clone());
        self.writers.insert(node.clone());
        self.dependencies
            .push(Dependency::new(node, DependencyType::ReadWrite));
        self.increment_user_count();
    }

    /// 移除读取依赖
    pub fn remove_reader(&mut self, node: &PlanNodeRef) -> bool {
        let removed = self.readers.remove(node);
        if removed {
            self.dependencies
                .retain(|dep| !(dep.node == *node && dep.dep_type == DependencyType::Read));
        }
        removed
    }

    /// 移除写入依赖
    pub fn remove_writer(&mut self, node: &PlanNodeRef) -> bool {
        let removed = self.writers.remove(node);
        if removed {
            self.dependencies
                .retain(|dep| !(dep.node == *node && dep.dep_type == DependencyType::Write));
        }
        removed
    }

    /// 获取所有读取者
    pub fn get_readers(&self) -> &HashSet<PlanNodeRef> {
        &self.readers
    }

    /// 获取所有写入者
    pub fn get_writers(&self) -> &HashSet<PlanNodeRef> {
        &self.writers
    }

    /// 获取所有依赖
    pub fn get_dependencies(&self) -> &[Dependency] {
        &self.dependencies
    }

    /// 检查是否有读取依赖
    pub fn has_readers(&self) -> bool {
        !self.readers.is_empty()
    }

    /// 检查是否有写入依赖
    pub fn has_writers(&self) -> bool {
        !self.writers.is_empty()
    }

    /// 检查节点是否是读取者
    pub fn is_reader(&self, node: &PlanNodeRef) -> bool {
        self.readers.contains(node)
    }

    /// 检查节点是否是写入者
    pub fn is_writer(&self, node: &PlanNodeRef) -> bool {
        self.writers.contains(node)
    }

    /// 增加使用计数
    pub fn increment_user_count(&self) {
        self.user_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// 获取使用计数
    pub fn get_user_count(&self) -> u64 {
        self.user_count.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// 获取依赖统计信息
    pub fn get_dependency_stats(&self) -> DependencyStats {
        DependencyStats {
            variable_name: self.variable_name.clone(),
            reader_count: self.readers.len(),
            writer_count: self.writers.len(),
            total_dependencies: self.dependencies.len(),
            user_count: self.get_user_count(),
        }
    }
}

/// 依赖统计信息
#[derive(Debug, Clone)]
pub struct DependencyStats {
    pub variable_name: String,
    pub reader_count: usize,
    pub writer_count: usize,
    pub total_dependencies: usize,
    pub user_count: u64,
}

/// 依赖关系跟踪器
#[derive(Debug, Clone)]
pub struct DependencyTracker {
    dependencies: HashMap<String, VariableDependencies>,
}

impl DependencyTracker {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
        }
    }

    /// 添加变量
    pub fn add_variable(&mut self, variable_name: String) {
        self.dependencies.insert(
            variable_name.clone(),
            VariableDependencies::new(variable_name),
        );
    }

    /// 检查变量是否存在
    pub fn has_variable(&self, variable_name: &str) -> bool {
        self.dependencies.contains_key(variable_name)
    }

    /// 获取变量依赖
    pub fn get_variable_dependencies(&self, variable_name: &str) -> Option<&VariableDependencies> {
        self.dependencies.get(variable_name)
    }

    /// 获取可变变量依赖
    pub fn get_variable_dependencies_mut(
        &mut self,
        variable_name: &str,
    ) -> Option<&mut VariableDependencies> {
        self.dependencies.get_mut(variable_name)
    }

    /// 添加读取依赖
    pub fn add_read_dependency(
        &mut self,
        variable_name: &str,
        node: PlanNodeRef,
    ) -> Result<(), String> {
        let deps = self
            .dependencies
            .entry(variable_name.to_string())
            .or_insert_with(|| VariableDependencies::new(variable_name.to_string()));
        deps.add_reader(node);
        Ok(())
    }

    /// 添加写入依赖
    pub fn add_write_dependency(
        &mut self,
        variable_name: &str,
        node: PlanNodeRef,
    ) -> Result<(), String> {
        let deps = self
            .dependencies
            .entry(variable_name.to_string())
            .or_insert_with(|| VariableDependencies::new(variable_name.to_string()));
        deps.add_writer(node);
        Ok(())
    }

    /// 添加读写依赖
    pub fn add_read_write_dependency(
        &mut self,
        variable_name: &str,
        node: PlanNodeRef,
    ) -> Result<(), String> {
        let deps = self
            .dependencies
            .entry(variable_name.to_string())
            .or_insert_with(|| VariableDependencies::new(variable_name.to_string()));
        deps.add_reader_writer(node);
        Ok(())
    }

    /// 移除读取依赖
    pub fn remove_read_dependency(
        &mut self,
        variable_name: &str,
        node: &PlanNodeRef,
    ) -> Result<bool, String> {
        if let Some(deps) = self.dependencies.get_mut(variable_name) {
            Ok(deps.remove_reader(node))
        } else {
            Err(format!("Variable '{}' not found", variable_name))
        }
    }

    /// 移除写入依赖
    pub fn remove_write_dependency(
        &mut self,
        variable_name: &str,
        node: &PlanNodeRef,
    ) -> Result<bool, String> {
        if let Some(deps) = self.dependencies.get_mut(variable_name) {
            Ok(deps.remove_writer(node))
        } else {
            Err(format!("Variable '{}' not found", variable_name))
        }
    }

    /// 更新变量名（重命名）
    pub fn rename_variable(&mut self, old_name: &str, new_name: &str) -> Result<(), String> {
        if let Some(mut deps) = self.dependencies.remove(old_name) {
            deps.variable_name = new_name.to_string();
            self.dependencies.insert(new_name.to_string(), deps);
            Ok(())
        } else {
            Err(format!("Variable '{}' not found", old_name))
        }
    }

    /// 删除变量
    pub fn remove_variable(&mut self, variable_name: &str) -> Result<bool, String> {
        Ok(self.dependencies.remove(variable_name).is_some())
    }

    /// 获取所有变量名
    pub fn get_all_variables(&self) -> Vec<String> {
        self.dependencies.keys().cloned().collect()
    }

    /// 获取所有依赖统计
    pub fn get_all_stats(&self) -> Vec<DependencyStats> {
        self.dependencies
            .values()
            .map(|deps| deps.get_dependency_stats())
            .collect()
    }

    /// 查找读取指定变量的所有节点
    pub fn find_readers_of(&self, variable_name: &str) -> Vec<&PlanNodeRef> {
        self.dependencies
            .get(variable_name)
            .map(|deps| deps.get_readers().iter().collect())
            .unwrap_or_else(Vec::new)
    }

    /// 查找写入指定变量的所有节点
    pub fn find_writers_of(&self, variable_name: &str) -> Vec<&PlanNodeRef> {
        self.dependencies
            .get(variable_name)
            .map(|deps| deps.get_writers().iter().collect())
            .unwrap_or_else(Vec::new)
    }

    /// 查找指定节点读取的所有变量
    pub fn find_variables_read_by(&self, node: &PlanNodeRef) -> Vec<String> {
        self.dependencies
            .iter()
            .filter(|(_, deps)| deps.is_reader(node))
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// 查找指定节点写入的所有变量
    pub fn find_variables_written_by(&self, node: &PlanNodeRef) -> Vec<String> {
        self.dependencies
            .iter()
            .filter(|(_, deps)| deps.is_writer(node))
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// 检查是否存在数据竞争（同一变量被多个节点写入）
    pub fn detect_write_conflicts(&self) -> Vec<(String, Vec<&PlanNodeRef>)> {
        self.dependencies
            .iter()
            .filter(|(_, deps)| deps.get_writers().len() > 1)
            .map(|(name, deps)| (name.clone(), deps.get_writers().iter().collect()))
            .collect()
    }

    /// 清空所有依赖
    pub fn clear(&mut self) {
        self.dependencies.clear();
    }

    /// 获取依赖数量
    pub fn len(&self) -> usize {
        self.dependencies.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.dependencies.is_empty()
    }
}

impl Default for DependencyTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_tracker() {
        let mut tracker = DependencyTracker::new();

        // 添加变量
        tracker.add_variable("test_var".to_string());
        assert!(tracker.has_variable("test_var"));

        // 添加依赖
        let node1 = PlanNodeRef::new("node1".to_string(), 1);
        let node2 = PlanNodeRef::new("node2".to_string(), 2);

        tracker
            .add_read_dependency("test_var", node1.clone())
            .expect("add_read_dependency should succeed in test");
        tracker
            .add_write_dependency("test_var", node2.clone())
            .expect("add_write_dependency should succeed in test");

        // 检查依赖
        let deps = tracker
            .get_variable_dependencies("test_var")
            .expect("get_variable_dependencies should return Some in test");
        assert!(deps.is_reader(&node1));
        assert!(deps.is_writer(&node2));

        // 查找读取者
        let readers = tracker.find_readers_of("test_var");
        assert_eq!(readers.len(), 1);
        assert_eq!(readers[0].id(), "node1");

        // 查找写入者
        let writers = tracker.find_writers_of("test_var");
        assert_eq!(writers.len(), 1);
        assert_eq!(writers[0].id(), "node2");
    }

    #[test]
    fn test_variable_dependencies() {
        let mut deps = VariableDependencies::new("test_var".to_string());

        let node1 = PlanNodeRef::new("node1".to_string(), 1);
        let node2 = PlanNodeRef::new("node2".to_string(), 2);

        deps.add_reader(node1.clone());
        deps.add_writer(node2.clone());

        assert!(deps.has_readers());
        assert!(deps.has_writers());
        assert!(deps.is_reader(&node1));
        assert!(deps.is_writer(&node2));
        assert!(!deps.is_writer(&node1));
        assert!(!deps.is_reader(&node2));

        // 测试移除
        assert!(deps.remove_reader(&node1));
        assert!(!deps.has_readers());
        assert!(deps.has_writers());
    }

    #[test]
    fn test_write_conflict_detection() {
        let mut tracker = DependencyTracker::new();

        tracker.add_variable("conflict_var".to_string());

        let node1 = PlanNodeRef::new("node1".to_string(), 1);
        let node2 = PlanNodeRef::new("node2".to_string(), 2);

        // 多个节点写入同一变量
        tracker
            .add_write_dependency("conflict_var", node1.clone())
            .expect("add_write_dependency should succeed in conflict test");
        tracker
            .add_write_dependency("conflict_var", node2.clone())
            .expect("add_write_dependency should succeed in conflict test");

        let conflicts = tracker.detect_write_conflicts();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].0, "conflict_var");
        assert_eq!(conflicts[0].1.len(), 2);
    }

    #[test]
    fn test_dependency_stats() {
        let mut tracker = DependencyTracker::new();

        tracker.add_variable("stats_var".to_string());

        let node1 = PlanNodeRef::new("node1".to_string(), 1);
        let node2 = PlanNodeRef::new("node2".to_string(), 2);

        tracker
            .add_read_dependency("stats_var", node1)
            .expect("add_read_dependency should succeed in stats test");
        tracker
            .add_write_dependency("stats_var", node2)
            .expect("add_write_dependency should succeed in stats test");

        let stats = tracker.get_all_stats();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].variable_name, "stats_var");
        assert_eq!(stats[0].reader_count, 1);
        assert_eq!(stats[0].writer_count, 1);
        assert_eq!(stats[0].total_dependencies, 2);
        assert_eq!(stats[0].user_count, 2);
    }
}
