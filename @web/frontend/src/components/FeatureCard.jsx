const FeatureCard = ({ icon: Icon, title, description }) => (
  <div className="flex flex-col items-center bg-white p-6 rounded-lg shadow hover:shadow-lg transition">
    <div className="mb-4">
      <Icon className="text-indigo-600 text-3xl" />
    </div>
    <h3 className="text-lg font-semibold mb-2">{title}</h3>
    <p className="text-gray-600">{description}</p>
  </div>
);
export default FeatureCard;
