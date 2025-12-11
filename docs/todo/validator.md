src\query\validator\aggregate_validator.rs:72-84
```
   /// 验证聚合表达式的合法性
    pub fn validate_aggregate_expr(&self, expr: &Expression) -> Result<(), String> {
        // 这里可以添加更详细的聚合函数验证逻辑
        // 例如检查聚合函数的参数类型、嵌套使用等

        if self.has_aggregate_expr(expr) {
            // 检查聚合函数的使用是否合法
            // 在实际实现中，这里会进行更详细的验证
        }

        Ok(())
    }
}
```

