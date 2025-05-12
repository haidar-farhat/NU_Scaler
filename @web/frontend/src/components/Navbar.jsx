import React, { useState, useEffect } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { useSelector, useDispatch } from 'react-redux';
import { logout } from '../auth/authSlice';

const Navbar = () => {
  const { isAuthenticated, user } = useSelector((state) => state.auth);
  const dispatch = useDispatch();
  const location = useLocation();
  const [isOpen, setIsOpen] = useState(false);
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const handleScroll = () => {
      const isScrolled = window.scrollY > 10;
      if (isScrolled !== scrolled) {
        setScrolled(isScrolled);
      }
    };

    window.addEventListener('scroll', handleScroll);
    return () => {
      window.removeEventListener('scroll', handleScroll);
    };
  }, [scrolled]);

  // Close mobile menu when route changes
  useEffect(() => {
    setIsOpen(false);
  }, [location.pathname]);

  const handleLogout = () => {
    dispatch(logout());
  };

  const isActive = (path) => {
    return location.pathname === path;
  };

  return (
    <nav className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 ${
      scrolled 
        ? 'bg-white/80 backdrop-blur-lg shadow-lg py-3' 
        : 'bg-transparent py-4'
    }`}>
      <div className="container mx-auto px-4 md:px-6 flex items-center justify-between">
        {/* Logo */}
        <Link to="/" className="flex items-center">
          <span className="text-2xl font-extrabold bg-gradient-to-r from-indigo-600 to-blue-500 bg-clip-text text-transparent">
            NuScaler
          </span>
        </Link>

        {/* Desktop Menu */}
        <div className="hidden md:flex items-center space-x-8">
          <div className="flex items-center space-x-6">
            <Link 
              to="/" 
              className={`text-sm font-medium transition-colors duration-200 hover:text-indigo-600 ${
                isActive('/') ? 'text-indigo-600' : 'text-gray-700'
              }`}
            >
              Home
            </Link>
            
            {isAuthenticated && (
              <Link 
                to="/download" 
                className={`text-sm font-medium transition-colors duration-200 hover:text-indigo-600 ${
                  isActive('/download') ? 'text-indigo-600' : 'text-gray-700'
                }`}
              >
                Download
              </Link>
            )}
            
            {user?.is_admin && (
              <Link 
                to="/admin" 
                className={`text-sm font-medium transition-colors duration-200 hover:text-indigo-600 ${
                  isActive('/admin') ? 'text-indigo-600' : 'text-gray-700'
                }`}
              >
                Admin Panel
              </Link>
            )}
          </div>

          <div className="flex items-center space-x-3">
            {!isAuthenticated ? (
              <>
                <Link 
                  to="/login" 
                  className="text-sm font-medium px-4 py-2 rounded-lg transition-colors duration-200 hover:bg-gray-100"
                >
                  Log in
                </Link>
                <Link 
                  to="/register" 
                  className="text-sm font-medium px-4 py-2 rounded-lg bg-indigo-600 text-white transition-all duration-200 hover:bg-indigo-700 hover:shadow-lg"
                >
                  Sign up
                </Link>
              </>
            ) : (
              <div className="flex items-center space-x-4">
                <div className="relative group">
                  <button className="flex items-center space-x-2 focus:outline-none">
                    <div className="w-8 h-8 rounded-full bg-gradient-to-r from-indigo-500 to-blue-500 flex items-center justify-center text-white font-medium">
                      {user?.name?.charAt(0).toUpperCase()}
                    </div>
                    <span className="text-sm font-medium">{user?.name}</span>
                  </button>
                  <div className="absolute right-0 mt-2 w-48 bg-white rounded-lg shadow-lg py-1 opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 transform origin-top-right scale-95 group-hover:scale-100">
                    <button
                      onClick={handleLogout}
                      className="block w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                    >
                      Log out
                    </button>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Mobile Menu Button */}
        <button 
          onClick={() => setIsOpen(!isOpen)} 
          className="md:hidden focus:outline-none"
          aria-label="Toggle menu"
        >
          <svg 
            className={`w-6 h-6 transition-transform duration-300 ${isOpen ? 'rotate-90' : ''}`} 
            fill="none" 
            stroke="currentColor" 
            viewBox="0 0 24 24"
          >
            {isOpen ? (
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            ) : (
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
            )}
          </svg>
        </button>
      </div>

      {/* Mobile Menu */}
      <div 
        className={`md:hidden absolute top-full left-0 w-full bg-white/95 backdrop-blur-md shadow-lg transition-all duration-300 ease-in-out overflow-hidden ${
          isOpen ? 'max-h-96 opacity-100' : 'max-h-0 opacity-0'
        }`}
      >
        <div className="container mx-auto px-4 py-3 space-y-1">
          <Link 
            to="/" 
            className={`block py-2 px-3 text-sm font-medium rounded-lg ${
              isActive('/') ? 'bg-indigo-50 text-indigo-600' : 'text-gray-700 hover:bg-gray-50'
            }`}
          >
            Home
          </Link>
          
          {isAuthenticated && (
            <Link 
              to="/download" 
              className={`block py-2 px-3 text-sm font-medium rounded-lg ${
                isActive('/download') ? 'bg-indigo-50 text-indigo-600' : 'text-gray-700 hover:bg-gray-50'
              }`}
            >
              Download
            </Link>
          )}
          
          {user?.is_admin && (
            <Link 
              to="/admin" 
              className={`block py-2 px-3 text-sm font-medium rounded-lg ${
                isActive('/admin') ? 'bg-indigo-50 text-indigo-600' : 'text-gray-700 hover:bg-gray-50'
              }`}
            >
              Admin Panel
            </Link>
          )}
          
          {!isAuthenticated ? (
            <div className="grid grid-cols-2 gap-2 mt-3 pt-3 border-t border-gray-100">
              <Link 
                to="/login" 
                className="py-2 px-3 text-center text-sm font-medium rounded-lg border border-gray-200 text-gray-700 hover:bg-gray-50"
              >
                Log in
              </Link>
              <Link 
                to="/register" 
                className="py-2 px-3 text-center text-sm font-medium rounded-lg bg-indigo-600 text-white hover:bg-indigo-700"
              >
                Sign up
              </Link>
            </div>
          ) : (
            <div className="mt-3 pt-3 border-t border-gray-100">
              <div className="flex items-center px-3 py-2">
                <div className="w-8 h-8 rounded-full bg-gradient-to-r from-indigo-500 to-blue-500 flex items-center justify-center text-white font-medium">
                  {user?.name?.charAt(0).toUpperCase()}
                </div>
                <span className="ml-3 text-sm font-medium">{user?.name}</span>
              </div>
              <button
                onClick={handleLogout}
                className="w-full mt-2 py-2 px-3 text-left text-sm font-medium rounded-lg text-red-600 hover:bg-red-50"
              >
                Log out
              </button>
            </div>
          )}
        </div>
      </div>
    </nav>
  );
};

export default Navbar; 