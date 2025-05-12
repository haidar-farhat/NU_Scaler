import React, { useEffect, useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { fetchUsers, updateUserRole, updateUserStatus } from '../../features/admin/usersSlice';
import { useToast } from '../../components/ToastContext';
import ConfirmDialog from '../../components/ConfirmDialog';
import '../../styles/admin.css';

export default function AdminUsersPage() {
  const dispatch = useDispatch();
  const { users, loading, error } = useSelector(state => state.adminUsers);
  const { user } = useSelector(state => state.auth);
  const { showToast } = useToast();
  const [dialog, setDialog] = useState({ open: false, type: '', targetUser: null });

  useEffect(() => {
    console.log('AdminUsersPage mounted, dispatching fetchUsers');
    dispatch(fetchUsers())
      .unwrap()
      .then(data => {
        console.log('Users fetched successfully:', data);
      })
      .catch(e => {
        console.error('Error fetching users:', e);
        showToast(e.message || 'Failed to load users', 'error');
      });
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
    <div className="admin-container">
      <h2 className="text-2xl font-bold mb-6">User Management</h2>
      
      {loading && (
        <div className="admin-loading">
          <div className="admin-loading-spinner" />
        </div>
      )}
      
      {error && (
        <div className="admin-error">
          <p className="admin-error-message">{error.message || error}</p>
        </div>
      )}

      <div className="admin-table-container">
        <table className="admin-table">
          <thead className="admin-table-header">
            <tr>
              <th>Name</th>
              <th>Email</th>
              <th>Role</th>
              <th>Status</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody className="admin-table-body">
            {users.map(u => (
              <tr key={u.id} className={`admin-table-row ${!u.is_active ? 'bg-red-50' : ''}`}>
                <td className="admin-table-cell">{u.name}</td>
                <td className="admin-table-cell">{u.email}</td>
                <td className="admin-table-cell">
                  <span className={`status-badge ${u.is_admin ? 'status-badge-active' : 'status-badge-pending'}`}>
                    {u.is_admin ? 'Admin' : 'User'}
                  </span>
                </td>
                <td className="admin-table-cell">
                  <span className={`status-badge ${u.is_active ? 'status-badge-active' : 'status-badge-inactive'}`}>
                    {u.is_active ? 'Active' : 'Inactive'}
                  </span>
                </td>
                <td className="admin-table-cell">
                  <button
                    className={`admin-button ${u.is_admin ? 'admin-button-secondary' : ''}`}
                    onClick={() => u.id !== user.id && openDialog('role', u)}
                    disabled={u.id === user.id}
                    title={u.id === user.id ? 'Cannot change your own role' : u.is_admin ? 'Demote to user' : 'Promote to admin'}
                  >
                    {u.is_admin ? 'Demote' : 'Promote'}
                  </button>
                  <button
                    className={`admin-button ${u.is_active ? 'admin-button-danger' : ''}`}
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
      </div>

      <ConfirmDialog
        isOpen={dialog.open}
        onClose={closeDialog}
        onConfirm={handleConfirm}
        title={dialog.type === 'role' ? 'Change User Role' : 'Change User Status'}
        message={dialogMessage}
      />
    </div>
  );
} 