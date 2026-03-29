import React from 'react';
import { Layout, Button, Space, Badge, Dropdown, Divider } from 'antd';
import { DatabaseOutlined, LogoutOutlined, UserOutlined, DisconnectOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useConnectionStore } from '@/stores/connection';
import SpaceSelector from '@/components/business/SpaceSelector';
import styles from './index.module.less';

const { Header: AntHeader } = Layout;

const Header: React.FC = () => {
  const navigate = useNavigate();
  const { isConnected, connectionInfo, logout, isLoading } = useConnectionStore();

  const handleLogout = async () => {
    try {
      await logout();
      navigate('/login');
    } catch (error) {
      console.error('Logout error:', error);
    }
  };

  const menuItems = [
    {
      key: 'logout',
      label: 'Logout',
      icon: <LogoutOutlined />,
      onClick: handleLogout,
    },
  ];

  return (
    <AntHeader className={styles.header}>
      <div className={styles.headerLeft}>
        <DatabaseOutlined className={styles.logo} />
        <span className={styles.title}>GraphDB Studio</span>
        {isConnected && (
          <>
            <Divider type="vertical" className={styles.divider} />
            <SpaceSelector />
          </>
        )}
      </div>

      <div className={styles.headerRight}>
        <Space size="large">
          <Badge
            status={isConnected ? 'success' : 'error'}
            text={
              <span className={styles.statusText}>
                {isConnected ? 'Connected' : 'Disconnected'}
              </span>
            }
          />

          {isConnected && (
            <>
              <Space size="small" className={styles.connectionInfo}>
                <UserOutlined />
                <span>{connectionInfo.username}</span>
              </Space>

              <Dropdown menu={{ items: menuItems }} placement="bottomRight">
                <Button
                  type="text"
                  icon={<DisconnectOutlined />}
                  loading={isLoading}
                  className={styles.disconnectBtn}
                >
                  Logout
                </Button>
              </Dropdown>
            </>
          )}
        </Space>
      </div>
    </AntHeader>
  );
};

export default Header;
