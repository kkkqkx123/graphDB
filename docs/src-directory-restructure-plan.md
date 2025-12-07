# GraphDB Src目录重构方案

## 当前问题分析

当前src目录下有22个子目录，导致根目录过于拥挤，不利于项目维护和理解。参考nebula-3.8.0的架构，我们需要重新组织目录结构。

## 新目录结构设计

### 1. 核心架构原则

- **减少根目录文件夹数量**：从22个减少到8-10个主要目录
- **功能相关性分组**：将功能相关的模块组织到同一目录下
- **保持core目录的核心地位**：core目录继续作为类型定义的核心
- **创建common目录**：集中通用工具和基础设施代码

### 2. 新目录结构

```
src/
├── lib.rs                    # 库入口文件
├── main.rs                   # 主程序入口
├── core/                     # 核心数据结构和类型定义（保持不变）
│   ├── mod.rs               # 核心类型：Vertex, Edge, Value等
│   ├── error.rs             # 错误类型定义
│   ├── allocator.rs         # 内存分配器
│   ├── lru_cache.rs         # LRU缓存实现
│   ├── cord.rs              # 字符串处理
│   ├── murmur.rs            # 哈希算法
│   ├── signal_handler.rs    # 信号处理
│   ├── collect_n_succeeded.rs
│   └── either.rs            # Either类型实现
├── common/                   # 通用基础设施（新增）
│   ├── mod.rs               # 通用模块入口
│   ├── base/                # 基础工具
│   │   ├── mod.rs
│   │   └── id.rs            # ID生成相关（从id/迁移）
│   ├── time/                # 时间处理（从time/迁移）
│   │   └── mod.rs
│   ├── memory/              # 内存管理（从memory/迁移）
│   │   └── mod.rs
│   ├── thread/              # 线程管理（从thread/迁移）
│   │   └── mod.rs
│   ├── process/             # 进程管理（从process/迁移）
│   │   └── mod.rs
│   ├── network/             # 网络工具（从network/迁移）
│   │   └── mod.rs
│   ├── fs/                  # 文件系统工具（从fs/迁移）
│   │   └── mod.rs
│   ├── log/                 # 日志系统（从log/迁移）
│   │   └── mod.rs
│   └── charset/             # 字符集处理（从charset/迁移）
│       └── mod.rs
├── storage/                  # 存储引擎（保持不变）
│   └── mod.rs               # 存储层实现
├── query/                    # 查询引擎（保持不变）
│   └── mod.rs               # 查询处理和执行
├── graph/                    # 图操作（扩展）
│   ├── mod.rs               # 图操作入口（从graph/迁移）
│   ├── transaction/         # 事务管理（从transaction/迁移）
│   │   └── mod.rs
│   ├── index/               # 索引系统（从index/迁移）
│   │   └── mod.rs
│   └── expression/          # 表达式计算（从expression/迁移）
│       └── mod.rs
├── api/                      # API接口层（保持不变）
│   └── mod.rs               # HTTP API和服务接口
├── config/                   # 配置管理（保持不变）
│   └── mod.rs               # 配置加载和管理
└── services/                 # 服务层（新增）
    ├── mod.rs               # 服务层入口
    ├── session/             # 会话管理（从session/迁移）
    │   └── mod.rs
    ├── stats/               # 统计服务（从stats/迁移）
    │   └── mod.rs
    ├── function/            # 函数服务（从function/迁移）
    │   └── mod.rs
    ├── algorithm/           # 算法服务（从algorithm/迁移）
    │   └── mod.rs
    └── context/             # 上下文管理（从context/迁移）
        └── mod.rs
```

### 3. 目录功能说明

#### core/ - 核心数据结构
- 保持现有结构不变
- 包含所有核心数据类型定义：Vertex, Edge, Value等
- 作为整个项目的基础类型系统

#### common/ - 通用基础设施
- **目的**：集中所有通用工具和基础设施代码
- **包含模块**：
  - `base/`：基础工具，如ID生成
  - `time/`：时间处理相关功能
  - `memory/`：内存管理工具
  - `thread/`：线程管理和同步工具
  - `process/`：进程管理工具
  - `network/`：网络相关工具
  - `fs/`：文件系统操作工具
  - `log/`：日志系统
  - `charset/`：字符集处理

#### graph/ - 图操作核心
- **目的**：集中所有图相关的核心操作
- **包含模块**：
  - 事务管理
  - 索引系统
  - 表达式计算
  - 图算法基础

#### services/ - 服务层
- **目的**：提供高级服务和功能
- **包含模块**：
  - 会话管理
  - 统计服务
  - 函数服务
  - 算法服务
  - 上下文管理

### 4. 模块依赖关系

```
api/
├── config/
├── query/
└── services/

query/
├── storage/
├── graph/
└── core/

graph/
├── core/
└── common/

storage/
├── core/
└── common/

services/
├── core/
├── common/
├── graph/
└── storage/

config/
└── common/

common/
└── core/
```

### 5. 迁移优势

1. **减少根目录复杂度**：从22个目录减少到8个主要目录
2. **更好的功能分组**：相关功能模块集中管理
3. **清晰的依赖层次**：从core→common→graph/services→api的清晰层次
4. **易于维护**：每个目录职责明确，便于定位和修改
5. **扩展性好**：新功能可以容易地归类到相应目录

### 6. 与nebula-3.8.0的对比

| nebula-3.8.0 | 新GraphDB结构 | 说明 |
|-------------|-------------|------|
| common/ | common/ | 都有通用模块，但新结构更细分 |
| graph/ | graph/ | 图相关功能，新结构包含事务和索引 |
| storage/ | storage/ | 存储层，基本保持一致 |
| daemons/ | api/ | 服务入口，新结构更简洁 |
| parser/ | query/ | 查询处理，新结构更全面 |
| meta/ | services/ | 元数据和服务管理 |

### 7. 实施步骤

1. **创建common目录结构**
2. **迁移基础模块到common**
3. **重组graph相关模块**
4. **创建services目录并迁移服务模块**
5. **更新所有模块的导入路径**
6. **更新lib.rs和main.rs**
7. **测试验证**

这个重构方案既保持了项目的核心架构，又显著减少了根目录的复杂度，同时提供了更好的功能组织和清晰的依赖关系。