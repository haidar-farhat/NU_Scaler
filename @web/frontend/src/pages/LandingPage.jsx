import { Link } from 'react-router-dom';
import { useSelector } from 'react-redux';
import HeroSection from '../components/HeroSection';
import FeaturesGrid from '../components/FeaturesGrid';
import TestimonialsCarousel from '../components/TestimonialsCarousel';
import FooterCTA from '../components/FooterCTA';

const LandingPage = () => {
  const { isAuthenticated, user } = useSelector((state) => state.auth);

  return (
    <div className="bg-white">
      <HeroSection />
      <FeaturesGrid />
      <TestimonialsCarousel />
      <FooterCTA />
    </div>
  );
};

export default LandingPage; 