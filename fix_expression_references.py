"""
批量修复 Expression -> Expr 类型引用

此脚本批量修复src目录中的Expression类型引用问题。
主要替换内容：
1. use crate::expression::Expression -> use crate::expression::Expr
2. &Expression -> &Expr
3. Expression:: -> Expr::

注意：此脚本仅进行安全的文本替换，不修改语句结构。
"""

import os
import re
from pathlib import Path

# 需要修复的文件列表（来自cargo_errors_report.md）
FILES_TO_FIX = [
    # query/visitor 目录
    "src/query/visitor/variable_visitor.rs",
    "src/query/visitor/find_visitor.rs",
    "src/query/visitor/extract_filter_expr_visitor.rs",
    "src/query/visitor/evaluable_expr_visitor.rs",
    "src/query/visitor/deduce_props_visitor.rs",
    "src/query/visitor/deduce_type_visitor.rs",
    "src/query/visitor/extract_prop_expr_visitor.rs",
    "src/query/visitor/extract_group_suite_visitor.rs",
    "src/query/visitor/rewrite_visitor.rs",
    "src/query/visitor/fold_constant_expr_visitor.rs",
    "src/query/visitor/deduce_alias_type_visitor.rs",
    "src/query/visitor/validate_pattern_expression_visitor.rs",
    "src/query/visitor/vid_extract_visitor.rs",
    "src/query/visitor/property_tracker_visitor.rs",

    # query/optimizer 目录
    "src/query/optimizer/prune_properties_visitor.rs",

    # expression 目录
    "src/expression/visitor.rs",
    "src/expression/evaluator/expression_evaluator.rs",
]

def fix_expression_references(content: str) -> str:
    """修复文件中的Expression引用"""

    # 1. 替换 import 语句
    # use crate::expression::Expression -> use crate::expression::Expr
    content = re.sub(
        r'use crate::expression::Expression\b',
        'use crate::expression::Expr',
        content
    )

    # use crate::core::types::expression::Expression -> use crate::core::types::expression::Expr
    content = re.sub(
        r'use crate::core::types::expression::Expression\b',
        'use crate::core::types::expression::Expr',
        content
    )

    # 2. 替换函数参数类型
    # &Expression -> &Expr
    content = re.sub(
        r'&Expression\b',
        '&Expr',
        content
    )

    # 3. 替换 match 分支和构造表达式
    # Expression::Literal -> Expr::Literal
    content = re.sub(
        r'Expression::Literal\b',
        'Expr::Literal',
        content
    )
    content = re.sub(
        r'Expression::Variable\b',
        'Expr::Variable',
        content
    )
    content = re.sub(
        r'Expression::Property\b',
        'Expr::Property',
        content
    )
    content = re.sub(
        r'Expression::Binary\b',
        'Expr::Binary',
        content
    )
    content = re.sub(
        r'Expression::Unary\b',
        'Expr::Unary',
        content
    )
    content = re.sub(
        r'Expression::Function\b',
        'Expr::Function',
        content
    )
    content = re.sub(
        r'Expression::Aggregate\b',
        'Expr::Aggregate',
        content
    )
    content = re.sub(
        r'Expression::List\b',
        'Expr::List',
        content
    )
    content = re.sub(
        r'Expression::Map\b',
        'Expr::Map',
        content
    )
    content = re.sub(
        r'Expression::Case\b',
        'Expr::Case',
        content
    )
    content = re.sub(
        r'Expression::TypeCast\b',
        'Expr::TypeCast',
        content
    )
    content = re.sub(
        r'Expression::Subscript\b',
        'Expr::Subscript',
        content
    )
    content = re.sub(
        r'Expression::Range\b',
        'Expr::Range',
        content
    )
    content = re.sub(
        r'Expression::Path\b',
        'Expr::Path',
        content
    )
    content = re.sub(
        r'Expression::Label\b',
        'Expr::Label',
        content
    )

    # 4. 替换 Vec<Expression> -> Vec<Expr>
    content = re.sub(
        r'Vec<Expression>',
        'Vec<Expr>',
        content
    )

    # 5. 替换 Box<Expression> -> Box<Expr>
    content = re.sub(
        r'Box<Expression>',
        'Box<Expr>',
        content
    )

    # 6. 替换 Option<Box<Expression>> -> Option<Box<Expr>>
    content = re.sub(
        r'Option<Box<Expression>>',
        'Option<Box<Expr>>',
        content
    )

    # 7. 替换 (Expression, Expression) -> (Expr, Expr)
    content = re.sub(
        r'\(Expression,\s*Expression\)',
        '(Expr, Expr)',
        content
    )

    # 8. 替换 &[(Expression, Expression)] -> &[(Expr, Expr)]
    content = re.sub(
        r'&\[\(Expression,\s*Expression\)\]',
        '&[(Expr, Expr)]',
        content
    )

    # 9. 替换 &[(String, Expression)] -> &[(String, Expr)]
    content = re.sub(
        r'&\[String,\s*Expression\]',
        '&[(String, Expr)]',
        content
    )

    return content

def process_file(file_path: str) -> bool:
    """处理单个文件"""
    if not os.path.exists(file_path):
        print(f"  [跳过] 文件不存在: {file_path}")
        return False

    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            original_content = f.read()

        new_content = fix_expression_references(original_content)

        if original_content != new_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(new_content)
            print(f"  [已修复] {file_path}")
            return True
        else:
            print(f"  [无需修改] {file_path}")
            return False
    except Exception as e:
        print(f"  [错误] {file_path}: {e}")
        return False

def main():
    base_path = os.path.dirname(os.path.abspath(__file__))
    print("=" * 60)
    print("批量修复 Expression -> Expr 引用")
    print("=" * 60)

    fixed_count = 0
    skipped_count = 0

    for relative_path in FILES_TO_FIX:
        file_path = os.path.join(base_path, relative_path)

        # 确保路径分隔符正确
        file_path = file_path.replace('/', os.sep).replace('\\', os.sep)

        if process_file(file_path):
            fixed_count += 1

    print("=" * 60)
    print(f"完成！共修复 {fixed_count} 个文件")
    print("=" * 60)

if __name__ == "__main__":
    main()
