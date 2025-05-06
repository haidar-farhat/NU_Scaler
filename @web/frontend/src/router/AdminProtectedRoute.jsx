import React from 'react';
import { useSelector } from 'react-redux';
import { Navigate } from 'react-router-dom';

export default function AdminProtectedRoute({ children }) {
  const { user } = useSelector(state => state.auth);
  if (!user) {
    return <Navigate to="/login" replace />;
  }
  if (!user.is_admin) {
    return <div style={{ padding: 32, color: 'red', fontWeight: 'bold' }}>Access denied: Admins only.</div>;
  }
  return children;
} 