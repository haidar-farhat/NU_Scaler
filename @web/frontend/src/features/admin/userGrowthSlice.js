import { createSlice, createAsyncThunk } from '@reduxjs/toolkit';
import api from '../../api/axios';

export const fetchUserGrowth = createAsyncThunk('userGrowth/fetch', async () => {
  const res = await api.get('/admin/metrics/user-growth');
  return res.data;
});

const userGrowthSlice = createSlice({
  name: 'userGrowth',
  initialState: { list: [], loading: false, error: null },
  reducers: {},
  extraReducers: (builder) => {
    builder
      .addCase(fetchUserGrowth.pending, (state) => { state.loading = true; })
      .addCase(fetchUserGrowth.fulfilled, (state, action) => {
        state.loading = false;
        state.list = action.payload;
      })
      .addCase(fetchUserGrowth.rejected, (state, action) => {
        state.loading = false;
        state.error = action.error.message;
      });
  },
});
export default userGrowthSlice.reducer; 