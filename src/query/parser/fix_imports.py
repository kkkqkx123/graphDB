#!/usr/bin/env python3
"""
批量修复查询解析器中的模块引用错误
"""

import os
import re
import glob

def fix_imports_in_file(filepath):
    """修复单个文件中的导入错误"""
    print(f"处理文件: {filepath}")
    
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original_content = content
    
    # 修复旧的 AST 模块引用
    replacements = [
        # 修复 statement 模块引用
        (r'crate::query::parser::ast::statement', 'crate::query::parser::ast::stmt'),
        (r'crate::query::parser::ast::span', 'crate::query::parser::ast::types'),
        
        # 修复 Expression trait 引用
        (r'Box<dyn Expression>', 'Expr'),
        (r'&Box<dyn Expression>', '&Expr'),
        (r'&dyn Expression', '&Expr'),
        (r'dyn Expression', 'Expr'),
        
        # 修复 Statement trait 引用
        (r'Box<dyn Statement>', 'Stmt'),
        (r'&Box<dyn Statement>', '&Stmt'),
        (r'&dyn Statement', '&Stmt'),
        (r'dyn Statement', 'Stmt'),
        
        # 修复 Pattern trait 引用
        (r'Box<dyn Pattern>', 'Pattern'),
        (r'&Box<dyn Pattern>', '&Pattern'),
        (r'&dyn Pattern', '&Pattern'),
        (r'dyn Pattern', 'Pattern'),
        
        # 修复具体的类型引用
        (r'crate::query::parser::ast::Expression', 'crate::query::parser::ast::Expr'),
        (r'crate::query::parser::ast::Statement', 'crate::query::parser::ast::Stmt'),
        (r'crate::query::parser::ast::Pattern', 'crate::query::parser::ast::Pattern'),
        
        # 修复旧的表达式类型
        (r'ExpressionType::', 'Expr::'),
        (r'StatementType::', 'Stmt::'),
        (r'PatternType::', 'Pattern::'),
        
        # 修复 BaseStatement 引用
        (r'BaseStatement::new', 'BaseStmt::new'),
        (r'BaseStatement', 'BaseStmt'),
        
        # 修复 MatchPath 等类型
        (r'MatchPath', 'Pattern'),
        (r'MatchPathSegment', 'PathElement'),
        (r'MatchClause', 'MatchClause'),
        (r'MatchClauseDetail', 'MatchClause'),
        
        # 修复语句类型
        (r'MatchStatement', 'MatchStmt'),
        (r'CreateStatement', 'CreateStmt'),
        (r'DeleteStatement', 'DeleteStmt'),
        (r'UpdateStatement', 'UpdateStmt'),
        (r'UseStatement', 'UseStmt'),
        (r'ShowStatement', 'ShowStmt'),
        (r'ExplainStatement', 'ExplainStmt'),
        (r'LookupStatement', 'LookupStmt'),
        (r'SubgraphStatement', 'SubgraphStmt'),
        (r'FindPathStatement', 'FindPathStmt'),
        
        # 修复表达式类型
        (r'ConstantExpr::new', 'ConstantExpr::new'),
        (r'VariableExpr::new', 'VariableExpr::new'),
        (r'BinaryExpr::new', 'BinaryExpr::new'),
        (r'UnaryExpr::new', 'UnaryExpr::new'),
        (r'FunctionCallExpr::new', 'FunctionCallExpr::new'),
        (r'PropertyAccessExpr::new', 'PropertyAccessExpr::new'),
        (r'ListExpr::new', 'ListExpr::new'),
        (r'MapExpr::new', 'MapExpr::new'),
        (r'CaseExpr::new', 'CaseExpr::new'),
        (r'SubscriptExpr::new', 'SubscriptExpr::new'),
        (r'PredicateExpr::new', 'PredicateExpr::new'),
        
        # 修复模式类型
        (r'NodePattern::new', 'NodePattern::new'),
        (r'EdgePattern::new', 'EdgePattern::new'),
        (r'PathPattern::new', 'PathPattern::new'),
        (r'VariablePattern::new', 'VariablePattern::new'),
        
        # 修复子句类型
        (r'ReturnClause', 'ReturnClause'),
        (r'WhereClause', 'WhereClause'),
        (r'SetClause', 'SetClause'),
        (r'FromClause', 'FromClause'),
        (r'OverClause', 'OverClause'),
        (r'YieldClause', 'YieldClause'),
        (r'OrderByClause', 'OrderByClause'),
        
        # 修复目标类型
        (r'CreateTarget', 'CreateTarget'),
        (r'DeleteTarget', 'DeleteTarget'),
        (r'UpdateTarget', 'UpdateTarget'),
        (r'FetchTarget', 'FetchTarget'),
        (r'ShowTarget', 'ShowTarget'),
        (r'LookupTarget', 'LookupTarget'),
        
        # 修复其他类型
        (r'Assignment', 'Assignment'),
        (r'Property', 'Property'),
        (r'ReturnItem', 'ReturnItem'),
        (r'OrderByItem', 'OrderByItem'),
        (r'Steps', 'Steps'),
        (r'EdgeDirection', 'EdgeDirection'),
        (r'EdgeRange', 'EdgeRange'),
        (r'PathElement', 'PathElement'),
        (r'PredicateType', 'PredicateType'),
        (r'BinaryOp', 'BinaryOp'),
        (r'UnaryOp', 'UnaryOp'),
        
        # 修复导入语句
        (r'use crate::query::parser::ast::expression::', 'use crate::query::parser::ast::expr::'),
        (r'use crate::query::parser::ast::statement::', 'use crate::query::parser::ast::stmt::'),
        (r'use crate::query::parser::ast::pattern::', 'use crate::query::parser::ast::pattern::'),
        (r'use crate::query::parser::ast::span::', 'use crate::query::parser::ast::types::'),
    ]
    
    # 应用替换
    for pattern, replacement in replacements:
        content = re.sub(pattern, replacement, content)
    
    # 修复特定的导入模式
    # 修复 use 语句中的模块路径
    content = re.sub(
        r'use crate::query::parser::ast::\{([^}]+)\}',
        lambda m: f"use crate::query::parser::ast::{{{m.group(1)}}}",
        content
    )
    
    # 如果内容有变化，写回文件
    if content != original_content:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"  ✓ 已修复: {filepath}")
        return True
    else:
        print(f"  - 无需修复: {filepath}")
        return False

