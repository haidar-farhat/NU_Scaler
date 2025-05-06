import { configureStore } from '@reduxjs/toolkit';
import authReducer from '../auth/authSlice';
import reviewsReducer from '../features/admin/reviewsSlice';
import bugReportsReducer from '../features/admin/bugReportsSlice';
import surveysReducer from '../features/admin/surveysSlice';
import userGrowthReducer from '../features/admin/userGrowthSlice';
import adminUsersReducer from '../features/admin/usersSlice';

const store = configureStore({
  reducer: {
    auth: authReducer,
    reviews: reviewsReducer,
    bugReports: bugReportsReducer,
    surveys: surveysReducer,
    userGrowth: userGrowthReducer,
    adminUsers: adminUsersReducer,
  },
});

export default store; 