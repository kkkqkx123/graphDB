import React from 'react';
import { Button, Space, Select, Tooltip } from 'antd';
import {
  ZoomInOutlined,
  ZoomOutOutlined,
  ExpandOutlined,
  ReloadOutlined,
  ClearOutlined,
} from '@ant-design/icons';
import { useGraphStore } from '@/stores/graph';
import { getLayoutOptions } from '@/utils/graphLayout';
import styles from './index.module.less';

interface GraphToolbarProps {
  cy?: cytoscape.Core;
}

const GraphToolbar: React.FC<GraphToolbarProps> = ({ cy }) => {
  const {
    layout,
    zoom,
    selectedNodes,
    selectedEdges,
    setLayout,
    fitToScreen,
    resetLayout,
    clearSelection,
  } = useGraphStore();

  const handleZoomIn = () => {
    if (cy) {
      cy.zoom(cy.zoom() * 1.2);
    }
  };

  const handleZoomOut = () => {
    if (cy) {
      cy.zoom(cy.zoom() * 0.8);
    }
  };

  const handleFit = () => {
    fitToScreen(cy);
  };

  const handleReset = () => {
    resetLayout(cy);
  };

  const selectionCount = selectedNodes.length + selectedEdges.length;

  return (
    <div className={styles.toolbar}>
      <Space>
        <Tooltip title="Zoom In">
          <Button icon={<ZoomInOutlined />} onClick={handleZoomIn} size="small" />
        </Tooltip>
        <Tooltip title="Zoom Out">
          <Button icon={<ZoomOutOutlined />} onClick={handleZoomOut} size="small" />
        </Tooltip>
        <span className={styles.zoomLevel}>{Math.round(zoom * 100)}%</span>
        <div className={styles.divider} />
        <Tooltip title="Fit to Screen">
          <Button icon={<ExpandOutlined />} onClick={handleFit} size="small">
            Fit
          </Button>
        </Tooltip>
        <Tooltip title="Reset Layout">
          <Button icon={<ReloadOutlined />} onClick={handleReset} size="small">
            Reset
          </Button>
        </Tooltip>
        <div className={styles.divider} />
        <Select
          value={layout}
          onChange={setLayout}
          options={getLayoutOptions()}
          size="small"
          style={{ width: 140 }}
        />
        {selectionCount > 0 && (
          <>
            <div className={styles.divider} />
            <Tooltip title="Clear Selection">
              <Button icon={<ClearOutlined />} onClick={clearSelection} size="small">
                Clear ({selectionCount})
              </Button>
            </Tooltip>
          </>
        )}
      </Space>
    </div>
  );
};

export default GraphToolbar;
