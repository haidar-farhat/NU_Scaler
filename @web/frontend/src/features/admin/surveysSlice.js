import { createSlice, createAsyncThunk } from '@reduxjs/toolkit';
import api from '../../api/axios';

export const fetchSurveys = createAsyncThunk('surveys/fetch', async (params = {}) => {
  const res = await api.get('/admin/hardware-surveys', { params });
  return res.data;
});

const surveysSlice = createSlice({
  name: 'surveys',
  initialState: { list: [], meta: {}, loading: false, error: null },
  reducers: {},
  extraReducers: (builder) => {
    builder
      .addCase(fetchSurveys.pending, (state) => { state.loading = true; })
      .addCase(fetchSurveys.fulfilled, (state, action) => {
        state.loading = false;
        state.list = action.payload.data || action.payload;
        state.meta = action.payload.meta || {};
      })
      .addCase(fetchSurveys.rejected, (state, action) => {
        state.loading = false;
        state.error = action.error.message;
      });
  },
});
export default surveysSlice.reducer; 