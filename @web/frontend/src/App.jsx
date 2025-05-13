import React, { Suspense, lazy } from 'react';
import { BrowserRouter, Routes, Route, createBrowserRouter, RouterProvider } from 'react-router-dom';
import Navbar from './components/Navbar';
import { ToastProvider } from './components/ToastContext';

// Layout component (optional, for shared structure like header/footer)
// import Layout from './components/Layout'; 

// Eagerly load common pages
import LandingPage from './pages/LandingPage';
import LoginPage from './auth/LoginPage';
import RegisterPage from './auth/RegisterPage';
import DownloadPage from './pages/DownloadPage';
import ProtectedRoute from './auth/ProtectedRoute';

// Lazy load less common or heavier pages (like admin)
const AdminDashboard = lazy(() => import('./pages/admin/AdminDashboard'));
const AdminUsersPage = lazy(() => import('./pages/admin/AdminUsersPage'));

// Simple loading spinner component (you can replace with a fancier one)
const LoadingSpinner = () => (
  <div className="min-h-screen flex items-center justify-center">
    <div className="animate-spin rounded-full h-16 w-16 border-t-2 border-b-2 border-indigo-600"></div>
  </div>
);

// Use future flags to prevent warnings
const router = createBrowserRouter([
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
          <ProtectedRoute>
            <DownloadPage />
          </ProtectedRoute>
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
], {
  // Use future flags to prevent warnings
  future: {
    v7_startTransition: true,
    v7_relativeSplatPath: true
  }
});

function App() {
  return (
    <ToastProvider>
      <RouterProvider router={router} />
    </ToastProvider>
  );
}

export default App;
