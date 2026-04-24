import React, { useState, useEffect } from 'react';
import { Header, SoftCard, AroChip, Button, cn } from './OneUI';
import { Share2, Link as LinkIcon, User, Trash2, Calendar, Lock } from 'lucide-react';
import { callMcpTool } from '../lib/mcp';

interface Share {
  path: string;
  grantee: string;
  can_read: boolean;
  can_write: boolean;
  expires_at: string | null;
}

export const ShareManagement = () => {
  const [shares, setShares] = useState<Share[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchShares = async () => {
    setLoading(true);
    try {
      // For now, listing 'alice' shares as a demo, but a true admin tool should list all.
      // Since MCP tool list_outgoing_shares is per-user, we use 'admin_dev' or loop.
      const result = await callMcpTool('list_outgoing_shares', { username: 'admin_dev' });
      const data = JSON.parse(result.content[0].text);
      setShares(data.shares || []);
    } catch (err) {
      console.error("Failed to fetch shares:", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchShares();
  }, []);

  return (
    <div className="flex flex-col w-full min-h-screen bg-transparent pb-20">
      <Header 
        title="Partages Globaux" 
        subtitle="Surveillance des accès et liens publics" 
      />

      <div className="px-10 grid grid-cols-1 gap-4">
        {loading ? (
          <div className="flex flex-col items-center justify-center py-20 opacity-50">
             <div className="w-10 h-10 border-4 border-blue/20 border-t-blue rounded-full animate-spin mb-4" />
             <p className="text-subtext0 font-bold tracking-tight">Récupération des partages...</p>
          </div>
        ) : shares.length === 0 ? (
          <SoftCard className="flex flex-col items-center justify-center py-32 border-dashed opacity-50">
            <Share2 size={64} className="text-surface2 mb-4 opacity-20" />
            <p className="text-subtext0 font-bold">Aucun partage actif sur le serveur</p>
          </SoftCard>
        ) : (
          shares.map((share, i) => (
            <SoftCard key={i} className="flex items-center justify-between group hover:border-mauve/20 transition-all py-6">
              <div className="flex items-center gap-6">
                <div className="w-14 h-14 rounded-2xl bg-mauve/10 text-mauve flex items-center justify-center shadow-inner">
                  {share.grantee.startsWith('http') ? <LinkIcon size={24} /> : <User size={24} />}
                </div>
                <div>
                  <div className="flex items-center gap-3 mb-1.5">
                    <h4 className="font-black text-lg tracking-tighter truncate max-w-md">{share.path}</h4>
                    <AroChip label={share.can_write ? "Lecture/Écriture" : "Lecture Seule"} color={share.can_write ? "peach" : "blue"} />
                  </div>
                  <div className="flex items-center gap-6 text-sm text-subtext0 font-medium opacity-70">
                    <span className="flex items-center gap-2">
                      <User size={14} /> Partagé avec <span className="text-text font-black">{share.grantee}</span>
                    </span>
                    <span className="flex items-center gap-2">
                      <Calendar size={14} /> 
                      {share.expires_at ? `Expire le ${new Date(share.expires_at).toLocaleDateString()}` : 'Accès illimité'}
                    </span>
                    <span className="flex items-center gap-2">
                      <Lock size={14} /> AES-256 GCM
                    </span>
                  </div>
                </div>
              </div>
              
              <div className="flex items-center gap-3">
                <button className="w-12 h-12 flex items-center justify-center bg-red/10 text-red hover:bg-red hover:text-crust rounded-2xl transition-all active:scale-90">
                  <Trash2 size={20} />
                </button>
              </div>
            </SoftCard>
          ))
        )}
      </div>
    </div>
  );
};
