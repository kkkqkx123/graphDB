添加以下节点：

/// 显示Meta领导者计划节点（已注释：单节点架构不需要）
// #[derive(Debug, Clone)]
// pub struct ShowMetaLeader {
//     pub id: i64,
//     pub cost: f64,
// }
//
// impl ShowMetaLeader {
//     pub fn new(id: i64, cost: f64) -> Self {
//         Self { id, cost }
//     }
// }
//
// impl ManagementNode for ShowMetaLeader {
//     fn id(&self) -> i64 {
//         self.id
//     }
//
//     fn name(&self) -> &'static str {
//         "ShowMetaLeader"
//     }
//
//     fn cost(&self) -> f64 {
//         self.cost
//     }
//
//     fn into_enum(self) -> ManagementNodeEnum {
//         ManagementNodeEnum::ShowMetaLeader(self)
//     }
// }

/// 显示分区计划节点（已注释：单节点架构不需要）
// #[derive(Debug, Clone)]
// pub struct ShowParts {
//     pub id: i64,
//     pub cost: f64,
//     pub space_name: Option<String>,
// }
//
// impl ShowParts {
//     pub fn new(id: i64, cost: f64, space_name: Option<String>) -> Self {
//         Self { id, cost, space_name }
//     }
//
//     pub fn space_name(&self) -> Option<&str> {
//         self.space_name.as_deref()
//     }
// }
//
// impl ManagementNode for ShowParts {
//     fn id(&self) -> i64 {
//         self.id
//     }
//
//     fn name(&self) -> &'static str {
//         "ShowParts"
//     }
//
//     fn cost(&self) -> f64 {
//         self.cost
//     }
//
//     fn into_enum(self) -> ManagementNodeEnum {
//         ManagementNodeEnum::ShowParts(self)
//     }
// }