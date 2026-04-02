# 阶段五：Space 统计与 TTL 配置实施方案

## 1. 概述

本阶段实现两个独立但重要的功能：Space 统计功能和 TTL（Time To Live）配置功能。Space 统计用于监控数据分布，TTL 用于管理数据生命周期。

## 2. Space 统计功能

### 2.1 功能需求

| 功能点 | 描述 | 优先级 |
|--------|------|--------|
| 统计信息展示 | 显示 Tag、Edge 的数据量统计 | 高 |
| 手动触发统计 | 支持手动提交统计任务 | 高 |
| 任务状态跟踪 | 显示统计任务执行状态 | 中 |
| 历史记录 | 显示统计历史更新时间 | 中 |
| 自动刷新 | 定时刷新统计状态 | 低 |

### 2.2 界面设计

```
+----------------------------------------------------------+
|  Space Statistics: production_space          [Refresh Stats]     |
+----------------------------------------------------------+
|                                                          |
|  Last Updated: 2024-01-15 10:30:25                      |
|  Status: ✅ Finished                                     |
|                                                          |
|  Summary                                                 |
|  +----------------------------------------------------+  |
|  | Total Vertices: 1,234,567    Total Edges: 5,678,901|  |
|  +----------------------------------------------------+  |
|                                                          |
|  Tag Statistics                                          |
|  +----------------+----------------+----------------+    |
|  | Type           | Name           | Count          |    |
|  +----------------+----------------+----------------+    |
|  | Tag            | Person         | 500,000        |    |
|  | Tag            | Company        | 100,000        |    |
|  | Tag            | Product        | 634,567        |    |
|  +----------------+----------------+----------------+    |
|                                                          |
|  Edge Statistics                                         |
|  +----------------+----------------+----------------+    |
|  | Type           | Name           | Count          |    |
|  +----------------+----------------+----------------+    |
|  | Edge           | WORKS_AT       | 2,000,000      |    |
|  | Edge           | FOLLOWS        | 3,678,901      |    |
|  +----------------+----------------+----------------+    |
+----------------------------------------------------------+
```

### 2.3 组件设计

