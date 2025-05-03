import { createSlice, createAsyncThunk } from '@reduxjs/toolkit';
import api from '../api/axios';

// Load initial state from localStorage if available
const getInitialState = () => {
  const token = localStorage.getItem('token');
  const user = localStorage.getItem('user');
  
  return {
    token: token || null,
    user: user ? JSON.parse(user) : null,
    isAuthenticated: !!token,
    loading: false,
    error: null,
  };
};

// Async thunks for auth operations
export const login = createAsyncThunk(
  'auth/login',
  async (credentials, { rejectWithValue }) => {
    try {
      const response = await api.post('/v1/login', credentials);
      const { token, user } = response.data;
      
      // Store in localStorage
      localStorage.setItem('token', token);
      localStorage.setItem('user', JSON.stringify(user));
      
      return { token, user };
    } catch (error) {
      return rejectWithValue(error.response?.data?.message || 'Login failed');
    }
  }
);

export const register = createAsyncThunk(
  'auth/register',
  async (userData, { rejectWithValue }) => {
    try {
      const response = await api.post('/v1/auth/register', userData);
      const { token, user } = response.data;
      
      // Store in localStorage
      localStorage.setItem('token', token);
      localStorage.setItem('user', JSON.stringify(user));
      
      return { token, user };
    } catch (error) {
      return rejectWithValue(error.response?.data?.message || 'Registration failed');
    }
  }
);

export const logout = createAsyncThunk(
  'auth/logout',
  async (_, { rejectWithValue }) => {
    try {
      await api.post('/v1/logout');
      
      // Clear localStorage
      localStorage.removeItem('token');
      localStorage.removeItem('user');
      
      return null;
    } catch (error) {
      // Still remove items from localStorage even if API call fails
      localStorage.removeItem('token');
      localStorage.removeItem('user');
      
      return rejectWithValue(error.response?.data?.message || 'Logout failed');
    }
  }
);

// Auth slice
const authSlice = createSlice({
  name: 'auth',
  initialState: getInitialState(),
  reducers: {
    clearError: (state) => {
      state.error = null;
    },
  },
  extraReducers: (builder) => {
    builder
      // Login reducers
      .addCase(login.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(login.fulfilled, (state, action) => {
        state.loading = false;
        state.token = action.payload.token;
        state.user = action.payload.user;
        state.isAuthenticated = true;
      })
      .addCase(login.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload;
      })
      
      // Register reducers
      .addCase(register.pending, (state) => {
        state.loading = true;
        state.error = null;
      })
      .addCase(register.fulfilled, (state, action) => {
        state.loading = false;
        state.token = action.payload.token;
        state.user = action.payload.user;
        state.isAuthenticated = true;
      })
      .addCase(register.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload;
      })
      
      // Logout reducers
      .addCase(logout.pending, (state) => {
        state.loading = true;
      })
      .addCase(logout.fulfilled, (state) => {
        state.loading = false;
        state.token = null;
        state.user = null;
        state.isAuthenticated = false;
      })
      .addCase(logout.rejected, (state) => {
        state.loading = false;
        state.token = null;
        state.user = null;
        state.isAuthenticated = false;
      });
  },
});

export const { clearError } = authSlice.actions;
export default authSlice.reducer; 