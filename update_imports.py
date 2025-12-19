#!/usr/bin/env python3
"""
批量更新导入语句的脚本
用于修复因模块重构导致的导入错误
"""

import os
import re
from pathlib import Path

def update_file(filepath, pattern, replacement):
    """更新单个文件中的内容"""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
    except UnicodeDecodeError:
        # 如果UTF-8失败，尝试其他编码
        with open(filepath, 'r', encoding='gbk') as f:
            content = f.read()

    # 使用正则表达式进行替换
    updated_content = re.sub(pattern, replacement, content)

    if content != updated_content:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(updated_content)
        print(f"更新了文件: {filepath}")
        return True
    return False

def main():
    # 项目根目录
    root_dir = Path(".")

    # 要搜索的目录
    search_dirs = [
        root_dir / "src" / "query",
        root_dir / "src" / "executor",
        root_dir / "src" / "planner",
        root_dir / "src" / "validator",
        root_dir / "src" / "context",
    ]

    # 需要修复的模式
    fixes = [
        # 修复base模块导入
        (r'crate::query::context::ast_context::base::AstContext', 'crate::query::context::ast_context::AstContext'),
        (r'crate::query::context::ast::base::AstContext', 'crate::query::context::ast::AstContext'),
        
        # 修复data_processing到result_processing的导入
        (r'use crate::query::executor::data_processing::filter::FilterExecutor', 'use crate::query::executor::result_processing::filter::FilterExecutor'),
        (r'use crate::query::executor::data_processing::pagination::LimitExecutor', 'use crate::query::executor::result_processing::limit::LimitExecutor'),
        (r'use crate::query::executor::data_processing::sort::(SortExecutor, SortKey, SortOrder)', 'use crate::query::executor::result_processing::sort::{SortExecutor, SortKey, SortOrder}'),
        (r'use crate::query::executor::data_processing::sort::(SortExecutor, SortKey, SortOrder)', 'use crate::query::executor::result_processing::sort::{SortExecutor, SortKey, SortOrder}'),
        (r'use crate::query::executor::data_processing::sort::SortExecutor', 'use crate::query::executor::result_processing::sort::SortExecutor'),
        (r'use crate::query::executor::data_processing::aggregation::AggregateExecutor', 'use crate::query::executor::result_processing::aggregation::AggregateExecutor'),
        
        # 修复validator::common_structs
        (r'use crate::query::validator::common_structs::', 'use crate::query::validator::'),
        
        # 修复validator::clause_structs
        (r'crate::query::validator::clause_structs', 'crate::query::validator'),
        
        # 修复validator::ValidateContext
        (r'crate::query::validator::ValidateContext', 'crate::query::context::validate::ValidateContext'),
        
        # 修复context::RequestContext
        (r'crate::query::context::RequestContext', 'crate::query::context::query_context::QueryContext as RequestContext'),
        
        # 修复Context类型
        (r'use crate::query::context::ast_context::CypherAstContext', 'use crate::query::context::ast_context::AstContext'),
        (r'CypherAstContext', 'AstContext'),
        
        # 修复其他Context类型
        (r'use crate::query::context::ast_context::FetchEdgesContext', 'use crate::query::context::ast_context::AstContext'),
        (r'use crate::query::context::ast_context::FetchVerticesContext', 'use crate::query::context::ast_context::AstContext'),
        (r'use crate::query::context::ast_context::GoContext', 'use crate::query::context::ast_context::AstContext'),
        (r'use crate::query::context::ast_context::LookupContext', 'use crate::query::context::ast_context::AstContext'),
        (r'use crate::query::context::ast_context::MaintainContext', 'use crate::query::context::ast_context::AstContext'),
        (r'use crate::query::context::ast_context::PathContext', 'use crate::query::context::ast_context::AstContext'),
        (r'use crate::query::context::ast_context::SubgraphContext', 'use crate::query::context::ast_context::AstContext'),
    ]

    # 遍历所有Rust文件
    for search_dir in search_dirs:
        if search_dir.exists():
            for rust_file in search_dir.rglob("*.rs"):
                print(f"正在处理: {rust_file}")
                for pattern, replacement in fixes:
                    update_file(rust_file, pattern, replacement)

    print("更新完成！")

if __name__ == "__main__":
    main()