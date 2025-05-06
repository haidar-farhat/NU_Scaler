import { useState, useCallback } from 'react';
import { useToast } from '../../components/ToastContext';
import { useSelector } from 'react-redux';

export function useDataExport() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const { showToast } = useToast();
  const { token } = useSelector(state => state.auth);

  const exportData = useCallback(async (type, format) => {
    setLoading(true);
    setError(null);
    const urlMap = {
      reviews: '/api/admin/reviews/export',
      bugReports: '/api/admin/bug-reports/export',
      surveys: '/api/admin/hardware-surveys/export',
    };
    const url = urlMap[type] + `?format=${format}`;
    try {
      const res = await fetch(url, {
        headers: { Authorization: `Bearer ${token || localStorage.getItem('token')}` },
      });
      if (!res.ok) {
        let msg = 'Export failed';
        try {
          const err = await res.json();
          msg = err.message || msg;
        } catch {}
        throw new Error(msg);
      }
      const blob = await res.blob();
      const a = document.createElement('a');
      a.href = window.URL.createObjectURL(blob);
      a.download = `${type}_${new Date().toISOString().slice(0,19).replace(/[-T:]/g,'')}.${format === 'xlsx' ? 'xlsx' : 'csv'}`;
      document.body.appendChild(a);
      a.click();
      a.remove();
      showToast('Export started. Check your downloads.', 'success');
    } catch (e) {
      setError(e.message || 'Export failed');
      showToast(e.message || 'Export failed', 'error');
    } finally {
      setLoading(false);
    }
  }, [token, showToast]);

  return { exportData, loading, error };
} 