import axios from 'axios';

/**
 * Admin API utility for making authenticated requests to admin endpoints
 */
const adminApi = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || 'http://localhost:8000/api',
  withCredentials: true, // Critical for Sanctum authentication
  headers: {
    'Content-Type': 'application/json',
    'Accept': 'application/json',
  }
});

/**
 * Get CSRF cookie and set up token for admin requests
 */
async function setupCSRF() {
  try {
    // Get CSRF cookie from Laravel Sanctum endpoint
    await axios.get('/sanctum/csrf-cookie', {
      baseURL: import.meta.env.VITE_APP_URL || 'http://localhost:8000',
      withCredentials: true
    });
    
    console.log('CSRF cookie obtained');
    return true;
  } catch (error) {
    console.error('Failed to get CSRF cookie:', error);
    return false;
  }
}

// Add request interceptor to attach the auth token
adminApi.interceptors.request.use(async (config) => {
  // Get the token from localStorage
  const token = localStorage.getItem('token');
  
  // If we have a token, add it to the Authorization header
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  
  // If this is a mutation (not GET), ensure we have CSRF protection
  if (!['get', 'head', 'options'].includes(config.method.toLowerCase())) {
    // Get the XSRF token from the cookie if available
    const xsrfToken = document.cookie
      .split('; ')
      .find(row => row.startsWith('XSRF-TOKEN='))
      ?.split('=')[1];
      
    if (xsrfToken) {
      // Decode the token (Laravel stores it URL encoded)
      config.headers['X-XSRF-TOKEN'] = decodeURIComponent(xsrfToken);
    } else {
      // If we don't have a CSRF token, try to get one
      await setupCSRF();
    }
  }
  
  console.log(`Admin API Request: ${config.method.toUpperCase()} ${config.url}`);
  return config;
});

// Add response interceptor for error handling
adminApi.interceptors.response.use(
  (response) => {
    return response;
  },
  async (error) => {
    const originalRequest = error.config;
    
    // If error is 401 (Unauthorized) or 419 (CSRF token mismatch)
    if ((error.response?.status === 401 || error.response?.status === 419) && !originalRequest._retry) {
      originalRequest._retry = true;
      
      try {
        // Try to refresh CSRF token
        const csrfSuccess = await setupCSRF();
        
        if (csrfSuccess) {
          // Retry the original request with new token
          return adminApi(originalRequest);
        }
      } catch (retryError) {
        console.error('Admin authentication refresh failed:', retryError);
      }
      
      // If we get here, redirect to login
      console.warn('Admin authentication failed. Redirecting to login.');
      localStorage.removeItem('token');
      localStorage.removeItem('user');
      window.location.href = '/login';
    }
    
    console.error('Admin API Error:', error.response?.status, error.response?.data || error.message);
    return Promise.reject(error);
  }
);

// Admin API endpoints
const adminApiService = {
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
  
  // Authentication check
  checkAdminAuth: () => adminApi.get('/debug/auth'),
};

export default adminApiService; 