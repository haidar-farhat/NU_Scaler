import { createApi, fetchBaseQuery } from '@reduxjs/toolkit/query/react';

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

// Helper function for fetching admin data
export const fetchAdminData = async (endpoint, params = {}) => {
  const token = localStorage.getItem('token');
  const queryParams = new URLSearchParams(params).toString();
  const url = `http://localhost:8000/api/admin/${endpoint}${queryParams ? `?${queryParams}` : ''}`;
  
  console.log(`Fetching admin data from: ${url}`);
  console.log('With token:', token ? `${token.substring(0, 10)}...` : 'No token');
  
  try {
    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`
      },
      credentials: 'include'
    });
    
    if (!response.ok) {
      console.error(`Error fetching ${endpoint}:`, response.status, response.statusText);
      throw new Error(`Error fetching ${endpoint}: ${response.statusText}`);
    }
    
    const data = await response.json();
    console.log(`Data received for ${endpoint}:`, data);
    return data;
  } catch (error) {
    console.error(`Error in fetchAdminData for ${endpoint}:`, error);
    throw error;
  }
}; 