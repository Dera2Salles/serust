import React, { useState, useEffect } from 'react';
import { Header, SoftCard, AroChip, cn } from './OneUI';
import { Activity, Clock, Globe, Shield, Terminal, ZapOff } from 'lucide-react';

interface Session {
  peer_addr: String;
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
    <div className="flex flex-col w-full min-h-screen bg-transparent pb-20">
      <Header 
        title="Connexions Actives" 
        subtitle="Surveillance en temps réel du serveur FTP" 
      />

      <div className="px-10 grid grid-cols-1 gap-4">
        {loading && sessions.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-20 opacity-50">
             <div className="w-10 h-10 border-4 border-blue/20 border-t-blue rounded-full animate-spin mb-4" />
             <p className="text-subtext0 font-bold">Analyse du trafic...</p>
          </div>
        ) : sessions.length === 0 ? (
          <SoftCard className="flex flex-col items-center justify-center py-24 border-dashed">
            <ZapOff size={64} className="text-surface2 mb-4 opacity-20" />
            <h4 className="text-xl font-black text-text mb-1">Aucune connexion</h4>
            <p className="text-subtext0 font-medium italic opacity-60">Le serveur est actuellement au repos.</p>
          </SoftCard>
        ) : (
          sessions.map((session, i) => (
            <SoftCard key={i} className="flex items-center justify-between group hover:border-blue/20 transition-all py-6">
              <div className="flex items-center gap-6">
                <div className={cn(
                  "w-16 h-16 rounded-[24px] flex items-center justify-center font-black text-2xl shadow-lg transition-transform group-hover:scale-105",
                  session.username ? "bg-gradient-to-br from-blue to-mauve text-crust shadow-blue/20" : "bg-surface0 text-subtext0 border border-surface1"
                )}>
                  {session.username ? session.username.charAt(0).toUpperCase() : <Globe size={28} />}
                </div>
                <div>
                  <div className="flex items-center gap-3 mb-1.5">
                    <h4 className="font-black text-xl tracking-tighter">{session.username || 'Client Anonyme'}</h4>
                    {session.username ? (
                      <AroChip label="Authentifié" color="blue" />
                    ) : (
                      <AroChip label="En attente" color="yellow" />
                    )}
                  </div>
                  <div className="flex flex-wrap gap-5 text-sm text-subtext0 font-medium">
                    <span className="flex items-center gap-2 bg-surface0/50 px-3 py-1 rounded-xl">
                      <Terminal size={14} className="text-blue" />
                      <span className="font-mono text-xs">{session.peer_addr}</span>
                    </span>
                    <span className="flex items-center gap-2">
                      <Clock size={14} className="opacity-50" /> 
                      Depuis {new Date(session.connected_at).toLocaleTimeString()}
                    </span>
                    {session.last_command && (
                      <span className="flex items-center gap-2">
                        <Activity size={14} className="text-mauve" /> 
                        Dernière cmd: <span className="font-black text-text tracking-tighter">{session.last_command}</span>
                      </span>
                    )}
                  </div>
                </div>
              </div>
              
              <div className="flex items-center gap-3">
                <button className="px-5 py-2.5 bg-red/10 text-red hover:bg-red hover:text-crust font-black text-xs uppercase tracking-widest rounded-full transition-all active:scale-95">
                  Kick Session
                </button>
              </div>
            </SoftCard>
          ))
        )}
      </div>
    </div>
  );
};
