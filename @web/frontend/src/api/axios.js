import axios from 'axios';

// Ensure we're using the correct server URL
const SERVER_URL = 'http://15.237.190.24:8000';
const API_BASE_URL = `${SERVER_URL}/api`;
const SANCTUM_URL = `${SERVER_URL}/sanctum`;

// Create axios instance with default config
const api = axios.create({
  baseURL: API_BASE_URL,
  withCredentials: true,
  headers: {
    'Content-Type': 'application/json',
    'Accept': 'application/json',
    'X-Requested-With': 'XMLHttpRequest',
  },
});

// Function to get CSRF token
const getCsrfToken = async () => {
  try {
    const response = await axios.get(`${SANCTUM_URL}/csrf-cookie`, {
      withCredentials: true,
      headers: {
        'Accept': 'application/json',
        'X-Requested-With': 'XMLHttpRequest',
      },
    });
    return response;
  } catch (error) {
    console.error('CSRF token fetch failed:', error);
    throw error;
  }
};

// Add a request interceptor
api.interceptors.request.use(async (config) => {
  const token = localStorage.getItem('token');
  console.log('API Request:', config.url, 'Token exists:', !!token);

  // Always get CSRF token before any request
  try {
    await getCsrfToken();
  } catch (error) {
    console.warn('Failed to get CSRF token:', error);
  }

  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Add a response interceptor
api.interceptors.response.use(
  (response) => {
    console.log('API Response:', response.config.url, 'Status:', response.status);
    return response;
  },
  (error) => {
    console.error('API Error:', error.config?.url, 'Status:', error.response?.status, 'Message:', error.message);
    
    if (error.response && error.response.status === 401) {
      console.warn('Unauthorized request - clearing token');
      localStorage.removeItem('token');
    }
    return Promise.reject(error);
  }
);

export default api; 