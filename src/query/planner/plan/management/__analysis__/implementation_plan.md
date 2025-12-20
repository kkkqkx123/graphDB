# GraphDB 管理计划节点实现计划

## 实现优先级

基于分析报告，我们将按照以下优先级实现缺失的功能：

### 第一阶段：基础功能完善

1. **空间管理**
   - `DropSpace` - 删除空间
   - `ClearSpace` - 清空空间
   - `AlterSpace` - 修改空间

2. **模式管理**
   - `DropTag` - 删除标签
   - `DropEdge` - 删除边
   - `ShowTags` - 显示标签列表
   - `ShowEdges` - 显示边列表
   - `ShowCreateTag` - 显示创建标签的语句
   - `ShowCreateEdge` - 显示创建边的语句

3. **数据操作**
   - `UpdateVertex` - 更新顶点
   - `UpdateEdge` - 更新边
   - `DeleteVertices` - 删除顶点
   - `DeleteEdges` - 删除边

4. **安全管理**
   - `ChangePassword` - 修改密码
   - `ListUsers` - 列出用户
   - `ListUserRoles` - 列出用户角色
   - `DescribeUser` - 描述用户

### 第二阶段：索引管理重构

1. **区分索引类型**
   - 将 `CreateIndex` 拆分为 `CreateTagIndex` 和 `CreateEdgeIndex`
   - 将 `DropIndex` 拆分为 `DropTagIndex` 和 `DropEdgeIndex`
   - 将 `ShowIndexes` 拆分为 `ShowTagIndexes` 和 `ShowEdgeIndexes`
   - 将 `DescIndex` 拆分为 `DescTagIndex` 和 `DescEdgeIndex`

2. **索引状态管理**
   - `ShowTagIndexStatus` - 显示标签索引状态
   - `ShowEdgeIndexStatus` - 显示边索引状态

### 第三阶段：高级功能

1. **系统管理**
   - `ShowMetaLeader` - 显示 Meta 领导者
   - `ShowParts` - 显示分区
   - 监听器管理相关操作
   - 区域管理相关操作
   - 会话和查询管理相关操作

2. **全文索引**
   - `CreateFTIndex` - 创建全文索引
   - `DropFTIndex` - 删除全文索引
   - `ShowFTIndexes` - 显示全文索引列表

## 实现细节

### 1. 空间管理实现

#### DropSpace
```rust
pub struct DropSpace {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub if_exists: bool,
    pub space_name: String,
}
```

#### ClearSpace
```rust
pub struct ClearSpace {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub if_exists: bool,
    pub space_name: String,
}
```

#### AlterSpace
```rust
pub struct AlterSpace {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_name: String,
    pub alter_options: AlterSpaceOptions,
}

pub enum AlterSpaceOptions {
    AddZone(String),
    RemoveZone(String),
    SetPartitionNum(i32),
    SetReplicaFactor(i32),
}
```

### 2. 模式管理实现

#### DropTag
```rust
pub struct DropTag {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub if_exists: bool,
    pub tag_name: String,
}
```

#### DropEdge
```rust
pub struct DropEdge {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub if_exists: bool,
    pub edge_name: String,
}
```

#### ShowTags
```rust
pub struct ShowTags {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}
```

#### ShowEdges
```rust
pub struct ShowEdges {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}
```

### 3. 数据操作实现

#### UpdateVertex
```rust
pub struct UpdateVertex {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub tag_id: i32,
    pub vid: String,
    pub updated_props: Vec<(String, String)>,
    pub insertable: bool,
    pub return_props: Vec<String>,
    pub condition: Option<String>,
}
```

#### UpdateEdge
```rust
pub struct UpdateEdge {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub edge_type_id: i32,
    pub src_id: String,
    pub dst_id: String,
    pub rank: i64,
    pub updated_props: Vec<(String, String)>,
    pub insertable: bool,
    pub return_props: Vec<String>,
    pub condition: Option<String>,
}
```

#### DeleteVertices
```rust
pub struct DeleteVertices {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub vid_ref: String,
}
```

#### DeleteEdges
```rust
pub struct DeleteEdges {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub edge_key_ref: String,
}
```

### 4. 安全管理实现

#### ChangePassword
```rust
pub struct ChangePassword {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub username: String,
    pub password: String,
    pub new_password: String,
}
```

#### ListUsers
```rust
pub struct ListUsers {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}
```

#### ListUserRoles
```rust
pub struct ListUserRoles {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub username: String,
}
```

#### DescribeUser
```rust
pub struct DescribeUser {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub username: String,
}
```

## 实现步骤

1. **创建新文件**：为每个新功能创建对应的 Rust 文件
2. **实现结构体**：按照上述定义实现结构体
3. **实现 Trait**：为每个结构体实现必要的 Trait
4. **更新模块**：更新相应的 mod.rs 文件以导出新功能
5. **测试验证**：编写测试用例验证实现

## 注意事项

1. **保持一致性**：确保所有新实现与现有代码风格一致
2. **错误处理**：为所有新功能添加适当的错误处理
3. **文档注释**：为所有新功能添加详细的文档注释
4. **参数验证**：在构造函数中添加参数验证逻辑
5. **性能考虑**：考虑性能影响，避免不必要的克隆和分配