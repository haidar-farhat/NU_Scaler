import { useState, useEffect } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { Link, useNavigate, useLocation } from 'react-router-dom';
import { login, clearError } from './authSlice';
import AuthForm from './AuthForm';
import axios from 'axios';

const LoginPage = () => {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [csrfLoaded, setCsrfLoaded] = useState(false);
  const [loginAttempts, setLoginAttempts] = useState(0);
  const dispatch = useDispatch();
  const navigate = useNavigate();
  const location = useLocation();
  
  // Get auth state from Redux
  const { loading, error, isAuthenticated, user } = useSelector((state) => state.auth);
  
  // Get the redirect path if user was redirected from a protected route
  const from = location.state?.from?.pathname || '/';
  
  // Get CSRF cookie on component mount
  useEffect(() => {
    const fetchCsrfToken = async () => {
      try {
        console.log('Fetching CSRF cookie...');
        await axios.get('http://localhost:8000/sanctum/csrf-cookie', { 
          withCredentials: true
        });
        console.log('CSRF cookie set successfully');
        setCsrfLoaded(true);
      } catch (err) {
        console.error('Failed to fetch CSRF cookie:', err);
      }
    };
    
    fetchCsrfToken();
  }, []);
  
  useEffect(() => {
    console.log('Auth state changed:', { isAuthenticated, user });
    if (isAuthenticated && user) {
      console.log('User is authenticated, redirecting to:', from);
      console.log('User details:', user);
      
      // Check if the user is an admin and log that information
      if (user.is_admin) {
        console.log('User has admin privileges');
      } else {
        console.log('User does not have admin privileges');
      }
      
      navigate(from, { replace: true });
    }
  }, [isAuthenticated, user, navigate, from]);

  const handleSubmit = async (form) => {
    try {
      console.log('Attempting login with:', { email: form.email });
      setLoginAttempts(prev => prev + 1);
      
      const result = await dispatch(login({ email: form.email, password: form.password })).unwrap();
      console.log('Login result:', result);
      
      if (result) {
        navigate(from, { replace: true });
      }
    } catch (err) {
      // Error is handled in the authSlice
      console.error('Login failed:', err);
    }
  };
  
  // Helper to determine the appropriate error message and styling
  const getErrorDisplay = () => {
    if (!error || loginAttempts === 0) return null;
    
    // Check if the error is an account disabled error
    const isAccountDisabled = typeof error === 'object' && error.account_disabled;
    const errorMessage = typeof error === 'object' ? error.message : error;
    
    return (
      <div className={`border px-4 py-3 rounded relative max-w-md mx-auto mb-4 ${
        isAccountDisabled ? 'bg-yellow-100 border-yellow-400 text-yellow-800' : 'bg-red-100 border-red-400 text-red-700'
      }`}>
        <strong className="font-bold">{isAccountDisabled ? 'Account Deactivated: ' : 'Error: '}</strong>
        <span className="block sm:inline">{errorMessage}</span>
        {isAccountDisabled && (
          <p className="mt-2">Please contact an administrator to reactivate your account.</p>
        )}
      </div>
    );
  };
  
  return (
    <>
      {getErrorDisplay()}
      <AuthForm mode="login" onSubmit={handleSubmit} loading={loading} error={error} />
    </>
  );
};

export default LoginPage; 