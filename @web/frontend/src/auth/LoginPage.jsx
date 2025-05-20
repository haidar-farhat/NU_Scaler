import { useState, useEffect } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { Link, useNavigate, useLocation } from 'react-router-dom';
import { login, clearError } from './authSlice';
import AuthForm from './AuthForm';

const LoginPage = () => {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [loginAttempts, setLoginAttempts] = useState(0);
  const [loginError, setLoginError] = useState(null); // Local error state as a backup
  const dispatch = useDispatch();
  const navigate = useNavigate();
  const location = useLocation();
  
  // Get auth state from Redux
  const { loading, error, isAuthenticated, user } = useSelector((state) => state.auth);
  
  // Get the redirect path if user was redirected from a protected route
  const from = location.state?.from?.pathname || '/';
  
  useEffect(() => {
    if (isAuthenticated && user) {
      navigate(from, { replace: true });
    }
  }, [isAuthenticated, user, navigate, from]);

  const handleSubmit = async (form) => {
    setLoginAttempts(prev => prev + 1);
    setLoginError(null); // Clear previous local errors
    try {
      const result = await dispatch(login({ email: form.email, password: form.password })).unwrap();
      if (result) {
        navigate(from, { replace: true });
      }
    } catch (err) {
      setLoginError(err);
    }
  };

  const getErrorDisplay = () => {
    if (loginAttempts === 0) return null;
    const currentError = error || loginError;
    if (!currentError) return null;
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