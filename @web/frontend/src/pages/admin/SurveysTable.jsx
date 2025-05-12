import React, { useState } from 'react';
import '../../styles/admin.css';

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
      <div className="admin-filters">
        <div className="column-selector">
          <button
            type="button"
            className="admin-button-secondary"
            onClick={() => setShowCols(v => !v)}
          >
            Columns
          </button>
          {showCols && (
            <div className="column-selector-menu">
              {ALL_COLUMNS.map(col => (
                <div key={col.key} className="column-selector-item">
                  <label className="flex items-center">
                    <input
                      type="checkbox"
                      checked={visibleCols.includes(col.key)}
                      onChange={() => toggleCol(col.key)}
                      className="mr-2"
                    />
                    {col.label}
                  </label>
                </div>
              ))}
            </div>
          )}
        </div>

        <form onSubmit={handleFilter} className="flex flex-wrap gap-4">
          <input
            value={search}
            onChange={e => setSearch(e.target.value)}
            placeholder="Search info..."
            className="admin-filter-input"
          />
          <input
            value={os}
            onChange={e => setOs(e.target.value)}
            placeholder="OS"
            className="admin-filter-input"
          />
          <input
            value={gpu}
            onChange={e => setGpu(e.target.value)}
            placeholder="GPU Model"
            className="admin-filter-input"
          />
          <input
            type="number"
            value={minRam}
            onChange={e => setMinRam(e.target.value)}
            placeholder="Min RAM (GB)"
            className="admin-filter-input w-32"
          />
          <input
            type="date"
            value={fromDate}
            onChange={e => setFromDate(e.target.value)}
            className="admin-filter-input"
          />
          <input
            type="date"
            value={toDate}
            onChange={e => setToDate(e.target.value)}
            className="admin-filter-input"
          />
          <button type="submit" className="admin-button">
            Filter
          </button>
        </form>
      </div>

      <div className="admin-table-container">
        <table className="admin-table">
          <thead className="admin-table-header">
            <tr>
              {ALL_COLUMNS.map(col => (
                visibleCols.includes(col.key) && (
                  <th
                    key={col.key}
                    onClick={() => handleSort(col.key)}
                    className="cursor-pointer hover:bg-gray-100"
                  >
                    {col.label}
                    {sortBy === col.key && (
                      <span className="ml-1">{sortDir === 'asc' ? '↑' : '↓'}</span>
                    )}
                  </th>
                )
              ))}
            </tr>
          </thead>
          <tbody className="admin-table-body">
            {surveys.map(survey => (
              <tr key={survey.id} className="admin-table-row">
                {ALL_COLUMNS.map(col => (
                  visibleCols.includes(col.key) && (
                    <td key={col.key} className="admin-table-cell">
                      {survey[col.key]}
                    </td>
                  )
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {meta.total_pages > 1 && (
        <div className="admin-pagination">
          <button
            className="admin-pagination-button"
            onClick={() => onPageChange(meta.current_page - 1)}
            disabled={meta.current_page === 1}
          >
            Previous
          </button>
          <span className="text-sm text-gray-700">
            Page {meta.current_page} of {meta.total_pages}
          </span>
          <button
            className="admin-pagination-button"
            onClick={() => onPageChange(meta.current_page + 1)}
            disabled={meta.current_page === meta.total_pages}
          >
            Next
          </button>
        </div>
      )}
    </div>
  );
};

export default SurveysTable; 