const SummaryCards = ({ title, value, icon }) => (
  <div className="summary-card">
    <div className="summary-card-icon">{icon}</div>
    <div className="summary-card-title">{title}</div>
    <div className="summary-card-value">{value}</div>
  </div>
);

export default SummaryCards; 