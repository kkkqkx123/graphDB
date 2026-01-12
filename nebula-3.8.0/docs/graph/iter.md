从你提供的 Nebula Graph 迭代器源码中，可以清晰地看到它执行**图遍历和节点迭代**的核心机制。这些迭代器是查询执行引擎访问图数据的**统一抽象接口**。

## 🏗️ 迭代器类层次结构

Nebula Graph 的迭代器系统采用**多态设计**，通过基类 `Iterator` 定义统一接口：

```
Iterator（基类）
├── SequentialIter（顺序迭代器 - 通用数据集）
├── GetNeighborsIter（邻居迭代器 - 核心图遍历）
├── PropIter（属性迭代器 - 属性查询）
└── DefaultIter（默认迭代器 - 简单包装）
```

## 🔍 核心迭代器工作原理

### **GetNeighborsIter：图遍历的核心**
这是执行 `GO` 查询（如 `GO FROM "player100" OVER follow`）的关键迭代器，它处理存储层返回的**邻居数据集**。

**数据结构关键点**：
```cpp
struct DataSetIndex {
    const DataSet* ds;  // 存储层返回的数据集
    // 列名到索引的映射：{ "_vid": 0, "_stats": 1, "_tag:player:name": 2, ... }
    std::unordered_map<std::string, size_t> colIndices;
    // 属性索引："_tag:player:name:age" -> { "player": {colIdx: 2, propIndices: {"name":0, "age":1}} }
    std::unordered_map<std::string, PropIndex> tagPropsMap;
    std::unordered_map<std::string, PropIndex> edgePropsMap;
};
```

**迭代执行流程（四层嵌套循环）**：
1. **数据集列表层**：`dsIndices_`（可能多个分片返回多个数据集）
2. **数据行层**：`currentRow_`（每个顶点一行）
3. **边列层**：`colIdx_`（每个边类型一列）
4. **边列表层**：`edgeIdx_`（每个邻接边一条记录）

### **PropIter：属性查询迭代器**
用于 `FETCH PROP ON` 查询，专门优化属性访问：
```cpp
// 属性索引结构：tag.prop -> 列索引
// {"player.name": 1, "player.age": 2, "team.name": 3}
std::unordered_map<std::string, std::unordered_map<std::string, size_t>> propsMap;
```

### **SequentialIter：通用顺序迭代器**
包装常规 `DataSet`，提供简单的行级迭代，用于中间结果处理。

## 🔄 节点迭代的具体过程

以 `GetNeighborsIter` 为例，遍历邻居节点的**完整流程**：

### **1. 数据准备阶段**
存储层返回的数据集格式示例：
```
行结构: [_vid, _stats, _tag:player:name:age, _edge:follow:_dst:_rank]
示例行: ["player100", stats, ["LeBron", 38], [["player101", 1], ["player102", 1]]]
```

### **2. 迭代初始化 (`goToFirstEdge()`)** 
```cpp
// 定位到第一个有效的边数据
for (遍历所有数据集) {
    for (遍历数据集中的所有行) {
        for (从colLowerBound遍历到colUpperBound) {
            if (当前列是边列且非空) {
                currentCol_ = &边列表;
                edgeIdx_ = 0;
                valid_ = true;  // 找到有效边，开始迭代
                break;
            }
        }
    }
}
```

### **3. 逐边迭代 (`next()`)** 
```cpp
while (++edgeIdx_ < edgeIdxUpperBound_) {
    if (当前边有效 && bitset_[bitIdx_] == true) {
        currentEdge_ = &边属性列表;
        break;  // 找到下一个有效边
    }
}
// 如果当前列遍历完，移动到下一列/下一行/下一个数据集
```

### **4. 属性访问接口**
迭代过程中，可以通过多种方式获取数据：
```cpp
// 获取顶点ID
Value vid = getColumn(0);  // 第一列总是_vid

// 获取顶点属性
Value playerName = getTagProp("player", "name");

// 获取边属性
Value edgeRank = getEdgeProp("follow", "_rank");

// 获取完整顶点对象
Vertex v = getVertex();  // 包含所有tag属性
```

### **5. 过滤与采样**
- **过滤**：通过 `bitset_` 标记被过滤掉的边，跳过不满足条件的边
- **采样**：`sample(count)` 方法使用**蓄水池抽样算法**随机采样指定数量的边

## 💡 设计特点与优化

### **1. 懒加载与缓存**
```cpp
// 顶点缓存：避免重复构建Vertex对象
Value prevVertex_;
if (!prevVertex_.empty() && prevVertex_.getVertex().vid == vidVal) {
    return prevVertex_;  // 直接返回缓存的顶点
}
```

### **2. 内存感知迭代**
```cpp
// 定期检查内存水位
bool hitsSysMemoryHighWatermark() const {
    if (numRowsModN_ >= FLAGS_num_rows_to_check_memory) {
        if (memory::MemoryUtils::kHitMemoryHighWatermark.load()) {
            throw std::runtime_error("内存超出限制");
        }
    }
}
```

### **3. 统一的多迭代器合并**
```cpp
// 支持多个迭代器结果合并
SequentialIter::SequentialIter(std::vector<std::unique_ptr<Iterator>> inputList) {
    // 合并所有输入迭代器的数据行
    for (auto& iter : iterators) {
        ds.rows.insert(ds.rows.end(), iter->begin(), iter->end());
    }
}
```

### **4. 灵活的属性访问**
通过预构建的索引快速定位属性：
```cpp
// 根据 "player.name" 快速找到列索引
auto it = dsIndex_.colIndices.find("player.name");
if (it != dsIndex_.colIndices.end()) {
    return currentRow_->values[it->second];
}
```

## 🎯 总结：Nebula Graph 节点迭代的核心思想

1. **分层迭代策略**：数据集 → 数据行 → 边列 → 边记录的四层遍历结构
2. **统一接口抽象**：所有迭代器实现相同的 `Iterator` 接口，执行引擎无需关心具体类型
3. **属性索引优化**：预构建属性名到列索引的映射，实现 O(1) 复杂度的属性访问
4. **内存敏感设计**：定期检查内存使用，防止大规模遍历导致 OOM
5. **分布式透明**：迭代器封装了跨分片的数据集，对上层表现为统一的迭代接口

这种设计使得 Nebula Graph 能够高效处理**大规模分布式图遍历**，同时为不同查询类型（`GO`、`FETCH`、`MATCH`）提供一致的迭代抽象。

如果你对特定迭代器的某个方法或某个查询类型的执行细节有更深入的疑问，我可以进一步分析。