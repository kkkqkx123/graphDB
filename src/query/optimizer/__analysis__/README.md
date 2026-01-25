拆分旧的优化器

依赖关系 ：
- core → 无依赖（基础类型）
- plan → 依赖 core
- engine → 依赖 core 和 plan
- rules → 依赖 plan 和 engine