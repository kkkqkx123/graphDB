# 阶段三：DDL 导出功能实施方案

## 1. 概述

本阶段实现 DDL（Data Definition Language）导出功能，允许用户将整个 Space 的 Schema 定义导出为可执行的 nGQL 脚本，便于备份、迁移和版本控制。

## 2. 功能需求

### 2.1 功能清单

| 功能点 | 描述 | 优先级 |
|--------|------|--------|
| 完整 DDL 导出 | 导出 Space、Tag、Edge、Index 的创建语句 | 高 |
| GQL 预览 | 在模态框中展示生成的 DDL | 高 |
| 复制到剪贴板 | 一键复制 DDL 内容 | 中 |
| 下载为文件 | 下载为 .ngql 文件 | 中 |
| 语法高亮 | 使用 Monaco Editor 进行语法高亮 | 低 |

### 2.2 界面设计

```
+----------------------------------------------------------+
|  DDL Export: production_space              [Copy] [Download] [Close] |
+----------------------------------------------------------+
|                                                          |
|  +----------------------------------------------------+  |
|  | # Create Space                                     |  |
|  | CREATE SPACE production_space (                    |  |
|  |   partition_num = 100,                             |  |
|  |   replica_factor = 1,                              |  |
|  |   vid_type = FIXED_STRING(32)                      |  |
|  | );                                                 |  |
|  | :sleep 20;                                         |  |
|  | USE production_space;                              |  |
|  |                                                    |  |
|  | # Create Tags                                      |  |
|  | CREATE TAG Person (                                |  |
|  |   name STRING,                                     |  |
|  |   age INT64,                                       |  |
|  |   email STRING                                     |  |
|  | ) ttl_duration = 0, ttl_col = "";                  |  |
|  |                                                    |  |
|  | CREATE TAG Company (                               |  |
|  |   name STRING,                                     |  |
|  |   founded_date DATE                                |  |
|  | );                                                 |  |
|  |                                                    |  |
|  | # Create Edges                                     |  |
|  | CREATE EDGE WORKS_AT (                             |  |
|  |   start_date DATE,                                 |  |
|  |   end_date DATE                                    |  |
|  | );                                                 |  |
|  |                                                    |  |
|  | # Create Indexes                                   |  |
|  | CREATE TAG INDEX idx_person_name ON Person(name);  |  |
|  | :sleep 20;                                         |  |
|  +----------------------------------------------------+  |
|                                                          |
|  Generated: 2024-01-15 10:30:25                          |
+----------------------------------------------------------+
```

## 3. 技术方案

### 3.1 目录结构

```
frontend/src/pages/Schema/
├── SpaceList/
│   ├── components/
│   │   └── DDLExportModal/      # 新增：DDL 导出模态框
│   │       ├── index.module.less
│   │       └── index.tsx
│   ├── index.module.less
│   └── index.tsx
```

### 3.2 组件设计

#### 3.2.1 DDLExportModal 组件

