import React, { useState } from 'react';

const BugReportsTable = ({ bugReports, meta = {}, onFilter, onPageChange, loading }) => {
  const [search, setSearch] = useState('');
  const [severity, setSeverity] = useState('');
  const [category, setCategory] = useState('');
  const [fromDate, setFromDate] = useState('');
  const [toDate, setToDate] = useState('');

  const handleFilter = (e) => {
    e.preventDefault();
    onFilter({ search, severity, category, from_date: fromDate, to_date: toDate, page: 1 });
  };

  return (
    <div>
      <form onSubmit={handleFilter} style={{ display: 'flex', gap: 8, marginBottom: 12, flexWrap: 'wrap' }}>
        <input value={search} onChange={e => setSearch(e.target.value)} placeholder="Search description..." style={{ padding: 4 }} />
        <select value={severity} onChange={e => setSeverity(e.target.value)} style={{ padding: 4 }}>
          <option value="">All Severities</option>
          {['critical','high','medium','low'].map(s => <option key={s} value={s}>{s}</option>)}
        </select>
        <input value={category} onChange={e => setCategory(e.target.value)} placeholder="Category" style={{ padding: 4 }} />
        <input type="date" value={fromDate} onChange={e => setFromDate(e.target.value)} style={{ padding: 4 }} />
        <input type="date" value={toDate} onChange={e => setToDate(e.target.value)} style={{ padding: 4 }} />
        <button type="submit" style={{ padding: '4px 12px' }}>Filter</button>
      </form>
      <table className="min-w-full bg-white rounded shadow mt-4">
        <thead>
          <tr>
            <th className="p-2">Severity</th>
            <th className="p-2">Category</th>
            <th className="p-2">Description</th>
            <th className="p-2">Date</th>
          </tr>
        </thead>
        <tbody>
          {bugReports.map((b) => (
            <tr key={b.id}>
              <td className="p-2">{b.severity}</td>
              <td className="p-2">{b.category}</td>
              <td className="p-2">{b.description.slice(0, 40)}...</td>
              <td className="p-2">{new Date(b.created_at).toLocaleDateString()}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <div style={{ display: 'flex', gap: 8, marginTop: 12, alignItems: 'center' }}>
        <button onClick={() => onPageChange(meta.current_page - 1)} disabled={meta.current_page <= 1 || loading}>Prev</button>
        <span>Page {meta.current_page || 1} of {meta.last_page || 1}</span>
        <button onClick={() => onPageChange(meta.current_page + 1)} disabled={meta.current_page >= meta.last_page || loading}>Next</button>
        <select value={meta.per_page || 15} onChange={e => onFilter({ per_page: e.target.value, page: 1 })}>
          {[10, 15, 25, 50, 100].map(n => <option key={n} value={n}>{n} / page</option>)}
        </select>
      </div>
    </div>
  );
};
export default BugReportsTable; 