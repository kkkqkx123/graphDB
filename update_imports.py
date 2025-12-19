#!/usr/bin/env python3
"""
更新所有引用的导入路径，从 crate::core 移到 crate::query::context
"""

import os
import re
from pathlib import Path

def update_imports(file_path):
    """更新单个文件的导入"""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original_content = content
    
    # 替换导入语句
    replacements = [
        (r'use crate::core::ExecutionContext;', 'use crate::query::context::ExecutionContext;'),
        (r'use crate::core::QueryContext;', 'use crate::query::context::QueryContext;'),
        (r'use crate::core::ValidateContext;', 'use crate::query::context::ValidateContext;'),
        (r'use crate::core::AstContext;', 'use crate::query::context::AstContext;'),
    ]
    
    for old, new in replacements:
        content = re.sub(old, new, content)
    
    # 如果有更改，写回文件
    if content != original_content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        return True
    return False

def main():
    root_dir = Path('d:/项目/database/graphDB/src')
    
    if not root_dir.exists():
        print(f"目录不存在: {root_dir}")
        return
    
    # 需要处理的目录
    target_dirs = [
        root_dir / 'query' / 'planner',
        root_dir / 'query' / 'optimizer',
        root_dir / 'query' / 'executor',
        root_dir / 'query' / 'validator',
        root_dir / 'query' / 'visitor',
        root_dir / 'api',
        root_dir / 'services',
    ]
    
    updated_files = []
    
    for target_dir in target_dirs:
        if not target_dir.exists():
            print(f"跳过不存在的目录: {target_dir}")
            continue
        
        print(f"处理目录: {target_dir}")
        for rs_file in target_dir.rglob('*.rs'):
            if update_imports(rs_file):
                updated_files.append(str(rs_file))
                print(f"  ✓ 已更新: {rs_file.relative_to(root_dir.parent)}")
    
    print(f"\n总共更新 {len(updated_files)} 个文件")
    
    if updated_files:
        print("\n更新的文件列表:")
        for f in updated_files:
            print(f"  {f}")

if __name__ == '__main__':
    main()
