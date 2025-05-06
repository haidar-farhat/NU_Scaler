import { configureStore } from '@reduxjs/toolkit';
import authReducer from '../auth/authSlice';
import reviewsReducer from '../features/admin/reviewsSlice';
import bugReportsReducer from '../features/admin/bugReportsSlice';
import surveysReducer from '../features/admin/surveysSlice';
import userGrowthReducer from '../features/admin/userGrowthSlice';

const store = configureStore({
  reducer: {
    auth: authReducer,
    reviews: reviewsReducer,
    bugReports: bugReportsReducer,
    surveys: surveysReducer,
    userGrowth: userGrowthReducer,
  },
});

export default store; 