import { useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { Link, useNavigate, useLocation } from 'react-router-dom';
import { login, clearError } from './authSlice';
import AuthForm from './AuthForm';

const LoginPage = () => {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const dispatch = useDispatch();
  const navigate = useNavigate();
  const location = useLocation();
  
  // Get auth state from Redux
  const { loading, error } = useSelector((state) => state.auth);
  
  // Get the redirect path if user was redirected from a protected route
  const from = location.state?.from?.pathname || '/';
  
  const handleSubmit = async (form) => {
    try {
      const result = await dispatch(login({ email: form.email, password: form.password })).unwrap();
      if (result) {
        navigate(from, { replace: true });
      }
    } catch (err) {
      // Error is handled in the authSlice
      console.error('Login failed:', err);
    }
  };
  
  return (
    <AuthForm mode="login" onSubmit={handleSubmit} loading={loading} error={error} />
  );
};

export default LoginPage; 