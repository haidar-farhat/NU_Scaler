import { createSlice, createAsyncThunk } from '@reduxjs/toolkit';
import adminApi from '../../api/adminApi';

export const fetchUsers = createAsyncThunk('adminUsers/fetchUsers', async (params, { rejectWithValue }) => {
  try {
    const res = await adminApi.getUsers(params);
    const { data } = res.data;
    return {
      users: data.data,
      meta: { ...data, data: undefined },
    };
  } catch (err) {
    return rejectWithValue(err.response?.data || err.message);
  }
});

export const updateUserRole = createAsyncThunk('adminUsers/updateUserRole', async ({ userId, is_admin }, { rejectWithValue }) => {
  try {
    const res = await adminApi.updateUserRole(userId, is_admin);
    return res.data.user;
  } catch (err) {
    return rejectWithValue(err.response?.data || err.message);
  }
});

export const updateUserStatus = createAsyncThunk('adminUsers/updateUserStatus', async ({ userId, is_active }, { rejectWithValue }) => {
  try {
    const res = await adminApi.updateUserStatus(userId, is_active);
    return res.data.user;
  } catch (err) {
    return rejectWithValue(err.response?.data || err.message);
  }
});

const adminUsersSlice = createSlice({
  name: 'adminUsers',
  initialState: {
    users: [],
    meta: null,
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
        state.users = action.payload.users || [];
        state.meta = action.payload.meta || null;
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