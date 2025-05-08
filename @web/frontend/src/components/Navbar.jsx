import { Link } from 'react-router-dom';
import { useSelector, useDispatch } from 'react-redux';
import { logout } from '../auth/authSlice';

const Navbar = () => {
  const { isAuthenticated, user } = useSelector((state) => state.auth);
  const dispatch = useDispatch();

  const handleLogout = () => {
    dispatch(logout());
  };

  return (
    <nav className="bg-white border-b shadow-sm px-4 py-2 flex items-center justify-between">
      <div className="flex items-center gap-4">
        <Link to="/" className="text-xl font-bold text-indigo-700">NuScaler</Link>
        {isAuthenticated && (
          <Link to="/download" className="text-gray-700 hover:text-indigo-600">Download</Link>
        )}
        {user?.is_admin && (
          <Link to="/admin" className="text-red-600 font-semibold hover:underline">Admin Panel</Link>
        )}
      </div>
      <div className="flex items-center gap-4">
        {!isAuthenticated ? (
          <>
            <Link to="/login" className="text-gray-700 hover:text-indigo-600">Login</Link>
            <Link to="/register" className="text-indigo-600 font-semibold">Register</Link>
          </>
        ) : (
          <>
            <span className="text-gray-600">{user?.name}</span>
            <button
              onClick={handleLogout}
              className="bg-gray-200 px-3 py-1 rounded hover:bg-gray-300"
            >
              Logout
            </button>
          </>
        )}
      </div>
    </nav>
  );
};

export default Navbar; 