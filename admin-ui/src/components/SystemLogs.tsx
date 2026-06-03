import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { RefreshCcw, ChevronDown } from 'lucide-react';
import { Header } from './OneUI';

export const SystemLogs: React.FC = () => {
  const [logs, setLogs] = useState<string[]>([]);
  const [autoScroll, setAutoScroll] = useState(true);
  const scrollRef = useRef<HTMLDivElement>(null);

  const fetchLogs = async () => {
    try {
      const c = await invoke<string>('read_server_logs');
      setLogs(c.split('\n'));
    } catch {}
  };

  useEffect(() => { fetchLogs(); const t = setInterval(fetchLogs, 3000); return () => clearInterval(t); }, []);
  useEffect(() => { if (autoScroll && scrollRef.current) scrollRef.current.scrollTop = scrollRef.current.scrollHeight; }, [logs, autoScroll]);

  const getLineStyle = (line: string): React.CSSProperties => {
    if (line.includes('ERROR') || line.includes('error')) return { color: '#c42b1c', fontWeight: 600 };
    if (line.includes('WARN') || line.includes('warn')) return { color: '#7a4100', fontWeight: 600 };
    if (line.includes('INFO') || line.includes('info')) return { color: '#107c10', fontWeight: 500 };
    return { color: 'var(--color-win-text)' };
  };

  return (
    <div style={{ padding: '0 0 40px 0', height: '100%', display: 'flex', flexDirection: 'column' }}>
      <Header title="Journaux Serveur" subtitle="Sortie temps réel du backend Kajy" />

      {/* Toolbar */}
      <div style={{ padding: '0 32px 16px', display: 'flex', gap: 8, alignItems: 'center' }}>
        <button className="fluent-btn flex items-center gap-2" onClick={fetchLogs}>
          <RefreshCcw size={14} /> Actualiser
        </button>
        <button
          className={`fluent-btn flex items-center gap-2 ${autoScroll ? 'fluent-btn-accent' : ''}`}
          onClick={() => setAutoScroll(!autoScroll)}
        >
          <ChevronDown size={14} />
          {autoScroll ? 'Auto-scroll actif' : 'Auto-scroll désactivé'}
        </button>
        <span style={{ marginLeft: 'auto', fontSize: 12, color: 'var(--color-win-text3)' }}>
          {logs.length} lignes
        </span>
      </div>

      {/* Log viewport */}
      <div style={{ padding: '0 32px', flex: 1 }}>
        <div
          ref={scrollRef}
          className="fluent-card"
          style={{
            height: 480,
            overflow: 'auto',
            background: 'var(--color-win-surface)',
            border: '1px solid var(--color-win-stroke2)',
            padding: '16px 20px',
            fontFamily: "'Cascadia Code', 'Consolas', 'Courier New', monospace",
            fontSize: 12,
            lineHeight: 1.6,
            boxShadow: 'inset 0 1px 3px rgba(0,0,0,0.05)',
          }}
        >
          {logs.map((line, i) => (
            <div key={i} style={{ ...getLineStyle(line), whiteSpace: 'pre-wrap', wordBreak: 'break-all' }}>
              {line || '\u00A0'}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
