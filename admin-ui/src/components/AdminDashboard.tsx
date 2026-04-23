import React, { useState, useEffect } from 'react';
import { Header, SoftCard, cn } from './OneUI';
import { 
  XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer,
  AreaChart, Area
} from 'recharts';
import { Activity, HardDrive, Users, ArrowUpRight, ArrowDownRight } from 'lucide-react';

interface SummaryData {
  total_accesses: number;
  total_bytes_transferred: number;
  unique_files_accessed: number;
  unique_ips: number;
  bandwidth_by_day: any[];
  recent_activity: any[];
}

export const AdminDashboard = () => {
  const [data, setData] = useState<SummaryData | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
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

  if (loading) return (
    <div className="flex items-center justify-center h-full">
      <div className="w-12 h-12 border-4 border-blue/20 border-t-blue rounded-full animate-spin" />
    </div>
  );

  return (
    <div className="flex flex-col w-full min-h-screen bg-transparent pb-20">
      <Header 
        title="Admin Console" 
        subtitle="AroSaina Server Overview" 
      />

      <div className="px-10 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <SoftCard className="flex items-center gap-5 group hover:border-blue/30 transition-all">
          <div className="w-14 h-14 bg-blue/10 rounded-2xl flex items-center justify-center text-blue group-hover:scale-110 transition-transform">
            <Activity size={28} />
          </div>
          <div>
            <p className="text-subtext0 text-xs font-bold tracking-widest uppercase opacity-70">Vues totales</p>
            <p className="text-3xl font-extrabold tracking-tighter">{data?.total_accesses || 0}</p>
          </div>
        </SoftCard>

        <SoftCard className="flex items-center gap-5 group hover:border-mauve/30 transition-all">
          <div className="w-14 h-14 bg-mauve/10 rounded-2xl flex items-center justify-center text-mauve group-hover:scale-110 transition-transform">
            <ArrowUpRight size={28} />
          </div>
          <div>
            <p className="text-subtext0 text-xs font-bold tracking-widest uppercase opacity-70">Trafic total</p>
            <p className="text-3xl font-extrabold tracking-tighter">{formatBytes(data?.total_bytes_transferred || 0)}</p>
          </div>
        </SoftCard>

        <SoftCard className="flex items-center gap-5 group hover:border-green/30 transition-all">
          <div className="w-14 h-14 bg-green/10 rounded-2xl flex items-center justify-center text-green group-hover:scale-110 transition-transform">
            <HardDrive size={28} />
          </div>
          <div>
            <p className="text-subtext0 text-xs font-bold tracking-widest uppercase opacity-70">Fichiers</p>
            <p className="text-3xl font-extrabold tracking-tighter">{data?.unique_files_accessed || 0}</p>
          </div>
        </SoftCard>

        <SoftCard className="flex items-center gap-5 group hover:border-peach/30 transition-all">
          <div className="w-14 h-14 bg-peach/10 rounded-2xl flex items-center justify-center text-peach group-hover:scale-110 transition-transform">
            <Users size={28} />
          </div>
          <div>
            <p className="text-subtext0 text-xs font-bold tracking-widest uppercase opacity-70">Visiteurs</p>
            <p className="text-3xl font-extrabold tracking-tighter">{data?.unique_ips || 0}</p>
          </div>
        </SoftCard>
      </div>

      <div className="px-10 mt-10 grid grid-cols-1 lg:grid-cols-3 gap-10">
        <SoftCard className="lg:col-span-2 p-8">
          <div className="flex items-center justify-between mb-10">
            <h3 className="text-2xl font-black tracking-tighter">Trafic Réseau</h3>
            <div className="px-3 py-1 bg-surface0 rounded-full text-xs font-bold text-subtext0 border border-surface1">30 Derniers Jours</div>
          </div>
          <div className="h-[350px] w-full">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={data?.bandwidth_by_day || []}>
                <defs>
                  <linearGradient id="colorBytes" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#89b4fa" stopOpacity={0.4}/>
                    <stop offset="95%" stopColor="#89b4fa" stopOpacity={0}/>
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="#313244" vertical={false} opacity={0.5} />
                <XAxis 
                  dataKey="date" 
                  stroke="#6c7086" 
                  fontSize={12} 
                  fontWeight={600}
                  tickLine={false}
                  axisLine={false}
                  dy={10}
                />
                <YAxis 
                  stroke="#6c7086" 
                  fontSize={12} 
                  fontWeight={600}
                  tickFormatter={(value) => formatBytes(value)} 
                  tickLine={false}
                  axisLine={false}
                  dx={-10}
                />
                <Tooltip 
                  contentStyle={{ backgroundColor: '#11111b', border: '1px solid #313244', borderRadius: '16px', boxShadow: '0 10px 30px rgba(0,0,0,0.3)' }}
                  itemStyle={{ color: '#cdd6f4', fontWeight: 700 }}
                  labelStyle={{ color: '#9399b2', marginBottom: '4px', fontWeight: 600 }}
                />
                <Area 
                  type="monotone" 
                  dataKey="bytes_total" 
                  stroke="#89b4fa" 
                  strokeWidth={4}
                  fillOpacity={1} 
                  fill="url(#colorBytes)" 
                  animationDuration={1500}
                />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </SoftCard>

        <SoftCard className="p-8">
          <h3 className="text-2xl font-black tracking-tighter mb-8">Activités Récentes</h3>
          <div className="space-y-6">
            {data?.recent_activity?.slice(0, 7).map((activity: any, i: number) => (
              <div key={i} className="flex items-center gap-4 group cursor-pointer">
                <div className={cn(
                  "p-3 rounded-2xl transition-transform group-hover:scale-110",
                  activity.action === 'read' ? 'bg-blue/10 text-blue' : 'bg-green/10 text-green'
                )}>
                  {activity.action === 'read' ? <ArrowDownRight size={20} /> : <ArrowUpRight size={20} />}
                </div>
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-bold text-text truncate group-hover:text-blue transition-colors">{activity.filename}</p>
                  <p className="text-xs text-subtext0 font-medium opacity-60 tracking-tight">{activity.ip_address || 'Interne'} • {activity.action.toUpperCase()}</p>
                </div>
                <p className="text-[10px] font-black text-overlay2 bg-surface0 px-2 py-1 rounded-lg">
                  {new Date(activity.accessed_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                </p>
              </div>
            ))}
            {!data?.recent_activity?.length && (
              <div className="text-center py-10">
                <p className="text-subtext0 font-medium opacity-50 italic">Aucune activité récente</p>
              </div>
            )}
          </div>
        </SoftCard>
      </div>
    </div>
  );
};
