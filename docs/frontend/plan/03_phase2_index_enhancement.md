# 阶段二：索引创建增强实施方案

## 1. 概述

本阶段增强索引创建功能，添加拖拽字段排序和字段选择模态框，提升用户创建复合索引的体验。

## 2. 功能需求

### 2.1 功能清单

| 功能点 | 描述 | 优先级 |
|--------|------|--------|
| 拖拽字段排序 | 通过拖拽调整索引字段顺序 | 高 |
| 字段选择模态框 | 从 Tag/Edge 属性中选择字段 | 高 |
| 字段顺序预览 | 实时显示索引字段顺序 | 中 |
| GQL 预览 | 实时显示 CREATE INDEX 语句 | 中 |

### 2.2 界面设计

```
+----------------------------------------------------------+
|  Create Index                                    [Cancel] [Create] |
+----------------------------------------------------------+
|                                                          |
|  Basic Info                                              |
|  +----------------------------------------------------+  |
|  | Index Name: [idx_person_name_age________________]  |  |
|  | Index Type: [Tag____v]  Entity: [Person_____v]     |  |
|  +----------------------------------------------------+  |
|                                                          |
|  Index Fields (drag to reorder)                          |
|  +----------------------------------------------------+  |
|  | [=] name     STRING    [x]                         |  |
|  | [=] age      INT64     [x]                         |  |
|  | [+ Add Field]                                      |  |
|  +----------------------------------------------------+  |
|                                                          |
|  GQL Preview                                             |
|  +----------------------------------------------------+  |
|  | CREATE TAG INDEX idx_person_name_age ON Person(     |  |
|  |   name,                                             |  |
|  |   age                                               |  |
|  | );                                                  |  |
|  +----------------------------------------------------+  |
+----------------------------------------------------------+

字段选择模态框:
+----------------------------------------------------------+
|  Select Fields                                 [Cancel] [Confirm]  |
+----------------------------------------------------------+
|                                                          |
|  Available Fields (Person)                               |
|  +----------------------------------------------------+  |
|  | [x] name        STRING                             |  |
|  | [ ] email       STRING                             |  |
|  | [x] age         INT64                              |  |
|  | [ ] created_at  DATETIME                           |  |
|  +----------------------------------------------------+  |
|                                                          |
|  Selected: name, age                                     |
+----------------------------------------------------------+
```

## 3. 技术方案

### 3.1 目录结构

```
frontend/src/pages/Schema/
├── IndexList/
│   ├── components/              # 新增：子组件目录
│   │   ├── DraggableField.tsx   # 可拖拽字段项
│   │   ├── FieldSelectModal.tsx # 字段选择模态框
│   │   └── CreateIndexModal.tsx # 重构后的创建模态框
│   ├── index.module.less
│   └── index.tsx
```

### 3.2 依赖库

```bash
npm install @dnd-kit/core @dnd-kit/sortable @dnd-kit/utilities
```

### 3.3 组件设计

#### 3.3.1 DraggableField 组件

```typescript
// frontend/src/pages/Schema/IndexList/components/DraggableField.tsx

import React from 'react';
import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { HolderOutlined, CloseOutlined } from '@ant-design/icons';
import { Tag, Button } from 'antd';
import styles from './index.module.less';

interface DraggableFieldProps {
  id: string;
  name: string;
  type: string;
  onRemove: () => void;
}

const DraggableField: React.FC<DraggableFieldProps> = ({
  id,
  name,
  type,
  onRemove,
}) => {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={styles.draggableField}
    >
      <HolderOutlined
        className={styles.dragHandle}
        {...attributes}
        {...listeners}
      />
      <span className={styles.fieldName}>{name}</span>
      <Tag size="small">{type}</Tag>
      <Button
        type="text"
        size="small"
        icon={<CloseOutlined />}
        onClick={onRemove}
        danger
      />
    </div>
  );
};

export default DraggableField;
```

#### 3.3.2 FieldSelectModal 组件

