const BugReportsTable = ({ bugReports }) => (
  <table className="min-w-full bg-white rounded shadow mt-4">
    <thead>
      <tr>
        <th className="p-2">Severity</th>
        <th className="p-2">Description</th>
        <th className="p-2">Date</th>
      </tr>
    </thead>
    <tbody>
      {bugReports.map((b) => (
        <tr key={b.id}>
          <td className="p-2">{b.severity}</td>
          <td className="p-2">{b.description.slice(0, 40)}...</td>
          <td className="p-2">{new Date(b.created_at).toLocaleDateString()}</td>
        </tr>
      ))}
    </tbody>
  </table>
);
export default BugReportsTable; 