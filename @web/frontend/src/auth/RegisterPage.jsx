import { useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { Link, useNavigate } from 'react-router-dom';
import { register, clearError } from './authSlice';
import AuthForm from './AuthForm';

const RegisterPage = () => {
  const dispatch = useDispatch();
  const navigate = useNavigate();
  const { loading, error } = useSelector((state) => state.auth);
  const [passwordError, setPasswordError] = useState('');

  const handleSubmit = async (form) => {
    if (form.password !== form.password_confirmation) {
      setPasswordError('Passwords do not match');
      return;
    }
    setPasswordError('');
    try {
      const result = await dispatch(register(form)).unwrap();
      if (result) {
        navigate('/', { replace: true });
      }
    } catch (err) {
      // Error is handled in the authSlice
      console.error('Registration failed:', err);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-md w-full space-y-8">
        <div>
          <h2 className="mt-6 text-center text-3xl font-extrabold text-gray-900">
            Create your account
          </h2>
        </div>
        
        {error && (
          <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative">
            <span className="block sm:inline">{error}</span>
            <span 
              className="absolute top-0 bottom-0 right-0 px-4 py-3 cursor-pointer"
              onClick={() => dispatch(clearError())}
            >
              <span className="sr-only">Close</span>
              &times;
            </span>
          </div>
        )}
        
        <AuthForm mode="register" onSubmit={handleSubmit} loading={loading} error={error || passwordError} />
        
        <div className="text-sm text-center">
          <Link 
            to="/login" 
            className="font-medium text-indigo-600 hover:text-indigo-500"
          >
            Already have an account? Sign in
          </Link>
        </div>
      </div>
    </div>
  );
};

export default RegisterPage; 