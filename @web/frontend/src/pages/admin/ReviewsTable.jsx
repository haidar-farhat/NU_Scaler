import React, { useState } from 'react';
import { useDispatch } from 'react-redux';
import { fetchReviews } from '../../features/admin/reviewsSlice';
import '../../styles/admin.css';

const ReviewsTable = ({ reviews, meta, loading, onFilter, onPageChange }) => {
  const dispatch = useDispatch();
  const [filters, setFilters] = useState({
    search: '',
    rating: '',
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

  // Helper to render star ratings
  const renderStars = (rating) => {
    const stars = [];
    for (let i = 0; i < 5; i++) {
      stars.push(
        <svg 
          key={i} 
          className={`w-5 h-5 ${i < rating ? 'text-yellow-400' : 'text-gray-300'}`} 
          fill="currentColor" 
          viewBox="0 0 20 20"
        >
          <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118l-2.8-2.034c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
        </svg>
      );
    }
    return <div className="flex">{stars}</div>;
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
              placeholder="Search reviews..."
              className="admin-filter-input"
            />
          </div>
          <div className="w-[200px]">
            <select
              name="rating"
              value={filters.rating}
              onChange={handleFilterChange}
              className="admin-filter-input"
            >
              <option value="">All Ratings</option>
              <option value="5">5 Stars</option>
              <option value="4">4 Stars</option>
              <option value="3">3 Stars</option>
              <option value="2">2 Stars</option>
              <option value="1">1 Star</option>
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
              <th onClick={() => handleSort('rating')} className="cursor-pointer">
                Rating
                {filters.sortBy === 'rating' && (
                  <span className="ml-1">{filters.sortOrder === 'asc' ? '↑' : '↓'}</span>
                )}
              </th>
              <th>Comment</th>
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
            {reviews.map(review => (
              <tr key={review.id} className="admin-table-row">
                <td className="admin-table-cell">{review.id}</td>
                <td className="admin-table-cell">{review.user_name}</td>
                <td className="admin-table-cell">{renderStars(review.rating)}</td>
                <td className="admin-table-cell">
                  <div className="max-w-xs truncate">{review.comment}</div>
                </td>
                <td className="admin-table-cell">{new Date(review.created_at).toLocaleDateString()}</td>
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

export default ReviewsTable; 