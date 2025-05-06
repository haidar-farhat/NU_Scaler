import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer } from 'recharts';

const SurveysChart = ({ data }) => (
  <div className="bg-white p-4 rounded shadow mt-4">
    <h3 className="font-semibold mb-2">GPU Brand Distribution</h3>
    <ResponsiveContainer width="100%" height={250}>
      <BarChart data={data}>
        <XAxis dataKey="gpu_brand" />
        <YAxis />
        <Tooltip />
        <Bar dataKey="count" fill="#6366f1" />
      </BarChart>
    </ResponsiveContainer>
  </div>
);
export default SurveysChart; 