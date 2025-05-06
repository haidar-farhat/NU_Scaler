import React, { useEffect, useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { fetchUsers, updateUserRole, updateUserStatus } from '../../features/admin/usersSlice';
import { useToast } from '../../components/ToastContext';
import ConfirmDialog from '../../components/ConfirmDialog';

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

  // Dialog state
  const [dialog, setDialog] = useState({ open: false, type: '', targetUser: null });

  useEffect(() => {
    dispatch(fetchUsers())
      .unwrap()
      .catch(e => showToast(e.message || 'Failed to load users', 'error'));
  }, [dispatch, showToast]);

  const openDialog = (type, targetUser) => setDialog({ open: true, type, targetUser });
  const closeDialog = () => setDialog({ open: false, type: '', targetUser: null });

  const handleConfirm = async () => {
    const { type, targetUser } = dialog;
    if (!targetUser) return closeDialog();
    try {
      if (type === 'role') {
        await dispatch(updateUserRole({ userId: targetUser.id, is_admin: !targetUser.is_admin })).unwrap();
        showToast(`User ${targetUser.is_admin ? 'demoted' : 'promoted'} successfully.`, 'success');
      } else if (type === 'status') {
        await dispatch(updateUserStatus({ userId: targetUser.id, is_active: !targetUser.is_active })).unwrap();
        showToast(`User ${targetUser.is_active ? 'deactivated' : 'activated'} successfully.`, 'success');
      }
    } catch (e) {
      showToast(e.message || 'Action failed', 'error');
    }
    closeDialog();
  };

  let dialogMessage = '';
  if (dialog.open && dialog.targetUser) {
    if (dialog.type === 'role') {
      dialogMessage = dialog.targetUser.is_admin
        ? 'Are you sure you want to demote this admin to user?'
        : 'Are you sure you want to promote this user to admin?';
    } else if (dialog.type === 'status') {
      dialogMessage = dialog.targetUser.is_active
        ? 'Are you sure you want to deactivate this user?'
        : 'Are you sure you want to activate this user?';
    }
  }

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
                  onClick={() => u.id !== user.id && openDialog('role', u)}
                  disabled={u.id === user.id}
                  title={u.id === user.id ? 'Cannot change your own role' : u.is_admin ? 'Demote to user' : 'Promote to admin'}
                >
                  {u.is_admin ? 'Demote' : 'Promote'}
                </button>
                <button
                  style={{ ...actionBtn, background: u.is_active ? '#dc3545' : '#28a745', color: '#fff' }}
                  onClick={() => u.id !== user.id && openDialog('status', u)}
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
      <ConfirmDialog open={dialog.open} message={dialogMessage} onConfirm={handleConfirm} onCancel={closeDialog} />
    </div>
  );
} 