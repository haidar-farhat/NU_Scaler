import React from 'react';
import { createBrowserRouter, RouterProvider } from 'react-router-dom';
import { ToastProvider } from './components/ToastContext';
import routes from './router/routes';

const router = createBrowserRouter(routes, {
  future: {
    v7_startTransition: true,
    v7_relativeSplatPath: true
  }
});

function App() {
  return (
    <ToastProvider>
      <RouterProvider router={router} />
    </ToastProvider>
  );
}

export default App;