```typescript
// frontend/src/pages/Schema/SpaceList/components/DDLExportModal/index.tsx

import React, { useState, useEffect, useCallback } from 'react';
import { Modal, Button, Spin, message, Tooltip } from 'antd';
import { CopyOutlined, DownloadOutlined } from '@ant-design/icons';
import { useSchemaStore } from '@/stores/schema';
import styles from './index.module.less';

interface DDLExportModalProps {
  visible: boolean;
  space: string;
  onCancel: () => void;
}

interface DDLData {
  space: string;
  tags: string[];
  edges: string[];
  indexes: string[];
}

const DDLExportModal: React.FC<DDLExportModalProps> = ({
  visible,
  space,
  onCancel,
}) => {
  const [loading, setLoading] = useState(false);
  const [ddl, setDDL] = useState('');
  const [generatedAt, setGeneratedAt] = useState<string>('');

  // 获取 DDL 数据
  const fetchDDL = useCallback(async () => {
    if (!visible || !space) return;
    
    setLoading(true);
    try {
      const data = await fetchSchemaDDL(space);
      const formattedDDL = formatDDL(data);
      setDDL(formattedDDL);
      setGeneratedAt(new Date().toLocaleString());
    } catch (err) {
      message.error('Failed to fetch DDL');
    } finally {
      setLoading(false);
    }
  }, [visible, space]);

  useEffect(() => {
    fetchDDL();
  }, [fetchDDL]);

  // 格式化 DDL
  const formatDDL = (data: DDLData): string => {
    const lines: string[] = [];
    
    // Space 创建
    lines.push('# Create Space');
    lines.push(data.space);
    lines.push(':sleep 20;');
    
    // 提取 Space 名称
    const spaceNameMatch = data.space.match(/CREATE SPACE (\w+)/);
    const spaceName = spaceNameMatch ? spaceNameMatch[1] : space;
    lines.push(`USE ${escapeIdentifier(spaceName)};`);
    lines.push('');
    
    // Tags
    if (data.tags.length > 0) {
      lines.push('# Create Tags');
      data.tags.forEach(tag => {
        lines.push(tag);
        lines.push('');
      });
    }
    
    // Edges
    if (data.edges.length > 0) {
      lines.push('# Create Edges');
      data.edges.forEach(edge => {
        lines.push(edge);
        lines.push('');
      });
    }
    
    // Indexes
    if (data.indexes.length > 0) {
      lines.push('# Create Indexes');
      lines.push(':sleep 20;');
      data.indexes.forEach(index => {
        lines.push(index);
        lines.push('');
      });
    }
    
    return lines.join('\n');
  };

  // 转义标识符
  const escapeIdentifier = (name: string): string => {
    // 如果名称包含特殊字符，使用反引号包裹
    if (/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(name)) {
      return name;
    }
    return `\`${name}\``;
  };

  // 复制到剪贴板
  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(ddl);
      message.success('DDL copied to clipboard');
    } catch (err) {
      message.error('Failed to copy');
    }
  };

  // 下载为文件
  const handleDownload = () => {
    const blob = new Blob([ddl], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `${space}_ddl.ngql`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
    message.success('DDL downloaded');
  };

  return (
    <Modal
      title={`DDL Export: ${space}`}
      open={visible}
      width={800}
      onCancel={onCancel}
      footer={[
        <Button key="copy" icon={<CopyOutlined />} onClick={handleCopy}>
          Copy
        </Button>,
        <Button
          key="download"
          type="primary"
          icon={<DownloadOutlined />}
          onClick={handleDownload}
          disabled={!ddl}
        >
          Download
        </Button>,
        <Button key="close" onClick={onCancel}>
          Close
        </Button>,
      ]}
    >
      <Spin spinning={loading} tip="Generating DDL...">
        <div className={styles.ddlContainer}>
          {ddl ? (
            <>
              <pre className={styles.ddlContent}>{ddl}</pre>
              <div className={styles.generatedTime}>
                Generated: {generatedAt}
              </div>
            </>
          ) : (
            <div className={styles.emptyState}>
              {!loading && 'No DDL available'}
            </div>
          )}
        </div>
      </Spin>
    </Modal>
  );
};

export default DDLExportModal;
```

### 3.3 API 服务

```typescript
// frontend/src/services/schema.ts

// 获取 Schema DDL
export const fetchSchemaDDL = async (space: string): Promise<DDLData> => {
  // 并行获取所有 DDL
  const [spaceDDL, tagsDDL, edgesDDL, indexesDDL] = await Promise.all([
    fetchSpaceDDL(space),
    fetchTagsDDL(space),
    fetchEdgesDDL(space),
    fetchIndexesDDL(space),
  ]);

  return {
    space: spaceDDL,
    tags: tagsDDL,
    edges: edgesDDL,
    indexes: indexesDDL,
  };
};

// 获取 Space DDL
const fetchSpaceDDL = async (space: string): Promise<string> => {
  const query = `SHOW CREATE SPACE ${space}`;
  const response = await api.post('/query', { query });
  // 解析返回结果
  return response.data.tables[0]['Create Space'];
};

// 获取所有 Tag DDL
const fetchTagsDDL = async (space: string): Promise<string[]> => {
  // 先获取所有 Tags
  const tagsQuery = `SHOW TAGS`;
  const tagsResponse = await api.post('/query', { space, query: tagsQuery });
  const tags = tagsResponse.data.tables.map((t: any) => t.Name);
  
  // 并行获取每个 Tag 的 DDL
  const ddlPromises = tags.map(async (tag: string) => {
    const query = `SHOW CREATE TAG ${tag}`;
    const response = await api.post('/query', { space, query });
    return response.data.tables[0]['Create Tag'];
  });
  
  return Promise.all(ddlPromises);
};

// 获取所有 Edge DDL
const fetchEdgesDDL = async (space: string): Promise<string[]> => {
  const edgesQuery = `SHOW EDGES`;
  const edgesResponse = await api.post('/query', { space, query: edgesQuery });
  const edges = edgesResponse.data.tables.map((e: any) => e.Name);
  
  const ddlPromises = edges.map(async (edge: string) => {
    const query = `SHOW CREATE EDGE ${edge}`;
    const response = await api.post('/query', { space, query });
    return response.data.tables[0]['Create Edge'];
  });
  
  return Promise.all(ddlPromises);
};

