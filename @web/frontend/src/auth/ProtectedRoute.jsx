import { Navigate, useLocation } from 'react-router-dom';
import { useSelector } from 'react-redux';

/**
 * ProtectedRoute component for guarding routes that require authentication
 * and optionally checking for admin role
 */
const ProtectedRoute = ({ children, role }) => {
  const { isAuthenticated, user } = useSelector((state) => state.auth);
  const location = useLocation();

  // Check if user is authenticated
  if (!isAuthenticated) {
    // Redirect to login page and save the location they were trying to access
    return <Navigate to="/login" state={{ from: location }} replace />;
  }

  // If a specific role is required, check for it
  if (role === 'admin' && (!user?.role || user.role !== 'admin')) {
    // Redirect to homepage if user doesn't have admin role
    return <Navigate to="/" replace />;
  }

  // If all checks pass, render the protected component
  return children;
};

export default ProtectedRoute; 