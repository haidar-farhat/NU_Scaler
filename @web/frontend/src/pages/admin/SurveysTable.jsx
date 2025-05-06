import React, { useState } from 'react';

const SurveysTable = ({ surveys, meta = {}, onFilter, onPageChange, loading }) => {
  const [search, setSearch] = useState('');
  const [os, setOs] = useState('');
  const [gpu, setGpu] = useState('');
  const [minRam, setMinRam] = useState('');
  const [fromDate, setFromDate] = useState('');
  const [toDate, setToDate] = useState('');

  const handleFilter = (e) => {
    e.preventDefault();
    onFilter({ search, os, gpu_model: gpu, min_ram: minRam, from_date: fromDate, to_date: toDate, page: 1 });
  };

  return (
    <div>
      <form onSubmit={handleFilter} style={{ display: 'flex', gap: 8, marginBottom: 12, flexWrap: 'wrap' }}>
        <input value={search} onChange={e => setSearch(e.target.value)} placeholder="Search info..." style={{ padding: 4 }} />
        <input value={os} onChange={e => setOs(e.target.value)} placeholder="OS" style={{ padding: 4 }} />
        <input value={gpu} onChange={e => setGpu(e.target.value)} placeholder="GPU Model" style={{ padding: 4 }} />
        <input type="number" value={minRam} onChange={e => setMinRam(e.target.value)} placeholder="Min RAM (GB)" style={{ padding: 4, width: 100 }} />
        <input type="date" value={fromDate} onChange={e => setFromDate(e.target.value)} style={{ padding: 4 }} />
        <input type="date" value={toDate} onChange={e => setToDate(e.target.value)} style={{ padding: 4 }} />
        <button type="submit" style={{ padding: '4px 12px' }}>Filter</button>
      </form>
      <table className="min-w-full bg-white rounded shadow mt-4">
        <thead>
          <tr>
            <th className="p-2">OS</th>
            <th className="p-2">GPU</th>
            <th className="p-2">RAM</th>
            <th className="p-2">Date</th>
            <th className="p-2">Info</th>
          </tr>
        </thead>
        <tbody>
          {surveys.map((s) => (
            <tr key={s.id}>
              <td className="p-2">{s.os}</td>
              <td className="p-2">{s.gpu_model}</td>
              <td className="p-2">{s.ram_size} GB</td>
              <td className="p-2">{new Date(s.created_at).toLocaleDateString()}</td>
              <td className="p-2">{s.additional_info?.slice(0, 40) || ''}</td>
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
export default SurveysTable; 