// 获取所有 Index DDL
const fetchIndexesDDL = async (space: string): Promise<string[]> => {
  const indexesQuery = `SHOW INDEXES`;
  const indexesResponse = await api.post('/query', { space, query: indexesQuery });
  const indexes = indexesResponse.data.tables.map((i: any) => ({
    name: i['Index Name'],
    type: i['By'] === 'Tag' ? 'TAG' : 'EDGE',
  }));
  
  const ddlPromises = indexes.map(async (index: any) => {
    const query = `SHOW CREATE ${index.type} INDEX ${index.name}`;
    const response = await api.post('/query', { space, query });
    return response.data.tables[0][`Create ${index.type} Index`];
  });
  
  return Promise.all(ddlPromises);
};
```

### 3.4 类型定义

```typescript
// frontend/src/types/schema.ts

export interface DDLData {
  space: string;
  tags: string[];
  edges: string[];
  indexes: string[];
}

export interface DDLExportOptions {
  includeSpace: boolean;
  includeTags: boolean;
  includeEdges: boolean;
  includeIndexes: boolean;
  addSleepCommands: boolean;
}
```

## 4. 样式文件

```less
// frontend/src/pages/Schema/SpaceList/components/DDLExportModal/index.module.less

.ddlContainer {
  min-height: 300px;
  max-height: 500px;
  overflow: auto;
}

.ddlContent {
  margin: 0;
  padding: 16px;
  background: #f6ffed;
  border: 1px solid #b7eb8f;
  border-radius: 4px;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 13px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-all;
  color: #1f1f1f;
}

.generatedTime {
  margin-top: 12px;
  text-align: right;
  color: #999;
  font-size: 12px;
}

.emptyState {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 200px;
  color: #999;
}
```

## 5. 集成到 SpaceList

```typescript
// frontend/src/pages/Schema/SpaceList/index.tsx

import React, { useState } from 'react';
import { Button, Tooltip } from 'antd';
import { FileTextOutlined } from '@ant-design/icons';
import DDLExportModal from './components/DDLExportModal';

const SpaceList: React.FC = () => {
  const [ddlModalVisible, setDDLModalVisible] = useState(false);
  const [selectedSpaceForDDL, setSelectedSpaceForDDL] = useState('');

  const handleShowDDL = (space: Space) => {
    setSelectedSpaceForDDL(space.name);
    setDDLModalVisible(true);
  };

  const columns = [
    // ... 其他列
    {
      title: 'Action',
      render: (_, record: Space) => (
        <Space>
          {/* ... 其他操作按钮 */}
          <Tooltip title="Export DDL">
            <Button
              type="text"
              icon={<FileTextOutlined />}
              onClick={() => handleShowDDL(record)}
            />
          </Tooltip>
        </Space>
      ),
    },
  ];

  return (
    <>
      {/* ... 列表渲染 */}
      
      <DDLExportModal
        visible={ddlModalVisible}
        space={selectedSpaceForDDL}
        onCancel={() => setDDLModalVisible(false)}
      />
    </>
  );
};
```

## 6. 实现步骤

### 步骤 1: 创建 DDLExportModal 组件 (1-2 天)

1. 创建组件目录和文件
2. 实现 DDL 获取逻辑
3. 实现格式化功能
4. 实现复制和下载功能

### 步骤 2: 添加 API 服务 (0.5 天)

1. 在 `services/schema.ts` 中添加 DDL 相关方法
2. 在 `types/schema.ts` 中添加类型定义

### 步骤 3: 集成到 SpaceList (0.5 天)

1. 添加导出按钮
2. 集成模态框

### 步骤 4: 测试与优化 (1 天)

1. 测试各种 Space 的导出
2. 测试复制和下载功能
3. UI 优化

## 7. 注意事项

1. **性能考虑**: 对于 Schema 较多的 Space，DDL 获取可能较慢，需要添加加载状态
2. **错误处理**: 某些对象可能因权限问题无法获取 DDL，需要优雅处理
3. **格式一致性**: 确保生成的 DDL 格式与 NebulaGraph 标准一致
4. **字符编码**: 下载文件时使用 UTF-8 编码，支持中文

## 8. 扩展功能（可选）

1. **选择性导出**: 允许用户选择导出哪些类型的对象
2. **语法高亮**: 集成 Monaco Editor 提供语法高亮
3. **DDL 对比**: 对比两个 Space 的 DDL 差异
4. **批量导出**: 支持导出多个 Space 的 DDL

## 9. 参考文档

- [总体分析文档](./01_schema_analysis.md)
- [阶段二：索引创建增强](./03_phase2_index_enhancement.md)
- [阶段四：Schema 可视化](./05_phase4_schema_visualization.md)
