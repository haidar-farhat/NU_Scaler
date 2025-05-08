import { useState } from 'react';

const AuthForm = ({ mode, onSubmit, loading, error }) => {
  const [form, setForm] = useState({ email: '', password: '', name: '', password_confirmation: '' });

  const handleChange = (e) => {
    setForm({ ...form, [e.target.name]: e.target.value });
  };

  const handleSubmit = (e) => {
    e.preventDefault();
    onSubmit(form);
  };

  return (
    <form onSubmit={handleSubmit} className="bg-white p-6 rounded-lg shadow max-w-md mx-auto">
      {mode === 'register' && (
        <div className="mb-4">
          <label className="block mb-1">Name</label>
          <input name="name" value={form.name} onChange={handleChange} required className="input w-full border px-3 py-2 rounded" />
        </div>
      )}
      <div className="mb-4">
        <label className="block mb-1">Email</label>
        <input name="email" type="email" value={form.email} onChange={handleChange} required className="input w-full border px-3 py-2 rounded" />
      </div>
      <div className="mb-4">
        <label className="block mb-1">Password</label>
        <input name="password" type="password" value={form.password} onChange={handleChange} required className="input w-full border px-3 py-2 rounded" />
      </div>
      {mode === 'register' && (
        <div className="mb-4">
          <label className="block mb-1">Confirm Password</label>
          <input name="password_confirmation" type="password" value={form.password_confirmation} onChange={handleChange} required className="input w-full border px-3 py-2 rounded" />
        </div>
      )}
      {error && <div className="text-red-600 mb-2">{error}</div>}
      <button type="submit" disabled={loading} className="btn-primary w-full mt-4 bg-indigo-600 text-white py-2 rounded">
        {loading ? 'Loading...' : mode === 'login' ? 'Login' : 'Register'}
      </button>
    </form>
  );
};

export default AuthForm; 