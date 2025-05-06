import React, { Suspense, lazy } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import Navbar from './components/Navbar';

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

// Simple loading spinner component (you can replace with a fancier one)
const LoadingSpinner = () => (
  <div className="min-h-screen flex items-center justify-center">
    <div className="animate-spin rounded-full h-16 w-16 border-t-2 border-b-2 border-indigo-600"></div>
  </div>
);

function App() {
  return (
    <BrowserRouter>
      <Navbar />
      {/* <Layout> You could wrap Routes in a Layout component */}
      <Suspense fallback={<LoadingSpinner />}>
        <Routes>
          {/* Public Routes */}
          <Route path="/" element={<LandingPage />} />
          <Route path="/login" element={<LoginPage />} />
          <Route path="/register" element={<RegisterPage />} />

          {/* Protected Routes */}
          <Route 
            path="/download" 
            element={
              <ProtectedRoute>
                <DownloadPage />
              </ProtectedRoute>
            }
          />
          <Route 
            path="/admin" 
            element={
              <ProtectedRoute role="admin">
                <AdminDashboard />
              </ProtectedRoute>
            }
          />
          
          {/* Add other routes here */}

          {/* Optional: 404 Not Found Route */}
          {/* <Route path="*" element={<NotFoundPage />} /> */}
        </Routes>
      </Suspense>
      {/* </Layout> */}
    </BrowserRouter>
  );
}

export default App;
