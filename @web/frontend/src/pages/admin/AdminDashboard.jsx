import React, { useEffect } from 'react';
import { useSelector, useDispatch } from 'react-redux';
import SummaryCards from './SummaryCards';
import ReviewsTable from './ReviewsTable';
import BugReportsTable from './BugReportsTable';
import SurveysChart from './SurveysChart';
import UserGrowthChart from './UserGrowthChart';
import { fetchReviews } from '../../features/admin/reviewsSlice';
import { fetchBugReports } from '../../features/admin/bugReportsSlice';
import { fetchSurveys } from '../../features/admin/surveysSlice';
import { fetchUserGrowth } from '../../features/admin/userGrowthSlice';
import { Link } from 'react-router-dom';

const AdminDashboard = () => {
  const dispatch = useDispatch();
  const { list: reviews, loading: reviewsLoading, error: reviewsError } = useSelector(state => state.reviews);
  const { list: bugReports, loading: bugReportsLoading, error: bugReportsError } = useSelector(state => state.bugReports);
  const { list: surveys, loading: surveysLoading, error: surveysError } = useSelector(state => state.surveys);
  const { list: userGrowth, loading: userGrowthLoading, error: userGrowthError } = useSelector(state => state.userGrowth);

  useEffect(() => {
    dispatch(fetchReviews());
    dispatch(fetchBugReports());
    dispatch(fetchSurveys());
    dispatch(fetchUserGrowth());
  }, [dispatch]);

  return (
    <div className="p-6">
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
        <SummaryCards title="Total Reviews" value={reviews.length} icon="⭐" />
        <SummaryCards title="Bug Reports" value={bugReports.length} icon="🐞" />
        <SummaryCards title="Surveys" value={surveys.length} icon="🖥️" />
        <SummaryCards title="New Users" value={userGrowth.length} icon="👤" />
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {reviewsLoading ? <div>Loading reviews...</div> : reviewsError ? <div className="text-red-600">{reviewsError}</div> : <ReviewsTable reviews={reviews} />}
        {bugReportsLoading ? <div>Loading bug reports...</div> : bugReportsError ? <div className="text-red-600">{bugReportsError}</div> : <BugReportsTable bugReports={bugReports} />}
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mt-6">
        {surveysLoading ? <div>Loading surveys...</div> : surveysError ? <div className="text-red-600">{surveysError}</div> : <SurveysChart data={surveys} />}
        {userGrowthLoading ? <div>Loading user growth...</div> : userGrowthError ? <div className="text-red-600">{userGrowthError}</div> : <UserGrowthChart data={userGrowth} />}
      </div>
      <div style={{ marginBottom: 24 }}>
        <Link to="/admin/users" style={{ fontWeight: 'bold', color: '#007bff', textDecoration: 'none' }}>
          Manage Users
        </Link>
      </div>
    </div>
  );
};

export default AdminDashboard; 