```typescript
// frontend/src/pages/Schema/IndexList/components/FieldSelectModal.tsx

import React, { useState, useEffect } from 'react';
import { Modal, Checkbox, List, Tag, Empty } from 'antd';
import { useSchemaStore } from '@/stores/schema';
import styles from './index.module.less';

interface FieldSelectModalProps {
  visible: boolean;
  space: string;
  entityType: 'TAG' | 'EDGE';
  entityName: string;
  selectedFields: string[];
  onConfirm: (fields: string[]) => void;
  onCancel: () => void;
}

interface FieldInfo {
  name: string;
  type: string;
  comment?: string;
}

const FieldSelectModal: React.FC<FieldSelectModalProps> = ({
  visible,
  space,
  entityType,
  entityName,
  selectedFields,
  onConfirm,
  onCancel,
}) => {
  const [fields, setFields] = useState<FieldInfo[]>([]);
  const [selected, setSelected] = useState<string[]>(selectedFields);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (visible && entityName) {
      loadFields();
    }
    setSelected(selectedFields);
  }, [visible, entityName, selectedFields]);

  const loadFields = async () => {
    setLoading(true);
    try {
      // 调用 API 获取 Tag/Edge 的属性列表
      const response = await fetchEntityFields(space, entityType, entityName);
      setFields(response.properties);
    } finally {
      setLoading(false);
    }
  };

  const handleToggle = (fieldName: string) => {
    setSelected(prev =>
      prev.includes(fieldName)
        ? prev.filter(f => f !== fieldName)
        : [...prev, fieldName]
    );
  };

  return (
    <Modal
      title="Select Fields"
      open={visible}
      width={500}
      onOk={() => onConfirm(selected)}
      onCancel={onCancel}
      confirmLoading={loading}
    >
      <div className={styles.fieldSelectModal}>
        <p className={styles.entityInfo}>
          Available fields from {entityType.toLowerCase()} <strong>{entityName}</strong>:
        </p>
        
        {fields.length === 0 ? (
          <Empty description="No fields available" />
        ) : (
          <List
            dataSource={fields}
            renderItem={field => (
              <List.Item
                className={styles.fieldItem}
                onClick={() => handleToggle(field.name)}
              >
                <Checkbox
                  checked={selected.includes(field.name)}
                  onChange={() => handleToggle(field.name)}
                />
                <span className={styles.fieldName}>{field.name}</span>
                <Tag size="small">{field.type}</Tag>
                {field.comment && (
                  <span className={styles.fieldComment}>{field.comment}</span>
                )}
              </List.Item>
            )}
          />
        )}
        
        <div className={styles.selectedSummary}>
          Selected: {selected.length > 0 ? selected.join(', ') : 'None'}
        </div>
      </div>
    </Modal>
  );
};

export default FieldSelectModal;
```

#### 3.3.3 CreateIndexModal 组件

