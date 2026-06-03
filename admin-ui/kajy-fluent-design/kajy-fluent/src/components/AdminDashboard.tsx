import React, { useState, useEffect } from 'react';
import { Header, SoftCard, cn } from './OneUI';
import {
  XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer,
  AreaChart, Area
} from 'recharts';
import { Activity, HardDrive, Users, ArrowUpRight, ArrowDownRight, ServerOff, CheckCircle2, Play, AlertCircle } from 'lucide-react';
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
    { label: 'Accès Totaux',     value: data?.total_accesses || 0,                      icon: Activity,    accent: 'var(--color-accent)' },
    { label: 'Trafic Global',    value: formatBytes(data?.total_bytes_transferred || 0), icon: ArrowUpRight, accent: '#107c10' },
    { label: 'Fichiers Uniques', value: data?.unique_files_accessed || 0,               icon: HardDrive,   accent: '#7a4100' },
    { label: 'IPs Visiteurs',    value: data?.unique_ips || 0,                           icon: Users,       accent: '#881798' },
  ];

  return (
    <div style={{ padding: '0 0 40px 0' }}>
      <Header title="Console d'administration" subtitle="Vue d'ensemble du serveur Kajy" />

      {/* Server status banner */}
      <div style={{ padding: '0 32px 24px' }}>
        <div
          className="fluent-card flex items-center justify-between"
          style={{
            borderLeft: `4px solid ${isRunning ? 'var(--color-success)' : 'var(--color-error)'}`,
            padding: '14px 20px',
          }}
        >
          <div className="flex items-center gap-3">
            <div style={{
              width: 36, height: 36, borderRadius: 8, flexShrink: 0,
              background: isRunning ? 'var(--color-success-bg)' : 'var(--color-error-bg)',
              display: 'flex', alignItems: 'center', justifyContent: 'center',
            }}>
              {isRunning
                ? <CheckCircle2 size={20} style={{ color: 'var(--color-success)' }} />
                : <AlertCircle size={20} style={{ color: 'var(--color-error)' }} />
              }
            </div>
            <div>
              <p style={{ fontWeight: 600, fontSize: 14, margin: 0, color: 'var(--color-win-text)' }}>
                Serveur {isRunning ? 'En ligne' : 'Hors ligne'}
              </p>
              <p style={{ fontSize: 12, color: 'var(--color-win-text3)', margin: 0 }}>
                {isRunning ? 'MCP Realtime actif — port 8081' : 'Le serveur n\'est pas démarré'}
              </p>
            </div>
          </div>
          {!isRunning && (
            <button
              className="fluent-btn fluent-btn-accent flex items-center gap-2"
              onClick={async () => { try { await invoke('start_server'); fetchData(); } catch(e) { alert(e); } }}
            >
              <Play size={14} fill="currentColor" />
              Démarrer
            </button>
          )}
        </div>
      </div>

      {/* Stat tiles */}
      <div style={{ padding: '0 32px', display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 12, marginBottom: 24 }}>
        {stats.map((s, i) => {
          const Icon = s.icon;
          return (
            <div className="fluent-stat" key={i}>
              <div className="flex items-center justify-between">
                <span style={{ fontSize: 12, color: 'var(--color-win-text3)', fontWeight: 400 }}>{s.label}</span>
                <Icon size={16} style={{ color: s.accent, opacity: 0.8 }} />
              </div>
              <p style={{ fontSize: 26, fontWeight: 600, color: 'var(--color-win-text)', margin: 0, letterSpacing: '-0.5px' }}>
                {s.value}
              </p>
            </div>
          );
        })}
      </div>

      {/* Charts row */}
      <div style={{ padding: '0 32px', display: 'grid', gridTemplateColumns: '2fr 1fr', gap: 16 }}>
        {/* Area chart */}
        <div className="fluent-card" style={{ padding: '20px 24px' }}>
          <div className="flex items-center justify-between mb-5">
            <p style={{ fontWeight: 600, fontSize: 15, margin: 0, color: 'var(--color-win-text)' }}>Flux Réseau</p>
            <span className="fluent-badge fluent-badge-gray">30 derniers jours</span>
          </div>
          <div style={{ height: 260 }}>
            {!isRunning ? (
              <div className="flex flex-col items-center justify-center h-full gap-3" style={{ opacity: 0.3 }}>
                <ServerOff size={40} />
                <span style={{ fontSize: 13, fontWeight: 600 }}>Données indisponibles</span>
              </div>
            ) : (
              <ResponsiveContainer width="100%" height="100%">
                <AreaChart data={data?.bandwidth_by_day || []} margin={{ top: 4, right: 4, bottom: 0, left: 0 }}>
                  <defs>
                    <linearGradient id="gBlue" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%"  stopColor="#0078d4" stopOpacity={0.15} />
                      <stop offset="95%" stopColor="#0078d4" stopOpacity={0} />
                    </linearGradient>
                  </defs>
                  <CartesianGrid strokeDasharray="3 3" stroke="var(--color-win-stroke)" vertical={false} />
                  <XAxis dataKey="date" stroke="var(--color-win-text3)" fontSize={11} tickLine={false} axisLine={false} dy={8} />
                  <YAxis stroke="var(--color-win-text3)" fontSize={11} tickFormatter={formatBytes} tickLine={false} axisLine={false} width={64} />
                  <Tooltip
                    contentStyle={{
                      background: 'var(--color-win-surface)',
                      border: '1px solid var(--color-win-stroke)',
                      borderRadius: 8,
                      boxShadow: 'var(--shadow-8)',
                      fontSize: 13,
                      padding: '10px 14px',
                    }}
                    itemStyle={{ color: 'var(--color-accent)', fontWeight: 600 }}
                    labelStyle={{ color: 'var(--color-win-text2)', marginBottom: 4, fontSize: 12 }}
                    cursor={{ stroke: 'var(--color-accent)', strokeWidth: 1.5, strokeDasharray: '4 4' }}
                  />
                  <Area type="monotone" dataKey="bytes_total" stroke="#0078d4" strokeWidth={2}
                    fillOpacity={1} fill="url(#gBlue)" animationDuration={1200} />
                </AreaChart>
              </ResponsiveContainer>
            )}
          </div>
        </div>

        {/* Recent activity */}
        <div className="fluent-card" style={{ padding: '20px 24px' }}>
          <p style={{ fontWeight: 600, fontSize: 15, margin: '0 0 16px', color: 'var(--color-win-text)' }}>
            Activité Récente
          </p>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            {isRunning && data?.recent_activity?.slice(0, 8).map((act: any, i: number) => (
              <div key={i} className="fluent-row" style={{ padding: '8px 10px' }}>
                <div style={{
                  width: 28, height: 28, borderRadius: 6, flexShrink: 0,
                  background: act.action === 'read' ? 'var(--color-accent-subtle)' : 'var(--color-success-bg)',
                  display: 'flex', alignItems: 'center', justifyContent: 'center',
                }}>
                  {act.action === 'read'
                    ? <ArrowDownRight size={14} style={{ color: 'var(--color-accent)' }} />
                    : <ArrowUpRight size={14} style={{ color: 'var(--color-success)' }} />
                  }
                </div>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <p style={{ fontSize: 13, fontWeight: 400, margin: 0, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', color: 'var(--color-win-text)' }}>
                    {act.filename}
                  </p>
                  <p style={{ fontSize: 11, color: 'var(--color-win-text3)', margin: 0 }}>
                    {act.ip_address || 'Local'} · {act.action}
                  </p>
                </div>
                <span style={{ fontSize: 11, color: 'var(--color-win-text3)', whiteSpace: 'nowrap' }}>
                  {new Date(act.accessed_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                </span>
              </div>
            ))}
            {(!isRunning || !data?.recent_activity?.length) && (
              <div className="flex flex-col items-center justify-center py-12 gap-2" style={{ opacity: 0.3 }}>
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
