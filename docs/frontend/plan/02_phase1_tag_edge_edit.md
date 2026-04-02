# 阶段一：Tag/Edge 编辑功能实施方案

## 1. 概述

本阶段实现 Tag 和 Edge 的编辑功能，包括属性编辑、TTL 配置和注释修改。这是 Schema 管理的核心功能，允许用户在创建后修改 Schema 定义。

## 2. 功能需求

### 2.1 Tag/Edge 编辑功能清单

| 功能点 | 描述 | 优先级 |
|--------|------|--------|
| 属性添加 | 向现有 Tag/Edge 添加新属性 | 高 |
| 属性删除 | 删除现有属性 | 高 |
| 属性修改 | 修改属性类型、默认值、注释 | 高 |
| TTL 配置 | 设置/修改 TTL 持续时间和列 | 中 |
| 注释编辑 | 修改 Tag/Edge 的注释 | 中 |
| GQL 预览 | 实时显示生成的 ALTER 语句 | 中 |

### 2.2 界面设计

```
+----------------------------------------------------------+
|  Edit Tag/Edge: <name>                          [Cancel] [Save]  |
+----------------------------------------------------------+
|                                                          |
|  Basic Info                                              |
|  +----------------------------------------------------+  |
|  | Name: [readonly: Person]                           |  |
|  | Comment: [____________________]                    |  |
|  +----------------------------------------------------+  |
|                                                          |
|  Properties                                              |
|  +----------------------------------------------------+  |
|  | Name        | Type      | Default | Nullable | Comment | 操作 |
|  |-------------|-----------|---------|----------|---------|------|
|  | name        | STRING    | -       | [x]      | -       | [Edit][Delete] |
|  | age         | INT64     | 0       | [ ]      | years   | [Edit][Delete] |
|  | email       | STRING    | -       | [x]      | -       | [Edit][Delete] |
|  +----------------------------------------------------+  |
|  [+ Add Property]                                        |
|                                                          |
|  TTL Configuration                                       |
|  +----------------------------------------------------+  |
|  | TTL Column: [age__________v]                       |  |
|  | Duration: [3600____] seconds                       |  |
|  +----------------------------------------------------+  |
|                                                          |
|  GQL Preview                                             |
|  +----------------------------------------------------+  |
|  | ALTER TAG Person                                     |  |
|  | ADD (address STRING)                                 |  |
|  | TTL_DURATION = 3600, TTL_COL = "age";                |  |
|  +----------------------------------------------------+  |
+----------------------------------------------------------+
```

## 3. 技术方案

### 3.1 目录结构

```
frontend/src/pages/Schema/
├── EdgeList/
├── IndexList/
├── SpaceList/
├── TagList/
├── components/
│   ├── SpaceCreateModal/
│   ├── SpaceDetailModal/
│   └── EditModal/              # 新增：通用编辑模态框
│       ├── PropertiesForm.tsx  # 属性表单组件
│       ├── TTLForm.tsx         # TTL 配置组件
│       ├── index.module.less
│       └── index.tsx
└── index.tsx
```

### 3.2 组件设计

#### 3.2.1 EditModal 组件

```typescript
// frontend/src/pages/Schema/components/EditModal/index.tsx

interface EditModalProps {
  visible: boolean;
  type: 'TAG' | 'EDGE';
  name: string;
  space: string;
  onCancel: () => void;
  onSuccess: () => void;
}

interface PropertyItem {
  name: string;
  type: string;
  default?: string;
  nullable: boolean;
  comment?: string;
  // 编辑状态标记
  status: 'existing' | 'added' | 'modified' | 'deleted';
  originalName?: string; // 用于重命名追踪
}

interface EditFormData {
  comment: string;
  properties: PropertyItem[];
  ttlCol?: string;
  ttlDuration?: number;
}
```

#### 3.2.2 核心方法

```typescript
// 获取 Tag/Edge 详情
const fetchSchemaDetail = async (space: string, type: 'TAG' | 'EDGE', name: string) => {
  // 调用 API: DESCRIBE TAG/EDGE <name>
  // 解析返回的属性列表、TTL 配置、注释
};

// 生成 ALTER GQL
const generateAlterGQL = (
  type: 'TAG' | 'EDGE',
  name: string,
  changes: EditFormData
): string => {
  const statements: string[] = [];
  
  // 处理属性变更
  changes.properties.forEach(prop => {
    switch (prop.status) {
      case 'added':
        statements.push(`ADD (${prop.name} ${prop.type}${prop.default ? ` DEFAULT ${prop.default}` : ''})`);
        break;
      case 'deleted':
        statements.push(`DROP ${prop.originalName || prop.name}`);
        break;
      case 'modified':
        statements.push(`CHANGE ${prop.originalName} ${prop.name} ${prop.type}`);
        break;
    }
  });
  
  // 处理 TTL
  if (changes.ttlCol && changes.ttlDuration) {
    statements.push(`TTL_DURATION = ${changes.ttlDuration}, TTL_COL = "${changes.ttlCol}"`);
  }
  
  // 处理注释
  if (changes.comment) {
    statements.push(`COMMENT = "${changes.comment}"`);
  }
  
  return `ALTER ${type} ${name} ${statements.join(', ')};`;
};
```

