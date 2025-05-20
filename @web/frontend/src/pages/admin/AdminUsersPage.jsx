import React, { useEffect, useState } from 'react';
import { useSelector, useDispatch } from 'react-redux';
import { Link } from 'react-router-dom';
import { fetchUsers, updateUserRole, updateUserStatus } from '../../features/admin/usersSlice';
import { useToast } from '../../components/ToastContext';
import '../../styles/admin.css';

const AdminUsersPage = () => {
  const dispatch = useDispatch();
  // Use more cautious state structure access with fallbacks
  const users = useSelector(state => state.adminUsers?.users || []);
  const loading = useSelector(state => state.adminUsers?.loading || false);
  const error = useSelector(state => state.adminUsers?.error || null);
  const meta = useSelector(state => state.adminUsers?.meta || null);
  
  const { showToast } = useToast();
  const [filters, setFilters] = useState({
    search: '',
    role: '',
    status: '',
    sortBy: 'created_at',
    sortOrder: 'desc'
  });
  const [confirmDialog, setConfirmDialog] = useState({
    show: false,
    userId: null,
    action: null,
    targetState: null
  });

  useEffect(() => {
    try {
      dispatch(fetchUsers());
    } catch (err) {
      console.error("Error fetching users:", err);
      showToast("Failed to load users", "error");
    }
  }, [dispatch, showToast]);

  const handleFilterChange = (e) => {
    const { name, value } = e.target;
    setFilters(prev => ({
      ...prev,
      [name]: value
    }));
  };

  const handleSubmit = (e) => {
    e.preventDefault();
    try {
      dispatch(fetchUsers(filters));
    } catch (err) {
      console.error("Error applying filters:", err);
      showToast("Failed to filter users", "error");
    }
  };

  const handleSort = (field) => {
    const newOrder = filters.sortBy === field && filters.sortOrder === 'asc' ? 'desc' : 'asc';
    setFilters(prev => ({
      ...prev,
      sortBy: field,
      sortOrder: newOrder
    }));
    try {
      dispatch(fetchUsers({
        ...filters,
        sortBy: field,
        sortOrder: newOrder
      }));
    } catch (err) {
      console.error("Error sorting users:", err);
      showToast("Failed to sort users", "error");
    }
  };

  const handlePageChange = (page) => {
    try {
      dispatch(fetchUsers({
        ...filters,
        page
      }));
    } catch (err) {
      console.error("Error changing page:", err);
      showToast("Failed to change page", "error");
    }
  };

  const confirmAction = (userId, action, targetState) => {
    setConfirmDialog({
      show: true,
      userId,
      action,
      targetState
    });
  };

  const handleConfirm = async () => {
    try {
      const { userId, action, targetState } = confirmDialog;
      setConfirmDialog({ show: false, userId: null, action: null, targetState: null });

      if (action === 'role') {
        await dispatch(updateUserRole({ userId, is_admin: targetState === 'admin' })).unwrap();
      } else if (action === 'status') {
        await dispatch(updateUserStatus({ userId, is_active: targetState === 'active' })).unwrap();
      }
      dispatch(fetchUsers(filters));
    } catch (error) {
      showToast(error.message || 'An error occurred', 'error');
    }
  };

  const handleCancel = () => {
    setConfirmDialog({ show: false, userId: null, action: null, targetState: null });
  };

  const handleRoleChange = (user, newRole) => {
    confirmAction(user.id, 'role', newRole);
  };

  const handleStatusChange = (user, newStatus) => {
    confirmAction(user.id, 'status', newStatus);
  };

  return (
    <div className="admin-container">
      <div className="flex justify-between items-center mb-8">
        <h1 className="text-3xl font-bold bg-gradient-to-r from-indigo-600 to-blue-500 bg-clip-text text-transparent">
          User Management
        </h1>
        <Link to="/admin" className="admin-button-secondary">
          <svg className="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 19l-7-7m0 0l7-7m-7 7h18" />
          </svg>
          Back to Dashboard
        </Link>
      </div>

      <div className="admin-table-container">
        <form onSubmit={handleSubmit} className="admin-filters">
          <div className="flex flex-wrap gap-4">
            <div className="flex-1 min-w-[200px]">
              <input
                type="text"
                name="search"
                value={filters.search}
                onChange={handleFilterChange}
                placeholder="Search users..."
                className="admin-filter-input"
              />
            </div>
            <div className="w-[150px]">
              <select
                name="role"
                value={filters.role}
                onChange={handleFilterChange}
                className="admin-filter-input"
              >
                <option value="">All Roles</option>
                <option value="admin">Admin</option>
                <option value="moderator">Moderator</option>
                <option value="user">User</option>
              </select>
            </div>
            <div className="w-[150px]">
              <select
                name="status"
                value={filters.status}
                onChange={handleFilterChange}
                className="admin-filter-input"
              >
                <option value="">All Status</option>
                <option value="active">Active</option>
                <option value="inactive">Inactive</option>
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

        {loading ? (
          <div className="admin-loading">
            <div className="admin-loading-spinner" />
          </div>
        ) : error ? (
          <div className="admin-error">
            <p className="admin-error-message">{error}</p>
          </div>
        ) : (
          <>
            <div className="overflow-x-auto">
              <table className="admin-table">
                <thead className="admin-table-header">
                  <tr>
                    <th onClick={() => handleSort('id')} className="cursor-pointer">
                      ID
                      {filters.sortBy === 'id' && (
                        <span className="ml-1">{filters.sortOrder === 'asc' ? '↑' : '↓'}</span>
                      )}
                    </th>
                    <th onClick={() => handleSort('name')} className="cursor-pointer">
                      Name
                      {filters.sortBy === 'name' && (
                        <span className="ml-1">{filters.sortOrder === 'asc' ? '↑' : '↓'}</span>
                      )}
                    </th>
                    <th onClick={() => handleSort('email')} className="cursor-pointer">
                      Email
                      {filters.sortBy === 'email' && (
                        <span className="ml-1">{filters.sortOrder === 'asc' ? '↑' : '↓'}</span>
                      )}
                    </th>
                    <th onClick={() => handleSort('role')} className="cursor-pointer">
                      Role
                      {filters.sortBy === 'role' && (
                        <span className="ml-1">{filters.sortOrder === 'asc' ? '↑' : '↓'}</span>
                      )}
                    </th>
                    <th onClick={() => handleSort('status')} className="cursor-pointer">
                      Status
                      {filters.sortBy === 'status' && (
                        <span className="ml-1">{filters.sortOrder === 'asc' ? '↑' : '↓'}</span>
                      )}
                    </th>
                    <th onClick={() => handleSort('created_at')} className="cursor-pointer">
                      Created
                      {filters.sortBy === 'created_at' && (
                        <span className="ml-1">{filters.sortOrder === 'asc' ? '↑' : '↓'}</span>
                      )}
                    </th>
                    <th>Actions</th>
                  </tr>
                </thead>
                <tbody className="admin-table-body">
                  {users.map(user => (
                    <tr key={user.id} className="admin-table-row">
                      <td className="admin-table-cell">{user.id}</td>
                      <td className="admin-table-cell">{user.name}</td>
                      <td className="admin-table-cell">{user.email}</td>
                      <td className="admin-table-cell">
                        <select
                          className="admin-form-input"
                          value={user.is_admin ? 'admin' : 'user'}
                          onChange={e => handleRoleChange(user, e.target.value)}
                        >
                          <option value="admin">Admin</option>
                          <option value="user">User</option>
                        </select>
                      </td>
                      <td className="admin-table-cell">
                        <span className={`status-badge ${user.is_active ? 'status-badge-active' : 'status-badge-inactive'}`}>
                          {user.is_active ? 'active' : 'inactive'}
                        </span>
                      </td>
                      <td className="admin-table-cell">{user.created_at ? new Date(user.created_at).toLocaleDateString() : 'N/A'}</td>
                      <td className="admin-table-cell">
                        <div className="flex space-x-2">
                          <button
                            onClick={() => handleStatusChange(user, user.is_active ? 'inactive' : 'active')}
                            className={user.is_active ? 'admin-button-danger' : 'admin-button'}
                          >
                            {user.is_active ? (
                              <>
                                <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
                                </svg>
                                Deactivate
                              </>
                            ) : (
                              <>
                                <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                </svg>
                                Activate
                              </>
                            )}
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
                  onClick={() => meta.current_page > 1 && handlePageChange(meta.current_page - 1)}
                  disabled={!meta.current_page || meta.current_page === 1}
                  className="admin-pagination-button"
                >
                  Previous
                </button>
                <span className="text-sm text-slate-700">
                  Page {meta.current_page || 1} of {meta.last_page || 1}
                </span>
                <button
                  onClick={() => meta.current_page < meta.last_page && handlePageChange(meta.current_page + 1)}
                  disabled={!meta.last_page || !meta.current_page || meta.current_page === meta.last_page}
                  className="admin-pagination-button"
                >
                  Next
                </button>
              </div>
            )}
          </>
        )}
      </div>

      {confirmDialog.show && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="admin-form max-w-lg w-full">
            <h2 className="text-xl font-bold mb-4">Confirm Action</h2>
            <p className="mb-6">
              {confirmDialog.action === 'role' 
                ? `Are you sure you want to change this user's role to ${confirmDialog.targetState}?` 
                : `Are you sure you want to ${confirmDialog.targetState === 'active' ? 'activate' : 'deactivate'} this user?`}
            </p>
            <div className="flex justify-end gap-4">
              <button onClick={handleCancel} className="admin-button-secondary">
                Cancel
              </button>
              <button onClick={handleConfirm} className="admin-button">
                Confirm
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default AdminUsersPage; 