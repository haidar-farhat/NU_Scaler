import React, { useState } from 'react';
import { useDispatch } from 'react-redux';
import { fetchBugReports } from '../../features/admin/bugReportsSlice';
import '../../styles/admin.css';

const BugReportsTable = ({ bugReports, meta, loading, onFilter, onPageChange }) => {
  const dispatch = useDispatch();
  const [filters, setFilters] = useState({
    search: '',
    status: '',
    severity: '',
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
    switch (status?.toLowerCase()) {
      case 'open':
        return 'status-badge-warning';
      case 'in progress':
        return 'status-badge-info';
      case 'resolved':
        return 'status-badge-success';
      case 'closed':
        return 'status-badge-inactive';
      default:
        return 'status-badge-default';
    }
  };

  const getSeverityBadgeClass = (severity) => {
    switch (severity?.toLowerCase()) {
      case 'critical':
        return 'status-badge-danger';
      case 'high':
        return 'status-badge-warning';
      case 'medium':
        return 'status-badge-info';
      case 'low':
        return 'status-badge-success';
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
              placeholder="Search bug reports..."
              className="admin-filter-input"
            />
          </div>
          <div className="w-[150px]">
            <select
              name="status"
              value={filters.status}
              onChange={handleFilterChange}
              className="admin-filter-input"
            >
              <option value="">All Status</option>
              <option value="open">Open</option>
              <option value="in progress">In Progress</option>
              <option value="resolved">Resolved</option>
              <option value="closed">Closed</option>
            </select>
          </div>
          <div className="w-[150px]">
            <select
              name="severity"
              value={filters.severity}
              onChange={handleFilterChange}
              className="admin-filter-input"
            >
              <option value="">All Severity</option>
              <option value="critical">Critical</option>
              <option value="high">High</option>
              <option value="medium">Medium</option>
              <option value="low">Low</option>
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
          <thead className="admin-table-header">
            <tr>
              <th onClick={() => handleSort('id')} className="cursor-pointer">
                ID
                {filters.sortBy === 'id' && (
                  <span className="ml-1">{filters.sortOrder === 'asc' ? '↑' : '↓'}</span>
                )}
              </th>
              <th onClick={() => handleSort('user_name')} className="cursor-pointer">
                User
                {filters.sortBy === 'user_name' && (
                  <span className="ml-1">{filters.sortOrder === 'asc' ? '↑' : '↓'}</span>
                )}
              </th>
              <th onClick={() => handleSort('title')} className="cursor-pointer">
                Title
                {filters.sortBy === 'title' && (
                  <span className="ml-1">{filters.sortOrder === 'asc' ? '↑' : '↓'}</span>
                )}
              </th>
              <th onClick={() => handleSort('status')} className="cursor-pointer">
                Status
                {filters.sortBy === 'status' && (
                  <span className="ml-1">{filters.sortOrder === 'asc' ? '↑' : '↓'}</span>
                )}
              </th>
              <th onClick={() => handleSort('severity')} className="cursor-pointer">
                Severity
                {filters.sortBy === 'severity' && (
                  <span className="ml-1">{filters.sortOrder === 'asc' ? '↑' : '↓'}</span>
                )}
              </th>
              <th onClick={() => handleSort('created_at')} className="cursor-pointer">
                Date
                {filters.sortBy === 'created_at' && (
                  <span className="ml-1">{filters.sortOrder === 'asc' ? '↑' : '↓'}</span>
                )}
              </th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody className="admin-table-body">
            {bugReports.map(report => (
              <tr key={report.id} className="admin-table-row">
                <td className="admin-table-cell">{report.id}</td>
                <td className="admin-table-cell">{report.user_name}</td>
                <td className="admin-table-cell">
                  <div className="max-w-xs truncate">{report.title}</div>
                </td>
                <td className="admin-table-cell">
                  <span className={`status-badge ${getStatusBadgeClass(report.status)}`}>
                    {report.status}
                  </span>
                </td>
                <td className="admin-table-cell">
                  <span className={`status-badge ${getSeverityBadgeClass(report.severity)}`}>
                    {report.severity}
                  </span>
                </td>
                <td className="admin-table-cell">{new Date(report.created_at).toLocaleDateString()}</td>
                <td className="admin-table-cell">
                  <div className="flex space-x-2">
                    <button
                      onClick={() => {/* View details */}}
                      className="admin-button-secondary"
                    >
                      <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                      </svg>
                      View
                    </button>
                    <button
                      onClick={() => {/* Update status */}}
                      className="admin-button"
                    >
                      <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                      </svg>
                      Update
                    </button>
                  </div>
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
            className="admin-pagination-button"
          >
            Previous
          </button>
          <span className="text-sm text-slate-700">
            Page {meta.current_page} of {meta.last_page}
          </span>
          <button
            onClick={() => onPageChange(meta.current_page + 1)}
            disabled={meta.current_page === meta.last_page}
            className="admin-pagination-button"
          >
            Next
          </button>
        </div>
      )}
    </div>
  );
};

export default BugReportsTable; 