def main():
    """主函数"""
    # 获取所有需要修复的 Rust 文件
    patterns = [
        '../../query/parser/**/*.rs',
        '../../query/visitor/**/*.rs',
        '../../query/expressions/**/*.rs',
        '../../query/statements/**/*.rs',
        './*.rs',
        './ast/*.rs',
        './parser/*.rs',
        './expressions/*.rs',
        './statements/*.rs',
    ]
    
    files_to_fix = []
    for pattern in patterns:
        files_to_fix.extend(glob.glob(pattern, recursive=True))
    
    # 排除某些文件
    exclude_patterns = [
        '**/target/**',
        '**/tests/**',
        '**/examples/**',
        '**/mod.rs',  # 模块文件通常不需要修复
    ]
    
    filtered_files = []
    for file in files_to_fix:
        should_exclude = False
        for exclude_pattern in exclude_patterns:
            if exclude_pattern.replace('**', '') in file:
                should_exclude = True
                break
        if not should_exclude:
            filtered_files.append(file)
    
    print(f"找到 {len(filtered_files)} 个文件需要修复")
    
    # 修复文件
    fixed_count = 0
    for filepath in filtered_files:
        if fix_imports_in_file(filepath):
            fixed_count += 1
    
    print(f"修复完成！共修复了 {fixed_count} 个文件")
    
    # 生成报告
    print("\n建议下一步：")
    print("1. 运行 cargo check 检查是否还有错误")
    print("2. 运行 cargo test 确保功能正常")
    print("3. 手动检查关键文件是否正确")
    print("4. 考虑清理旧的 ast_old 和 parser_old 目录")

if __name__ == '__main__':
    main()