import { createApi, fetchBaseQuery } from '@reduxjs/toolkit/query/react';
import adminApiService from '../../api/adminApi';

// Base query with auth
export const baseQuery = fetchBaseQuery({
  baseURL: 'http://localhost:8000/api',
  credentials: 'include',
  prepareHeaders: (headers) => {
    const token = localStorage.getItem('token');
    console.log('API Request Header - Token Exists:', !!token);
    if (token) {
      headers.set('Authorization', `Bearer ${token}`);
    }
    headers.set('Accept', 'application/json');
    return headers;
  },
});

// Helper function for fetching admin data using our specialized admin API service
export const fetchAdminData = async (endpoint, params = {}) => {
  console.log(`Fetching admin data from endpoint: ${endpoint}`, params);
  
  try {
    let response;
    
    switch (endpoint) {
      case 'reviews':
        response = await adminApiService.getReviews(params);
        break;
      case 'bug-reports':
        response = await adminApiService.getBugReports(params);
        break;
      case 'hardware-surveys':
        response = await adminApiService.getHardwareSurveys(params);
        break;
      case 'metrics/user-growth':
        response = await adminApiService.getUserGrowth();
        break;
      case 'users':
        response = await adminApiService.getUsers(params);
        break;
      default:
        // For any other endpoints, use a direct API call through our service
        response = await adminApiService.get(`/admin/${endpoint}`, { params });
    }
    
    console.log(`Data received for ${endpoint}:`, response.data);
    return response.data;
  } catch (error) {
    console.error(`Error in fetchAdminData for ${endpoint}:`, error);
    throw new Error(`Error fetching ${endpoint}: ${error.response?.data?.message || error.message}`);
  }
}; 