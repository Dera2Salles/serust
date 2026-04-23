import React, { useState, useEffect } from 'react';
import { Header, Card, cn } from './OneUI';
import { 
  LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer,
  AreaChart, Area
} from 'recharts';
import { Activity, HardDrive, Users, ArrowUpRight, ArrowDownRight, Share2 } from 'lucide-react';

interface SummaryData {
  total_accesses: number;
  total_bytes_transferred: number;
  unique_files_accessed: number;
  unique_ips: number;
  bandwidth_by_day: any[];
}

export const AdminDashboard = () => {
  const [data, setData] = useState<SummaryData | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // In a real app, this would be an actual fetch to localhost:8081
    const fetchData = async () => {
      try {
        const response = await fetch('http://localhost:8081/api/analytics/summary?username=admin_dev');
        const json = await response.json();
        setData(json);
      } catch (error) {
        console.error("Failed to fetch summary:", error);
      } finally {
        setLoading(false);
      }
    };
    fetchData();
  }, []);

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  if (loading) return <div className="p-10 text-center">Loading Dashboard...</div>;

  return (
    <div className="flex flex-col w-full min-h-screen bg-base pb-20">
      <Header 
        title="Admin Console" 
        subtitle="TCP Framework Server Overview" 
      />

      <div className="px-8 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <Card className="flex items-center gap-4">
          <div className="p-3 bg-blue/10 rounded-2xl text-blue">
            <Activity size={28} />
          </div>
          <div>
            <p className="text-subtext0 text-sm">Total Accesses</p>
            <p className="text-2xl font-semibold">{data?.total_accesses || 0}</p>
          </div>
        </Card>

        <Card className="flex items-center gap-4">
          <div className="p-3 bg-mauve/10 rounded-2xl text-mauve">
            <ArrowUpRight size={28} />
          </div>
          <div>
            <p className="text-subtext0 text-sm">Data Transferred</p>
            <p className="text-2xl font-semibold">{formatBytes(data?.total_bytes_transferred || 0)}</p>
          </div>
        </Card>

        <Card className="flex items-center gap-4">
          <div className="p-3 bg-green/10 rounded-2xl text-green">
            <HardDrive size={28} />
          </div>
          <div>
            <p className="text-subtext0 text-sm">Unique Files</p>
            <p className="text-2xl font-semibold">{data?.unique_files_accessed || 0}</p>
          </div>
        </Card>

        <Card className="flex items-center gap-4">
          <div className="p-3 bg-peach/10 rounded-2xl text-peach">
            <Users size={28} />
          </div>
          <div>
            <p className="text-subtext0 text-sm">Unique IPs</p>
            <p className="text-2xl font-semibold">{data?.unique_ips || 0}</p>
          </div>
        </Card>
      </div>

      <div className="px-8 mt-8 grid grid-cols-1 lg:grid-cols-3 gap-8">
        <Card className="lg:col-span-2">
          <h3 className="text-xl font-medium mb-6">Bandwidth Usage</h3>
          <div className="h-[300px] w-full">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={data?.bandwidth_by_day || []}>
                <defs>
                  <linearGradient id="colorBytes" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#89b4fa" stopOpacity={0.3}/>
                    <stop offset="95%" stopColor="#89b4fa" stopOpacity={0}/>
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="#313244" vertical={false} />
                <XAxis dataKey="date" stroke="#9399b2" />
                <YAxis stroke="#9399b2" tickFormatter={(value) => formatBytes(value)} />
                <Tooltip 
                  contentStyle={{ backgroundColor: '#181825', border: 'none', borderRadius: '16px' }}
                  labelStyle={{ color: '#cdd6f4' }}
                />
                <Area 
                  type="monotone" 
                  dataKey="bytes_total" 
                  stroke="#89b4fa" 
                  strokeWidth={3}
                  fillOpacity={1} 
                  fill="url(#colorBytes)" 
                />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </Card>

        <Card>
          <h3 className="text-xl font-medium mb-6">Recent Activities</h3>
          <div className="space-y-4">
            {(data as any)?.recent_activity?.slice(0, 6).map((activity: any, i: number) => (
              <div key={i} className="flex items-center gap-3">
                <div className={cn(
                  "p-2 rounded-xl",
                  activity.action === 'read' ? 'bg-blue/10 text-blue' : 'bg-green/10 text-green'
                )}>
                  {activity.action === 'read' ? <ArrowDownRight size={18} /> : <ArrowUpRight size={18} />}
                </div>
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium truncate">{activity.filename}</p>
                  <p className="text-xs text-subtext0">{activity.ip_address || 'Internal'}</p>
                </div>
                <p className="text-xs text-overlay2">{new Date(activity.accessed_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</p>
              </div>
            ))}
          </div>
        </Card>
      </div>
    </div>
  );
};
