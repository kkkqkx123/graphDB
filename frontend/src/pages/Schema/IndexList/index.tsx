import React, { useEffect, useState } from 'react';
import {
  Card,
  Table,
  Button,
  Space as AntSpace,
  Tooltip,
  Popconfirm,
  message,
  Empty,
  Spin,
  Typography,
  Tag,
  Input,
  Modal,
  Form,
  Select,
  Badge,
} from 'antd';
import {
  PlusOutlined,
  ReloadOutlined,
  EyeOutlined,
  DeleteOutlined,
  SyncOutlined,
  FileSearchOutlined,
} from '@ant-design/icons';
import { useSchemaStore } from '@/stores/schema';
import type { IndexInfo } from '@/types/schema';
import styles from './index.module.less';

const { Title, Text } = Typography;

type IndexStatus = 'creating' | 'finished' | 'failed' | 'rebuilding';

interface IndexWithStatus extends IndexInfo {
  status?: IndexStatus;
  progress?: number;
}

const IndexList: React.FC = () => {
  const {
    indexes,
    isLoadingIndexes,
    indexesError,
    currentSpace,
    tags,
    edgeTypes,
    fetchIndexes,
    fetchTags,
    fetchEdgeTypes,
    createIndex,
    deleteIndex,
    rebuildIndex,
  } = useSchemaStore();

  const [searchText, setSearchText] = useState('');
  const [createModalVisible, setCreateModalVisible] = useState(false);
  const [detailModalVisible, setDetailModalVisible] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState<IndexWithStatus | null>(null);
  const [form] = Form.useForm();
  const [indexType, setIndexType] = useState<'TAG' | 'EDGE'>('TAG');
  const [selectedEntity, setSelectedEntity] = useState<string>('');
  const [selectedFields, setSelectedFields] = useState<string[]>([]);

  useEffect(() => {
    if (currentSpace) {
      fetchIndexes(currentSpace);
      fetchTags(currentSpace);
      fetchEdgeTypes(currentSpace);
    }
  }, [currentSpace, fetchIndexes, fetchTags, fetchEdgeTypes]);

  const handleRefresh = () => {
    if (currentSpace) {
      fetchIndexes(currentSpace);
      message.success('Index list refreshed');
    }
  };

  const handleCreate = async () => {
    try {
      const values = await form.validateFields();
      if (currentSpace) {
        await createIndex(currentSpace, {
          name: values.name,
          index_type: values.indexType,
          entity_type: values.indexType,
          entity_name: selectedEntity,
          fields: selectedFields,
        });
        message.success(`Index "${values.name}" created successfully`);
        setCreateModalVisible(false);
        form.resetFields();
        setSelectedEntity('');
        setSelectedFields([]);
      }
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to create index';
      message.error(errorMessage);
    }
  };

  const handleDelete = async (index: IndexWithStatus) => {
    try {
      if (currentSpace) {
        await deleteIndex(currentSpace, index.name);
        message.success(`Index "${index.name}" deleted successfully`);
      }
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to delete index';
      message.error(errorMessage);
    }
  };

  const handleRebuild = async (index: IndexWithStatus) => {
    try {
      if (currentSpace) {
        await rebuildIndex(currentSpace, index.name);
        message.success(`Index "${index.name}" rebuild started`);
      }
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to rebuild index';
      message.error(errorMessage);
    }
  };

  const handleViewDetail = (index: IndexWithStatus) => {
    setSelectedIndex(index);
    setDetailModalVisible(true);
  };

  const getEntityProperties = (entityType: 'TAG' | 'EDGE', entityName: string): string[] => {
    if (entityType === 'TAG') {
      const tag = tags.find((t) => t.name === entityName);
      return tag?.properties.map((p) => p.name) || [];
    } else {
      const edge = edgeTypes.find((e) => e.name === entityName);
      return edge?.properties.map((p) => p.name) || [];
    }
  };

  const getStatusBadge = (status?: IndexStatus) => {
    switch (status) {
      case 'finished':
        return <Badge status="success" text="Finished" />;
      case 'creating':
        return <Badge status="processing" text="Creating" />;
      case 'rebuilding':
        return <Badge status="warning" text="Rebuilding" />;
      case 'failed':
        return <Badge status="error" text="Failed" />;
      default:
        return <Badge status="default" text="Unknown" />;
    }
  };

  const filteredIndexes = indexes.filter((index) =>
    index.name.toLowerCase().includes(searchText.toLowerCase())
  );

  const columns = [
    {
      title: 'Name',
      dataIndex: 'name',
      key: 'name',
      render: (name: string) => (
        <AntSpace>
          <FileSearchOutlined />
          <Text strong>{name}</Text>
        </AntSpace>
      ),
      sorter: (a: IndexInfo, b: IndexInfo) => a.name.localeCompare(b.name),
    },
    {
      title: 'Type',
      dataIndex: 'entity_type',
      key: 'type',
      render: (type: string) => (
        <Tag color={type === 'TAG' ? 'blue' : 'green'}>{type}</Tag>
      ),
    },
    {
      title: 'Entity',
      dataIndex: 'entity_name',
      key: 'entity',
    },
    {
      title: 'Fields',
      dataIndex: 'fields',
      key: 'fields',
      render: (fields: string[]) => fields.join(', '),
    },
    {
      title: 'Status',
      key: 'status',
      render: (_: unknown, record: IndexWithStatus) => getStatusBadge(record.status),
    },
    {
      title: 'Created At',
      dataIndex: 'created_at',
      key: 'created_at',
      render: (timestamp: number) => new Date(timestamp * 1000).toLocaleString(),
    },
    {
      title: 'Actions',
      key: 'actions',
      render: (_: unknown, record: IndexWithStatus) => (
        <AntSpace>
          <Tooltip title="View Details">
            <Button
              type="text"
              icon={<EyeOutlined />}
              onClick={() => handleViewDetail(record)}
            />
          </Tooltip>
          <Tooltip title="Rebuild">
            <Button
              type="text"
              icon={<SyncOutlined />}
              onClick={() => handleRebuild(record)}
              disabled={record.status === 'creating' || record.status === 'rebuilding'}
            />
          </Tooltip>
          <Tooltip title="Delete">
            <Popconfirm
              title="Delete Index"
              description={`Are you sure you want to delete index "${record.name}"?`}
              onConfirm={() => handleDelete(record)}
              okText="Yes"
              cancelText="No"
            >
              <Button type="text" danger icon={<DeleteOutlined />} />
            </Popconfirm>
          </Tooltip>
        </AntSpace>
      ),
    },
  ];

  if (!currentSpace) {
    return (
      <Card>
        <Empty description="Please select a space first" />
      </Card>
    );
  }

  return (
    <div className={styles.container}>
      <Card
        title={
          <AntSpace>
            <Title level={4} style={{ margin: 0 }}>Indexes</Title>
            <Text type="secondary">({filteredIndexes.length})</Text>
          </AntSpace>
        }
        extra={
          <AntSpace>
            <Input.Search
              placeholder="Search indexes..."
              value={searchText}
              onChange={(e) => setSearchText(e.target.value)}
              style={{ width: 200 }}
            />
            <Tooltip title="Refresh">
              <Button icon={<ReloadOutlined />} onClick={handleRefresh} />
            </Tooltip>
            <Button
              type="primary"
              icon={<PlusOutlined />}
              onClick={() => setCreateModalVisible(true)}
            >
              Create Index
            </Button>
          </AntSpace>
        }
      >
        <Spin spinning={isLoadingIndexes}>
          {indexesError ? (
            <Empty description={indexesError} />
          ) : (
            <Table
              dataSource={filteredIndexes}
              columns={columns}
              rowKey="id"
              pagination={{ pageSize: 10 }}
              locale={{ emptyText: <Empty description="No indexes found" /> }}
            />
          )}
        </Spin>
      </Card>

      {/* Create Index Modal */}
      <Modal
        title="Create Index"
        open={createModalVisible}
        onOk={handleCreate}
        onCancel={() => {
          setCreateModalVisible(false);
          form.resetFields();
          setSelectedEntity('');
          setSelectedFields([]);
        }}
        width={600}
      >
        <Form form={form} layout="vertical">
          <Form.Item
            name="name"
            label="Index Name"
            rules={[
              { required: true, message: 'Please enter index name' },
              { pattern: /^[a-zA-Z][a-zA-Z0-9_]*$/, message: 'Must start with letter, alphanumeric and underscores only' },
            ]}
          >
            <Input placeholder="Enter index name" />
          </Form.Item>

          <Form.Item
            name="indexType"
            label="Index Type"
            rules={[{ required: true, message: 'Please select index type' }]}
          >
            <Select
              placeholder="Select index type"
              onChange={(value: 'TAG' | 'EDGE') => {
                setIndexType(value);
                setSelectedEntity('');
                setSelectedFields([]);
              }}
              options={[
                { label: 'Tag', value: 'TAG' },
                { label: 'Edge', value: 'EDGE' },
              ]}
            />
          </Form.Item>

          <Form.Item
            label="Select Entity"
            required
          >
            <Select
              placeholder={`Select ${indexType.toLowerCase()}`}
              value={selectedEntity}
              onChange={(value) => {
                setSelectedEntity(value);
                setSelectedFields([]);
              }}
              options={
                indexType === 'TAG'
                  ? tags.map((tag) => ({ label: tag.name, value: tag.name }))
                  : edgeTypes.map((edge) => ({ label: edge.name, value: edge.name }))
              }
            />
          </Form.Item>

          <Form.Item
            label="Select Fields"
            required
          >
            <Select
              mode="multiple"
              placeholder="Select fields to index"
              value={selectedFields}
              onChange={setSelectedFields}
              disabled={!selectedEntity}
              options={getEntityProperties(indexType, selectedEntity).map((field) => ({
                label: field,
                value: field,
              }))}
            />
          </Form.Item>
        </Form>
      </Modal>

      {/* Detail Modal */}
      <Modal
        title="Index Details"
        open={detailModalVisible}
        onCancel={() => setDetailModalVisible(false)}
        footer={[
          <Button key="close" onClick={() => setDetailModalVisible(false)}>
            Close
          </Button>,
        ]}
        width={600}
      >
        {selectedIndex && (
          <div>
            <p>
              <Text strong>Name:</Text> {selectedIndex.name}
            </p>
            <p>
              <Text strong>Type:</Text>{' '}
              <Tag color={selectedIndex.entity_type === 'TAG' ? 'blue' : 'green'}>
                {selectedIndex.entity_type}
              </Tag>
            </p>
            <p>
              <Text strong>Entity:</Text> {selectedIndex.entity_name}
            </p>
            <p>
              <Text strong>Fields:</Text> {selectedIndex.fields.join(', ')}
            </p>
            <p>
              <Text strong>Status:</Text> {getStatusBadge(selectedIndex.status)}
            </p>
            <p>
              <Text strong>Created At:</Text>{' '}
              {new Date(selectedIndex.created_at * 1000).toLocaleString()}
            </p>
          </div>
        )}
      </Modal>
    </div>
  );
};

export default IndexList;
