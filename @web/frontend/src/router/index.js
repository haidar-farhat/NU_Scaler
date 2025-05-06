import AdminUsersPage from '../pages/admin/AdminUsersPage';

const routes = [
  {
    path: '/admin/users',
    element: <ProtectedRoute adminOnly><AdminUsersPage /></ProtectedRoute>,
  },
]; 