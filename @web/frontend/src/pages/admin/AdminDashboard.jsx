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
import '../../styles/admin.css';

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
      <div className="admin-container flex items-center justify-center min-h-screen">
        <div className="text-center">
          <div className="admin-loading-spinner mx-auto mb-4" />
          <p className="text-lg text-slate-600 font-medium">Verifying admin access...</p>
        </div>
      </div>
    );
  }

  if (!authChecked) {
    return null;
  }

  return (
    <div className="admin-container">
      <div className="flex justify-between items-center mb-8">
        <h1 className="text-3xl font-bold bg-gradient-to-r from-indigo-600 to-blue-500 bg-clip-text text-transparent">
          Admin Dashboard
        </h1>
        <div className="flex gap-4">
          <Link to="/admin/users" className="admin-button">
            <svg className="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
            </svg>
            Manage Users
          </Link>
        </div>
      </div>

      <div className="admin-grid">
        <SummaryCards 
          title="Total Reviews" 
          value={reviews.length} 
          icon={
            <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
            </svg>
          } 
        />
        <SummaryCards 
          title="Bug Reports" 
          value={bugReports.length} 
          icon={
            <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
          } 
        />
        <SummaryCards 
          title="Surveys" 
          value={surveys.length} 
          icon={
            <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
            </svg>
          } 
        />
        <SummaryCards 
          title="New Users" 
          value={userGrowth.length} 
          icon={
            <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
            </svg>
          } 
        />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8 mb-8">
        <div className="admin-chart-container">
          <h2 className="admin-chart-title">User Growth</h2>
          {userGrowthLoading ? (
            <div className="admin-loading">
              <div className="admin-loading-spinner" />
            </div>
          ) : userGrowthError ? (
            <div className="admin-error">
              <p className="admin-error-message">{userGrowthError}</p>
            </div>
          ) : (
            <UserGrowthChart data={userGrowth} />
          )}
        </div>

        <div className="admin-chart-container">
          <div className="flex justify-between items-center mb-4">
            <h2 className="admin-chart-title">Recent Activity</h2>
            <button
              className="export-button"
              onClick={() => exportData('activity', 'csv')}
              disabled={exportLoading}
            >
              <svg className="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
              </svg>
              Export
            </button>
          </div>
          {/* Add your activity chart or list here */}
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <div className="admin-table-container">
          <div className="flex justify-between items-center p-4 border-b border-slate-200">
            <h2 className="text-lg font-semibold text-slate-800">Recent Reviews</h2>
            <button
              className="export-button"
              onClick={() => exportData('reviews', 'csv')}
              disabled={exportLoading}
            >
              <svg className="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
              </svg>
              Export
            </button>
          </div>
          {reviewsLoading ? (
            <div className="admin-loading">
              <div className="admin-loading-spinner" />
            </div>
          ) : reviewsError ? (
            <div className="admin-error">
              <p className="admin-error-message">{reviewsError}</p>
            </div>
          ) : (
            <ReviewsTable
              reviews={reviews}
              meta={reviewsMeta}
              loading={reviewsLoading}
              onFilter={handleReviewsFilter}
              onPageChange={handleReviewsPage}
            />
          )}
        </div>

        <div className="admin-table-container">
          <div className="flex justify-between items-center p-4 border-b border-slate-200">
            <h2 className="text-lg font-semibold text-slate-800">Bug Reports</h2>
            <button
              className="export-button"
              onClick={() => exportData('bugReports', 'csv')}
              disabled={exportLoading}
            >
              <svg className="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
              </svg>
              Export
            </button>
          </div>
          {bugReportsLoading ? (
            <div className="admin-loading">
              <div className="admin-loading-spinner" />
            </div>
          ) : bugReportsError ? (
            <div className="admin-error">
              <p className="admin-error-message">{bugReportsError}</p>
            </div>
          ) : (
            <BugReportsTable
              bugReports={bugReports}
              meta={bugReportsMeta}
              loading={bugReportsLoading}
              onFilter={handleBugReportsFilter}
              onPageChange={handleBugReportsPage}
            />
          )}
        </div>
      </div>

      <div className="admin-table-container mt-8">
        <div className="flex justify-between items-center p-4 border-b border-slate-200">
          <h2 className="text-lg font-semibold text-slate-800">Hardware Surveys</h2>
          <button
            className="export-button"
            onClick={() => exportData('surveys', 'csv')}
            disabled={exportLoading}
          >
            <svg className="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
            </svg>
            Export
          </button>
        </div>
        {surveysLoading ? (
          <div className="admin-loading">
            <div className="admin-loading-spinner" />
          </div>
        ) : surveysError ? (
          <div className="admin-error">
            <p className="admin-error-message">{surveysError}</p>
          </div>
        ) : (
          <SurveysTable
            surveys={surveys}
            meta={surveysMeta}
            loading={surveysLoading}
            onFilter={handleSurveysFilter}
            onPageChange={handleSurveysPage}
          />
        )}
      </div>

      {exportError && (
        <div className="admin-error mt-4">
          <p className="admin-error-message">{exportError}</p>
        </div>
      )}
    </div>
  );
};

export default AdminDashboard; 