```typescript
// frontend/src/pages/Schema/IndexList/components/CreateIndexModal.tsx

import React, { useState, useCallback, useEffect } from 'react';
import { Modal, Form, Input, Select, Button, message } from 'antd';
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
} from '@dnd-kit/core';
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { PlusOutlined } from '@ant-design/icons';
import { useSchemaStore } from '@/stores/schema';
import DraggableField from './DraggableField';
import FieldSelectModal from './FieldSelectModal';
import styles from './index.module.less';

const { Option } = Select;

interface CreateIndexModalProps {
  visible: boolean;
  space: string;
  onCancel: () => void;
  onSuccess: () => void;
}

interface IndexField {
  id: string;
  name: string;
  type: string;
}

const CreateIndexModal: React.FC<CreateIndexModalProps> = ({
  visible,
  space,
  onCancel,
  onSuccess,
}) => {
  const [form] = Form.useForm();
  const { tags, edgeTypes, createIndex } = useSchemaStore();
  
  const [indexType, setIndexType] = useState<'TAG' | 'EDGE'>('TAG');
  const [selectedEntity, setSelectedEntity] = useState<string>('');
  const [fields, setFields] = useState<IndexField[]>([]);
  const [fieldSelectVisible, setFieldSelectVisible] = useState(false);
  const [gqlPreview, setGqlPreview] = useState('');

  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  // 生成 GQL 预览
  const updateGQLPreview = useCallback(() => {
    const values = form.getFieldsValue();
    if (!values.name || !selectedEntity || fields.length === 0) {
      setGqlPreview('');
      return;
    }

    const fieldList = fields.map(f => f.name).join(', ');
    const entityType = indexType === 'TAG' ? 'TAG' : 'EDGE';
    
    setGqlPreview(
      `CREATE ${entityType} INDEX ${values.name} ON ${selectedEntity}(${fieldList});`
    );
  }, [form, indexType, selectedEntity, fields]);

  useEffect(() => {
    updateGQLPreview();
  }, [fields, selectedEntity, updateGQLPreview]);

  // 拖拽排序处理
  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    
    if (active.id !== over?.id) {
      setFields(items => {
        const oldIndex = items.findIndex(i => i.id === active.id);
        const newIndex = items.findIndex(i => i.id === over?.id);
        return arrayMove(items, oldIndex, newIndex);
      });
    }
  };

  // 添加字段
  const handleAddFields = (selectedFieldNames: string[]) => {
    // 获取字段类型信息
    const entityList = indexType === 'TAG' ? tags : edgeTypes;
    const entity = entityList.find(e => e.name === selectedEntity);
    
    const newFields = selectedFieldNames.map(name => {
      const prop = entity?.properties.find(p => p.name === name);
      return {
        id: `${name}_${Date.now()}`,
        name,
        type: prop?.type || 'STRING',
      };
    });

    setFields(prev => [...prev, ...newFields]);
    setFieldSelectVisible(false);
  };

  // 移除字段
  const handleRemoveField = (id: string) => {
    setFields(prev => prev.filter(f => f.id !== id));
  };

  // 提交创建
  const handleSubmit = async () => {
    try {
      await form.validateFields();
      
      if (fields.length === 0) {
        message.error('Please select at least one field');
        return;
      }

      const values = form.getFieldsValue();
      await createIndex(space, {
        name: values.name,
        index_type: indexType,
        entity_type: indexType,
        entity_name: selectedEntity,
        fields: fields.map(f => f.name),
      });

      message.success(`Index "${values.name}" created successfully`);
      onSuccess();
    } catch (err) {
      message.error('Failed to create index');
    }
  };

  const entityOptions = indexType === 'TAG' ? tags : edgeTypes;

  return (
    <>
      <Modal
        title="Create Index"
        open={visible}
        width={600}
        onCancel={onCancel}
        onOk={handleSubmit}
      >
        <Form
          form={form}
          layout="vertical"
          onValuesChange={updateGQLPreview}
        >
          <Form.Item
            name="name"
            label="Index Name"
            rules={[{ required: true, message: 'Please enter index name' }]}
          >
            <Input placeholder="e.g., idx_person_name" />
          </Form.Item>

          <Form.Item label="Index Type">
            <Select value={indexType} onChange={setIndexType}>
              <Option value="TAG">Tag</Option>
              <Option value="EDGE">Edge</Option>
            </Select>
          </Form.Item>

          <Form.Item label="Entity">
            <Select
              value={selectedEntity}
              onChange={setSelectedEntity}
              placeholder={`Select ${indexType.toLowerCase()}`}
            >
              {entityOptions.map(e => (
                <Option key={e.name} value={e.name}>{e.name}</Option>
              ))}
            </Select>
          </Form.Item>

          <div className={styles.fieldsSection}>
            <label>Index Fields (drag to reorder)</label>
            
            <DndContext
              sensors={sensors}
              collisionDetection={closestCenter}
              onDragEnd={handleDragEnd}
            >
              <SortableContext
                items={fields.map(f => f.id)}
                strategy={verticalListSortingStrategy}
              >
                <div className={styles.fieldsList}>
                  {fields.map(field => (
                    <DraggableField
                      key={field.id}
                      id={field.id}
                      name={field.name}
                      type={field.type}
                      onRemove={() => handleRemoveField(field.id)}
                    />
                  ))}
                </div>
              </SortableContext>
            </DndContext>

            <Button
              type="dashed"
              icon={<PlusOutlined />}
              onClick={() => setFieldSelectVisible(true)}
              disabled={!selectedEntity}
            >
              Add Field
            </Button>
          </div>

          {gqlPreview && (
            <div className={styles.gqlPreview}>
              <label>GQL Preview:</label>
              <pre>{gqlPreview}</pre>
            </div>
          )}
        </Form>
      </Modal>

      <FieldSelectModal
        visible={fieldSelectVisible}
        space={space}
        entityType={indexType}
        entityName={selectedEntity}
        selectedFields={fields.map(f => f.name)}
        onConfirm={handleAddFields}
        onCancel={() => setFieldSelectVisible(false)}
      />
    </>
  );
};

export default CreateIndexModal;
```

