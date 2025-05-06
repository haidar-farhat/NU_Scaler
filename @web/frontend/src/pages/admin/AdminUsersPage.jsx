import React, { useEffect } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { fetchUsers, updateUserRole, updateUserStatus } from '../../features/admin/usersSlice';
import { useToast } from '../../components/ToastContext';

const tableStyle = {
  width: '100%',
  borderCollapse: 'collapse',
  marginTop: 24,
};
const thtd = {
  border: '1px solid #ddd',
  padding: 8,
  textAlign: 'left',
};
const actionBtn = {
  marginRight: 8,
  padding: '4px 10px',
  border: 'none',
  borderRadius: 4,
  cursor: 'pointer',
};

export default function AdminUsersPage() {
  const dispatch = useDispatch();
  const { users, loading, error } = useSelector(state => state.adminUsers);
  const { user } = useSelector(state => state.auth);
  const { showToast } = useToast();

  useEffect(() => {
    dispatch(fetchUsers())
      .unwrap()
      .catch(e => showToast(e.message || 'Failed to load users', 'error'));
  }, [dispatch, showToast]);

  const handleRole = async (u) => {
    if (u.id === user.id) return;
    const action = u.is_admin ? 'demote this admin to user' : 'promote this user to admin';
    if (!window.confirm(`Are you sure you want to ${action}?`)) return;
    try {
      await dispatch(updateUserRole({ userId: u.id, is_admin: !u.is_admin })).unwrap();
      showToast(`User ${u.is_admin ? 'demoted' : 'promoted'} successfully.`, 'success');
    } catch (e) {
      showToast(e.message || 'Failed to update user role', 'error');
    }
  };
  const handleStatus = async (u) => {
    if (u.id === user.id) return;
    const action = u.is_active ? 'deactivate this user' : 'activate this user';
    if (!window.confirm(`Are you sure you want to ${action}?`)) return;
    try {
      await dispatch(updateUserStatus({ userId: u.id, is_active: !u.is_active })).unwrap();
      showToast(`User ${u.is_active ? 'deactivated' : 'activated'} successfully.`, 'success');
    } catch (e) {
      showToast(e.message || 'Failed to update user status', 'error');
    }
  };

  return (
    <div style={{ maxWidth: 900, margin: '0 auto', padding: 24 }}>
      <h2>User Management</h2>
      {loading && <p>Loading users...</p>}
      {error && <p style={{ color: 'red' }}>{error.message || error}</p>}
      <table style={tableStyle}>
        <thead>
          <tr>
            <th style={thtd}>Name</th>
            <th style={thtd}>Email</th>
            <th style={thtd}>Role</th>
            <th style={thtd}>Status</th>
            <th style={thtd}>Actions</th>
          </tr>
        </thead>
        <tbody>
          {users.map(u => (
            <tr key={u.id} style={{ background: u.is_active ? '#fff' : '#f8d7da' }}>
              <td style={thtd}>{u.name}</td>
              <td style={thtd}>{u.email}</td>
              <td style={thtd}>{u.is_admin ? 'Admin' : 'User'}</td>
              <td style={thtd}>{u.is_active ? 'Active' : 'Inactive'}</td>
              <td style={thtd}>
                <button
                  style={{ ...actionBtn, background: u.is_admin ? '#ffc107' : '#007bff', color: '#fff' }}
                  onClick={() => handleRole(u)}
                  disabled={u.id === user.id}
                  title={u.id === user.id ? 'Cannot change your own role' : u.is_admin ? 'Demote to user' : 'Promote to admin'}
                >
                  {u.is_admin ? 'Demote' : 'Promote'}
                </button>
                <button
                  style={{ ...actionBtn, background: u.is_active ? '#dc3545' : '#28a745', color: '#fff' }}
                  onClick={() => handleStatus(u)}
                  disabled={u.id === user.id}
                  title={u.id === user.id ? 'Cannot change your own status' : u.is_active ? 'Deactivate' : 'Activate'}
                >
                  {u.is_active ? 'Deactivate' : 'Activate'}
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
} 