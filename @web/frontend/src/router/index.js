import AdminUsersPage from '../pages/admin/AdminUsersPage';
import AdminProtectedRoute from './AdminProtectedRoute';
import UserProtectedRoute from './ProtectedRoute';

const routes = [
  {
    path: '/admin/users',
    element: <AdminProtectedRoute><AdminUsersPage /></AdminProtectedRoute>,
  },
  // Example for user-protected route:
  // {
  //   path: '/dashboard',
  //   element: <UserProtectedRoute><DashboardPage /></UserProtectedRoute>,
  // },
]; 