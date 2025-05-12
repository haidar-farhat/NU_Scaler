import React, { useEffect, useRef } from 'react';
import Chart from 'chart.js/auto';
import '../../styles/admin.css';

const UserGrowthChart = ({ data }) => {
  const chartRef = useRef(null);
  const chartInstance = useRef(null);

  useEffect(() => {
    if (!data || data.length === 0) return;

    // Destroy existing chart if it exists
    if (chartInstance.current) {
      chartInstance.current.destroy();
    }

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

    // Cleanup chart on unmount
    return () => {
      if (chartInstance.current) {
        chartInstance.current.destroy();
      }
    };
  }, [data]);

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

  if (!data || data.length === 0) {
    return (
      <div className="flex items-center justify-center h-64">
        <p className="text-slate-500">No user growth data available</p>
      </div>
    );
  }

  return (
    <div className="chart-container h-64">
      <canvas ref={chartRef} />
    </div>
  );
};

export default UserGrowthChart; 