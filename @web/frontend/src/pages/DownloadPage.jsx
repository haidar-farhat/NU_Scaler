import { useState, useEffect } from 'react';
import { useSelector } from 'react-redux';
import { Link } from 'react-router-dom';
import api from '../api/axios';

const DownloadPage = () => {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [downloadInfo, setDownloadInfo] = useState(null);
  const [downloadInProgress, setDownloadInProgress] = useState(false);
  const { user } = useSelector((state) => state.auth);
  
  // Direct download URL - no need for API calls to get this
  const directDownloadUrl = 'http://localhost:8000/api/v1/download/direct';

  useEffect(() => {
    // Set some basic download info
    setDownloadInfo({
      version: '2.1.0',
      sizeMb: 223, // We know this from checking the file
      expiresAt: new Date(Date.now() + 86400000).toISOString() // 24 hours from now
    });
    setLoading(false);
  }, []);

  // Handle file download directly
  const handleDownload = async (e, platform) => {
    e.preventDefault();
    
    try {
      setDownloadInProgress(true);
      
      // Create a hidden anchor element to trigger the download
      const link = document.createElement('a');
      link.href = directDownloadUrl;
      link.download = 'NuScaler.exe';
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      
      // Wait a bit before resetting the download state
      setTimeout(() => {
        setDownloadInProgress(false);
      }, 1000);
    } catch (err) {
      setError(`Download failed: ${err.message}. Please try again.`);
      console.error('Download error:', err);
      setDownloadInProgress(false);
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600 mx-auto"></div>
          <p className="mt-4 text-indigo-600 font-semibold">Loading download information...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 p-4">
        <div className="max-w-md w-full bg-white shadow-lg rounded-lg p-6">
          <div className="text-red-600 mb-4">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-10 w-10 mx-auto mb-3"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
              />
            </svg>
            <h3 className="text-xl font-bold text-center">Error</h3>
          </div>
          <p className="text-gray-600 text-center">{error}</p>
          <button
            onClick={() => window.location.reload()}
            className="mt-6 w-full bg-indigo-600 text-white py-2 px-4 rounded hover:bg-indigo-700 transition duration-200"
          >
            Try Again
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-3xl mx-auto">
        <div className="bg-white shadow rounded-lg overflow-hidden">
          <div className="bg-indigo-600 px-6 py-4">
            <h2 className="text-2xl font-bold text-white">Download Nu Scaler</h2>
          </div>
          <div className="p-6">
            <div className="mb-8">
              <p className="text-gray-700 mb-4">
                Thank you for being a valued user of Nu Scaler. You have access to download our premium upscaling software.
              </p>
              {user ? (
                <>
                  <p className="text-gray-700 mb-2">
                    <strong>Your License:</strong> Personal Use
                  </p>
                  <p className="text-gray-700">
                    <strong>User:</strong> {user.name}
                  </p>
                </>
              ) : (
                <p className="text-gray-700 mb-2">
                  <strong>License Type:</strong> Free Trial 
                  <span className="ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                    <Link to="/login" className="text-blue-600 hover:text-blue-800 ml-1">Sign in</Link> for full version
                  </span>
                </p>
              )}
            </div>

            <div className="bg-gray-50 p-6 rounded-lg mb-8">
              <h3 className="text-lg font-semibold text-gray-900 mb-3">Available Downloads</h3>
              <div className="space-y-4">
                {downloadInProgress ? (
                  <div className="flex items-center justify-center p-4">
                    <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-indigo-600 mr-3"></div>
                    <p className="text-indigo-600">Download in progress...</p>
                  </div>
                ) : (
                  <>
                    <div className="flex items-center justify-between border-b pb-4">
                      <div>
                        <p className="font-medium">Nu Scaler for Windows</p>
                        <p className="text-sm text-gray-500">v{downloadInfo?.version || '2.1.0'} (64-bit)</p>
                        {downloadInfo?.sizeMb && (
                          <p className="text-sm text-gray-500">{downloadInfo.sizeMb} MB</p>
                        )}
                      </div>
                      <button
                        onClick={(e) => handleDownload(e, 'windows')}
                        className="bg-indigo-600 text-white py-2 px-4 rounded hover:bg-indigo-700 transition duration-200"
                      >
                        Download
                      </button>
                    </div>
                    <div className="flex items-center justify-between border-b pb-4 opacity-60">
                      <div>
                        <p className="font-medium">Nu Scaler for macOS</p>
                        <p className="text-sm text-gray-500">v{downloadInfo?.version || '2.1.0'} (Universal)</p>
                      </div>
                      <button
                        disabled
                        className="bg-gray-300 text-gray-600 py-2 px-4 rounded cursor-not-allowed"
                      >
                        Coming Soon
                      </button>
                    </div>
                    <div className="flex items-center justify-between opacity-60">
                      <div>
                        <p className="font-medium">Nu Scaler for Linux</p>
                        <p className="text-sm text-gray-500">v{downloadInfo?.version || '2.1.0'} (.deb package)</p>
                      </div>
                      <button
                        disabled
                        className="bg-gray-300 text-gray-600 py-2 px-4 rounded cursor-not-allowed"
                      >
                        Coming Soon
                      </button>
                    </div>
                  </>
                )}
              </div>
            </div>

            <div className="bg-yellow-50 border-l-4 border-yellow-400 p-4 mb-6">
              <div className="flex">
                <div className="flex-shrink-0">
                  <svg
                    className="h-5 w-5 text-yellow-400"
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                  >
                    <path
                      fillRule="evenodd"
                      d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
                      clipRule="evenodd"
                    />
                  </svg>
                </div>
                <div className="ml-3">
                  <p className="text-sm text-yellow-700">
                    Download size is approximately 223MB. Make sure you have a stable internet connection.
                  </p>
                </div>
              </div>
            </div>

            <div>
              <h3 className="text-lg font-semibold text-gray-900 mb-3">Installation Instructions</h3>
              <p className="text-gray-700 mb-2">
                1. Download the version for your operating system.
              </p>
              <p className="text-gray-700 mb-2">
                2. Run the installer and follow the on-screen instructions.
              </p>
              <p className="text-gray-700 mb-2">
                3. The software will automatically activate with your account credentials.
              </p>
              <p className="text-gray-700">
                4. For any installation issues, please refer to our <a href="#" className="text-indigo-600 hover:text-indigo-800">documentation</a> or <a href="#" className="text-indigo-600 hover:text-indigo-800">contact support</a>.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default DownloadPage; 