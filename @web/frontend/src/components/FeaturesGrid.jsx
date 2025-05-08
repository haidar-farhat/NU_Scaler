import FeatureCard from './FeatureCard';
import features from '../data/features';

const FeaturesGrid = () => (
  <div className="bg-gray-50 py-16 sm:py-24">
    <div className="mx-auto max-w-7xl px-6 lg:px-8">
      <div className="mx-auto mt-16 max-w-2xl sm:mt-20 lg:mt-24 lg:max-w-none">
        <dl className="grid max-w-xl grid-cols-1 gap-x-8 gap-y-16 lg:max-w-none lg:grid-cols-3">
          {features.map((f, i) => (
            <FeatureCard key={i} {...f} />
          ))}
        </dl>
      </div>
    </div>
  </div>
);
export default FeaturesGrid; 