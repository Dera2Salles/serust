import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { RefreshCcw, ChevronDown, Activity, Terminal, Clock, User, FileText, Database, Trash2, Search } from 'lucide-react';
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
  const [searchQuery, setSearchQuery] = useState('');
  const scrollRef = useRef<HTMLDivElement>(null);

  const fetchLogs = async () => {
    try {
      const c = await invoke<string>('read_server_logs');
      setLogs(c.split('\n'));
    } catch (e) { console.error("Failed to read system logs", e); }
  };

  const fetchActivity = async () => {
    try {
      const data = await invoke<ActivityLog[]>('get_activity_logs_db');
      setActivityLogs(data || []);
    } catch (e) { console.error("Failed to read activity logs", e); }
  };

  const refresh = async () => {
    setLoading(true);
    if (tab === 'system') await fetchLogs();
    else await fetchActivity();
    setLoading(false);
  };

  const clearLogs = async () => {
    if (!window.confirm("Voulez-vous vraiment vider les journaux techniques ?")) return;
    try {
      await invoke('clear_server_logs');
      setLogs([]);
    } catch (e) { alert(`Erreur: ${e}`); }
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
    const l = line.toLowerCase();
    if (l.includes('error') || l.includes('fail')) return { color: '#ff6b6b', fontWeight: 600 };
    if (l.includes('warn')) return { color: '#ffd93d', fontWeight: 600 };
    if (l.includes('info')) return { color: '#6bff6b', fontWeight: 500 };
    if (l.includes('debug')) return { color: '#888', fontStyle: 'italic' };
    return { color: '#e0e0e0' };
  };

  const filteredLogs = tab === 'system' 
    ? logs.filter(line => line.toLowerCase().includes(searchQuery.toLowerCase()))
    : activityLogs.filter(log => 
        (log.username?.toLowerCase() || '').includes(searchQuery.toLowerCase()) ||
        log.filename.toLowerCase().includes(searchQuery.toLowerCase()) ||
        log.action.toLowerCase().includes(searchQuery.toLowerCase())
      );

  return (
    <div style={{ padding: '0 0 40px 0', height: '100%', display: 'flex', flexDirection: 'column' }}>
      <Header title="Journaux Système" subtitle="Surveillance des actions importantes et logs techniques" />

      {/* Toolbar */}
      <div style={{ padding: '0 28px 16px', display: 'flex', gap: 12, alignItems: 'center' }}>
        <div style={{ display: 'flex', background: 'var(--color-win-surface)', borderRadius: 8, padding: 2, border: '1px solid var(--color-win-stroke)' }}>
          <button 
            onClick={() => { setTab('activity'); setSearchQuery(''); }}
            className={cn('fluent-btn', tab === 'activity' && 'fluent-btn-accent')}
            style={{ gap: 8, border: 'none', boxShadow: 'none' }}
          >
            <Activity size={15} /> Historique
          </button>
          <button 
            onClick={() => { setTab('system'); setSearchQuery(''); }}
            className={cn('fluent-btn', tab === 'system' && 'fluent-btn-accent')}
            style={{ gap: 8, border: 'none', boxShadow: 'none' }}
          >
            <Terminal size={15} /> Logs techniques
          </button>
        </div>

        <div style={{ position: 'relative', flex: 1, maxWidth: 400 }}>
          <Search size={14} style={{ position: 'absolute', left: 12, top: '50%', transform: 'translateY(-50%)', color: 'var(--color-win-text3)' }} />
          <input 
            className="fluent-input" 
            placeholder="Rechercher dans les journaux..." 
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            style={{ paddingLeft: 36, width: '100%' }}
          />
        </div>

        <button className="fluent-btn" onClick={refresh} disabled={loading} title="Actualiser">
          <RefreshCcw size={14} className={loading ? 'animate-spin' : ''} />
        </button>

        {tab === 'system' && (
          <button className="fluent-btn fluent-btn-danger" onClick={clearLogs} title="Vider les logs">
            <Trash2 size={14} />
          </button>
        )}
      </div>

      <div style={{ padding: '0 28px', flex: 1, overflow: 'hidden' }}>
        <div className="fluent-card" style={{ height: '100%', display: 'flex', flexDirection: 'column', padding: 0, overflow: 'hidden' }}>
          
          {tab === 'system' ? (
            <>
              <div style={{ padding: '8px 20px', borderBottom: '1px solid var(--color-win-stroke)', display: 'flex', alignItems: 'center', justifyContent: 'space-between', background: 'var(--color-win-nav)' }}>
                <span style={{ fontSize: 11, fontWeight: 700, color: 'var(--color-win-text3)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
                  Fichier: server.log ({logs.length} lignes)
                </span>
                <button
                  className={`fluent-btn ${autoScroll ? 'fluent-btn-accent' : ''}`}
                  style={{ padding: '2px 10px', fontSize: 10, height: 24 }}
                  onClick={() => setAutoScroll(!autoScroll)}
                >
                  <ChevronDown size={12} style={{ transform: autoScroll ? 'none' : 'rotate(180deg)', transition: 'transform 0.2s' }} /> 
                  {autoScroll ? 'AUTO-SCROLL ON' : 'AUTO-SCROLL OFF'}
                </button>
              </div>
              <div
                ref={scrollRef}
                style={{
                  flex: 1,
                  overflow: 'auto',
                  background: '#121212', // Darker black for better contrast
                  padding: '16px 20px',
                  fontFamily: "'Cascadia Code', 'JetBrains Mono', 'Consolas', monospace",
                  fontSize: 12,
                  lineHeight: 1.6,
                }}
              >
                {filteredLogs.length === 0 ? (
                  <div style={{ color: '#555', textAlign: 'center', marginTop: 40 }}>
                    {searchQuery ? 'Aucun résultat pour cette recherche' : 'Le fichier de log est vide'}
                  </div>
                ) : (filteredLogs as string[]).map((line, i) => (
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
                  {filteredLogs.length === 0 ? (
                    <tr>
                      <td colSpan={5} style={{ padding: 40, textAlign: 'center', opacity: 0.5 }}>
                        <Database size={32} style={{ margin: '0 auto 12px' }} />
                        <p>{searchQuery ? 'Aucun résultat correspondant' : 'Aucune activité enregistrée'}</p>
                      </td>
                    </tr>
                  ) : (filteredLogs as ActivityLog[]).map((log) => (
                    <tr key={log.id} style={{ borderBottom: '1px solid var(--color-win-stroke)', transition: 'background 0.1s' }} className="hover:bg-[--color-win-nav]">
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
                          padding: '2px 8px', borderRadius: 4, fontSize: 11, fontWeight: 700, textTransform: 'uppercase',
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

