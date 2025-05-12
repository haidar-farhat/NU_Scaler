import React, { useMemo } from 'react';
import '../../styles/admin.css';

const UserGrowthChart = ({ data }) => {
  // Process data for the chart
  const chartData = useMemo(() => {
    if (!data || data.length === 0) return { labels: [], values: [] };
    
    // Sort by date
    const sortedData = [...data].sort((a, b) => 
      new Date(a.created_at) - new Date(b.created_at)
    );

    // Group by month or week
    const groupedData = {};
    sortedData.forEach(item => {
      const date = new Date(item.created_at);
      const month = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}`;
      
      if (!groupedData[month]) {
        groupedData[month] = 0;
      }
      groupedData[month]++;
    });

    // Format for chart
    const labels = Object.keys(groupedData).map(month => {
      const [year, monthNum] = month.split('-');
      return `${monthNum}/${year}`;
    });
    
    const values = Object.values(groupedData);

    return { labels, values };
  }, [data]);

  // Calculate chart dimensions and scales
  const maxValue = Math.max(...chartData.values, 1);
  const barCount = chartData.labels.length;
  
  // No data available
  if (!data || data.length === 0) {
    return (
      <div className="flex items-center justify-center h-64">
        <p className="text-slate-500">No user growth data available</p>
      </div>
    );
  }

  return (
    <div className="chart-container h-64">
      <h3 className="text-md font-medium text-slate-700 mb-3">User Growth</h3>
      <div className="relative h-52 w-full">
        {/* Y-axis grid lines */}
        {[...Array(5)].map((_, i) => (
          <div 
            key={`grid-${i}`}
            className="absolute w-full border-t border-slate-200 left-0 z-0"
            style={{ 
              bottom: `${(i / 4) * 100}%`,
              borderStyle: i === 0 ? 'solid' : 'dashed'
            }}
          >
            <span className="absolute -left-6 -top-2 text-xs text-slate-500">
              {Math.round((maxValue * i) / 4)}
            </span>
          </div>
        ))}
        
        {/* Chart Bars */}
        <div className="absolute bottom-0 left-0 right-0 top-0 flex items-end justify-between px-6">
          {chartData.labels.map((label, index) => {
            const heightPercent = (chartData.values[index] / maxValue) * 100;
            return (
              <div 
                key={`bar-${index}`}
                className="group flex flex-col items-center"
                style={{ width: `${100 / (barCount + 1)}%` }}
              >
                {/* Bar tooltip */}
                <div className="opacity-0 group-hover:opacity-100 transition-opacity duration-200 mb-2 px-2 py-1 bg-white rounded shadow-md text-xs">
                  {chartData.values[index]} users
                </div>
                
                {/* Bar */}
                <div 
                  className="w-full max-w-[30px] bg-gradient-to-t from-indigo-500 to-indigo-400 rounded-t-md transition-all duration-500 ease-out"
                  style={{ 
                    height: `${heightPercent}%`,
                    minHeight: '4px' 
                  }}
                >
                  <div className="h-full w-full bg-indigo-300/30 rounded-t-md opacity-0 group-hover:opacity-100 transition-opacity duration-200"></div>
                </div>
                
                {/* X-axis label */}
                <div className="mt-2 text-xs text-slate-600">{label}</div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};

export default UserGrowthChart; 