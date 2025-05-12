import React, { useState } from 'react';
import { useDispatch } from 'react-redux';
import { fetchSurveys } from '../../features/admin/surveysSlice';
import '../../styles/admin.css';

const SurveysTable = ({ surveys, meta, loading, onFilter, onPageChange }) => {
  const dispatch = useDispatch();
  const [filters, setFilters] = useState({
    search: '',
    status: '',
    sortBy: 'created_at',
    sortOrder: 'desc'
  });

  const handleFilterChange = (e) => {
    const { name, value } = e.target;
    setFilters(prev => ({ ...prev, [name]: value }));
  };

  const handleSubmit = (e) => {
    e.preventDefault();
    onFilter(filters);
  };

  const handleSort = (field) => {
    const newOrder = filters.sortBy === field && filters.sortOrder === 'asc' ? 'desc' : 'asc';
    setFilters(prev => ({ ...prev, sortBy: field, sortOrder: newOrder }));
    onFilter({ ...filters, sortBy: field, sortOrder: newOrder });
  };

  const getStatusBadgeClass = (status) => {
    switch (status) {
      case 'completed':
        return 'status-badge-success';
      case 'in_progress':
        return 'status-badge-warning';
      case 'pending':
        return 'status-badge-info';
      default:
        return 'status-badge-default';
    }
  };

  if (loading) {
    return (
      <div className="admin-loading">
        <div className="admin-loading-spinner" />
      </div>
    );
  }

  return (
    <div className="admin-table-wrapper">
      <form onSubmit={handleSubmit} className="admin-filters">
        <div className="flex flex-wrap gap-4">
          <div className="flex-1 min-w-[200px]">
            <input
              type="text"
              name="search"
              value={filters.search}
              onChange={handleFilterChange}
              placeholder="Search surveys..."
              className="admin-filter-input"
            />
          </div>
          <div className="w-[200px]">
            <select
              name="status"
              value={filters.status}
              onChange={handleFilterChange}
              className="admin-filter-input"
            >
              <option value="">All Status</option>
              <option value="completed">Completed</option>
              <option value="in_progress">In Progress</option>
              <option value="pending">Pending</option>
            </select>
          </div>
          <button type="submit" className="admin-button">
            <svg className="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
            Filter
          </button>
        </div>
      </form>

      <div className="admin-table-container">
        <table className="admin-table">
          <thead>
            <tr>
              <th onClick={() => handleSort('id')} className="cursor-pointer hover:bg-slate-50">
                ID
                {filters.sortBy === 'id' && (
                  <span className="ml-1">{filters.sortOrder === 'asc' ? '↑' : '↓'}</span>
                )}
              </th>
              <th onClick={() => handleSort('user_name')} className="cursor-pointer hover:bg-slate-50">
                User
                {filters.sortBy === 'user_name' && (
                  <span className="ml-1">{filters.sortOrder === 'asc' ? '↑' : '↓'}</span>
                )}
              </th>
              <th onClick={() => handleSort('hardware_type')} className="cursor-pointer hover:bg-slate-50">
                Hardware Type
                {filters.sortBy === 'hardware_type' && (
                  <span className="ml-1">{filters.sortOrder === 'asc' ? '↑' : '↓'}</span>
                )}
              </th>
              <th onClick={() => handleSort('status')} className="cursor-pointer hover:bg-slate-50">
                Status
                {filters.sortBy === 'status' && (
                  <span className="ml-1">{filters.sortOrder === 'asc' ? '↑' : '↓'}</span>
                )}
              </th>
              <th onClick={() => handleSort('created_at')} className="cursor-pointer hover:bg-slate-50">
                Created At
                {filters.sortBy === 'created_at' && (
                  <span className="ml-1">{filters.sortOrder === 'asc' ? '↑' : '↓'}</span>
                )}
              </th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {surveys.map(survey => (
              <tr key={survey.id} className="hover:bg-slate-50 transition-colors">
                <td>{survey.id}</td>
                <td>{survey.user_name}</td>
                <td>{survey.hardware_type}</td>
                <td>
                  <span className={`status-badge ${getStatusBadgeClass(survey.status)}`}>
                    {survey.status}
                  </span>
                </td>
                <td>{new Date(survey.created_at).toLocaleDateString()}</td>
                <td>
                  <button
                    onClick={() => {/* Add view details handler */}}
                    className="admin-button-secondary"
                  >
                    <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                    </svg>
                    View
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {meta && (
        <div className="admin-pagination">
          <button
            onClick={() => onPageChange(meta.current_page - 1)}
            disabled={meta.current_page === 1}
            className="admin-button-secondary"
          >
            Previous
          </button>
          <span className="admin-pagination-text">
            Page {meta.current_page} of {meta.last_page}
          </span>
          <button
            onClick={() => onPageChange(meta.current_page + 1)}
            disabled={meta.current_page === meta.last_page}
            className="admin-button-secondary"
          >
            Next
          </button>
        </div>
      )}
    </div>
  );
};

export default SurveysTable; 