# AST 上下文模块设计文档
**模块位置**: `src/query/context/ast/`

## 1. 模块概述

AST 上下文模块负责管理查询执行过程中的抽象语法树（AST）相关上下文信息，为查询解析、验证和执行提供必要的上下文环境。

### 1.1 核心职责
- 存储和管理 AST 节点信息
- 提供查询语句的上下文环境
- 管理命名空间和变量作用域
- 支持查询重写和优化

### 1.2 设计原则
- **单一职责**: 专注于 AST 上下文管理
- **不可变性**: 上下文信息在创建后不可修改
- **轻量级**: 最小化内存占用和性能开销
- **可扩展**: 支持未来功能扩展

## 2. 架构设计

### 2.1 核心数据结构

```rust
// src/query/context/ast/base.rs
pub struct AstContext {
    // 查询上下文（可选，用于与执行上下文关联）
    qctx: Option<Arc<QueryContext>>,
    
    // 查询语句的 AST 表示
    sentence: Option<Arc<dyn Sentence>>,
    
    // 命名空间信息
    space: SpaceInfo,
    
    // 变量作用域管理
    variable_scope: VariableScope,
    
    // 查询类型标识
    query_type: QueryType,
}

// 命名空间信息
pub struct SpaceInfo {
    pub space_name: String,
    pub space_id: Option<u32>,
    pub is_default: bool,
}

// 变量作用域
pub struct VariableScope {
    pub current_scope: HashMap<String, VariableInfo>,
    pub parent_scope: Option<Arc<VariableScope>>,
}

// 查询类型枚举
pub enum QueryType {
    ReadQuery,
    WriteQuery,
    AdminQuery,
    SchemaQuery,
}
```

### 2.2 接口设计

```rust
// AST 上下文接口
trait AstContextTrait {
    // 获取查询上下文
    fn get_query_context(&self) -> Option<Arc<QueryContext>>;
    
    // 获取查询语句
    fn get_sentence(&self) -> Option<Arc<dyn Sentence>>;
    
    // 获取命名空间信息
    fn get_space_info(&self) -> &SpaceInfo;
    
    // 变量查找
    fn lookup_variable(&self, name: &str) -> Option<VariableInfo>;
    
    // 上下文验证
    fn validate(&self) -> Result<(), ValidationError>;
}
```

## 3. NebulaGraph 对标分析

### 3.1 NebulaGraph 实现参考

NebulaGraph 中的 AST 上下文主要通过以下类实现：

```cpp
// nebula-3.8.0/src/graph/context/AstContext.h
class AstContext {
public:
    explicit AstContext(QueryContext* qctx);
    
    // 查询语句管理
    Sentence* sentence() const;
    void setSentence(Sentence* sentence);
    
    // 命名空间管理
    const std::string& spaceName() const;
    void setSpaceName(const std::string& name);
    
    // 变量作用域
    VariableScope* variableScope() const;
    
    // 验证接口
    Status validate();
};
```

### 3.2 关键差异与改进

| 特性 | NebulaGraph | GraphDB 设计 | 改进点 |
|------|-------------|--------------|--------|
| 生命周期管理 | 手动管理 | RAII + Arc | 自动内存管理 |
| 线程安全 | 非线程安全 | 线程安全设计 | 支持并发访问 |
| 错误处理 | 异常机制 | Result 类型 | 更安全的错误处理 |
| 扩展性 | 继承扩展 | Trait 扩展 | 更灵活的扩展机制 |

## 4. 核心功能实现

### 4.1 上下文创建

```rust
impl AstContext {
    // 从查询字符串创建上下文
    pub fn from_query(query_text: &str) -> Result<Self, QueryError> {
        // 1. 解析查询字符串
        let sentence = parse_query(query_text)?;
        
        // 2. 提取命名空间信息
        let space_info = extract_space_info(&sentence)?;
        
        // 3. 构建上下文
        Ok(AstContext {
            qctx: None,
            sentence: Some(Arc::new(sentence)),
            space: space_info,
            variable_scope: VariableScope::new(),
            query_type: determine_query_type(query_text),
        })
    }
    
    // 从现有上下文创建（用于查询重写）
    pub fn from_existing(
        existing: &AstContext,
        new_sentence: Arc<dyn Sentence>,
    ) -> Self {
        AstContext {
            qctx: existing.qctx.clone(),
            sentence: Some(new_sentence),
            space: existing.space.clone(),
            variable_scope: existing.variable_scope.clone(),
            query_type: existing.query_type,
        }
    }
}
```

