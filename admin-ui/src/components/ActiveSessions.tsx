import { useState, useEffect } from 'react';
import { Header, AroChip, cn } from './OneUI';
import { Activity, Clock, Globe, Terminal, ZapOff, User } from 'lucide-react';

interface Session {
  id: string;
  peer_addr: string;
  connected_at: string;
  last_command: string | null;
  username: string | null;
}

export const ActiveSessions = () => {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchStatus = async () => {
    try {
      const response = await fetch('http://localhost:8081/api/server/status');
      const data = await response.json();
      setSessions(data.sessions || []);
    } catch (err) {
      console.error("Failed to fetch server status:", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, 3000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="pb-10">
      <Header 
        title="Connexions" 
        subtitle="Surveillance en temps réel des accès réseau" 
      />

      <div className="px-8 grid grid-cols-1 gap-5">
        {loading && sessions.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-20 opacity-40">
             <div className="w-10 h-10 border-4 border-[--color-accent]/20 border-t-[--color-accent] rounded-full animate-spin mb-4" />
             <p className="text-[--color-win-text3] font-semibold uppercase text-[10px] tracking-widest">Écoute du port 8081...</p>
          </div>
        ) : sessions.length === 0 ? (
          <div className="fluent-card flex flex-col items-center justify-center py-32 border-dashed border-2 border-[--color-win-stroke]  shadow-none opacity-50">
            <ZapOff size={64} className="text-surface2 mb-4 opacity-10" />
            <h4 className="text-xl font-semibold text-[--color-win-text] mb-1 tracking-tight">Aucune activité réseau</h4>
            <p className="text-sm font-medium text-[--color-win-text3]">Le serveur est en attente de connexions.</p>
          </div>
        ) : (
          sessions.map((session, i) => (
            <div key={i} className="fluent-card flex items-center justify-between group transition-all py-8 bg-white shadow-sm border border-[--color-win-stroke]/50">
              <div className="flex items-center gap-7">
                <div className={cn(
                  "w-14 h-14 rounded-xl flex items-center justify-center transition-all duration-300",
                  session.username 
                    ? "bg-[--color-accent] text-white" 
                    : "bg-[--color-win-nav] text-[--color-win-text3] border border-[--color-win-stroke]"
                )}>
                  {session.username ? <User size={28} /> : <Globe size={28} className="opacity-40" />}
                </div>
                <div>
                  <div className="flex items-center gap-3 mb-2">
                    <h4 className="font-semibold text-xl tracking-tighter text-[--color-win-text]">{session.username || 'Client Anonyme'}</h4>
                    {session.username ? (
                      <AroChip label="Authentifié" color="blue" />
                    ) : (
                      <AroChip label="Non authentifié" color="red" />
                    )}
                  </div>
                  <div className="flex flex-wrap items-center gap-6 text-sm text-[--color-win-text3] font-medium">
                    <span className="flex items-center gap-2 bg-[--color-win-nav] px-4 py-1.5 rounded-full border border-[--color-win-stroke] shadow-inner">
                      <Terminal size={14} className="text-[--color-accent]" />
                      <span className="font-mono text-xs font-bold">{session.peer_addr}</span>
                    </span>
                    <span className="flex items-center gap-2 opacity-60">
                      <Clock size={14} /> 
                      Depuis {new Date(session.connected_at).toLocaleTimeString()}
                    </span>
                    {session.last_command && (
                      <span className="flex items-center gap-2">
                        <Activity size={14} className="text-[--color-accent] opacity-50" /> 
                        Action : <span className="font-semibold text-[--color-win-text] tracking-tight uppercase text-xs bg-[--color-accent]/5 px-2 py-0.5 rounded">{session.last_command}</span>
                      </span>
                    )}
                  </div>
                </div>
              </div>
              
              <div className="flex items-center gap-3">
                <button className="px-6 py-3 bg-[--color-error]/5 text-[--color-error] hover:bg-[--color-error] hover:text-white font-semibold text-[10px] uppercase tracking-widest rounded-full transition-all active:scale-95 border border-red/10 shadow-sm">
                  Kick Session
                </button>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};