```typescript
// frontend/src/pages/Schema/SpaceStats/index.tsx

import React, { useEffect, useState, useCallback, useRef } from 'react';
import { Card, Table, Button, Spin, message, Badge, Statistic, Row, Col } from 'antd';
import { ReloadOutlined, ClockCircleOutlined } from '@ant-design/icons';
import { useSchemaStore } from '@/stores/schema';
import { submitStatsJob, getStatsResult, getJobStatus, JobStatus } from '@/services/stats';
import styles from './index.module.less';

interface StatsData {
  tags: Array<{ name: string; count: number }>;
  edges: Array<{ name: string; count: number }>;
  totalVertices: number;
  totalEdges: number;
}

const SpaceStats: React.FC = () => {
  const { currentSpace } = useSchemaStore();
  const [loading, setLoading] = useState(false);
  const [statsData, setStatsData] = useState<StatsData | null>(null);
  const [lastUpdated, setLastUpdated] = useState<string>('');
  const [jobStatus, setJobStatus] = useState<JobStatus | null>(null);
  const timerRef = useRef<NodeJS.Timeout | null>(null);

  // 获取统计结果
  const fetchStats = useCallback(async () => {
    if (!currentSpace) return;

    try {
      const result = await getStatsResult(currentSpace);
      if (result.code === 0) {
        const data = processStatsData(result.data);
        setStatsData(data);
        setLastUpdated(new Date().toLocaleString());
      }
    } catch (err) {
      console.error('Failed to fetch stats:', err);
    }
  }, [currentSpace]);

  // 检查任务状态
  const checkJobStatus = useCallback(async (jobId: number) => {
    if (!currentSpace) return;

    try {
      const status = await getJobStatus(currentSpace, jobId);
      setJobStatus(status);

      if (status === 'FINISHED') {
        await fetchStats();
        if (timerRef.current) {
          clearTimeout(timerRef.current);
          timerRef.current = null;
        }
      } else if (status === 'RUNNING' || status === 'QUEUE') {
        // 继续轮询
        timerRef.current = setTimeout(() => checkJobStatus(jobId), 2000);
      }
    } catch (err) {
      console.error('Failed to check job status:', err);
    }
  }, [currentSpace, fetchStats]);

  // 提交统计任务
  const handleSubmitStats = async () => {
    if (!currentSpace) {
      message.warning('Please select a space first');
      return;
    }

    setLoading(true);
    try {
      const result = await submitStatsJob(currentSpace);
      if (result.code === 0) {
        message.success('Statistics job submitted');
        setJobStatus('QUEUE');
        // 开始轮询任务状态
        checkJobStatus(result.data.job_id);
      }
    } catch (err) {
      message.error('Failed to submit statistics job');
    } finally {
      setLoading(false);
    }
  };

  // 初始加载
  useEffect(() => {
    fetchStats();
    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
    };
  }, [fetchStats]);

  // 处理统计数据
  const processStatsData = (rawData: any[]): StatsData => {
    const tags: Array<{ name: string; count: number }> = [];
    const edges: Array<{ name: string; count: number }> = [];
    let totalVertices = 0;
    let totalEdges = 0;

    rawData.forEach(item => {
      if (item.Type === 'Tag') {
        tags.push({ name: item.Name, count: item.Count });
        totalVertices += item.Count;
      } else if (item.Type === 'Edge') {
        edges.push({ name: item.Name, count: item.Count });
        totalEdges += item.Count;
      }
    });

    return { tags, edges, totalVertices, totalEdges };
  };

  const getStatusBadge = () => {
    switch (jobStatus) {
      case 'FINISHED':
        return <Badge status="success" text="Finished" />;
      case 'RUNNING':
        return <Badge status="processing" text="Running" />;
      case 'QUEUE':
        return <Badge status="warning" text="Queued" />;
      case 'FAILED':
        return <Badge status="error" text="Failed" />;
      default:
        return <Badge status="default" text="Unknown" />;
    }
  };

  const tagColumns = [
    { title: 'Type', dataIndex: 'type', render: () => 'Tag' },
    { title: 'Name', dataIndex: 'name' },
    { title: 'Count', dataIndex: 'count', align: 'right' as const },
  ];

  const edgeColumns = [
    { title: 'Type', dataIndex: 'type', render: () => 'Edge' },
    { title: 'Name', dataIndex: 'name' },
    { title: 'Count', dataIndex: 'count', align: 'right' as const },
  ];

  if (!currentSpace) {
    return (
      <Card>
        <div className={styles.emptyState}>
          Please select a space to view statistics
        </div>
      </Card>
    );
  }

  return (
    <div className={styles.container}>
      <Card
        title={`Space Statistics: ${currentSpace}`}
        extra={
          <Button
            type="primary"
            icon={<ReloadOutlined />}
            onClick={handleSubmitStats}
            loading={loading || jobStatus === 'RUNNING' || jobStatus === 'QUEUE'}
          >
            Refresh Stats
          </Button>
        }
      >
        <Spin spinning={loading}>
          <div className={styles.header}>
            <div className={styles.metaInfo}>
              <span>
                <ClockCircleOutlined /> Last Updated: {lastUpdated || 'Never'}
              </span>
              <span className={styles.status}>Status: {getStatusBadge()}</span>
            </div>
          </div>

          {statsData && (
            <>
              <Row gutter={16} className={styles.summary}>
                <Col span={12}>
                  <Statistic
                    title="Total Vertices"
                    value={statsData.totalVertices}
                    formatter={(value) => value?.toLocaleString()}
                  />
                </Col>
                <Col span={12}>
                  <Statistic
                    title="Total Edges"
                    value={statsData.totalEdges}
                    formatter={(value) => value?.toLocaleString()}
                  />
                </Col>
              </Row>

              <div className={styles.tables}>
                <h3>Tag Statistics</h3>
                <Table
                  dataSource={statsData.tags}
                  columns={tagColumns}
                  pagination={false}
                  size="small"
                  rowKey="name"
                />

                <h3>Edge Statistics</h3>
                <Table
                  dataSource={statsData.edges}
                  columns={edgeColumns}
                  pagination={false}
                  size="small"
                  rowKey="name"
                />
              </div>
            </>
          )}
        </Spin>
      </Card>
    </div>
  );
};

export default SpaceStats;
```

### 2.4 API 服务

