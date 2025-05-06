import React, { useEffect } from 'react';
import { useSelector } from 'react-redux';
import { Navigate } from 'react-router-dom';

export default function AdminProtectedRoute({ children }) {
  const auth = useSelector(state => state.auth);
  const { user } = auth;
  
  useEffect(() => {
    console.log('AdminProtectedRoute - Auth State:', auth);
    console.log('User from auth state:', user);
    console.log('Token exists:', !!localStorage.getItem('token'));
    
    if (!user) {
      console.log('No user in state - redirecting to login');
    } else if (!user.is_admin) {
      console.log('User is not admin - access denied');
    } else {
      console.log('User is authenticated and has admin privileges');
    }
  }, [auth, user]);

  if (!user) {
    return <Navigate to="/login" replace />;
  }
  
  if (!user.is_admin) {
    return <div style={{ padding: 32, color: 'red', fontWeight: 'bold' }}>Access denied: Admins only.</div>;
  }
  
  return children;
} 