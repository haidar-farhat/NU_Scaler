import React from 'react';

export default function ConfirmDialog({ open, message, onConfirm, onCancel }) {
  if (!open) return null;
  return (
    <div style={{
      position: 'fixed', top: 0, left: 0, width: '100vw', height: '100vh',
      background: 'rgba(0,0,0,0.3)', zIndex: 10000, display: 'flex', alignItems: 'center', justifyContent: 'center'
    }}>
      <div style={{ background: '#fff', borderRadius: 8, padding: 32, minWidth: 320, boxShadow: '0 2px 16px rgba(0,0,0,0.2)' }}>
        <div style={{ marginBottom: 24, fontSize: 18 }}>{message}</div>
        <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 12 }}>
          <button onClick={onCancel} style={{ padding: '6px 18px', borderRadius: 4, border: 'none', background: '#ccc', color: '#222', fontWeight: 500 }}>Cancel</button>
          <button onClick={onConfirm} style={{ padding: '6px 18px', borderRadius: 4, border: 'none', background: '#007bff', color: '#fff', fontWeight: 500 }}>Confirm</button>
        </div>
      </div>
    </div>
  );
} 