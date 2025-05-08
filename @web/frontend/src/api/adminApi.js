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
    const csrfResponse = await axios.get('/sanctum/csrf-cookie', {
      baseURL: import.meta.env.VITE_APP_URL || 'http://localhost:8000',
      withCredentials: true
    });
    
    console.log('CSRF cookie obtained', csrfResponse.headers);
    
    // Check if we have the cookie in document.cookie
    const hasCsrfCookie = document.cookie.includes('XSRF-TOKEN');
    console.log('CSRF cookie exists in document.cookie:', hasCsrfCookie);
    
    return true;
  } catch (error) {
    console.error('Failed to get CSRF cookie:', error);
    return false;
  }
}

// Initialize by getting CSRF token
setupCSRF().then(success => {
  console.log('Initial CSRF setup:', success ? 'success' : 'failed');
});

// Add request interceptor to attach the auth token
adminApi.interceptors.request.use(async (config) => {
  // Get the token from localStorage
  const token = localStorage.getItem('token');
  
  // Log request details for debugging
  console.log(`Admin API Request: ${config.method.toUpperCase()} ${config.url}`);
  
  // If we have a token, add it to the Authorization header
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
    console.log('Using auth token:', token.substring(0, 15) + '...');
  } else {
    console.warn('No auth token available for request');
  }
  
  // Get the XSRF token from the cookie if available
  const xsrfToken = document.cookie
    .split('; ')
    .find(row => row.startsWith('XSRF-TOKEN='))
    ?.split('=')[1];
    
  if (xsrfToken) {
    // Decode the token (Laravel stores it URL encoded)
    config.headers['X-XSRF-TOKEN'] = decodeURIComponent(xsrfToken);
    console.log('Using XSRF token:', xsrfToken.substring(0, 15) + '...');
  } else {
    console.warn('No XSRF token found in cookies, fetching new one...');
    // If we don't have a CSRF token, try to get one
    await setupCSRF();
  }
  
  return config;
});

// Add response interceptor for error handling
adminApi.interceptors.response.use(
  (response) => {
    console.log(`Admin API Response: ${response.config.method.toUpperCase()} ${response.config.url} - Status ${response.status}`);
    return response;
  },
  async (error) => {
    console.error('Admin API Error:', error.response?.status, error.response?.data || error.message);
    
    const originalRequest = error.config;
    
    // If error is 401 (Unauthorized) or 419 (CSRF token mismatch)
    if ((error.response?.status === 401 || error.response?.status === 419) && !originalRequest._retry) {
      originalRequest._retry = true;
      
      try {
        console.log('Auth error detected, refreshing CSRF token and retrying...');
        // Try to refresh CSRF token
        const csrfSuccess = await setupCSRF();
        
        if (csrfSuccess) {
          // Check if token exists in localStorage
          const token = localStorage.getItem('token');
          if (!token) {
            console.warn('No token found in localStorage during retry');
            throw new Error('Authentication token not found');
          }
          
          // Retry the original request with new token
          console.log('Retrying request with refreshed CSRF token');
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
    
    return Promise.reject(error);
  }
);

// Admin API endpoints
const adminApiService = {
  // Debug helpers
  ensureCSRF: setupCSRF,
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