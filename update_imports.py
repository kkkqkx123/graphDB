#!/usr/bin/env python3
"""
批量更新 Rust 导入路径的脚本
将 src/core/context/expression 的导入路径更新为 src/core/expressions
"""

import os
import re
import sys
from pathlib import Path

def update_imports_in_file(file_path):
    """更新单个文件中的导入路径"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # 更新各种导入路径模式
        patterns = [
            # 基本导入路径
            (r'use crate::core::context::expression::', 'use crate::core::expressions::'),
            
            # 具体模块导入
            (r'use crate::core::context::expression::default_context::', 'use crate::core::expressions::default_context::'),
            (r'use crate::core::context::expression::basic_context::', 'use crate::core::expressions::basic_context::'),
            (r'use crate::core::context::expression::cache::', 'use crate::core::expressions::cache::'),
            (r'use crate::core::context::expression::functions::', 'use crate::core::expressions::functions::'),
            (r'use crate::core::context::expression::error::', 'use crate::core::expressions::error::'),
            (r'use crate::core::context::expression::evaluation::', 'use crate::core::expressions::evaluation::'),
            
            # 路径引用
            (r'crate::core::context::expression::', 'crate::core::expressions::'),
            
            # 类型路径
            (r'super::expression::', 'super::expressions::'),
            
            # 完整路径引用
            (r'crate::core::context::expression\{', 'crate::core::expressions\{'),
        ]
        
        # 应用所有模式
        for pattern, replacement in patterns:
            content = re.sub(pattern, replacement, content)
        
        # 如果内容有变化，写回文件
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"已更新: {file_path}")
            return True
        else:
            print(f"无需更新: {file_path}")
            return False
            
    except Exception as e:
        print(f"处理文件 {file_path} 时出错: {e}")
        return False

def find_rust_files(directory):
    """查找所有 Rust 源文件"""
    rust_files = []
    for root, dirs, files in os.walk(directory):
        # 跳过 target 目录
        if 'target' in dirs:
            dirs.remove('target')
        
        for file in files:
            if file.endswith('.rs'):
                rust_files.append(os.path.join(root, file))
    
    return rust_files

def main():
    """主函数"""
    print("开始批量更新导入路径...")
    
    # 查找所有 Rust 文件
    rust_files = find_rust_files('src')
    
    updated_files = 0
    total_files = len(rust_files)
    
    print(f"找到 {total_files} 个 Rust 文件")
    
    for file_path in rust_files:
        if update_imports_in_file(file_path):
            updated_files += 1
    
    print(f"\n更新完成!")
    print(f"总文件数: {total_files}")
    print(f"已更新文件数: {updated_files}")
    print(f"未更新文件数: {total_files - updated_files}")

if __name__ == '__main__':
    main()