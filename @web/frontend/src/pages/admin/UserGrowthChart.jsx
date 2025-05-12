import React, { useEffect, useRef, useState } from 'react';
import '../../styles/admin.css';

// Fallback chart component that doesn't require Chart.js
const FallbackChart = ({ data }) => {
  // Process data similar to the chart component
  const processedData = (() => {
    if (!data || data.length === 0) return { labels: [], counts: [] };
    
    // Sort by date
    const sortedData = [...data].sort((a, b) => 
      new Date(a.created_at) - new Date(b.created_at)
    );
    
    // Group by month
    const groupedData = {};
    sortedData.forEach(item => {
      const date = new Date(item.created_at);
      const month = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}`;
      
      if (!groupedData[month]) {
        groupedData[month] = 0;
      }
      groupedData[month]++;
    });
    
    return {
      labels: Object.keys(groupedData).map(month => {
        const [year, monthNum] = month.split('-');
        return `${monthNum}/${year}`;
      }),
      counts: Object.values(groupedData)
    };
  })();
  
  // Find max value for scaling
  const maxValue = Math.max(...processedData.counts, 1);
  
  return (
    <div className="h-64 w-full">
      <div className="text-sm font-medium text-slate-700 mb-4">User Growth Trend</div>
      <div className="h-48 flex items-end space-x-2">
        {processedData.labels.map((label, index) => {
          const height = (processedData.counts[index] / maxValue) * 100;
          return (
            <div key={label} className="flex flex-col items-center flex-1">
              <div 
                className="w-full bg-indigo-500 rounded-t-lg transition-all duration-500"
                style={{ height: `${height}%` }}
              />
              <div className="text-xs mt-2 text-slate-600">{label}</div>
            </div>
          );
        })}
        {processedData.labels.length === 0 && (
          <div className="w-full h-full flex items-center justify-center text-slate-400">
            No data available
          </div>
        )}
      </div>
    </div>
  );
};

const UserGrowthChart = ({ data }) => {
  const chartRef = useRef(null);
  const chartInstance = useRef(null);
  const [chartError, setChartError] = useState(false);
  const [Chart, setChart] = useState(null);

  // Load Chart.js dynamically
  useEffect(() => {
    const loadChart = async () => {
      try {
        const chartModule = await import('chart.js/auto');
        setChart(chartModule.default);
      } catch (err) {
        console.error("Failed to load Chart.js:", err);
        setChartError(true);
      }
    };
    
    loadChart();
  }, []);

  useEffect(() => {
    if (!Chart || !data || data.length === 0 || chartError) return;

    // Destroy existing chart if it exists
    if (chartInstance.current) {
      chartInstance.current.destroy();
    }

    try {
      // Prepare the data
      const chartData = processData(data);

      // Get the context
      const ctx = chartRef.current.getContext('2d');

      // Create gradient for the area
      const gradient = ctx.createLinearGradient(0, 0, 0, 400);
      gradient.addColorStop(0, 'rgba(99, 102, 241, 0.5)');
      gradient.addColorStop(1, 'rgba(99, 102, 241, 0.0)');

      // Create the chart
      chartInstance.current = new Chart(ctx, {
        type: 'line',
        data: {
          labels: chartData.labels,
          datasets: [
            {
              label: 'New Users',
              data: chartData.counts,
              backgroundColor: gradient,
              borderColor: 'rgb(99, 102, 241)',
              borderWidth: 3,
              pointBackgroundColor: 'rgb(99, 102, 241)',
              pointBorderColor: '#fff',
              pointBorderWidth: 2,
              pointRadius: 5,
              pointHoverRadius: 7,
              tension: 0.3,
              fill: true
            }
          ]
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          plugins: {
            legend: {
              display: false
            },
            tooltip: {
              backgroundColor: 'rgba(255, 255, 255, 0.9)',
              titleColor: '#1e293b',
              bodyColor: '#1e293b',
              borderColor: 'rgba(99, 102, 241, 0.1)',
              borderWidth: 1,
              padding: 12,
              boxPadding: 6,
              usePointStyle: true,
              titleFont: {
                size: 14,
                weight: 'bold'
              },
              bodyFont: {
                size: 12
              },
              callbacks: {
                title: function(tooltipItems) {
                  return tooltipItems[0].label;
                },
                label: function(context) {
                  return `New users: ${context.parsed.y}`;
                }
              }
            }
          },
          scales: {
            x: {
              grid: {
                color: 'rgba(226, 232, 240, 0.5)',
                borderDash: [5, 5]
              },
              ticks: {
                color: '#64748b',
                font: {
                  size: 11
                }
              }
            },
            y: {
              beginAtZero: true,
              grid: {
                color: 'rgba(226, 232, 240, 0.5)',
                borderDash: [5, 5]
              },
              ticks: {
                color: '#64748b',
                font: {
                  size: 11
                },
                precision: 0
              }
            }
          },
          interaction: {
            mode: 'index',
            intersect: false
          },
          animation: {
            duration: 1000,
            easing: 'easeOutQuart'
          }
        }
      });
    } catch (err) {
      console.error("Failed to create chart:", err);
      setChartError(true);
    }

    // Cleanup chart on unmount
    return () => {
      if (chartInstance.current) {
        chartInstance.current.destroy();
      }
    };
  }, [Chart, data]);

  // Process data to get labels and counts by week/month
  const processData = (data) => {
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
    const labels = Object.keys(groupedData);
    const counts = Object.values(groupedData);

    return {
      labels: labels.map(month => {
        const [year, monthNum] = month.split('-');
        return `${monthNum}/${year}`;
      }),
      counts
    };
  };

  // Show loading state when Chart.js is loading
  if (!Chart && !chartError) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="admin-loading-spinner" />
        <p className="ml-3 text-slate-500">Loading chart...</p>
      </div>
    );
  }

  // Show fallback chart if Chart.js failed to load
  if (chartError) {
    return <FallbackChart data={data} />;
  }

  // No data available
  if (!data || data.length === 0) {
    return (
      <div className="flex items-center justify-center h-64">
        <p className="text-slate-500">No user growth data available</p>
      </div>
    );
  }

  // Render the chart canvas when everything is ready
  return (
    <div className="chart-container h-64">
      <canvas ref={chartRef} />
    </div>
  );
};

export default UserGrowthChart; 