import React from 'react';
import { Card, Descriptions, Button, Space, Tag, Tooltip } from 'antd';
import { CopyOutlined, CloseOutlined } from '@ant-design/icons';
import { useGraphStore } from '@/stores/graph';
import { copyToClipboard } from '@/utils/function';
import styles from './index.module.less';

const DetailPanel: React.FC = () => {
  const { detailPanelVisible, detailData, detailType, hideDetail } = useGraphStore();

  if (!detailPanelVisible || !detailData) return null;

  const handleCopyId = () => {
    copyToClipboard(detailData.id);
  };

  const renderNodeDetail = () => {
    const node = detailData as import('@/stores/graph').NodeDetail;
    return (
      <>
        <Descriptions column={1} size="small" bordered>
          <Descriptions.Item label="ID">
            <Space>
              <span className={styles.idText}>{node.id}</span>
              <Tooltip title="Copy ID">
                <Button
                  icon={<CopyOutlined />}
                  size="small"
                  type="text"
                  onClick={handleCopyId}
                />
              </Tooltip>
            </Space>
          </Descriptions.Item>
          <Descriptions.Item label="Tag">
            <Tag color="blue">{node.tag}</Tag>
          </Descriptions.Item>
        </Descriptions>
        {Object.keys(node.properties).length > 0 && (
          <div className={styles.propertiesSection}>
            <div className={styles.sectionTitle}>Properties</div>
            <Descriptions column={1} size="small" bordered>
              {Object.entries(node.properties).map(([key, value]) => (
                <Descriptions.Item key={key} label={key}>
                  {String(value)}
                </Descriptions.Item>
              ))}
            </Descriptions>
          </div>
        )}
      </>
    );
  };

  const renderEdgeDetail = () => {
    const edge = detailData as import('@/stores/graph').EdgeDetail;
    return (
      <>
        <Descriptions column={1} size="small" bordered>
          <Descriptions.Item label="ID">
            <Space>
              <span className={styles.idText}>{edge.id}</span>
              <Tooltip title="Copy ID">
                <Button
                  icon={<CopyOutlined />}
                  size="small"
                  type="text"
                  onClick={handleCopyId}
                />
              </Tooltip>
            </Space>
          </Descriptions.Item>
          <Descriptions.Item label="Type">
            <Tag color="green">{edge.type}</Tag>
          </Descriptions.Item>
          <Descriptions.Item label="Source">
            <span className={styles.idText}>{edge.source}</span>
          </Descriptions.Item>
          <Descriptions.Item label="Target">
            <span className={styles.idText}>{edge.target}</span>
          </Descriptions.Item>
          <Descriptions.Item label="Rank">{edge.rank}</Descriptions.Item>
        </Descriptions>
        {Object.keys(edge.properties).length > 0 && (
          <div className={styles.propertiesSection}>
            <div className={styles.sectionTitle}>Properties</div>
            <Descriptions column={1} size="small" bordered>
              {Object.entries(edge.properties).map(([key, value]) => (
                <Descriptions.Item key={key} label={key}>
                  {String(value)}
                </Descriptions.Item>
              ))}
            </Descriptions>
          </div>
        )}
      </>
    );
  };

  return (
    <Card
      title={detailType === 'node' ? 'Node Detail' : 'Edge Detail'}
      size="small"
      className={styles.panel}
      extra={
        <Button
          icon={<CloseOutlined />}
          size="small"
          type="text"
          onClick={hideDetail}
        />
      }
    >
      {detailType === 'node' ? renderNodeDetail() : renderEdgeDetail()}
    </Card>
  );
};

export default DetailPanel;
