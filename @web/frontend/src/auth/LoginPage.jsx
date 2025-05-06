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
  const [loginError, setLoginError] = useState(null); // Local error state as a backup
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
    setLoginAttempts(prev => prev + 1);
    setLoginError(null); // Clear previous local errors
    
    try {
      console.log('Attempting login with:', { email: form.email });
      
      const result = await dispatch(login({ email: form.email, password: form.password })).unwrap();
      console.log('Login result:', result);
      
      if (result) {
        navigate(from, { replace: true });
      }
    } catch (err) {
      console.error('Login failed:', err);
      // Store error in local state as a backup
      setLoginError(err);
    }
  };
  
  // Helper to determine the appropriate error message and styling
  const getErrorDisplay = () => {
    if (loginAttempts === 0) return null;
    
    // Use either Redux error or local error state
    const currentError = error || loginError;
    if (!currentError) return null;
    
    // Check if this might be a deactivated account message
    const errorStr = String(currentError);
    const isAccountDisabled = errorStr.toLowerCase().includes('deactivated') || 
                              errorStr.toLowerCase().includes('disabled');
    
    return (
      <div className={`border px-4 py-3 rounded relative max-w-md mx-auto mb-4 ${
        isAccountDisabled ? 'bg-yellow-100 border-yellow-400 text-yellow-800' : 'bg-red-100 border-red-400 text-red-700'
      }`}>
        <strong className="font-bold">{isAccountDisabled ? 'Account Deactivated: ' : 'Error: '}</strong>
        <span className="block sm:inline">{errorStr}</span>
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