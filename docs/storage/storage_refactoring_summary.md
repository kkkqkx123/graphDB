# Storage 重构总结

本文用于记录当前 storage 重构已经完成的收敛点，以及接下来还值得继续推进的边界。

## 已完成

1. `StorageClient` 已拆成更细的能力接口，并删除了过时的 `StorageContextOps` / `StorageRuntimeContextOps`。
2. `GraphStorageContext` 已拆为 `GraphStoragePersistent`、`GraphStorageRuntime` 和 `GraphStorageLayout`。
3. `MockStorage` 已固定在测试条件编译路径中，不再混入默认运行时构建。
4. `graphdb-query` 中的只读数据访问执行器已经开始向 `StorageReader` 收缩。
5. `graphdb-query` 的基础执行器基座已经去掉了对 `StorageClient` 的直接硬绑定，只有真正需要链式执行和总分发的路径仍然保留完整 `StorageClient`。
6. 近期又继续收缩了一批 admin / mutation 执行器：
   - `AnalyzeExecutor` 现在只依赖 `StorageReader`
   - `Create/Alter/Drop/Show Tag` 与 `Create/Drop/Clear/Show/Switch/Desc Space` 已分别按 reader / schema 能力收口
   - `Insert/Update/Delete/DeleteTag` 已收缩到 `StorageReader + StorageWriter`，`DeleteExecutor` 额外保留 `StorageSchemaOps`，`Create/Drop Index` 已收缩到 `StorageSchemaOps`

## 当前状态

1. 读路径已经明显收窄，例如顶点、边、属性、索引、全文检索、向量检索等只读执行器已经不再直接依赖完整 `StorageClient`。
2. 依赖 `ExecutorEnum` 的链式 traversal / pipeline 仍然需要 `StorageClient`，这是当前结构下的现实边界。
3. `ExecutorEnum` 和管理类子枚举仍然承担“全量分发”的角色，因此它们暂时不能像单一只读执行器那样继续缩窄。
4. `MockStorage` 已经足够明确地停留在测试支持路径，不需要再回退到默认运行时。

## 下一步建议

1. 继续把不依赖 `ExecutorEnum` 的辅助函数和局部执行器收缩到更小的能力接口。
2. 若后续要继续缩窄 traversal 链路，需要先拆分 `ExecutorEnum` 的职责，或者把链式输入从 `ExecutorEnum` 改成更轻的输入协议。
3. 对剩余 admin / mutation 路径继续按写入、schema、auth、同步等能力拆分，不再回到完整 `StorageClient` 作为默认入口。
4. 优先继续收缩 `admin/index`、`admin/user`、`query_management/show_stats` 等仍然直接绑定 `StorageClient` 的 executor。
5. 保持文档精简，只保留总述和当前下一步，不再继续堆积长篇 `analysis` / `plan` 文档。