## 4. 样式文件

```less
// frontend/src/pages/Schema/IndexList/components/index.module.less

.draggableField {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: #f5f5f5;
  border-radius: 4px;
  margin-bottom: 8px;
  cursor: default;

  .dragHandle {
    cursor: grab;
    color: #999;
    
    &:active {
      cursor: grabbing;
    }
  }

  .fieldName {
    flex: 1;
    font-weight: 500;
  }
}

.fieldsSection {
  margin: 16px 0;

  label {
    display: block;
    margin-bottom: 8px;
    font-weight: 500;
  }
}

.fieldsList {
  margin-bottom: 12px;
}

.gqlPreview {
  margin-top: 16px;
  padding: 12px;
  background: #f6ffed;
  border: 1px solid #b7eb8f;
  border-radius: 4px;

  label {
    display: block;
    margin-bottom: 8px;
    font-weight: 500;
    color: #52c41a;
  }

  pre {
    margin: 0;
    font-size: 12px;
    white-space: pre-wrap;
    word-break: break-all;
  }
}

// FieldSelectModal styles
.fieldSelectModal {
  .entityInfo {
    margin-bottom: 16px;
    color: #666;
  }

  .fieldItem {
    cursor: pointer;
    padding: 8px;
    border-radius: 4px;
    transition: background 0.2s;

    &:hover {
      background: #f5f5f5;
    }

    .fieldName {
      flex: 1;
      margin-left: 8px;
    }

    .fieldComment {
      color: #999;
      font-size: 12px;
    }
  }

  .selectedSummary {
    margin-top: 16px;
    padding-top: 16px;
    border-top: 1px solid #e8e8e8;
    color: #666;
  }
}
```

## 5. 实现步骤

### 步骤 1: 安装依赖 (0.5 天)

```bash
cd frontend
npm install @dnd-kit/core @dnd-kit/sortable @dnd-kit/utilities
```

### 步骤 2: 创建组件 (2-3 天)

1. 创建 `DraggableField.tsx`
2. 创建 `FieldSelectModal.tsx`
3. 创建 `CreateIndexModal.tsx`
4. 添加样式文件

### 步骤 3: 集成到 IndexList (1 天)

1. 替换原有的创建逻辑
2. 添加拖拽排序功能
3. 集成字段选择模态框

### 步骤 4: 测试与优化 (1-2 天)

1. 测试拖拽排序
2. 测试字段选择
3. 测试 GQL 生成
4. UI 优化

## 6. 注意事项

1. **拖拽库选择**: @dnd-kit 是一个现代化的拖拽库，支持 React 18，API 设计合理
2. **字段唯一性**: 确保同一字段不能被重复添加到索引中
3. **排序持久化**: 字段顺序会直接影响索引效率，需要正确生成 GQL
4. **空状态处理**: 当没有可用字段时，需要友好的空状态提示

## 7. 参考文档

- [总体分析文档](./01_schema_analysis.md)
- [阶段一：Tag/Edge 编辑功能](./02_phase1_tag_edge_edit.md)
- [@dnd-kit 官方文档](https://docs.dndkit.com/)
