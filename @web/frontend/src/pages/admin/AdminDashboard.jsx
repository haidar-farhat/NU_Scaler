import React, { useEffect, useState } from 'react';
import { useSelector, useDispatch } from 'react-redux';
import SummaryCards from './SummaryCards';
import ReviewsTable from './ReviewsTable';
import BugReportsTable from './BugReportsTable';
import SurveysTable from './SurveysTable';
import UserGrowthChart from './UserGrowthChart';
import { fetchReviews } from '../../features/admin/reviewsSlice';
import { fetchBugReports } from '../../features/admin/bugReportsSlice';
import { fetchSurveys } from '../../features/admin/surveysSlice';
import { fetchUserGrowth } from '../../features/admin/userGrowthSlice';
import { Link, useNavigate } from 'react-router-dom';
import { useToast } from '../../components/ToastContext';
import { useDataExport } from '../../features/admin/useDataExport';
import adminApiService from '../../api/adminApi';

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
  const navigate = useNavigate();
  const { list: reviews, meta: reviewsMeta, loading: reviewsLoading, error: reviewsError } = useSelector(state => state.reviews);
  const { list: bugReports, meta: bugReportsMeta, loading: bugReportsLoading, error: bugReportsError } = useSelector(state => state.bugReports);
  const { list: surveys, meta: surveysMeta, loading: surveysLoading, error: surveysError } = useSelector(state => state.surveys);
  const { list: userGrowth, loading: userGrowthLoading, error: userGrowthError } = useSelector(state => state.userGrowth);
  const { showToast } = useToast();
  const { exportData, loading: exportLoading, error: exportError } = useDataExport();
  const { isAuthenticated, user } = useSelector(state => state.auth);
  const [authChecked, setAuthChecked] = useState(false);
  const [authLoading, setAuthLoading] = useState(true);

  // Check if the user is authenticated and is an admin
  useEffect(() => {
    const checkAdminAuth = async () => {
      try {
        // Make sure CSRF cookie is set up
        await adminApiService.ensureCSRF();
        
        // Check admin-specific authentication status
        console.log('Checking admin authentication status...');
        
        // First check if we have a token in localStorage
        const token = localStorage.getItem('token');
        if (!token) {
          console.error('No authentication token found');
          showToast('Please log in first', 'error');
          navigate('/login');
          return;
        }
        
        // Now check admin session specifically
        try {
          const adminSessionResponse = await adminApiService.checkAdminSession();
          console.log('Admin session check response:', adminSessionResponse.data);
          
          if (adminSessionResponse.data.authenticated && adminSessionResponse.data.is_admin) {
            console.log('Admin authentication confirmed');
            setAuthChecked(true);
            
            // Load the dashboard data
            dispatch(fetchReviews());
            dispatch(fetchBugReports());
            dispatch(fetchSurveys());
            dispatch(fetchUserGrowth());
            return;
          }
        } catch (adminError) {
          console.error('Admin session check failed:', adminError);
          // Fall through to the general auth check
        }
        
        // If admin check failed, try general auth check as fallback
        const response = await adminApiService.checkAuthStatus();
        console.log('General auth check response:', response.data);
        
        const isAdmin = response.data.user?.is_admin === true;
        const authenticated = response.data.authenticated === true;
        
        if (!authenticated) {
          console.error('User is not authenticated for admin dashboard');
          showToast('Please log in as an admin to access this page', 'error');
          navigate('/login');
          return;
        }
        
        if (!isAdmin) {
          console.error('User is authenticated but not an admin');
          showToast('You do not have admin privileges', 'error');
          navigate('/');
          return;
        }
        
        // If we get here, the user is authenticated and is an admin
        console.log('Admin authentication confirmed through fallback');
        setAuthChecked(true);
        
        // Load the dashboard data
        dispatch(fetchReviews());
        dispatch(fetchBugReports());
        dispatch(fetchSurveys());
        dispatch(fetchUserGrowth());
      } catch (error) {
        console.error('Error checking admin auth:', error);
        showToast('Authentication error: ' + (error.message || 'Unknown error'), 'error');
        navigate('/login');
      } finally {
        setAuthLoading(false);
      }
    };
    
    checkAdminAuth();
  }, [dispatch, navigate, showToast]);

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

  // Filter/pagination handlers
  const handleReviewsFilter = (params) => dispatch(fetchReviews(params));
  const handleReviewsPage = (page) => dispatch(fetchReviews({ ...reviewsMeta, page }));
  const handleBugReportsFilter = (params) => dispatch(fetchBugReports(params));
  const handleBugReportsPage = (page) => dispatch(fetchBugReports({ ...bugReportsMeta, page }));
  const handleSurveysFilter = (params) => dispatch(fetchSurveys(params));
  const handleSurveysPage = (page) => dispatch(fetchSurveys({ ...surveysMeta, page }));

  if (authLoading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <div className="text-center">
          <div className="spinner-border animate-spin inline-block w-8 h-8 border-4 rounded-full text-blue-600 mb-4" role="status">
            <span className="visually-hidden">Loading...</span>
          </div>
          <p className="text-lg">Verifying admin access...</p>
        </div>
      </div>
    );
  }

  if (!authChecked) {
    return null;
  }

  return (
    <div className="p-6">
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
        <SummaryCards title="Total Reviews" value={reviews.length} icon="⭐" />
        <SummaryCards title="Bug Reports" value={bugReports.length} icon="🐞" />
        <SummaryCards title="Surveys" value={surveys.length} icon="🖥️" />
        <SummaryCards title="New Users" value={userGrowth.length} icon="👤" />
      </div>
      <div style={{ marginBottom: 8 }}>
        <button
          style={exportBtnStyle}
          onClick={() => exportData('reviews', 'csv')}
          disabled={exportLoading}
          aria-busy={exportLoading}
          title="Export all reviews as CSV"
        >
          {exportLoading ? 'Exporting...' : 'Export Reviews CSV'}
        </button>
        <button
          style={exportBtnStyle}
          onClick={() => exportData('reviews', 'xlsx')}
          disabled={exportLoading}
          aria-busy={exportLoading}
          title="Export all reviews as Excel (XLSX)"
        >
          {exportLoading ? 'Exporting...' : 'Export Reviews Excel'}
        </button>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {reviewsLoading ? <div>Loading reviews...</div> : reviewsError ? <div className="text-red-600">{reviewsError}</div> :
          <ReviewsTable
            reviews={reviews}
            meta={reviewsMeta}
            loading={reviewsLoading}
            onFilter={handleReviewsFilter}
            onPageChange={handleReviewsPage}
          />}
        <div>
          <div style={{ marginBottom: 8 }}>
            <button
              style={exportBtnStyle}
              onClick={() => exportData('bugReports', 'csv')}
              disabled={exportLoading}
              aria-busy={exportLoading}
              title="Export all bug reports as CSV"
            >
              {exportLoading ? 'Exporting...' : 'Export Bug Reports CSV'}
            </button>
            <button
              style={exportBtnStyle}
              onClick={() => exportData('bugReports', 'xlsx')}
              disabled={exportLoading}
              aria-busy={exportLoading}
              title="Export all bug reports as Excel (XLSX)"
            >
              {exportLoading ? 'Exporting...' : 'Export Bug Reports Excel'}
            </button>
          </div>
          {bugReportsLoading ? <div>Loading bug reports...</div> : bugReportsError ? <div className="text-red-600">{bugReportsError}</div> :
            <BugReportsTable
              bugReports={bugReports}
              meta={bugReportsMeta}
              loading={bugReportsLoading}
              onFilter={handleBugReportsFilter}
              onPageChange={handleBugReportsPage}
            />}
        </div>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mt-6">
        <div>
          <div style={{ marginBottom: 8 }}>
            <button
              style={exportBtnStyle}
              onClick={() => exportData('surveys', 'csv')}
              disabled={exportLoading}
              aria-busy={exportLoading}
              title="Export all surveys as CSV"
            >
              {exportLoading ? 'Exporting...' : 'Export Surveys CSV'}
            </button>
            <button
              style={exportBtnStyle}
              onClick={() => exportData('surveys', 'xlsx')}
              disabled={exportLoading}
              aria-busy={exportLoading}
              title="Export all surveys as Excel (XLSX)"
            >
              {exportLoading ? 'Exporting...' : 'Export Surveys Excel'}
            </button>
          </div>
          {surveysLoading ? <div>Loading surveys...</div> : surveysError ? <div className="text-red-600">{surveysError}</div> :
            <SurveysTable
              surveys={surveys}
              meta={surveysMeta}
              loading={surveysLoading}
              onFilter={handleSurveysFilter}
              onPageChange={handleSurveysPage}
            />}
        </div>
        {userGrowthLoading ? <div>Loading user growth...</div> : userGrowthError ? <div className="text-red-600">{userGrowthError}</div> : <UserGrowthChart data={userGrowth} />}
      </div>
      {exportError && <div style={{ color: 'red', marginTop: 8 }}>{exportError}</div>}
      <div style={{ marginBottom: 24 }}>
        <Link to="/admin/users" style={{ fontWeight: 'bold', color: '#007bff', textDecoration: 'none' }}>
          Manage Users
        </Link>
      </div>
    </div>
  );
};

export default AdminDashboard; 