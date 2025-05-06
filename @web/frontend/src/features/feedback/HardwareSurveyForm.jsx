import { useState } from 'react';
import { submitSurvey } from './useFeedbackApi';

const HardwareSurveyForm = () => {
  const [form, setForm] = useState({
    cpu_model: '',
    gpu_model: '',
    ram_size: '',
    os: '',
    resolution: '',
    monitor_refresh_rate: '',
    additional_info: '',
  });
  const [loading, setLoading] = useState(false);
  const [success, setSuccess] = useState(false);
  const [error, setError] = useState('');

  const handleChange = (e) => {
    setForm({ ...form, [e.target.name]: e.target.value });
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    setLoading(true);
    setError('');
    setSuccess(false);
    try {
      await submitSurvey(form);
      setSuccess(true);
      setForm({
        cpu_model: '', gpu_model: '', ram_size: '', os: '', resolution: '', monitor_refresh_rate: '', additional_info: '',
      });
    } catch (err) {
      setError(err.message || 'Failed to submit survey');
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="bg-white p-6 rounded shadow max-w-md mx-auto mt-8">
      <h2 className="text-xl font-bold mb-4">Hardware Survey</h2>
      <div className="mb-4">
        <label className="block mb-1">CPU Model</label>
        <input name="cpu_model" value={form.cpu_model} onChange={handleChange} className="w-full border rounded px-3 py-2" />
      </div>
      <div className="mb-4">
        <label className="block mb-1">GPU Model</label>
        <input name="gpu_model" value={form.gpu_model} onChange={handleChange} className="w-full border rounded px-3 py-2" />
      </div>
      <div className="mb-4">
        <label className="block mb-1">RAM Size (GB)</label>
        <input name="ram_size" value={form.ram_size} onChange={handleChange} className="w-full border rounded px-3 py-2" />
      </div>
      <div className="mb-4">
        <label className="block mb-1">Operating System</label>
        <input name="os" value={form.os} onChange={handleChange} className="w-full border rounded px-3 py-2" />
      </div>
      <div className="mb-4">
        <label className="block mb-1">Resolution</label>
        <input name="resolution" value={form.resolution} onChange={handleChange} className="w-full border rounded px-3 py-2" />
      </div>
      <div className="mb-4">
        <label className="block mb-1">Monitor Refresh Rate (Hz)</label>
        <input name="monitor_refresh_rate" value={form.monitor_refresh_rate} onChange={handleChange} className="w-full border rounded px-3 py-2" />
      </div>
      <div className="mb-4">
        <label className="block mb-1">Additional Info</label>
        <textarea name="additional_info" value={form.additional_info} onChange={handleChange} className="w-full border rounded px-3 py-2" />
      </div>
      {error && <div className="text-red-600 mb-2">{error}</div>}
      {success && <div className="text-green-600 mb-2">Thank you for your submission!</div>}
      <button type="submit" disabled={loading} className="bg-indigo-600 text-white px-4 py-2 rounded w-full">
        {loading ? 'Submitting...' : 'Submit Survey'}
      </button>
    </form>
  );
};
export default HardwareSurveyForm; 