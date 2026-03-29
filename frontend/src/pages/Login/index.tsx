import React, { useEffect, useCallback } from 'react';
import { Form, Input, Button, Card, Checkbox, Spin, message } from 'antd';
import { useNavigate } from 'react-router-dom';
import { useConnectionStore } from '@/stores/connection';
import styles from './index.module.less';

const Login: React.FC = () => {
  const navigate = useNavigate();
  const { login, isConnected, isLoading, error, clearError, loadSavedConnection } = useConnectionStore();
  const [form] = Form.useForm();

  useEffect(() => {
    loadSavedConnection();
    
    const savedConnection = localStorage.getItem('graphdb_connection');
    if (savedConnection) {
      try {
        const connectionInfo = JSON.parse(savedConnection);
        form.setFieldsValue({
          username: connectionInfo.username,
          password: connectionInfo.password || '',
          rememberMe: true,
        });
      } catch (e) {
        console.error('Failed to parse saved connection', e);
      }
    }
  }, [form, loadSavedConnection]);

  useEffect(() => {
    if (isConnected) {
      navigate('/');
    }
  }, [isConnected, navigate]);

  useEffect(() => {
    if (error) {
      message.error(error);
      clearError();
    }
  }, [error, clearError]);

  const handleSubmit = async (values: {
    username: string;
    password: string;
    rememberMe: boolean;
  }) => {
    const { username, password, rememberMe } = values;
    
    try {
      await login(username, password, rememberMe);
      message.success('Logged in successfully');
      navigate('/');
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Login failed';
      message.error(errorMessage);
    }
  };

  const handleRememberMeChange = useCallback((checked: boolean) => {
    console.log('Remember me:', checked);
  }, []);

  return (
    <div className={styles.loginPage}>
      <Card className={styles.loginCard} title="GraphDB Studio">
        <Spin spinning={isLoading}>
          <Form
            form={form}
            name="login"
            onFinish={handleSubmit}
            layout="vertical"
            initialValues={{
              username: 'root',
              rememberMe: false,
            }}
          >
            <Form.Item
              name="username"
              label="Username"
              rules={[{ required: true, message: 'Please enter username' }]}
            >
              <Input placeholder="Enter username" />
            </Form.Item>

            <Form.Item
              name="password"
              label="Password"
              rules={[{ required: true, message: 'Please enter password' }]}
            >
              <Input.Password placeholder="Enter password" />
            </Form.Item>

            <Form.Item name="rememberMe" valuePropName="checked">
              <Checkbox onChange={(e) => handleRememberMeChange(e.target.checked)}>
                Remember me
              </Checkbox>
            </Form.Item>

            <Form.Item>
              <Button type="primary" htmlType="submit" block loading={isLoading}>
                Login
              </Button>
            </Form.Item>
          </Form>
        </Spin>
      </Card>
    </div>
  );
};

export default Login;
