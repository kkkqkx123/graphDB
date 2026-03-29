import React, { useEffect } from 'react';
import { Layout, Spin } from 'antd';
import Header from '../Header';
import Sidebar from '../Sidebar';
import { useHealthCheck } from '@/hooks/useHealthCheck';
import { useConnectionStore } from '@/stores/connection';
import styles from './index.module.less';

const { Content } = Layout;

const MainLayout: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { checkHealth, isConnected, connectionInfo, rememberMe, login } = useConnectionStore();
  const [isInitialCheck, setIsInitialCheck] = React.useState(true);

  useEffect(() => {
    const performInitialCheck = async () => {
      setIsInitialCheck(true);
      
      if (rememberMe && connectionInfo.username && connectionInfo.password) {
        try {
          await login(connectionInfo.username, connectionInfo.password, true);
        } catch (error) {
          console.error('Auto login failed:', error);
        }
      } else if (isConnected) {
        await checkHealth();
      }
      
      setIsInitialCheck(false);
    };

    performInitialCheck();
  }, [checkHealth, connectionInfo.password, connectionInfo.username, isConnected, login, rememberMe]);

  useHealthCheck(true);

  if (isInitialCheck) {
    return (
      <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100vh' }}>
        <Spin size="large" />
      </div>
    );
  }

  return (
    <Layout className={styles.layout}>
      <Header />
      <Layout className={styles.mainLayout}>
        <Sidebar />
        <Content className={styles.content}>{children}</Content>
      </Layout>
    </Layout>
  );
};

export default MainLayout;