### 3.3 API 接口

#### 3.3.1 新增 API 方法

```typescript
// frontend/src/services/schema.ts

// 获取 Tag/Edge 详情
export const getSchemaDetail = async (
  space: string,
  type: 'TAG' | 'EDGE',
  name: string
): Promise<SchemaDetail> => {
  const query = type === 'TAG' 
    ? `DESCRIBE TAG ${name}` 
    : `DESCRIBE EDGE ${name}`;
  return api.post('/query', { space, query });
};

// 获取创建语句
export const getSchemaCreateGQL = async (
  space: string,
  type: 'TAG' | 'EDGE',
  name: string
): Promise<string> => {
  const query = type === 'TAG'
    ? `SHOW CREATE TAG ${name}`
    : `SHOW CREATE EDGE ${name}`;
  return api.post('/query', { space, query });
};

// 执行 ALTER 语句
export const alterSchema = async (
  space: string,
  gql: string
): Promise<ApiResponse> => {
  return api.post('/query', { space, query: gql });
};
```

#### 3.3.2 数据类型定义

```typescript
// frontend/src/types/schema.ts

export interface SchemaProperty {
  name: string;
  type: string;
  default?: string;
  nullable: boolean;
  comment?: string;
}

export interface SchemaDetail {
  name: string;
  comment?: string;
  properties: SchemaProperty[];
  ttl?: {
    col: string;
    duration: number;
  };
}

export interface SchemaChangeSet {
  added: SchemaProperty[];
  deleted: string[]; // property names
  modified: Array<{
    originalName: string;
    property: SchemaProperty;
  }>;
  ttl?: {
    col: string;
    duration: number;
  } | null; // null 表示删除 TTL
  comment?: string;
}
```

## 4. 实现步骤

### 步骤 1: 扩展 API 服务 (1 天)

1. 在 `services/schema.ts` 中添加：
   - `getSchemaDetail`
   - `getSchemaCreateGQL`
   - `alterSchema`

2. 在 `types/schema.ts` 中添加类型定义

### 步骤 2: 创建 EditModal 组件 (3-4 天)

1. 创建目录结构
2. 实现基础模态框框架
3. 实现属性列表展示
4. 实现属性添加/删除/编辑功能
5. 实现 TTL 配置表单
6. 实现 GQL 预览

### 步骤 3: 集成到列表页面 (1 天)

1. 在 `TagList/index.tsx` 中添加编辑按钮
2. 在 `EdgeList/index.tsx` 中添加编辑按钮
3. 实现编辑成功后的刷新逻辑

### 步骤 4: 测试与优化 (2-3 天)

1. 单元测试
2. 集成测试
3. UI 优化

## 5. 代码示例

### 5.1 EditModal 核心实现

```typescript
// frontend/src/pages/Schema/components/EditModal/index.tsx

import React, { useEffect, useState, useCallback } from 'react';
import { Modal, Form, Input, Button, Table, Space, message } from 'antd';
import { useSchemaStore } from '@/stores/schema';
import { getSchemaDetail, alterSchema } from '@/services/schema';
import PropertiesForm from './PropertiesForm';
import TTLForm from './TTLForm';
import styles from './index.module.less';

const EditModal: React.FC<EditModalProps> = ({
  visible,
  type,
  name,
  space,
  onCancel,
  onSuccess,
}) => {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);
  const [detail, setDetail] = useState<SchemaDetail | null>(null);
  const [gql, setGql] = useState('');

  // 加载详情
  useEffect(() => {
    if (visible && space && name) {
      loadDetail();
    }
  }, [visible, space, name]);

  const loadDetail = async () => {
    setLoading(true);
    try {
      const data = await getSchemaDetail(space, type, name);
      setDetail(data);
      form.setFieldsValue({
        comment: data.comment,
        properties: data.properties.map(p => ({ ...p, status: 'existing' })),
        ttlCol: data.ttl?.col,
        ttlDuration: data.ttl?.duration,
      });
    } catch (err) {
      message.error('Failed to load schema detail');
    } finally {
      setLoading(false);
    }
  };

  // 生成并更新 GQL 预览
  const updateGQLPreview = useCallback(() => {
    const values = form.getFieldsValue();
    const generated = generateAlterGQL(type, name, values);
    setGql(generated);
  }, [form, type, name]);

  // 提交修改
  const handleSubmit = async () => {
    try {
      await form.validateFields();
      const values = form.getFieldsValue();
      const alterGQL = generateAlterGQL(type, name, values);
      
      setLoading(true);
      await alterSchema(space, alterGQL);
      message.success(`${type} "${name}" updated successfully`);
      onSuccess();
    } catch (err) {
      message.error('Failed to update schema');
    } finally {
      setLoading(false);
    }
  };

  return (
    <Modal
      title={`Edit ${type}: ${name}`}
      open={visible}
      width={800}
      onCancel={onCancel}
      footer={[
        <Button key="cancel" onClick={onCancel}>
          Cancel
        </Button>,
        <Button key="submit" type="primary" loading={loading} onClick={handleSubmit}>
          Save
        </Button>,
      ]}
    >
      <Form form={form} layout="vertical" onValuesChange={updateGQLPreview}>
        <Form.Item name="comment" label="Comment">
          <Input placeholder="Enter comment" />
        </Form.Item>

        <PropertiesForm form={form} onChange={updateGQLPreview} />
        
        <TTLForm form={form} properties={detail?.properties || []} />

        <div className={styles.gqlPreview}>
          <label>GQL Preview:</label>
          <pre>{gql}</pre>
        </div>
      </Form>
    </Modal>
  );
};

export default EditModal;
```

