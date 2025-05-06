import { createSlice, createAsyncThunk } from '@reduxjs/toolkit';
import axios from '../../api/axios';

export const fetchUsers = createAsyncThunk('adminUsers/fetchUsers', async (_, { rejectWithValue }) => {
  try {
    const res = await axios.get('/admin/users');
    console.log('Fetching users response:', res.data);
    return res.data;
  } catch (err) {
    console.error('Error fetching users:', err);
    return rejectWithValue(err.response?.data || err.message);
  }
});

export const updateUserRole = createAsyncThunk('adminUsers/updateUserRole', async ({ userId, is_admin }, { rejectWithValue }) => {
  try {
    const res = await axios.patch(`/admin/users/${userId}/role`, { is_admin });
    return res.data.user;
  } catch (err) {
    return rejectWithValue(err.response?.data || err.message);
  }
});

export const updateUserStatus = createAsyncThunk('adminUsers/updateUserStatus', async ({ userId, is_active }, { rejectWithValue }) => {
  try {
    const res = await axios.patch(`/admin/users/${userId}/status`, { is_active });
    return res.data.user;
  } catch (err) {
    return rejectWithValue(err.response?.data || err.message);
  }
});

const adminUsersSlice = createSlice({
  name: 'adminUsers',
  initialState: {
    users: [],
    loading: false,
    error: null,
  },
  reducers: {},
  extraReducers: builder => {
    builder
      .addCase(fetchUsers.pending, state => {
        state.loading = true;
        state.error = null;
      })
      .addCase(fetchUsers.fulfilled, (state, action) => {
        state.loading = false;
        if (action.payload && action.payload.data) {
          state.users = action.payload.data;
        } else {
          state.users = action.payload || [];
        }
      })
      .addCase(fetchUsers.rejected, (state, action) => {
        state.loading = false;
        state.error = action.payload;
      })
      .addCase(updateUserRole.fulfilled, (state, action) => {
        const idx = state.users.findIndex(u => u.id === action.payload.id);
        if (idx !== -1) state.users[idx] = action.payload;
      })
      .addCase(updateUserStatus.fulfilled, (state, action) => {
        const idx = state.users.findIndex(u => u.id === action.payload.id);
        if (idx !== -1) state.users[idx] = action.payload;
      });
  },
});

export default adminUsersSlice.reducer; 