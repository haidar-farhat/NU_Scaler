import React from 'react';
import '../../styles/admin.css';

const SummaryCards = ({ title, value, icon }) => {
  return (
    <div className="summary-card group">
      <div className="summary-card-icon">
        {icon}
      </div>
      <h3 className="summary-card-title">{title}</h3>
      <p className="summary-card-value">{value}</p>
      <div className="absolute inset-0 bg-gradient-to-br from-indigo-500/10 to-blue-500/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
    </div>
  );
};

export default SummaryCards; 