### 4.2 变量作用域管理

```rust
impl VariableScope {
    pub fn new() -> Self {
        VariableScope {
            current_scope: HashMap::new(),
            parent_scope: None,
        }
    }
    
    // 添加变量到当前作用域
    pub fn add_variable(&mut self, name: String, info: VariableInfo) -> Result<(), ScopeError> {
        if self.current_scope.contains_key(&name) {
            return Err(ScopeError::VariableAlreadyExists(name));
        }
        self.current_scope.insert(name, info);
        Ok(())
    }
    
    // 查找变量（支持作用域链）
    pub fn lookup(&self, name: &str) -> Option<VariableInfo> {
        if let Some(info) = self.current_scope.get(name) {
            return Some(info.clone());
        }
        
        if let Some(parent) = &self.parent_scope {
            return parent.lookup(name);
        }
        
        None
    }
}
```

### 4.3 上下文验证

```rust
impl AstContext {
    pub fn validate(&self) -> Result<(), ValidationError> {
        // 1. 验证查询语句存在
        if self.sentence.is_none() {
            return Err(ValidationError::MissingSentence);
        }
        
        // 2. 验证命名空间
        if self.space.space_name.is_empty() {
            return Err(ValidationError::InvalidSpaceName);
        }
        
        // 3. 验证变量作用域一致性
        self.validate_variable_scope()?;
        
        // 4. 验证查询类型与语句匹配
        self.validate_query_type()?;
        
        Ok(())
    }
}
```

## 5. 集成设计

### 5.1 与查询上下文集成

```rust
// 在 QueryContext 中集成 AST 上下文
pub struct QueryContext {
    // ... 其他字段
    ast_context: Option<Arc<AstContext>>,
}

impl QueryContext {
    pub fn set_ast_context(&mut self, ast_ctx: AstContext) {
        self.ast_context = Some(Arc::new(ast_ctx));
    }
    
    pub fn get_ast_context(&self) -> Option<Arc<AstContext>> {
        self.ast_context.clone()
    }
}
```

### 5.2 与验证器集成

```rust
// 验证器使用 AST 上下文进行语义验证
pub struct Validator {
    ast_context: Arc<AstContext>,
}

impl Validator {
    pub fn validate_statement(&self, statement: &dyn Statement) -> Result<(), ValidationError> {
        // 使用 AST 上下文进行验证
        self.ast_context.validate()?;
        
        // 语句特定验证逻辑
        self.validate_specific_statement(statement)
    }
}
```

## 6. 性能优化

### 6.1 内存优化
- 使用 `Arc` 共享不可变数据
- 延迟加载非必要字段
- 使用字符串池减少内存分配

### 6.2 访问优化
- 缓存常用查询结果
- 使用快速哈希表实现变量查找
- 优化作用域链遍历算法

## 7. 测试策略

### 7.1 单元测试
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ast_context_creation() {
        let ctx = AstContext::from_query("USE test_space; MATCH (n) RETURN n").unwrap();
        assert_eq!(ctx.get_space_info().space_name, "test_space");
    }
    
    #[test]
    fn test_variable_scope_management() {
        let mut scope = VariableScope::new();
        scope.add_variable("x".to_string(), VariableInfo::new()).unwrap();
        assert!(scope.lookup("x").is_some());
    }
}
```

### 7.2 集成测试
- 与解析器集成测试
- 与验证器集成测试
- 端到端查询执行测试

## 8. 未来扩展

### 8.1 计划功能
- 支持查询重写上下文
- 添加性能分析信息
- 支持分布式查询上下文

### 8.2 优化方向
- 上下文序列化支持
- 增量更新机制
- 多版本上下文管理

## 9. 总结

AST 上下文模块为 GraphDB 提供了 NebulaGraph 级别的 AST 管理能力，通过现代化的 Rust 设计实现了更好的内存安全性和并发性能。该设计遵循 NebulaGraph 的架构理念，同时充分利用 Rust 语言特性进行优化。