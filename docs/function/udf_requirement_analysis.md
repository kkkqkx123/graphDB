# UDF 需求分析与必要性评估

## 1. 当前函数覆盖情况分析

### 1.1 当前项目已实现函数

根据对 [registry.rs](file:///d:/项目/database/graphDB/src/expression/functions/registry.rs) 的分析，当前项目已实现以下内置函数：

#### 数学函数 (13个)
| 函数 | 描述 | 状态 |
|------|------|------|
| abs | 绝对值 | ✅ |
| ceil | 向上取整 | ✅ |
| floor | 向下取整 | ✅ |
| round | 四舍五入 | ✅ |
| sqrt | 平方根 | ✅ |
| pow | 幂运算 | ✅ |
| exp | 指数 | ✅ |
| log | 自然对数 | ✅ |
| log10 | 以10为底对数 | ✅ |
| sin | 正弦 | ✅ |
| cos | 余弦 | ✅ |
| tan | 正切 | ✅ |

#### 字符串函数 (10个)
| 函数 | 描述 | 状态 |
|------|------|------|
| length | 字符串长度 | ✅ |
| upper/toupper | 转大写 | ✅ |
| lower/tolower | 转小写 | ✅ |
| trim | 去除首尾空白 | ✅ |
| ltrim | 去除左侧空白 | ✅ |
| rtrim | 去除右侧空白 | ✅ |
| concat | 字符串连接 | ✅ |
| replace | 字符串替换 | ✅ |
| contains | 包含检查 | ✅ |
| starts_with | 前缀检查 | ✅ |
| ends_with | 后缀检查 | ✅ |

#### 正则表达式函数 (3个)
| 函数 | 描述 | 状态 |
|------|------|------|
| regex_match | 正则匹配 | ✅ |
| regex_replace | 正则替换 | ✅ |
| regex_find | 正则查找 | ✅ |

#### 类型转换函数 (4个)
| 函数 | 描述 | 状态 |
|------|------|------|
| to_string | 转字符串 | ✅ |
| to_int | 转整数 | ✅ |
| to_float | 转浮点数 | ✅ |
| to_bool | 转布尔值 | ✅ |

#### 日期时间函数 (8个)
| 函数 | 描述 | 状态 |
|------|------|------|
| now | 当前时间戳 | ✅ |
| date | 创建日期 | ✅ |
| year | 提取年份 | ✅ |
| month | 提取月份 | ✅ |
| day | 提取日 | ✅ |
| hour | 提取小时 | ✅ |
| minute | 提取分钟 | ✅ |
| second | 提取秒 | ✅ |

**当前总计：约 40+ 个内置函数**

---

### 1.2 Nebula-Graph 函数对比

根据对 [FunctionManager.cpp](file:///d:/项目/database/graphDB/nebula-3.8.0/src/common/function/FunctionManager.cpp) 的分析，Nebula 实现了以下额外函数：

#### Nebula 特有但当前项目缺失的函数

**数学函数**
| 函数 | 描述 | 复杂度 | 是否适合UDF |
|------|------|--------|-------------|
| bit_and | 按位与 | 低 | ❌ 应内置 |
| bit_or | 按位或 | 低 | ❌ 应内置 |
| bit_xor | 按位异或 | 低 | ❌ 应内置 |
| cbrt | 立方根 | 低 | ❌ 应内置 |
| hypot | 欧几里得距离 | 低 | ❌ 应内置 |
| exp2 | 2的幂 | 低 | ❌ 应内置 |
| log2 | 以2为底对数 | 低 | ❌ 应内置 |
| asin | 反正弦 | 低 | ❌ 应内置 |
| acos | 反余弦 | 低 | ❌ 应内置 |
| atan | 反正切 | 低 | ❌ 应内置 |
| sign | 符号函数 | 低 | ❌ 应内置 |
| rand | 随机数 | 中 | ⚠️ 可UDF |
| rand32/rand64 | 随机整数 | 中 | ⚠️ 可UDF |
| pi | 圆周率 | 低 | ❌ 应内置 |
| e | 自然常数 | 低 | ❌ 应内置 |
| radians | 角度转弧度 | 低 | ❌ 应内置 |

**字符串函数**
| 函数 | 描述 | 复杂度 | 是否适合UDF |
|------|------|--------|-------------|
| strcasecmp | 不区分大小写比较 | 低 | ❌ 应内置 |
| left | 左侧截取 | 低 | ❌ 应内置 |
| right | 右侧截取 | 低 | ❌ 应内置 |
| reverse | 字符串反转 | 低 | ❌ 应内置 |
| split | 字符串分割 | 低 | ❌ 应内置 |
| lpad | 左侧填充 | 低 | ❌ 应内置 |
| rpad | 右侧填充 | 低 | ❌ 应内置 |
| substr/substring | 子字符串 | 低 | ❌ 应内置 |
| hash | 哈希值 | 中 | ⚠️ 可UDF |

**图相关函数**
| 函数 | 描述 | 复杂度 | 是否适合UDF |
|------|------|--------|-------------|
| id | 获取顶点ID | 低 | ❌ 必须内置 |
| tags/labels | 获取顶点标签 | 低 | ❌ 必须内置 |
| properties | 获取属性映射 | 低 | ❌ 必须内置 |
| type | 获取边类型 | 低 | ❌ 必须内置 |
| typeid | 获取边类型ID | 低 | ❌ 必须内置 |
| rank | 获取边rank | 低 | ❌ 必须内置 |
| startnode | 获取边起点 | 低 | ❌ 必须内置 |
| endnode | 获取边终点 | 低 | ❌ 必须内置 |
| keys | 获取属性键列表 | 低 | ❌ 必须内置 |
| nodes | 获取路径节点 | 低 | ❌ 必须内置 |
| relationships | 获取路径边 | 低 | ❌ 必须内置 |
| head | 获取列表首元素 | 低 | ❌ 应内置 |
| tail | 获取列表尾元素 | 低 | ❌ 应内置 |
| last | 获取列表末元素 | 低 | ❌ 应内置 |
| range | 生成范围列表 | 低 | ❌ 应内置 |
| size | 获取容器大小 | 低 | ❌ 应内置 |
| hassameedgeinpath | 路径是否有重复边 | 中 | ⚠️ 可UDF |
| hassamevertexinpath | 路径是否有重复顶点 | 中 | ⚠️ 可UDF |
| reversepath | 反转路径 | 低 | ❌ 应内置 |

**日期时间函数**
| 函数 | 描述 | 复杂度 | 是否适合UDF |
|------|------|--------|-------------|
| time | 创建时间 | 低 | ❌ 应内置 |
| datetime | 创建日期时间 | 低 | ❌ 应内置 |
| timestamp | 获取时间戳 | 低 | ❌ 应内置 |

**地理空间函数**
| 函数 | 描述 | 复杂度 | 是否适合UDF |
|------|------|--------|-------------|
| st_point | 创建地理点 | 中 | ✅ 适合UDF |
| st_geogfromtext | WKT解析 | 高 | ✅ 适合UDF |
| st_astext | WKT输出 | 高 | ✅ 适合UDF |
| st_centroid | 计算质心 | 高 | ✅ 适合UDF |
| st_isvalid | 验证地理数据 | 中 | ✅ 适合UDF |
| st_intersects | 空间相交判断 | 高 | ✅ 适合UDF |
| st_covers | 空间覆盖判断 | 高 | ✅ 适合UDF |
| st_coveredby | 空间被覆盖判断 | 高 | ✅ 适合UDF |
| st_dwithin | 距离内判断 | 高 | ✅ 适合UDF |
| st_distance | 空间距离计算 | 高 | ✅ 适合UDF |
| s2_cellidfrompoint | S2单元格ID | 高 | ✅ 适合UDF |
| s2_coveringcellids | S2覆盖单元格 | 高 | ✅ 适合UDF |

**其他函数**
| 函数 | 描述 | 复杂度 | 是否适合UDF |
|------|------|--------|-------------|
| toset | 转集合 | 低 | ❌ 应内置 |
| udf_is_in | 成员检查 | 低 | ❌ 应内置 |
| cos_similarity | 余弦相似度 | 中 | ✅ 适合UDF |
| extract | 正则提取 | 中 | ⚠️ 可UDF |
| json_extract | JSON提取 | 高 | ✅ 适合UDF |
| score | 获取搜索分数 | 中 | ⚠️ 可UDF |
| duration | 持续时间 | 中 | ⚠️ 可UDF |
| datasetrowcol | 数据集行列访问 | 低 | ❌ 应内置 |

---

## 2. 需要通过 UDF 实现的功能识别

### 2.1 适合 UDF 实现的功能特征

适合通过 UDF 实现的功能通常具有以下特征：

1. **领域特定**：只在特定领域使用（如地理空间、机器学习）
2. **复杂度高**：实现复杂，依赖大量外部库
3. **使用频率低**：不是核心查询功能
4. **可选依赖**：不希望增加核心系统的依赖
5. **用户定制**：需要用户自定义逻辑

### 2.2 推荐通过 UDF 实现的功能

#### 高优先级（强烈建议UDF）

| 功能类别 | 函数示例 | 理由 |
|----------|----------|------|
| **地理空间函数** | st_point, st_distance, st_intersects | 依赖复杂几何库（如 geos、s2geometry），不是核心功能 |
| **机器学习函数** | cos_similarity, vector_distance | 依赖 ML 库，向量计算复杂 |
| **JSON处理** | json_extract, json_path | 依赖 JSON 解析库，可选功能 |
| **加密/哈希** | md5, sha256, aes_encrypt | 安全敏感，可选功能 |

#### 中优先级（可考虑UDF）

| 功能类别 | 函数示例 | 理由 |
|----------|----------|------|
| **文本分析** | tokenize, stemming | NLP 功能，依赖外部库 |
| **统计函数** | percentile, correlation | 统计分析，使用频率较低 |
| **编码/解码** | base64, url_encode | 实用但不核心 |
| **网络相关** | inet_aton, inet_ntoa | 特定场景使用 |

#### 低优先级（建议内置）

| 功能类别 | 函数示例 | 理由 |
|----------|----------|------|
| **缺失的数学函数** | bit_and, cbrt, asin | 简单数学运算，应内置 |
| **缺失的字符串函数** | left, right, split | 基础字符串处理，应内置 |
| **图相关函数** | id, tags, properties | 核心图查询功能，必须内置 |
| **列表函数** | head, tail, range | 基础容器操作，应内置 |

---

## 3. 引入 UDF 的必要性评估

### 3.1 当前阶段评估

#### 项目现状
- **当前阶段**：基础功能开发阶段
- **核心需求**：稳定的图查询功能
- **用户规模**：个人/小规模使用
- **部署方式**：单节点本地部署

#### 必要性分析

| 评估维度 | 评分 | 说明 |
|----------|------|------|
| **功能需求紧迫性** | ⭐⭐☆☆☆ | 当前内置函数已覆盖80%常见需求 |
| **技术复杂度** | ⭐⭐⭐⭐☆ | UDF框架设计复杂，涉及动态加载安全 |
| **维护成本** | ⭐⭐⭐⭐☆ | 需要维护UDF API兼容性、安全问题 |
| **用户价值** | ⭐⭐⭐☆☆ | 对个人用户价值有限，企业用户更需 |
| **实现成本** | ⭐⭐⭐⭐⭐ | 需要大量开发和测试工作 |

**综合评估：当前阶段不建议引入 UDF**

### 3.2 各阶段建议

#### Phase 1: 基础功能完善期（当前）
- **时间**：0-6个月
- **重点**：完善内置函数，特别是图相关函数
- **UDF 建议**：❌ 不引入
- **替代方案**：通过代码注册方式添加函数

#### Phase 2: 功能稳定期
- **时间**：6-12个月
- **重点**：性能优化、稳定性提升
- **UDF 建议**：⚠️ 可考虑简单版本（仅支持Rust插件）
- **实现范围**：仅支持编译时链接的插件

#### Phase 3: 生态扩展期
- **时间**：12个月以后
- **重点**：扩展生态、企业级功能
- **UDF 建议**：✅ 完整UDF支持
- **实现范围**：动态加载、多语言支持（WASM）

---

## 4. 替代方案建议

### 4.1 短期方案：完善内置函数

优先实现以下缺失的核心函数：

```rust
// 优先级1：图相关函数（必须）
- id(vertex) -> ID
- tags(vertex) -> List<String>
- properties(vertex|edge) -> Map
- type(edge) -> String
- startnode(edge) -> Vertex
- endnode(edge) -> Vertex

// 优先级2：容器函数（应内置）
- head(list) -> Any
- tail(list) -> List
- last(list) -> Any
- range(start, end, step) -> List
- size(container) -> Int

// 优先级3：数学函数（应内置）
- bit_and, bit_or, bit_xor
- asin, acos, atan
- cbrt, hypot
- rand, rand32, rand64

// 优先级4：字符串函数（应内置）
- left, right
- substr/substring
- split
- lpad, rpad
- reverse
```

### 4.2 中期方案：静态插件系统

在引入完整 UDF 之前，可实现简化版静态插件：

```rust
// src/expression/functions/plugin.rs

/// 静态插件 trait（编译时链接）
pub trait FunctionPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn register(&self, registry: &mut FunctionRegistry);
}

/// 插件管理器
pub struct PluginManager {
    plugins: Vec<Box<dyn FunctionPlugin>>,
}

impl PluginManager {
    /// 注册插件（编译时确定）
    pub fn register_plugin(&mut self, plugin: Box<dyn FunctionPlugin>) {
        self.plugins.push(plugin);
    }
    
    /// 加载所有插件函数
    pub fn load_all(&self, registry: &mut FunctionRegistry) {
        for plugin in &self.plugins {
            plugin.register(registry);
        }
    }
}
```

**优势：**
- 无需动态加载，安全性高
- 编译时类型检查
- 性能无损耗
- 实现简单

### 4.3 长期方案：完整 UDF

当项目进入生态扩展期后，再考虑实现完整 UDF 支持：

```
建议技术路线：
1. Rust 动态库加载（libloading）
2. WASM 支持（wasmer/wasmtime）
   - 更好的安全性（沙箱）
   - 跨语言支持
   - 标准化接口
```

---

## 5. 结论与建议

### 5.1 核心结论

1. **当前不建议引入 UDF**
   - 项目处于基础功能开发阶段
   - 内置函数已覆盖大部分需求
   - UDF 实现复杂度高，维护成本大

2. **优先完善内置函数**
   - 特别是图相关函数（id, tags, properties 等）
   - 补充常用数学和字符串函数

3. **未来可考虑静态插件**
   - 作为 UDF 的过渡方案
   - 编译时链接，安全可控

### 5.2 实施路线图

```
当前阶段（0-6个月）
├── 完善核心图函数（id, tags, properties等）
├── 补充基础数学/字符串函数
└── 稳定函数注册系统

中期阶段（6-12个月）
├── 评估静态插件系统
├── 实现可选功能（地理空间、JSON）
└── 性能优化

长期阶段（12个月+）
├── 评估完整 UDF 需求
├── 设计 UDF 安全模型
└── 实现动态加载/WASM支持
```

### 5.3 决策树

```
是否需要添加新函数？
├── 是核心图查询功能？
│   └── 是 → 内置实现
├── 是基础数学/字符串操作？
│   └── 是 → 内置实现
├── 依赖复杂外部库？
│   └── 是 → 考虑静态插件
├── 领域特定（地理空间/ML）？
│   └── 是 → 未来考虑UDF
└── 用户自定义逻辑？
    └── 是 → 长期考虑UDF
```
