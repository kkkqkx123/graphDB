import React from 'react';
import { Layout, Menu } from 'antd';
import { useLocation, useNavigate } from 'react-router-dom';
import {
  ConsoleSqlOutlined,
  DatabaseOutlined,
  ApartmentOutlined,
  TableOutlined,
} from '@ant-design/icons';
import styles from './index.module.less';

const { Sider } = Layout;

const Sidebar: React.FC = () => {
  const location = useLocation();
  const navigate = useNavigate();

  const menuItems = [
    {
      key: '/console',
      icon: <ConsoleSqlOutlined />,
      label: 'Console',
      onClick: () => navigate('/console'),
    },
    {
      key: '/schema',
      icon: <DatabaseOutlined />,
      label: 'Schema',
      onClick: () => navigate('/schema'),
    },
    {
      key: '/graph',
      icon: <ApartmentOutlined />,
      label: 'Graph',
      onClick: () => navigate('/graph'),
    },
    {
      key: '/data-browser',
      icon: <TableOutlined />,
      label: 'Data Browser',
      onClick: () => navigate('/data-browser'),
    },
  ];

  const getSelectedKey = () => {
    const path = location.pathname;
    if (path.startsWith('/console')) return '/console';
    if (path.startsWith('/schema')) return '/schema';
    if (path.startsWith('/graph')) return '/graph';
    if (path.startsWith('/data-browser')) return '/data-browser';
    return path;
  };

  return (
    <Sider className={styles.sider} width={240} theme="light">
      <Menu
        mode="inline"
        selectedKeys={[getSelectedKey()]}
        items={menuItems}
        className={styles.menu}
      />
    </Sider>
  );
};

export default Sidebar;