```typescript
// frontend/src/services/stats.ts

import api from './api';

export type JobStatus = 'QUEUE' | 'RUNNING' | 'FINISHED' | 'FAILED';

// 提交统计任务
export const submitStatsJob = async (space: string): Promise<ApiResponse<{ job_id: number }>> => {
  const query = 'SUBMIT JOB STATS';
  return api.post('/query', { space, query });
};

// 获取统计结果
export const getStatsResult = async (space: string): Promise<ApiResponse<any[]>> => {
  const query = 'SHOW STATS';
  return api.post('/query', { space, query });
};

// 获取任务状态
export const getJobStatus = async (space: string, jobId: number): Promise<JobStatus> => {
  const query = `SHOW JOB ${jobId}`;
  const response = await api.post('/query', { space, query });
  // 解析任务状态
  return response.data.tables[0]?.Status as JobStatus;
};
```

## 3. TTL 配置功能

### 3.1 功能需求

| 功能点 | 描述 | 优先级 |
|--------|------|--------|
| TTL 配置表单 | 在创建/编辑 Tag/Edge 时配置 TTL | 高 |
| 列选择 | 选择 TTL 基于的列 | 高 |
| 持续时间设置 | 设置数据存活时间（秒） | 高 |
| 实时预览 | 显示 TTL 配置的 GQL 效果 | 中 |

### 3.2 界面设计

```
TTL Configuration (在 Create/Edit Modal 中)
+----------------------------------------------------------+
|  TTL Configuration                                       |
|  +----------------------------------------------------+  |
|  |                                                    |  |
|  |  Enable TTL: [Toggle]                              |  |
|  |                                                    |  |
|  |  TTL Column: [age______________v]                  |  |
|  |  (Select a column of type INT64 or TIMESTAMP)      |  |
|  |                                                    |  |
|  |  Duration: [3600_______________] seconds           |  |
|  |  = 1 hour(s)                                       |  |
|  |                                                    |  |
|  |  [?] Data will be automatically deleted after      |  |
|  |      the specified duration from the TTL column    |  |
|  |      timestamp                                     |  |
|  +----------------------------------------------------+  |
+----------------------------------------------------------+
```

### 3.3 TTLForm 组件

```typescript
// frontend/src/pages/Schema/components/TTLForm/index.tsx

import React, { useEffect, useState } from 'react';
import { Form, Select, InputNumber, Switch, Tooltip, Alert } from 'antd';
import { InfoCircleOutlined } from '@ant-design/icons';
import type { PropertyDef } from '@/types/schema';
import styles from './index.module.less';

const { Option } = Select;

interface TTLFormProps {
  form: FormInstance;
  properties: PropertyDef[];
}

// 支持 TTL 的数据类型
const TTL_SUPPORTED_TYPES = ['INT64', 'TIMESTAMP', 'DATETIME'];

const TTLForm: React.FC<TTLFormProps> = ({ form, properties }) => {
  const [enabled, setEnabled] = useState(false);
  const [duration, setDuration] = useState<number | null>(null);

  // 过滤支持 TTL 的属性
  const ttlEligibleProperties = properties.filter(p =>
    TTL_SUPPORTED_TYPES.includes(p.type)
  );

  // 监听启用状态变化
  const handleEnableChange = (checked: boolean) => {
    setEnabled(checked);
    if (!checked) {
      form.setFieldsValue({
        ttlCol: undefined,
        ttlDuration: undefined,
      });
    }
  };

  // 格式化持续时间显示
  const formatDuration = (seconds: number | null): string => {
    if (!seconds) return '';
    if (seconds < 60) return `${seconds} second(s)`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)} minute(s)`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)} hour(s)`;
    return `${Math.floor(seconds / 86400)} day(s)`;
  };

  return (
    <div className={styles.ttlForm}>
      <div className={styles.header}>
        <h4>TTL Configuration</h4>
        <Switch
          checked={enabled}
          onChange={handleEnableChange}
          checkedChildren="Enabled"
          unCheckedChildren="Disabled"
        />
      </div>

      {enabled && (
        <>
          {ttlEligibleProperties.length === 0 ? (
            <Alert
              type="warning"
              message="No eligible properties"
              description="TTL requires a property of type INT64, TIMESTAMP, or DATETIME. Please add such a property first."
              showIcon
            />
          ) : (
            <>
              <Form.Item
                name="ttlCol"
                label="TTL Column"
                rules={[{ required: true, message: 'Please select TTL column' }]}
              >
                <Select placeholder="Select a column">
                  {ttlEligibleProperties.map(prop => (
                    <Option key={prop.name} value={prop.name}>
                      {prop.name} ({prop.type})
                    </Option>
                  ))}
                </Select>
              </Form.Item>

              <Form.Item
                name="ttlDuration"
                label={
                  <span>
                    Duration (seconds)
                    <Tooltip title="Data will be automatically deleted after this duration from the TTL column timestamp">
                      <InfoCircleOutlined style={{ marginLeft: 8 }} />
                    </Tooltip>
                  </span>
                }
                rules={[{ required: true, message: 'Please enter duration' }]}
              >
                <InputNumber
                  min={1}
                  style={{ width: '100%' }}
                  placeholder="e.g., 3600"
                  onChange={setDuration}
                />
              </Form.Item>

              {duration && (
                <div className={styles.durationHint}>
                  = {formatDuration(duration)}
                </div>
              )}

              <Alert
                type="info"
                message="TTL Behavior"
                description="When TTL is enabled, data will be automatically marked as expired and deleted after the specified duration from the timestamp in the TTL column."
                showIcon
              />
            </>
          )}
        </>
      )}
    </div>
  );
};

export default TTLForm;
```

