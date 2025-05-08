import { useState } from 'react';
import { submitReview } from './useFeedbackApi';

const ReviewForm = () => {
  const [rating, setRating] = useState(5);
  const [comment, setComment] = useState('');
  const [loading, setLoading] = useState(false);
  const [success, setSuccess] = useState(false);
  const [error, setError] = useState('');

  const handleSubmit = async (e) => {
    e.preventDefault();
    setLoading(true);
    setError('');
    setSuccess(false);
    try {
      await submitReview({ rating, comment });
      setSuccess(true);
      setComment('');
      setRating(5);
    } catch (err) {
      setError(err.message || 'Failed to submit review');
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="bg-white p-6 rounded shadow max-w-md mx-auto mt-8">
      <h2 className="text-xl font-bold mb-4">Leave a Review</h2>
      <div className="mb-4">
        <label className="block mb-1">Rating</label>
        <select value={rating} onChange={e => setRating(Number(e.target.value))} className="w-full border rounded px-3 py-2">
          {[5,4,3,2,1].map(n => <option key={n} value={n}>{n} Star{n > 1 ? 's' : ''}</option>)}
        </select>
      </div>
      <div className="mb-4">
        <label className="block mb-1">Comment</label>
        <textarea value={comment} onChange={e => setComment(e.target.value)} required className="w-full border rounded px-3 py-2" />
      </div>
      {error && <div className="text-red-600 mb-2">{error}</div>}
      {success && <div className="text-green-600 mb-2">Thank you for your review!</div>}
      <button type="submit" disabled={loading} className="bg-indigo-600 text-white px-4 py-2 rounded w-full">
        {loading ? 'Submitting...' : 'Submit Review'}
      </button>
    </form>
  );
};
export default ReviewForm; 