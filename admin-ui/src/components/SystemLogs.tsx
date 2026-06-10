import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { RefreshCcw, ChevronDown, Activity, Terminal, Clock, User, FileText, Database } from 'lucide-react';
import { Header, cn } from './OneUI';

interface ActivityLog {
  id: number;
  username: string | null;
  filename: string;
  action: string;
  accessed_at: string;
  ip_address: string | null;
  bytes_transferred: number | null;
}

const formatSize = (bytes: number | null) => {
  if (bytes === null || bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let val = bytes, idx = 0;
  while (val >= 1024 && idx < units.length - 1) { val /= 1024; idx++; }
  return `${val.toFixed(1)} ${units[idx]}`;
};

export const SystemLogs: React.FC = () => {
  const [logs, setLogs] = useState<string[]>([]);
  const [activityLogs, setActivityLogs] = useState<ActivityLog[]>([]);
  const [tab, setTab] = useState<'system' | 'activity'>('activity');
  const [autoScroll, setAutoScroll] = useState(true);
  const [loading, setLoading] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);

  const fetchLogs = async () => {
    setLoading(true);
    try {
      const c = await invoke<string>('read_server_logs');
      setLogs(c.split('\n'));
    } catch (e) { console.error("Failed to read system logs", e); }
    finally { setLoading(false); }
  };

  const fetchActivity = async () => {
    setLoading(true);
    try {
      const data = await invoke<ActivityLog[]>('get_activity_logs_db');
      setActivityLogs(data || []);
    } catch (e) { console.error("Failed to read activity logs", e); }
    finally { setLoading(false); }
  };

  const refresh = () => {
    if (tab === 'system') fetchLogs();
    else fetchActivity();
  };

  useEffect(() => {
    refresh();
    const t = setInterval(refresh, 5000);
    return () => clearInterval(t);
  }, [tab]);

  useEffect(() => { 
    if (autoScroll && scrollRef.current && tab === 'system') {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [logs, autoScroll, tab]);

  const getLineStyle = (line: string): React.CSSProperties => {
    if (line.includes('ERROR') || line.includes('error')) return { color: '#c42b1c', fontWeight: 600 };
    if (line.includes('WARN') || line.includes('warn')) return { color: '#7a4100', fontWeight: 600 };
    if (line.includes('INFO') || line.includes('info')) return { color: '#107c10', fontWeight: 500 };
    return { color: 'var(--color-win-text)' };
  };

  return (
    <div style={{ padding: '0 0 40px 0', height: '100%', display: 'flex', flexDirection: 'column' }}>
      <Header title="Journaux Système" subtitle="Surveillance des actions important et logs techniques" />

      {/* Tabs */}
      <div style={{ padding: '0 28px 16px', display: 'flex', gap: 12 }}>
        <button 
          onClick={() => setTab('activity')}
          className={cn('fluent-btn', tab === 'activity' && 'fluent-btn-accent')}
          style={{ gap: 8 }}
        >
          <Activity size={15} /> Historique d'activité
        </button>
        <button 
          onClick={() => setTab('system')}
          className={cn('fluent-btn', tab === 'system' && 'fluent-btn-accent')}
          style={{ gap: 8 }}
        >
          <Terminal size={15} /> Logs techniques (Serveur)
        </button>

        <div style={{ flex: 1 }} />

        <button className="fluent-btn" onClick={refresh} disabled={loading}>
          <RefreshCcw size={14} className={loading ? 'animate-spin' : ''} />
        </button>
      </div>

      <div style={{ padding: '0 28px', flex: 1, overflow: 'hidden' }}>
        <div className="fluent-card" style={{ height: '100%', display: 'flex', flexDirection: 'column', padding: 0, overflow: 'hidden' }}>
          
          {tab === 'system' ? (
            <>
              <div style={{ padding: '12px 20px', borderBottom: '1px solid var(--color-win-stroke)', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <span style={{ fontSize: 12, fontWeight: 600, color: 'var(--color-win-text3)' }}>FICHIER: server.log</span>
                <button
                  className={`fluent-btn ${autoScroll ? 'fluent-btn-accent' : ''}`}
                  style={{ padding: '4px 10px', fontSize: 11 }}
                  onClick={() => setAutoScroll(!autoScroll)}
                >
                  <ChevronDown size={12} /> {autoScroll ? 'Auto-scroll' : 'Manuel'}
                </button>
              </div>
              <div
                ref={scrollRef}
                style={{
                  flex: 1,
                  overflow: 'auto',
                  background: '#1e1e1e', // Dark mode for technical logs
                  padding: '16px 20px',
                  fontFamily: "'Cascadia Code', 'Consolas', 'Courier New', monospace",
                  fontSize: 12,
                  lineHeight: 1.6,
                }}
              >
                {logs.map((line, i) => (
                  <div key={i} style={{ ...getLineStyle(line), whiteSpace: 'pre-wrap', wordBreak: 'break-all' }}>
                    {line || '\u00A0'}
                  </div>
                ))}
              </div>
            </>
          ) : (
            <div style={{ flex: 1, overflow: 'auto' }}>
              <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 13 }}>
                <thead style={{ position: 'sticky', top: 0, background: 'var(--color-win-surface)', zIndex: 10, borderBottom: '1px solid var(--color-win-stroke2)' }}>
                  <tr style={{ textAlign: 'left' }}>
                    <th style={{ padding: '12px 20px', fontWeight: 600, color: 'var(--color-win-text3)', width: 180 }}>Date</th>
                    <th style={{ padding: '12px 20px', fontWeight: 600, color: 'var(--color-win-text3)' }}>Utilisateur</th>
                    <th style={{ padding: '12px 20px', fontWeight: 600, color: 'var(--color-win-text3)' }}>Action</th>
                    <th style={{ padding: '12px 20px', fontWeight: 600, color: 'var(--color-win-text3)' }}>Fichier</th>
                    <th style={{ padding: '12px 20px', fontWeight: 600, color: 'var(--color-win-text3)', textAlign: 'right' }}>Détails</th>
                  </tr>
                </thead>
                <tbody>
                  {activityLogs.length === 0 ? (
                    <tr>
                      <td colSpan={5} style={{ padding: 40, textAlign: 'center', opacity: 0.5 }}>
                        <Database size={32} style={{ margin: '0 auto 12px' }} />
                        <p>Aucune activité enregistrée</p>
                      </td>
                    </tr>
                  ) : activityLogs.map((log) => (
                    <tr key={log.id} style={{ borderBottom: '1px solid var(--color-win-stroke)' }}>
                      <td style={{ padding: '10px 20px', whiteSpace: 'nowrap', color: 'var(--color-win-text3)' }}>
                        <span style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                          <Clock size={12} /> {new Date(log.accessed_at).toLocaleString('fr-FR')}
                        </span>
                      </td>
                      <td style={{ padding: '10px 20px' }}>
                        <span style={{ display: 'flex', alignItems: 'center', gap: 6, fontWeight: 500 }}>
                          <User size={12} style={{ color: 'var(--color-accent)' }} />
                          {log.username || 'Public (Anonyme)'}
                        </span>
                      </td>
                      <td style={{ padding: '10px 20px' }}>
                        <span style={{ 
                          padding: '2px 8px', borderRadius: 4, fontSize: 11, fontWeight: 600, textTransform: 'uppercase',
                          background: log.action === 'upload' ? 'var(--color-success-bg)' : 'var(--color-accent-light)',
                          color: log.action === 'upload' ? 'var(--color-success)' : 'var(--color-accent)'
                        }}>
                          {log.action}
                        </span>
                      </td>
                      <td style={{ padding: '10px 20px' }}>
                        <span style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                          <FileText size={12} style={{ opacity: 0.5 }} />
                          {log.filename}
                        </span>
                      </td>
                      <td style={{ padding: '10px 20px', textAlign: 'right', color: 'var(--color-win-text4)', fontSize: 11 }}>
                        {log.bytes_transferred ? formatSize(log.bytes_transferred) : (log.ip_address || '—')}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
