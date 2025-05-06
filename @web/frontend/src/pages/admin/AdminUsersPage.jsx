import React, { useEffect } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { fetchUsers, updateUserRole, updateUserStatus } from '../../features/admin/usersSlice';

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

  useEffect(() => {
    dispatch(fetchUsers());
  }, [dispatch]);

  const handleRole = (u) => {
    if (u.id === user.id) return;
    dispatch(updateUserRole({ userId: u.id, is_admin: !u.is_admin }));
  };
  const handleStatus = (u) => {
    if (u.id === user.id) return;
    dispatch(updateUserStatus({ userId: u.id, is_active: !u.is_active }));
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