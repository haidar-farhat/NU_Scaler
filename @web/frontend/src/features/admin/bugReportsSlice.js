import { createSlice, createAsyncThunk } from '@reduxjs/toolkit';
import { fetchAdminData } from './baseAdminSlice';

export const fetchBugReports = createAsyncThunk('bugReports/fetch', async (params = {}) => {
  return await fetchAdminData('bug-reports', params);
});

const bugReportsSlice = createSlice({
  name: 'bugReports',
  initialState: { list: [], meta: {}, loading: false, error: null },
  reducers: {},
  extraReducers: (builder) => {
    builder
      .addCase(fetchBugReports.pending, (state) => { state.loading = true; })
      .addCase(fetchBugReports.fulfilled, (state, action) => {
        state.loading = false;
        state.list = action.payload.data || action.payload;
        state.meta = action.payload.meta || {};
      })
      .addCase(fetchBugReports.rejected, (state, action) => {
        state.loading = false;
        state.error = action.error.message;
      });
  },
});
export default bugReportsSlice.reducer; 