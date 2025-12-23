#!/usr/bin/env python3
"""
批量修复剩余编译错误的脚本
主要处理导入缺失、类型不匹配等问题
"""

import os
import re
from pathlib import Path

def fix_file(file_path):
    """修复单个文件中的编译错误"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # 1. 添加缺失的导入
        # 添加 ExecutionPlan 导入
        if 'ExecutionPlan' in content and 'use crate::query::context::ExecutionPlan;' not in content:
            # 在现有导入后添加
            import_pattern = r'(use crate::[^;]+;)'
            if re.search(import_pattern, content):
                content = re.sub(
                    import_pattern,
                    r'\1\nuse crate::query::context::ExecutionPlan;',
                    content,
                    count=1
                )
            else:
                # 如果没有现有导入，在文件开头添加
                content = 'use crate::query::context::ExecutionPlan;\n' + content
        
        # 添加 PlanNodeEnum 导入
        if 'PlanNodeEnum' in content and 'use crate::query::planner::plan::core::nodes::PlanNodeEnum;' not in content:
            import_pattern = r'(use crate::[^;]+;)'
            if re.search(import_pattern, content):
                content = re.sub(
                    import_pattern,
                    r'\1\nuse crate::query::planner::plan::core::nodes::PlanNodeEnum;',
                    content,
                    count=1
                )
            else:
                content = 'use crate::query::planner::plan::core::nodes::PlanNodeEnum;\n' + content
        
        # 2. 修复方法调用
        # 将 .clone_plan_node() 替换为 .clone()
        content = re.sub(r'\.clone_plan_node\(\)', r'.clone()', content)
        
        # 3. 修复 Arc::new() 调用 - 包装为 PlanNodeEnum
        # 这是一个简化的处理，可能需要更复杂的逻辑
        content = re.sub(
            r'Arc::new\(([^)]+)\)',
            lambda m: f'PlanNodeEnum::{identify_node_type(m.group(1))}({m.group(1)})',
            content
        )
        
        # 4. 修复 trait object 引用
        content = re.sub(r'&dyn PlanNode', r'&PlanNodeEnum', content)
        content = re.sub(r'Box<dyn PlanNode>', r'PlanNodeEnum', content)
        
        # 5. 修复函数参数类型
        content = re.sub(r'fn\s+(\w+)\s*\([^)]*:\s*&dyn PlanNode[^)]*\)', r'fn \1(node: &PlanNodeEnum)', content)
        
        # 6. 修复返回类型
        content = re.sub(r'->\s*&dyn PlanNode', r'-> &PlanNodeEnum', content)
        
        # 7. 移除多余的 trait 导入
        content = re.sub(r'use.*PlanNode.*trait.*;', '', content)
        content = re.sub(r'use.*PlanNodeDependencies.*;', '', content)
        content = re.sub(r'use.*PlanNodeClonable.*;', '', content)
        
        # 8. 修复泛型约束
        content = re.sub(r'where.*T:\s*PlanNode', '', content)
        content = re.sub(r'where.*T:\s*PlanNodeClonable', '', content)
        
        # 9. 修复结构体字段类型
        content = re.sub(r'pub\s+(\w+):\s*Box<dyn PlanNode>,', r'pub \1: PlanNodeEnum,', content)
        
        # 10. 修复变量声明
        content = re.sub(r'let\s+(\w+):\s*Box<dyn PlanNode>\s*=', r'let \1: PlanNodeEnum =', content)
        
        # 如果内容有变化，写回文件
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"已修复: {file_path}")
            return True
        else:
            print(f"无需修复: {file_path}")
            return False
            
    except Exception as e:
        print(f"修复文件 {file_path} 时出错: {e}")
        return False

def identify_node_type(node_content):
    """识别节点类型的辅助函数"""
    # 这是一个简化的实现，实际可能需要更复杂的逻辑
    if 'StartNode' in node_content:
        return 'Start'
    elif 'ProjectNode' in node_content:
        return 'Project'
    elif 'FilterNode' in node_content:
        return 'Filter'
    elif 'SortNode' in node_content:
        return 'Sort'
    elif 'JoinNode' in node_content:
        return 'InnerJoin'
    else:
        return 'Placeholder'  # 默认使用占位符

def fix_imports_file():
    """修复导入文件，确保所有必要的类型都被导出"""
    mod_file = Path("src/query/planner/plan/core/nodes/mod.rs")
    
    if not mod_file.exists():
        return False
    
    try:
        with open(mod_file, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # 确保导出了 PlanNodeEnum
        if 'pub use plan_node_enum::PlanNodeEnum;' not in content:
            # 在其他 pub use 语句后添加
            import_pattern = r'(pub use [^;]+;)'
            if re.search(import_pattern, content):
                content = re.sub(
                    import_pattern,
                    r'\1\npub use plan_node_enum::PlanNodeEnum;',
                    content,
                    count=1
                )
            else:
                content += '\npub use plan_node_enum::PlanNodeEnum;'
        
        with open(mod_file, 'w', encoding='utf-8') as f:
            f.write(content)
        
        print(f"已修复导入文件: {mod_file}")
        return True
        
    except Exception as e:
        print(f"修复导入文件时出错: {e}")
        return False

def main():
    """主函数"""
    print("开始修复编译错误...")
    
    # 首先修复导入文件
    fix_imports_file()
    
    # 搜索 src 目录下的所有 Rust 文件
    src_dir = Path("src")
    rust_files = list(src_dir.rglob("*.rs"))
    
    fixed_count = 0
    total_count = 0
    
    for file_path in rust_files:
        total_count += 1
        if fix_file(file_path):
            fixed_count += 1
    
    print(f"\n修复完成！")
    print(f"总文件数: {total_count}")
    print(f"已修复文件数: {fixed_count}")
    
    # 创建一个总结文件，记录可能需要手动修复的地方
    with open("manual_fixes_needed.md", "w", encoding="utf-8") as f:
        f.write("# 需要手动修复的问题\n\n")
        f.write("以下问题可能需要手动修复：\n\n")
        f.write("1. Arc::new() 调用的自动类型识别可能不准确\n")
        f.write("2. 某些复杂的泛型约束可能需要手动调整\n")
        f.write("3. 测试文件中的错误可能需要特殊处理\n")
        f.write("4. 某些特定的 trait 实现可能需要手动更新\n")
        f.write("5. 宏定义中的类型引用可能需要手动修复\n\n")
        f.write("建议运行 `cargo check` 查看剩余的编译错误。")

if __name__ == "__main__":
    main()