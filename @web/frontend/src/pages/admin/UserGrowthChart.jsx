import React, { useMemo } from 'react';
import '../../styles/admin.css';

const UserGrowthChart = ({ data }) => {
  // Safely process data for the chart
  const chartData = useMemo(() => {
    try {
      // Handle null, undefined or non-array data
      if (!data || !Array.isArray(data) || data.length === 0) {
        return { labels: [], values: [] };
      }
      
      // Safely filter out invalid entries and sort by date
      const sortedData = [...data].filter(item => item && item.created_at)
        .sort((a, b) => {
          try {
            return new Date(a.created_at) - new Date(b.created_at);
          } catch (e) {
            return 0; // In case of invalid dates
          }
        });

      // Group by month or week
      const groupedData = {};
      sortedData.forEach(item => {
        try {
          const date = new Date(item.created_at);
          // Check if date is valid
          if (isNaN(date.getTime())) return;
          
          const month = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}`;
          
          if (!groupedData[month]) {
            groupedData[month] = 0;
          }
          groupedData[month]++;
        } catch (e) {
          // Skip invalid entries
          console.error('Error processing item:', e);
        }
      });

      // Format for chart - handle empty case
      const months = Object.keys(groupedData);
      if (months.length === 0) {
        return { labels: [], values: [] };
      }

      const labels = months.map(month => {
        try {
          const [year, monthNum] = month.split('-');
          return `${monthNum}/${year}`;
        } catch (e) {
          return month; // Fallback to the raw month string
        }
      });
      
      const values = Object.values(groupedData);

      return { labels, values };
    } catch (error) {
      console.error('Error processing chart data:', error);
      return { labels: [], values: [] };
    }
  }, [data]);

  // Safely calculate chart dimensions and scales
  const maxValue = useMemo(() => {
    try {
      if (!chartData.values || !Array.isArray(chartData.values) || chartData.values.length === 0) {
        return 1;
      }
      return Math.max(...chartData.values, 1);
    } catch (e) {
      console.error('Error calculating max value:', e);
      return 1;
    }
  }, [chartData.values]);
  
  const barCount = chartData.labels?.length || 0;
  
  // Render empty state if no data or error
  if (!chartData || !chartData.labels || !chartData.values || 
      !Array.isArray(chartData.labels) || !Array.isArray(chartData.values) ||
      chartData.labels.length === 0) {
    return (
      <div className="flex items-center justify-center h-64">
        <p className="text-slate-500">No user growth data available</p>
      </div>
    );
  }

  // Safe render function
  try {
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
              // Safely calculate height percentage
              const value = chartData.values[index] || 0;
              const heightPercent = maxValue > 0 ? (value / maxValue) * 100 : 0;
              
              return (
                <div 
                  key={`bar-${index}`}
                  className="group flex flex-col items-center"
                  style={{ width: `${barCount > 0 ? 100 / (barCount + 1) : 100}%` }}
                >
                  {/* Bar tooltip */}
                  <div className="opacity-0 group-hover:opacity-100 transition-opacity duration-200 mb-2 px-2 py-1 bg-white rounded shadow-md text-xs">
                    {value} users
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
                  <div className="mt-2 text-xs text-slate-600">{label || ''}</div>
                </div>
              );
            })}
          </div>
        </div>
      </div>
    );
  } catch (error) {
    console.error('Error rendering UserGrowthChart:', error);
    return (
      <div className="flex items-center justify-center h-64">
        <p className="text-slate-500">Error displaying chart. Please try again later.</p>
      </div>
    );
  }
};

export default UserGrowthChart; 