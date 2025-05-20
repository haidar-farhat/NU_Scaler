import React, { Suspense, lazy } from 'react';
import Navbar from '../components/Navbar';
import LandingPage from '../pages/LandingPage';
import LoginPage from '../auth/LoginPage';
import RegisterPage from '../auth/RegisterPage';
import DownloadPage from '../pages/DownloadPage';
import ProtectedRoute from '../auth/ProtectedRoute';

const AdminDashboard = lazy(() => import('../pages/admin/AdminDashboard'));
const AdminUsersPage = lazy(() => import('../pages/admin/AdminUsersPage'));

const LoadingSpinner = () => (
  <div className="min-h-screen flex items-center justify-center">
    <div className="animate-spin rounded-full h-16 w-16 border-t-2 border-b-2 border-indigo-600"></div>
  </div>
);

const routes = [
  {
    path: "/",
    element: (
      <>
        <Navbar />
        <Suspense fallback={<LoadingSpinner />}>
          <LandingPage />
        </Suspense>
      </>
    )
  },
  {
    path: "/login",
    element: (
      <>
        <Navbar />
        <Suspense fallback={<LoadingSpinner />}>
          <LoginPage />
        </Suspense>
      </>
    )
  },
  {
    path: "/register",
    element: (
      <>
        <Navbar />
        <Suspense fallback={<LoadingSpinner />}>
          <RegisterPage />
        </Suspense>
      </>
    )
  },
  {
    path: "/download",
    element: (
      <>
        <Navbar />
        <Suspense fallback={<LoadingSpinner />}>
          <DownloadPage />
        </Suspense>
      </>
    )
  },
  {
    path: "/admin",
    element: (
      <>
        <Navbar />
        <Suspense fallback={<LoadingSpinner />}>
          <ProtectedRoute role="admin">
            <AdminDashboard />
          </ProtectedRoute>
        </Suspense>
      </>
    )
  },
  {
    path: "/admin/users",
    element: (
      <>
        <Navbar />
        <Suspense fallback={<LoadingSpinner />}>
          <ProtectedRoute role="admin">
            <AdminUsersPage />
          </ProtectedRoute>
        </Suspense>
      </>
    )
  }
];

export default routes; 