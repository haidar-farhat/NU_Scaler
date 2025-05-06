import { useState } from 'react';
import { submitBug } from './useFeedbackApi';

const BugReportForm = () => {
  const [description, setDescription] = useState('');
  const [logFile, setLogFile] = useState(null);
  const [loading, setLoading] = useState(false);
  const [success, setSuccess] = useState(false);
  const [error, setError] = useState('');

  const handleSubmit = async (e) => {
    e.preventDefault();
    setLoading(true);
    setError('');
    setSuccess(false);
    try {
      await submitBug({ description, logFile });
      setSuccess(true);
      setDescription('');
      setLogFile(null);
    } catch (err) {
      setError(err.message || 'Failed to submit bug report');
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="bg-white p-6 rounded shadow max-w-md mx-auto mt-8">
      <h2 className="text-xl font-bold mb-4">Report a Bug</h2>
      <div className="mb-4">
        <label className="block mb-1">Description</label>
        <textarea value={description} onChange={e => setDescription(e.target.value)} required className="w-full border rounded px-3 py-2" />
      </div>
      <div className="mb-4">
        <label className="block mb-1">Log File (optional)</label>
        <input type="file" onChange={e => setLogFile(e.target.files[0])} className="w-full" />
      </div>
      {error && <div className="text-red-600 mb-2">{error}</div>}
      {success && <div className="text-green-600 mb-2">Thank you for your report!</div>}
      <button type="submit" disabled={loading} className="bg-indigo-600 text-white px-4 py-2 rounded w-full">
        {loading ? 'Submitting...' : 'Submit Bug Report'}
      </button>
    </form>
  );
};
export default BugReportForm; 