import { lazy, Suspense } from 'react';
import { createBrowserRouter, Navigate, Outlet } from 'react-router-dom';
import MainLayout from '@/components/layout/MainLayout';
import ProtectedRoute from '@/components/layout/ProtectedRoute';

const Login = lazy(() => import('@/pages/Login'));
const MainPage = lazy(() => import('@/pages/MainPage'));
const Console = lazy(() => import('@/pages/Console'));
const Schema = lazy(() => import('@/pages/Schema'));
const Graph = lazy(() => import('@/pages/Graph'));
const DataBrowser = lazy(() => import('@/pages/DataBrowser'));

const LoadingFallback = () => (
  <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100vh' }}>
    Loading...
  </div>
);

const router = createBrowserRouter([
  {
    path: '/login',
    element: (
      <Suspense fallback={<LoadingFallback />}>
        <Login />
      </Suspense>
    ),
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
        element: (
          <Suspense fallback={<LoadingFallback />}>
            <MainPage />
          </Suspense>
        ),
      },
      {
        path: 'console',
        element: (
          <Suspense fallback={<LoadingFallback />}>
            <Console />
          </Suspense>
        ),
      },
      {
        path: 'schema',
        element: (
          <Suspense fallback={<LoadingFallback />}>
            <Schema />
          </Suspense>
        ),
      },
      {
        path: 'graph',
        element: (
          <Suspense fallback={<LoadingFallback />}>
            <Graph />
          </Suspense>
        ),
      },
      {
        path: 'data-browser',
        element: (
          <Suspense fallback={<LoadingFallback />}>
            <DataBrowser />
          </Suspense>
        ),
      },
    ],
  },
  {
    path: '*',
    element: <Navigate to="/" replace />,
  },
]);

export default router;
