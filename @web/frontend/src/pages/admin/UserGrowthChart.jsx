import { LineChart, Line, XAxis, YAxis, Tooltip, ResponsiveContainer } from 'recharts';

const UserGrowthChart = ({ data }) => (
  <div className="bg-white p-4 rounded shadow mt-4">
    <h3 className="font-semibold mb-2">User Growth</h3>
    <ResponsiveContainer width="100%" height={250}>
      <LineChart data={data}>
        <XAxis dataKey="date" />
        <YAxis />
        <Tooltip />
        <Line type="monotone" dataKey="registrations" stroke="#6366f1" />
      </LineChart>
    </ResponsiveContainer>
  </div>
);
export default UserGrowthChart; 