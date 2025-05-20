import axios from 'axios';

/**
 * Admin API utility for making authenticated requests to admin endpoints
 */
const adminApi = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || 'http://15.237.190.24:8000/api',
  headers: {
    'Content-Type': 'application/json',
    'Accept': 'application/json',
  }
});

adminApi.interceptors.request.use((config) => {
  const token = localStorage.getItem('token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

adminApi.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response && error.response.status === 401) {
      localStorage.removeItem('token');
      localStorage.removeItem('user');
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);

// Admin API endpoints
const adminApiService = {
  // Debug helpers
  checkAuthStatus: () => adminApi.get('/debug/auth'),
  
  // Admin session
  checkAdminSession: () => adminApi.get('/admin/session/check'),
  
  // Reviews
  getReviews: (params) => adminApi.get('/admin/reviews', { params }),
  exportReviews: (format = 'csv') => adminApi.get(`/admin/reviews/export?format=${format}`, { responseType: 'blob' }),
  
  // Bug Reports
  getBugReports: (params) => adminApi.get('/admin/bug-reports', { params }),
  exportBugReports: (format = 'csv') => adminApi.get(`/admin/bug-reports/export?format=${format}`, { responseType: 'blob' }),
  
  // Hardware Surveys
  getHardwareSurveys: (params) => adminApi.get('/admin/hardware-surveys', { params }),
  exportHardwareSurveys: (format = 'csv') => adminApi.get(`/admin/hardware-surveys/export?format=${format}`, { responseType: 'blob' }),
  
  // Metrics and Analytics
  getUserGrowth: () => adminApi.get('/admin/metrics/user-growth'),
  getFeedbackTrends: () => adminApi.get('/admin/metrics/feedback-trends'),
  getDashboardMetrics: () => adminApi.get('/admin/metrics/dashboard'),
  
  // User Management
  getUsers: (params) => adminApi.get('/admin/users', { params }),
  updateUserRole: (userId, isAdmin) => adminApi.patch(`/admin/users/${userId}/role`, { is_admin: isAdmin }),
  updateUserStatus: (userId, isActive) => adminApi.patch(`/admin/users/${userId}/status`, { is_active: isActive }),
};

export default adminApiService; 