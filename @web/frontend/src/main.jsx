import React from 'react'
import ReactDOM from 'react-dom/client'
import './index.css'

// Temporary component until dependencies are installed properly
const TemporaryApp = () => {
  return (
    <div className="p-4">
      <h1 className="text-2xl font-bold">NU_Scaler Temporary Page</h1>
      <p className="mt-2">The application is currently unavailable due to dependency issues. Please try again later.</p>
      <div className="mt-2">
        <h2 className="font-bold">Troubleshooting:</h2>
        <ul style={{ listStyle: 'disc', paddingLeft: '20px' }}>
          <li>Check internet connection</li>
          <li>Try changing npm registry (npm config set registry https://registry.npmjs.org/)</li>
          <li>Run npm install with --force or --legacy-peer-deps flag</li>
        </ul>
      </div>
    </div>
  )
}

ReactDOM.createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <TemporaryApp />
  </React.StrictMode>,
)
