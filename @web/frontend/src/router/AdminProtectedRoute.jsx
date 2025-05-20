import React, { useEffect, useState } from 'react';
import { useSelector } from 'react-redux';
import { Navigate } from 'react-router-dom';

export default function AdminProtectedRoute({ children }) {
  const auth = useSelector(state => state.auth);
  const { user, token, isAuthenticated } = auth;
  const [error, setError] = useState(null);
  
  useEffect(() => {
    // Validate token format
    const storedToken = localStorage.getItem('token');
    if (!storedToken) {
      setError('No token found in localStorage. Please log in again.');
    } else if (storedToken.length < 20) {
      setError(`Token found but appears invalid (length: ${storedToken.length}). Please log in again.`);
    }
    
    if (!user) {
      setError('No user data found. Please log in again.');
    } else if (!user.is_admin) {
      setError(`User ${user.name} (${user.email}) does not have admin privileges.`);
    } else {
      setError(null);
    }
  }, [auth, user, token, isAuthenticated]);

  // Not authenticated - redirect to login
  if (!user) {
    return <Navigate to="/login" replace />;
  }
  
  // Not an admin - show access denied
  if (!user.is_admin) {
    return (
      <div className="p-8 bg-red-50 text-center">
        <div className="text-red-600 text-3xl font-bold mb-4">Access Denied</div>
        <div className="text-red-800 mb-4">Your account does not have administrator privileges.</div>
        <div className="bg-white p-4 rounded shadow-md text-left mb-6">
          <div><strong>Username:</strong> {user.name}</div>
          <div><strong>Email:</strong> {user.email}</div>
          <div><strong>Admin Status:</strong> {user.is_admin ? 'Yes' : 'No'}</div>
        </div>
        <button
          onClick={() => window.location.href = '/'}
          className="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600"
        >
          Return to Homepage
        </button>
      </div>
    );
  }
  
  // Show any authentication errors if present
  if (error) {
    return (
      <div className="p-8 bg-yellow-50 text-center">
        <div className="text-yellow-600 text-2xl font-bold mb-4">Authentication Warning</div>
        <div className="bg-white p-4 rounded shadow-md text-left mb-6 text-red-600">{error}</div>
        <div className="text-gray-700 mb-4">
          You appear to be logged in, but there may be issues with your authentication.
          <br />Try logging out and logging back in.
        </div>
        <div className="space-x-4">
          <button
            onClick={() => window.location.href = '/login'}
            className="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600"
          >
            Go to Login
          </button>
          <button
            onClick={() => {
              localStorage.removeItem('token');
              localStorage.removeItem('user');
              window.location.href = '/login';
            }}
            className="bg-red-500 text-white px-4 py-2 rounded hover:bg-red-600"
          >
            Clear Session & Logout
          </button>
        </div>
      </div>
    );
  }
  
  // All good - render the protected content
  return children;
} 