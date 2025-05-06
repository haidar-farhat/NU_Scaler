import { createSlice, createAsyncThunk } from '@reduxjs/toolkit';
import { fetchAdminData } from './baseAdminSlice';

export const fetchReviews = createAsyncThunk('reviews/fetch', async (params = {}) => {
  return await fetchAdminData('reviews', params);
});

const reviewsSlice = createSlice({
  name: 'reviews',
  initialState: { list: [], meta: {}, loading: false, error: null },
  reducers: {},
  extraReducers: (builder) => {
    builder
      .addCase(fetchReviews.pending, (state) => { state.loading = true; })
      .addCase(fetchReviews.fulfilled, (state, action) => {
        state.loading = false;
        state.list = action.payload.data || action.payload;
        state.meta = action.payload.meta || {};
      })
      .addCase(fetchReviews.rejected, (state, action) => {
        state.loading = false;
        state.error = action.error.message;
      });
  },
});
export default reviewsSlice.reducer; 