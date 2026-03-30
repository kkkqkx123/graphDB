#!/usr/bin/env python3
"""
Translate remaining Chinese text in storage directory to English
"""

import re
from pathlib import Path

def translate_file(file_path: Path) -> bool:
    """Translate Chinese text in a single file"""
    try:
        content = file_path.read_text(encoding='utf-8')
        original_content = content
        
        # Error messages - mixed Chinese/English patterns
        translations = [
            # Mixed patterns in error messages
            (r'Failed to query 名称索引', 'Failed to query name index'),
            (r'Failed to query 空间', 'Failed to query space'),
            (r'Failed to iterate 空间', 'Failed to iterate space'),
            (r'Failed to update 空间', 'Failed to update space'),
            (r'Failed to query ID计数器', 'Failed to query ID counter'),
            (r'Failed to update ID计数器', 'Failed to update ID counter'),
            (r'Failed to insert 标签', 'Failed to insert tag'),
            (r'Failed to delete 标签', 'Failed to delete tag'),
            (r'Failed to iterate 标签', 'Failed to iterate tag'),
            (r'Failed to update 标签', 'Failed to update tag'),
            (r'Failed to query 边类型', 'Failed to query edge type'),
            (r'Failed to insert 边类型', 'Failed to insert edge type'),
            (r'Failed to delete 边类型', 'Failed to delete edge type'),
            (r'Failed to iterate 边类型', 'Failed to iterate edge type'),
            (r'Failed to update 边类型', 'Failed to update edge type'),
            (r'Failed to open 索引数据表', 'Failed to open INDEX_DATA_TABLE'),
            (r'Failed to insert 索引数据', 'Failed to insert index data'),
            (r'Failed to insert 反向索引', 'Failed to insert reverse index'),
            (r'Failed to insert 正向索引', 'Failed to insert forward index'),
            (r'Failed to delete 反向索引', 'Failed to delete reverse index'),
            (r'Failed to delete 正向索引', 'Failed to delete forward index'),
            (r'Failed to delete 索引', 'Failed to delete index'),
            (r'Failed to insert 边索引数据', 'Failed to insert edge index data'),
            (r'Failed to insert 边反向索引', 'Failed to insert edge reverse index'),
            (r'Failed to insert 顶点索引数据', 'Failed to insert vertex index data'),
            (r'Failed to insert 顶点反向索引', 'Failed to insert vertex reverse index'),
            (r'Failed to iterate 索引数据', 'Failed to iterate index data'),
            
            # Iteration errors
            (r'迭代空间失败', 'Failed to iterate space'),
            (r'迭代标签失败', 'Failed to iterate tag'),
            (r'迭代边类型失败', 'Failed to iterate edge type'),
            
            # Extended schema errors
            (r'反Serialization failed', 'Deserialization failed'),
            (r'Failed to get schema版本', 'Failed to get schema version'),
            (r'创建schema版本应该成功', 'Failed to create schema version'),
            (r'Failed to save schema快照', 'Failed to save schema snapshot'),
            (r'Failed to record schema变更', 'Failed to record schema change'),
            (r'Failed to get schema变更', 'Failed to get schema change'),
            (r'创建 Person 标签', 'Create Person tag'),
            
            # Comments
            (r'扫描的项目数', 'Number of items scanned'),
            (r'返回的项目数', 'Number of items returned'),
            (r'缓存命中次数', 'Number of cache hits'),
            (r'缓存未命中次数', 'Number of cache misses'),
            (r'各操作类型的计数', 'Count of each operation type'),
            (r'格式:', r'Format:'),
            (r'顶点ID', 'vertex ID'),
            (r'边对象', 'edge object'),
            (r'边列表', 'edge list'),
            (r'标签名称', 'tag name'),
            (r'用于索引重建操作', 'used for index rebuild operation'),
        ]
        
        for pattern, replacement in translations:
            content = re.sub(pattern, replacement, content)
        
        if content != original_content:
            file_path.write_text(content, encoding='utf-8')
            return True
        return False
        
    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False

def main():
    """Process all Rust files in storage directory"""
    storage_dir = Path('src/storage')
    
    rust_files = list(storage_dir.rglob('*.rs'))
    
    translated_count = 0
    for file_path in rust_files:
        if translate_file(file_path):
            translated_count += 1
            print(f"Translated: {file_path}")
    
    print(f"\nTotal files translated: {translated_count}")

if __name__ == '__main__':
    main()