### 5.2 PropertiesForm 组件

```typescript
// frontend/src/pages/Schema/components/EditModal/PropertiesForm.tsx

import React from 'react';
import { Form, Table, Button, Input, Select, Checkbox, Space } from 'antd';
import { PlusOutlined, DeleteOutlined } from '@ant-design/icons';

const DATA_TYPES = ['STRING', 'INT64', 'DOUBLE', 'BOOL', 'DATETIME', 'DATE', 'TIME', 'TIMESTAMP'];

interface PropertiesFormProps {
  form: FormInstance;
  onChange: () => void;
}

const PropertiesForm: React.FC<PropertiesFormProps> = ({ form, onChange }) => {
  const properties = Form.useWatch('properties', form) || [];

  const handleAdd = () => {
    const current = form.getFieldValue('properties') || [];
    form.setFieldsValue({
      properties: [
        ...current,
        { name: '', type: 'STRING', nullable: true, status: 'added' },
      ],
    });
    onChange();
  };

  const handleDelete = (index: number) => {
    const current = form.getFieldValue('properties') || [];
    const prop = current[index];
    
    if (prop.status === 'existing') {
      // 标记为删除而不是直接移除
      current[index] = { ...prop, status: 'deleted' };
    } else {
      current.splice(index, 1);
    }
    
    form.setFieldsValue({ properties: [...current] });
    onChange();
  };

  const columns = [
    { title: 'Name', dataIndex: 'name', render: (_, __, index) => (
      <Form.Item name={['properties', index, 'name']} rules={[{ required: true }]}>
        <Input />
      </Form.Item>
    )},
    { title: 'Type', dataIndex: 'type', render: (_, __, index) => (
      <Form.Item name={['properties', index, 'type']}>
        <Select options={DATA_TYPES.map(t => ({ value: t, label: t }))} />
      </Form.Item>
    )},
    { title: 'Default', dataIndex: 'default', render: (_, __, index) => (
      <Form.Item name={['properties', index, 'default']}>
        <Input placeholder="Optional" />
      </Form.Item>
    )},
    { title: 'Nullable', dataIndex: 'nullable', render: (_, __, index) => (
      <Form.Item name={['properties', index, 'nullable']} valuePropName="checked">
        <Checkbox />
      </Form.Item>
    )},
    { title: 'Action', render: (_, __, index) => (
      <Button type="link" danger icon={<DeleteOutlined />} onClick={() => handleDelete(index)}>
        Delete
      </Button>
    )},
  ];

  return (
    <div>
      <h4>Properties</h4>
      <Form.List name="properties">
        {() => (
          <Table
            dataSource={properties.filter(p => p.status !== 'deleted')}
            columns={columns}
            pagination={false}
            size="small"
          />
        )}
      </Form.List>
      <Button type="dashed" icon={<PlusOutlined />} onClick={handleAdd}>
        Add Property
      </Button>
    </div>
  );
};

export default PropertiesForm;
```

## 6. 注意事项

1. **数据一致性**: 编辑前需要重新获取最新 Schema 定义，避免并发修改冲突
2. **变更追踪**: 需要准确追踪属性的增删改状态，生成正确的 ALTER 语句
3. **错误处理**: ALTER 语句可能因数据不兼容而失败，需要良好的错误提示
4. **权限检查**: 编辑 Schema 需要相应权限，需在 UI 层面进行控制

## 7. 参考文档

- [总体分析文档](./01_schema_analysis.md)
- [阶段二：索引创建增强](./03_phase2_index_enhancement.md)
