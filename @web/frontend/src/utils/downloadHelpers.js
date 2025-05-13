/**
 * Helper functions for handling file downloads
 */

/**
 * Download a file from the given URL
 * @param {string} url - The URL to download the file from
 */
export const downloadFile = async (url) => {
  try {
    const response = await fetch(url, {
      method: 'GET',
      credentials: 'include', // Include cookies
      headers: {
        'Authorization': `Bearer ${localStorage.getItem('token')}` 
      }
    });
    
    if (!response.ok) {
      throw new Error(`Download failed with status: ${response.status}`);
    }
    
    // Get filename from Content-Disposition header or default to NuScaler.exe
    const contentDisposition = response.headers.get('Content-Disposition');
    let filename = 'NuScaler.exe';
    if (contentDisposition) {
      const matches = /filename="([^"]+)"/.exec(contentDisposition);
      if (matches && matches[1]) {
        filename = matches[1];
      }
    }
    
    // Convert the response to a blob
    const blob = await response.blob();
    
    // Create an object URL for the blob
    const url = window.URL.createObjectURL(blob);
    
    // Create a temporary anchor element to trigger the download
    const a = document.createElement('a');
    a.style.display = 'none';
    a.href = url;
    a.download = filename;
    
    // Add to the DOM, click it, and then remove it
    document.body.appendChild(a);
    a.click();
    window.URL.revokeObjectURL(url);
    document.body.removeChild(a);
    
    return true;
  } catch (error) {
    console.error('File download error:', error);
    throw error;
  }
}; 