### 3.4 样式文件

```less
// frontend/src/pages/Schema/components/TTLForm/index.module.less

.ttlForm {
  margin: 16px 0;
  padding: 16px;
  background: #f6ffed;
  border: 1px solid #b7eb8f;
  border-radius: 4px;

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;

    h4 {
      margin: 0;
    }
  }

  .durationHint {
    margin-top: -16px;
    margin-bottom: 16px;
    color: #666;
    font-size: 12px;
  }
}
```

### 3.5 集成到创建/编辑表单

```typescript
// 在 CreateTagModal 或 EditModal 中使用

import TTLForm from '../components/TTLForm';

// 在 Form 中添加 TTL 字段
<Form form={form} layout="vertical">
  {/* ... 其他表单项 ... */}
  
  <TTLForm form={form} properties={properties} />
</Form>

// 生成 GQL 时处理 TTL
const generateCreateGQL = (values: any) => {
  const { name, properties, comment, ttlCol, ttlDuration } = values;
  
  let gql = `CREATE TAG ${name} (`;
  gql += properties.map((p: any) => `${p.name} ${p.type}`).join(', ');
  gql += ')';
  
  // 添加 TTL
  if (ttlCol && ttlDuration) {
    gql += ` ttl_duration = ${ttlDuration}, ttl_col = "${ttlCol}"`;
  }
  
  // 添加注释
  if (comment) {
    gql += ` comment = "${comment}"`;
  }
  
  gql += ';';
  return gql;
};
```

## 4. 实现步骤

### 4.1 Space 统计功能 (2-3 天)

1. 创建 `services/stats.ts` API 服务
2. 创建 `SpaceStats` 页面组件
3. 添加路由配置
4. 实现任务状态轮询
5. 测试统计功能

### 4.2 TTL 配置功能 (1-2 天)

1. 创建 `TTLForm` 组件
2. 集成到 Tag/Edge 创建模态框
3. 集成到 Tag/Edge 编辑模态框
4. 测试 TTL 配置

## 5. 注意事项

### 5.1 Space 统计

1. **异步任务**: 统计是异步任务，需要轮询检查状态
2. **性能影响**: 统计任务可能影响数据库性能，避免频繁执行
3. **错误处理**: 任务可能失败，需要友好的错误提示

### 5.2 TTL 配置

1. **数据类型限制**: TTL 只支持 INT64、TIMESTAMP、DATETIME 类型
2. **单位**: 持续时间以秒为单位
3. **行为理解**: 用户需要理解 TTL 是标记过期而非立即删除
4. **修改限制**: 某些情况下 TTL 配置修改可能受限

## 6. 参考文档

- [总体分析文档](./01_schema_analysis.md)
- [阶段四：Schema 可视化](./05_phase4_schema_visualization.md)
- [NebulaGraph TTL 文档](https://docs.nebula-graph.io/3.6.0/3.ngql-guide/10.tag-statements/1.create-tag/#ttl)
