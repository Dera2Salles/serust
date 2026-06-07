import { useState, useEffect } from 'react';
import { Header } from './OneUI';
import {
  XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer,
  AreaChart, Area
} from 'recharts';
import {
  Activity, HardDrive, Users, ArrowUpRight, ArrowDownRight,
  ServerOff, CheckCircle2, Play, AlertCircle,
  FolderOpen, Database, Wifi, Clock
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface SummaryData {
  total_accesses: number;
  total_bytes_transferred: number;
  unique_files_accessed: number;
  unique_ips: number;
  bandwidth_by_day: any[];
  recent_activity: any[];
}

const formatBytes = (bytes: number) => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

/* Quick Access mock data — replaced with real data when available */
const quickItems = [
  { label: 'Stockage actif',  sub: 'Données utilisateurs', icon: Database, color: '#2563eb', bg: '#eff4ff' },
  { label: 'Fichiers publics', sub: 'Liens partagés',      icon: FolderOpen, color: '#059669', bg: '#ecfdf5' },
  { label: 'Trafic réseau',   sub: 'Dernières 24h',        icon: Wifi,      color: '#d97706', bg: '#fffbeb' },
  { label: 'Sessions live',   sub: 'Connexions actives',   icon: Activity,  color: '#7c3aed', bg: '#f5f3ff' },
];

export const AdminDashboard = () => {
  const [data, setData] = useState<SummaryData | null>(null);
  const [loading, setLoading] = useState(true);
  const [isRunning, setIsRunning] = useState(false);

  const fetchData = async () => {
    try {
      const status = await invoke<boolean>('get_server_status');
      setIsRunning(status);
      if (status) {
        const response = await fetch('http://localhost:8081/api/analytics/summary?username=admin_dev');
        if (response.ok) setData(await response.json());
      }
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
    const t = setInterval(fetchData, 5000);
    return () => clearInterval(t);
  }, []);

  if (loading) return (
    <div className="flex items-center justify-center h-64">
      <div style={{ width: 28, height: 28, border: '3px solid var(--color-win-stroke2)', borderTopColor: 'var(--color-accent)', borderRadius: '50%', animation: 'spin 0.8s linear infinite' }} />
    </div>
  );

  const stats = [
    { label: 'Accès Totaux',     value: data?.total_accesses || 0,                      icon: Activity,    color: 'var(--color-accent)',   bg: 'var(--color-accent-light)' },
    { label: 'Trafic Global',    value: formatBytes(data?.total_bytes_transferred || 0), icon: ArrowUpRight, color: 'var(--color-success)', bg: 'var(--color-success-bg)' },
    { label: 'Fichiers Uniques', value: data?.unique_files_accessed || 0,               icon: HardDrive,   color: 'var(--color-warning)',   bg: 'var(--color-warning-bg)' },
    { label: 'IPs Visiteurs',    value: data?.unique_ips || 0,                           icon: Users,       color: '#7c3aed',               bg: '#f5f3ff' },
  ];

  return (
    <div style={{ padding: '0 0 48px 0' }}>

      {/* Page header */}
      <div style={{ padding: '28px 28px 20px' }}>
        <p style={{ fontSize: 11, fontWeight: 600, color: 'var(--color-accent)', textTransform: 'uppercase', letterSpacing: '0.08em', margin: '0 0 4px' }}>
          Vue d'ensemble
        </p>
        <h1 style={{ fontSize: 24, fontWeight: 700, color: 'var(--color-win-text)', margin: 0, letterSpacing: '-0.4px' }}>
          Console d'administration
        </h1>
      </div>

      {/* Server status banner */}
      <div style={{ padding: '0 28px 22px' }}>
        <div style={{
          display: 'flex', alignItems: 'center', justifyContent: 'space-between',
          padding: '14px 18px',
          borderRadius: 12,
          background: isRunning ? 'var(--color-success-bg)' : 'var(--color-error-bg)',
          border: `1px solid ${isRunning ? '#a7f3d0' : '#fca5a5'}`,
        }}>
          <div className="flex items-center gap-3">
            <div style={{
              width: 32, height: 32, borderRadius: 8, flexShrink: 0,
              background: isRunning ? '#d1fae5' : '#fee2e2',
              display: 'flex', alignItems: 'center', justifyContent: 'center',
            }}>
              {isRunning
                ? <CheckCircle2 size={18} style={{ color: 'var(--color-success)' }} />
                : <AlertCircle size={18} style={{ color: 'var(--color-error)' }} />
              }
            </div>
            <div>
              <p style={{ fontWeight: 600, fontSize: 13.5, margin: 0, color: isRunning ? 'var(--color-success)' : 'var(--color-error)' }}>
                Serveur {isRunning ? 'En ligne' : 'Hors ligne'}
              </p>
              <p style={{ fontSize: 12, color: 'var(--color-win-text3)', margin: 0 }}>
                {isRunning ? 'MCP Realtime actif — port 8081' : "Le serveur n'est pas démarré"}
              </p>
            </div>
          </div>
          {!isRunning && (
            <button
              className="fluent-btn fluent-btn-accent"
              style={{ gap: 6 }}
              onClick={async () => { try { await invoke('start_server'); fetchData(); } catch (e) { alert(e); } }}
            >
              <Play size={13} fill="currentColor" />
              Démarrer
            </button>
          )}
        </div>
      </div>

      {/* Quick Access */}
      <div style={{ padding: '0 28px 22px' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 12 }}>
          <p style={{ fontSize: 14, fontWeight: 600, color: 'var(--color-win-text)', margin: 0 }}>Accès Rapide</p>
        </div>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 12 }}>
          {quickItems.map((item, i) => {
            const Icon = item.icon;
            return (
              <div key={i} className="quick-card">
                <div className="icon-tile" style={{ background: item.bg }}>
                  <Icon size={18} style={{ color: item.color }} />
                </div>
                <div>
                  <p style={{ fontSize: 13.5, fontWeight: 600, color: 'var(--color-win-text)', margin: '0 0 2px' }}>{item.label}</p>
                  <p style={{ fontSize: 11.5, color: 'var(--color-win-text3)', margin: 0 }}>{item.sub}</p>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Stat tiles */}
      <div style={{ padding: '0 28px', display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 12, marginBottom: 22 }}>
        {stats.map((s, i) => {
          const Icon = s.icon;
          return (
            <div className="fluent-stat" key={i}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <span style={{ fontSize: 11.5, color: 'var(--color-win-text3)', fontWeight: 500 }}>{s.label}</span>
                <div className="icon-tile" style={{ background: s.bg, width: 30, height: 30, borderRadius: 7 }}>
                  <Icon size={15} style={{ color: s.color }} />
                </div>
              </div>
              <p style={{ fontSize: 28, fontWeight: 700, color: 'var(--color-win-text)', margin: 0, letterSpacing: '-0.5px', lineHeight: 1.1 }}>
                {s.value}
              </p>
            </div>
          );
        })}
      </div>

      {/* Charts row */}
      <div style={{ padding: '0 28px', display: 'grid', gridTemplateColumns: '2fr 1fr', gap: 16 }}>

        {/* Area chart */}
        <div className="fluent-card" style={{ padding: '22px 24px' }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 20 }}>
            <div>
              <p style={{ fontWeight: 600, fontSize: 14.5, margin: '0 0 2px', color: 'var(--color-win-text)' }}>Flux Réseau</p>
              <p style={{ fontSize: 12, color: 'var(--color-win-text3)', margin: 0 }}>Bande passante journalière</p>
            </div>
            <span className="fluent-badge fluent-badge-blue">30 derniers jours</span>
          </div>
          <div style={{ height: 240 }}>
            {!isRunning ? (
              <div className="flex flex-col items-center justify-center h-full gap-3" style={{ opacity: 0.25 }}>
                <ServerOff size={36} />
                <span style={{ fontSize: 13, fontWeight: 500 }}>Données indisponibles</span>
              </div>
            ) : (
              <ResponsiveContainer width="100%" height="100%">
                <AreaChart data={data?.bandwidth_by_day || []} margin={{ top: 4, right: 4, bottom: 0, left: 0 }}>
                  <defs>
                    <linearGradient id="gBlue" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%"  stopColor="#2563eb" stopOpacity={0.12} />
                      <stop offset="95%" stopColor="#2563eb" stopOpacity={0} />
                    </linearGradient>
                  </defs>
                  <CartesianGrid strokeDasharray="3 3" stroke="var(--color-win-stroke)" vertical={false} />
                  <XAxis dataKey="date" stroke="var(--color-win-text3)" fontSize={11} tickLine={false} axisLine={false} dy={8} />
                  <YAxis stroke="var(--color-win-text3)" fontSize={11} tickFormatter={formatBytes} tickLine={false} axisLine={false} width={64} />
                  <Tooltip
                    contentStyle={{
                      background: 'var(--color-win-surface)',
                      border: '1px solid var(--color-win-stroke)',
                      borderRadius: 10,
                      boxShadow: 'var(--shadow-8)',
                      fontSize: 13,
                      padding: '10px 14px',
                    }}
                    itemStyle={{ color: 'var(--color-accent)', fontWeight: 600 }}
                    labelStyle={{ color: 'var(--color-win-text2)', marginBottom: 4, fontSize: 12 }}
                    cursor={{ stroke: 'var(--color-accent)', strokeWidth: 1.5, strokeDasharray: '4 4' }}
                  />
                  <Area type="monotone" dataKey="bytes_total" stroke="#2563eb" strokeWidth={2}
                    fillOpacity={1} fill="url(#gBlue)" animationDuration={1200} />
                </AreaChart>
              </ResponsiveContainer>
            )}
          </div>
        </div>

        {/* Recent activity */}
        <div className="fluent-card" style={{ padding: '22px 20px' }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 16 }}>
            <p style={{ fontWeight: 600, fontSize: 14.5, margin: 0 }}>Activité Récente</p>
            <Clock size={14} style={{ color: 'var(--color-win-text3)' }} />
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            {isRunning && data?.recent_activity?.slice(0, 8).map((act: any, i: number) => (
              <div key={i} className="fluent-row" style={{ padding: '8px 10px' }}>
                <div style={{
                  width: 30, height: 30, borderRadius: 8, flexShrink: 0,
                  background: act.action === 'read' ? 'var(--color-accent-light)' : 'var(--color-success-bg)',
                  display: 'flex', alignItems: 'center', justifyContent: 'center',
                }}>
                  {act.action === 'read'
                    ? <ArrowDownRight size={14} style={{ color: 'var(--color-accent)' }} />
                    : <ArrowUpRight size={14} style={{ color: 'var(--color-success)' }} />
                  }
                </div>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <p style={{ fontSize: 13, fontWeight: 500, margin: 0, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {act.filename}
                  </p>
                  <p style={{ fontSize: 11, color: 'var(--color-win-text3)', margin: 0 }}>
                    {act.ip_address || 'Local'} · {act.action}
                  </p>
                </div>
                <span style={{ fontSize: 11, color: 'var(--color-win-text4)', whiteSpace: 'nowrap' }}>
                  {new Date(act.accessed_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                </span>
              </div>
            ))}
            {(!isRunning || !data?.recent_activity?.length) && (
              <div className="flex flex-col items-center justify-center py-10 gap-2" style={{ opacity: 0.25 }}>
                <Activity size={32} />
                <span style={{ fontSize: 13 }}>Aucune donnée</span>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
