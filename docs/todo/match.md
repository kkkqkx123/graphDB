、
 src\query\planner\match_planning\label_index_seek.rs:22-88
```

    /// 构建标签索引查找计划
    pub fn build_plan(&self) -> Result<SubPlan, PlannerError> {
        // 检查是否只有一个标签，目前不支持多标签索引查找
        if self.node_info.labels.len() != 1 {
            return Err(PlannerError::UnsupportedOperation(
                "Multiple tag index seek is not supported now.".to_string(),
            ));
        }

        // 创建索引扫描节点
        let mut index_scan_node = Box::new(SingleInputNode::new(
            PlanNodeKind::IndexScan,
            create_start_node()?,
        ));

        // 设置标签索引信息
        // 根据node_info.labels和node_info.tids设置要扫描的标签索引
        if !self.node_info.labels.is_empty() && !self.node_info.tids.is_empty() {
            // 获取标签ID和标签名称
            let label_id = self.node_info.tids[0];
            let label_name = &self.node_info.labels[0];
            
            // 设置索引查询上下文
            // 在实际实现中，这里应该查询元数据获取对应的索引ID
            // 目前简化处理，直接使用标签ID作为索引ID
            let index_id = label_id;
            
            // 设置输出变量
            index_scan_node.set_output_var(Some(format!("index_scan_{}", label_name)));
            
            // 设置列名，包含顶点ID
            let col_names = vec!["vid".to_string()];
            index_scan_node.set_col_names(col_names);
            
            // 存储索引信息以便后续使用
            // 在实际实现中，这些信息应该存储在节点的附加数据中
            // 这里简化处理，通过输出变量名传递部分信息
            let index_info = format!("{}:{}", label_id, index_id);
            index_scan_node.set_output_var(Some(format!("index_{}_{}", label_name, index_info)));
        }

        // 处理节点属性过滤
        if let Some(props) = &self.node_info.props {
            // 在实际实现中，属性过滤应该嵌入到索引扫描中
            // 这里简化处理，标记有属性过滤
            index_scan_node.set_output_var(Some(format!("with_props_{}",
                index_scan_node.output_var().as_ref().unwrap_or(&"default".to_string()))));
        }

        // 处理节点过滤条件
        if let Some(filter) = &self.node_info.filter {
            // 在实际实现中，过滤条件应该嵌入到索引扫描中
            // 这里简化处理，标记有过滤条件
            index_scan_node.set_output_var(Some(format!("with_filter_{}",
                index_scan_node.output_var().as_ref().unwrap_or(&"default".to_string()))));
        }

        Ok(SubPlan::new(Some(index_scan_node.clone()), Some(index_scan_node)))
    }

    /// 检查是否可以使用标签索引查找
    pub fn match_node(&self) -> bool {
        // 如果节点有标签，可以使用标签索引查找
        !self.node_info.labels.is_empty()
    }
}
```

src\query\planner\match_planning\match_planner.rs:241-286
```
impl Planner for MatchPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // 验证这是MATCH语句
        if !Self::match_ast_ctx(ast_ctx) {
            return Err(PlannerError::InvalidAstContext(
                "Only MATCH statements are accepted by MatchPlanner".to_string(),
            ));
        }

        // 从AST上下文中提取Cypher上下文
        // 这里需要解析AST并构建相应的Cypher上下文结构
        // 在实际实现中，应该根据AST内容构建CypherContext
        // 由于当前简化实现，我们创建一个基本的查询计划
        
        let mut query_plan = SubPlan::new(None, None);

        // 创建起始节点
        let start_node = Box::new(SingleDependencyNode {
            id: -1,
            kind: PlanNodeKind::Start,
            dependencies: vec![],
            output_var: None,
            col_names: vec![],
            cost: 0.0,
        });

        // 创建获取邻居节点
        let get_neighbors_node = Box::new(SingleDependencyNode {
            id: -1,
            kind: PlanNodeKind::GetNeighbors,
            dependencies: vec![start_node],
            output_var: None,
            col_names: vec!["vertex".to_string()],
            cost: 1.0,
        });

        query_plan.root = Some(get_neighbors_node.clone());
        query_plan.tail = Some(get_neighbors_node);

        Ok(query_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}
```

