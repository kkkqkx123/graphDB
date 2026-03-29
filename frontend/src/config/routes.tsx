import { createBrowserRouter, Navigate, Outlet } from 'react-router-dom';
import Login from '@/pages/Login';
import MainPage from '@/pages/MainPage';
import Console from '@/pages/Console';
import Schema from '@/pages/Schema';
import Graph from '@/pages/Graph';
import DataBrowser from '@/pages/DataBrowser';
import MainLayout from '@/components/layout/MainLayout';
import ProtectedRoute from '@/components/layout/ProtectedRoute';

const router = createBrowserRouter([
  {
    path: '/login',
    element: <Login />,
  },
  {
    path: '/',
    element: (
      <ProtectedRoute>
        <MainLayout>
          <Outlet />
        </MainLayout>
      </ProtectedRoute>
    ),
    children: [
      {
        index: true,
        element: <MainPage />,
      },
      {
        path: 'console',
        element: <Console />,
      },
      {
        path: 'schema',
        element: <Schema />,
      },
      {
        path: 'graph',
        element: <Graph />,
      },
      {
        path: 'data-browser',
        element: <DataBrowser />,
      },
    ],
  },
  {
    path: '*',
    element: <Navigate to="/" replace />,
  },
]);

export default router;
