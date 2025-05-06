const ReviewsTable = ({ reviews }) => (
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
);
export default ReviewsTable; 