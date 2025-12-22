#!/usr/bin/env python3
"""
批量更新导入路径脚本
将使用 expression:: 的导入语句更新为使用 core::
"""

import os
import re
import sys

def update_imports_in_file(filepath):
    """更新单个文件中的导入路径"""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # 定义需要更新的导入模式
        import_patterns = [
            (r'use\s+crate::expression::Expression;', 'use crate::core::Expression;'),
            (r'use\s+crate::expression::\{([^}]*?)Expression([^}]*?)\};', r'use crate::core::{\1Expression\2};'),
            (r'use\s+crate::expression::\{([^}]*?)BinaryOperator([^}]*?)\};', r'use crate::core::{\1BinaryOperator\2};'),
            (r'use\s+crate::expression::\{([^}]*?)UnaryOperator([^}]*?)\};', r'use crate::core::{\1UnaryOperator\2};'),
            (r'use\s+crate::expression::\{([^}]*?)LiteralValue([^}]*?)\};', r'use crate::core::{\1LiteralValue\2};'),
            (r'use\s+crate::expression::\{([^}]*?)AggregateFunction([^}]*?)\};', r'use crate::core::{\1AggregateFunction\2};'),
            (r'use\s+crate::expression::\{([^}]*?)DataType([^}]*?)\};', r'use crate::core::{\1DataType\2};'),
            (r'crate::expression::Expression', 'crate::core::Expression'),
            (r'crate::expression::BinaryOperator', 'crate::core::BinaryOperator'),
            (r'crate::expression::UnaryOperator', 'crate::core::UnaryOperator'),
            (r'crate::expression::LiteralValue', 'crate::core::LiteralValue'),
            (r'crate::expression::AggregateFunction', 'crate::core::AggregateFunction'),
            (r'crate::expression::DataType', 'crate::core::DataType'),
        ]
        
        # 应用所有模式替换
        for pattern, replacement in import_patterns:
            content = re.sub(pattern, replacement, content)
        
        # 如果内容有变化，写回文件
        if content != original_content:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(content)
            return True
        return False
        
    except Exception as e:
        print(f"处理文件 {filepath} 时出错: {e}")
        return False

def find_and_update_files():
    """查找并更新所有需要更新的文件"""
    updated_files = []
    
    # 遍历src目录下的所有.rs文件
    for root, dirs, files in os.walk('src'):
        # 跳过expression目录本身，因为我们只更新其他模块的引用
        if 'expression' in root and 'src/expression' in root:
            continue
            
        for file in files:
            if file.endswith('.rs'):
                filepath = os.path.join(root, file)
                if update_imports_in_file(filepath):
                    updated_files.append(filepath)
    
    return updated_files

if __name__ == '__main__':
    print("开始批量更新导入路径...")
    updated_files = find_and_update_files()
    
    if updated_files:
        print(f"更新了 {len(updated_files)} 个文件:")
        for f in updated_files:
            print(f"  - {f}")
    else:
        print("没有找到需要更新的文件")
    
    print("导入路径更新完成！")