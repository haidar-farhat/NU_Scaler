import React, { useState } from 'react';

const ReviewsTable = ({ reviews, meta = {}, onFilter, onPageChange, loading }) => {
  const [search, setSearch] = useState('');
  const [rating, setRating] = useState('');
  const [fromDate, setFromDate] = useState('');
  const [toDate, setToDate] = useState('');

  const handleFilter = (e) => {
    e.preventDefault();
    onFilter({ search, rating, from_date: fromDate, to_date: toDate, page: 1 });
  };

  return (
    <div>
      <form onSubmit={handleFilter} style={{ display: 'flex', gap: 8, marginBottom: 12, flexWrap: 'wrap' }}>
        <input value={search} onChange={e => setSearch(e.target.value)} placeholder="Search comment..." style={{ padding: 4 }} />
        <select value={rating} onChange={e => setRating(e.target.value)} style={{ padding: 4 }}>
          <option value="">All Ratings</option>
          {[1,2,3,4,5].map(r => <option key={r} value={r}>{r}</option>)}
        </select>
        <input type="date" value={fromDate} onChange={e => setFromDate(e.target.value)} style={{ padding: 4 }} />
        <input type="date" value={toDate} onChange={e => setToDate(e.target.value)} style={{ padding: 4 }} />
        <button type="submit" style={{ padding: '4px 12px' }}>Filter</button>
      </form>
      <table className="min-w-full bg-white rounded shadow mt-4">
        <thead>
          <tr>
            <th className="p-2">Rating</th>
            <th className="p-2">Comment</th>
            <th className="p-2">Date</th>
          </tr>
        </thead>
        <tbody>
          {reviews.map((r) => (
            <tr key={r.id}>
              <td className="p-2">{r.rating}</td>
              <td className="p-2">{r.comment.slice(0, 40)}...</td>
              <td className="p-2">{new Date(r.created_at).toLocaleDateString()}</td>
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
export default ReviewsTable; 