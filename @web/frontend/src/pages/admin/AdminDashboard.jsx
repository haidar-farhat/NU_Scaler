import React from 'react';
import SummaryCards from './SummaryCards';
import ReviewsTable from './ReviewsTable';
import BugReportsTable from './BugReportsTable';
import SurveysChart from './SurveysChart';
import UserGrowthChart from './UserGrowthChart';

const dummyReviews = [
  { id: 1, rating: 5, comment: 'Great!', created_at: new Date().toISOString() },
];
const dummyBugReports = [
  { id: 1, severity: 'high', description: 'Crash on launch', created_at: new Date().toISOString() },
];
const dummySurveys = [
  { gpu_brand: 'NVIDIA', count: 10 },
  { gpu_brand: 'AMD', count: 5 },
];
const dummyUserGrowth = [
  { date: '2024-05-01', registrations: 2 },
  { date: '2024-05-02', registrations: 5 },
];

const AdminDashboard = () => {
  // Replace dummy data with Redux selectors or API data as needed
  return (
    <div className="p-6">
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
        <SummaryCards title="Total Reviews" value={dummyReviews.length} icon="⭐" />
        <SummaryCards title="Bug Reports" value={dummyBugReports.length} icon="🐞" />
        <SummaryCards title="Surveys" value={dummySurveys.length} icon="🖥️" />
        <SummaryCards title="New Users" value={dummyUserGrowth.length} icon="👤" />
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <ReviewsTable reviews={dummyReviews} />
        <BugReportsTable bugReports={dummyBugReports} />
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mt-6">
        <SurveysChart data={dummySurveys} />
        <UserGrowthChart data={dummyUserGrowth} />
      </div>
    </div>
  );
};

export default AdminDashboard; 