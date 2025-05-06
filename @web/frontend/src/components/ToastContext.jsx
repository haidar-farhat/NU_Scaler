import React, { createContext, useContext, useState, useCallback } from 'react';

const ToastContext = createContext();

export function ToastProvider({ children }) {
  const [toasts, setToasts] = useState([]);

  const showToast = useCallback((message, type = 'info', duration = 3000) => {
    const id = Date.now() + Math.random();
    setToasts(ts => [...ts, { id, message, type }]);
    setTimeout(() => {
      setToasts(ts => ts.filter(t => t.id !== id));
    }, duration);
  }, []);

  return (
    <ToastContext.Provider value={{ showToast }}>
      {children}
      <div style={{ position: 'fixed', top: 24, right: 24, zIndex: 9999 }}>
        {toasts.map(t => (
          <div key={t.id} style={{
            marginBottom: 12,
            padding: '12px 24px',
            borderRadius: 6,
            background: t.type === 'error' ? '#dc3545' : t.type === 'success' ? '#28a745' : '#333',
            color: '#fff',
            boxShadow: '0 2px 8px rgba(0,0,0,0.15)',
            minWidth: 200,
            fontWeight: 500,
          }}>
            {t.message}
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  );
}

export function useToast() {
  return useContext(ToastContext);
} 