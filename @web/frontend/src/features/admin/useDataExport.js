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
    
    // Add http://localhost:8000 prefix if needed
    const baseUrl = 'http://localhost:8000';
    const url = `${baseUrl}${urlMap[type]}?format=${format}`;
    
    // Get the token
    const authToken = token || localStorage.getItem('token');
    console.log('Export request with token:', authToken ? `${authToken.substring(0, 10)}...` : 'No token');
    
    try {
      const res = await fetch(url, {
        headers: { 
          'Authorization': `Bearer ${authToken}`,
          'Accept': 'application/json, application/octet-stream',
          'Content-Type': 'application/json'
        },
        credentials: 'include' // Important for CORS and authentication cookies
      });
      
      if (!res.ok) {
        console.error('Export failed:', res.status, res.statusText);
        let msg = `Export failed: ${res.status} ${res.statusText}`;
        try {
          const err = await res.json();
          msg = err.message || msg;
        } catch (e) {
          console.error('Error parsing error response:', e);
        }
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
      console.error('Export error:', e);
      setError(e.message || 'Export failed');
      showToast(e.message || 'Export failed', 'error');
    } finally {
      setLoading(false);
    }
  }, [token, showToast]);

  return { exportData, loading, error };
} 