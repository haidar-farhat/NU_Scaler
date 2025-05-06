import React, { useState } from 'react';

const ALL_COLUMNS = [
  { key: 'os', label: 'OS' },
  { key: 'gpu_model', label: 'GPU' },
  { key: 'ram_size', label: 'RAM' },
  { key: 'date', label: 'Date' },
  { key: 'additional_info', label: 'Info' },
];

const SurveysTable = ({ surveys, meta = {}, onFilter, onPageChange, loading }) => {
  const [search, setSearch] = useState('');
  const [os, setOs] = useState('');
  const [gpu, setGpu] = useState('');
  const [minRam, setMinRam] = useState('');
  const [fromDate, setFromDate] = useState('');
  const [toDate, setToDate] = useState('');
  const [visibleCols, setVisibleCols] = useState(ALL_COLUMNS.map(c => c.key));
  const [showCols, setShowCols] = useState(false);
  const [sortBy, setSortBy] = useState('');
  const [sortDir, setSortDir] = useState('asc');

  const handleFilter = (e) => {
    e.preventDefault();
    onFilter({ search, os, gpu_model: gpu, min_ram: minRam, from_date: fromDate, to_date: toDate, page: 1, sort_by: sortBy, sort_dir: sortDir });
  };

  const handleSort = (col) => {
    let dir = 'asc';
    if (sortBy === col) dir = sortDir === 'asc' ? 'desc' : 'asc';
    setSortBy(col);
    setSortDir(dir);
    onFilter({ search, os, gpu_model: gpu, min_ram: minRam, from_date: fromDate, to_date: toDate, page: 1, sort_by: col, sort_dir: dir });
  };

  const toggleCol = (col) => {
    setVisibleCols(cols => cols.includes(col) ? cols.filter(c => c !== col) : [...cols, col]);
  };

  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 8 }}>
        <button type="button" onClick={() => setShowCols(v => !v)} style={{ padding: '4px 12px' }}>Columns</button>
        {showCols && (
          <div style={{ background: '#fff', border: '1px solid #ddd', borderRadius: 4, padding: 8, position: 'absolute', zIndex: 10 }}>
            {ALL_COLUMNS.map(col => (
              <label key={col.key} style={{ display: 'block', marginBottom: 4 }}>
                <input type="checkbox" checked={visibleCols.includes(col.key)} onChange={() => toggleCol(col.key)} /> {col.label}
              </label>
            ))}
          </div>
        )}
      </div>
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
            {ALL_COLUMNS.filter(col => visibleCols.includes(col.key)).map(col => (
              <th
                key={col.key}
                className="p-2 cursor-pointer select-none"
                onClick={() => handleSort(col.key)}
                style={{ userSelect: 'none' }}
              >
                {col.label}
                {sortBy === col.key && (sortDir === 'asc' ? ' ▲' : ' ▼')}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {surveys.map((s) => (
            <tr key={s.id}>
              {visibleCols.includes('os') && <td className="p-2">{s.os}</td>}
              {visibleCols.includes('gpu_model') && <td className="p-2">{s.gpu_model}</td>}
              {visibleCols.includes('ram_size') && <td className="p-2">{s.ram_size} GB</td>}
              {visibleCols.includes('date') && <td className="p-2">{new Date(s.created_at).toLocaleDateString()}</td>}
              {visibleCols.includes('additional_info') && <td className="p-2">{s.additional_info?.slice(0, 40) || ''}</td>}
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