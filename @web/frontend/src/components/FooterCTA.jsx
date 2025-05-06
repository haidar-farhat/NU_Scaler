import { Link } from 'react-router-dom';

const FooterCTA = () => (
  <div className="bg-indigo-600 py-8 text-center">
    <h2 className="text-2xl font-bold text-white mb-2">Ready to upscale your images?</h2>
    <Link to="/register" className="bg-white text-indigo-600 px-6 py-2 rounded font-semibold shadow hover:bg-gray-100">
      Get Started
    </Link>
  </div>
);
export default FooterCTA; 