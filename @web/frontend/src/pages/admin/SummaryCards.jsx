const SummaryCards = ({ title, value, icon }) => (
  <div className="bg-white p-4 rounded shadow flex items-center gap-4">
    <div className="text-3xl text-indigo-600">{icon}</div>
    <div>
      <div className="text-lg font-semibold">{title}</div>
      <div className="text-2xl font-bold">{value}</div>
    </div>
  </div>
);
export default SummaryCards; 