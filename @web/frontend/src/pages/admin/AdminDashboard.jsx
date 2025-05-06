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
import { useToast } from '../../components/ToastContext';

const exportBtnStyle = {
  marginRight: 8,
  marginBottom: 8,
  padding: '6px 16px',
  borderRadius: 4,
  border: 'none',
  background: '#007bff',
  color: '#fff',
  fontWeight: 500,
  cursor: 'pointer',
};

const AdminDashboard = () => {
  const dispatch = useDispatch();
  const { list: reviews, loading: reviewsLoading, error: reviewsError } = useSelector(state => state.reviews);
  const { list: bugReports, loading: bugReportsLoading, error: bugReportsError } = useSelector(state => state.bugReports);
  const { list: surveys, loading: surveysLoading, error: surveysError } = useSelector(state => state.surveys);
  const { list: userGrowth, loading: userGrowthLoading, error: userGrowthError } = useSelector(state => state.userGrowth);
  const { showToast } = useToast();

  useEffect(() => {
    dispatch(fetchReviews());
    dispatch(fetchBugReports());
    dispatch(fetchSurveys());
    dispatch(fetchUserGrowth());
  }, [dispatch]);

  const handleExport = async (type, format) => {
    const urlMap = {
      reviews: '/api/admin/reviews/export',
      bugReports: '/api/admin/bug-reports/export',
      surveys: '/api/admin/hardware-surveys/export',
    };
    const url = urlMap[type] + `?format=${format}`;
    try {
      const res = await fetch(url, {
        headers: { Authorization: `Bearer ${localStorage.getItem('token')}` },
      });
      if (!res.ok) throw new Error('Export failed');
      const blob = await res.blob();
      const a = document.createElement('a');
      a.href = window.URL.createObjectURL(blob);
      a.download = `${type}_${new Date().toISOString().slice(0,19).replace(/[-T:]/g,'')}.${format === 'xlsx' ? 'xlsx' : 'csv'}`;
      document.body.appendChild(a);
      a.click();
      a.remove();
      showToast('Export started. Check your downloads.', 'success');
    } catch (e) {
      showToast(e.message || 'Export failed', 'error');
    }
  };

  return (
    <div className="p-6">
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
        <SummaryCards title="Total Reviews" value={reviews.length} icon="⭐" />
        <SummaryCards title="Bug Reports" value={bugReports.length} icon="🐞" />
        <SummaryCards title="Surveys" value={surveys.length} icon="🖥️" />
        <SummaryCards title="New Users" value={userGrowth.length} icon="👤" />
      </div>
      <div style={{ marginBottom: 8 }}>
        <button style={exportBtnStyle} onClick={() => handleExport('reviews', 'csv')}>Export Reviews CSV</button>
        <button style={exportBtnStyle} onClick={() => handleExport('reviews', 'xlsx')}>Export Reviews Excel</button>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {reviewsLoading ? <div>Loading reviews...</div> : reviewsError ? <div className="text-red-600">{reviewsError}</div> : <ReviewsTable reviews={reviews} />}
        <div>
          <div style={{ marginBottom: 8 }}>
            <button style={exportBtnStyle} onClick={() => handleExport('bugReports', 'csv')}>Export Bug Reports CSV</button>
            <button style={exportBtnStyle} onClick={() => handleExport('bugReports', 'xlsx')}>Export Bug Reports Excel</button>
          </div>
          {bugReportsLoading ? <div>Loading bug reports...</div> : bugReportsError ? <div className="text-red-600">{bugReportsError}</div> : <BugReportsTable bugReports={bugReports} />}
        </div>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mt-6">
        <div>
          <div style={{ marginBottom: 8 }}>
            <button style={exportBtnStyle} onClick={() => handleExport('surveys', 'csv')}>Export Surveys CSV</button>
            <button style={exportBtnStyle} onClick={() => handleExport('surveys', 'xlsx')}>Export Surveys Excel</button>
          </div>
          {surveysLoading ? <div>Loading surveys...</div> : surveysError ? <div className="text-red-600">{surveysError}</div> : <SurveysChart data={surveys} />}
        </div>
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