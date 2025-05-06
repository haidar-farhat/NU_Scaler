import { useState } from 'react';
import adminApiService from '../../api/adminApi';

export const useDataExport = () => {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const exportData = async (type, format = 'csv') => {
    setLoading(true);
    setError(null);

    try {
      let response;
      
      // Use our centralized admin API service to handle exports
      switch (type) {
        case 'reviews':
          response = await adminApiService.exportReviews(format);
          break;
        case 'bugReports':
          response = await adminApiService.exportBugReports(format);
          break;
        case 'surveys':
          response = await adminApiService.exportHardwareSurveys(format);
          break;
        default:
          throw new Error(`Unknown export type: ${type}`);
      }

      // Create and trigger download
      const filename = `${type}_${new Date().toISOString().split('T')[0]}.${format}`;
      const blob = new Blob([response.data], { 
        type: format === 'csv' 
          ? 'text/csv' 
          : 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' 
      });
      
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = filename;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      a.remove();
      
      setLoading(false);
      return true;
    } catch (err) {
      console.error('Export error:', err);
      
      // Handle server error messages that come as JSON
      if (err.response?.data) {
        try {
          // If the error data is a Blob, read it as JSON
          if (err.response.data instanceof Blob) {
            const textData = await new Response(err.response.data).text();
            const jsonData = JSON.parse(textData);
            setError(jsonData.message || `Export failed: ${err.message}`);
          } else {
            // If it's already a parsed object
            setError(err.response.data.message || `Export failed: ${err.message}`);
          }
        } catch (jsonError) {
          // If we can't parse the error response
          setError(`Export failed: ${err.message}`);
        }
      } else {
        setError(`Export failed: ${err.message}`);
      }
      
      setLoading(false);
      return false;
    }
  };

  return { exportData, loading, error };
}; 