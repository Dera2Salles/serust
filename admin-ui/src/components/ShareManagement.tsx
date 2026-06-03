import { useState, useEffect } from 'react';
import { Header, SoftCard, AroChip, cn } from './OneUI';
import { Share2, Link as LinkIcon, User, Trash2, Calendar, RefreshCw, FileText, ExternalLink, ShieldCheck } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface Share {
  id: string;
  type: 'direct' | 'link';
  owner: string;
  grantee: string;
  filename: string;
  path: string;
  token?: string;
  label?: string | null;
  can_read: boolean;
  can_write: boolean;
  expires_at: string | null;
  is_active?: boolean;
}

export const ShareManagement = () => {
  const [shares, setShares] = useState<Share[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchShares = async () => {
    setLoading(true);
    try {
      const data = await invoke<Share[]>('get_all_shares_db');
      setShares(data || []);
    } catch (err) {
      console.error("Failed to fetch shares:", err);
    } finally {
      setLoading(false);
    }
  };

  const handleRevoke = async (share: Share) => {
    if (!confirm(`Voulez-vous vraiment révoquer définitivement le partage de "${share.filename}" ?`)) return;
    try {
      if (share.type === 'direct') {
        await invoke('revoke_share_grant_db', { id: share.id });
      } else {
        await invoke('revoke_share_link_db', { id: share.id });
      }
      fetchShares();
    } catch (err) {
      alert("Erreur lors de la révocation : " + err);
    }
  };

  useEffect(() => {
    fetchShares();
  }, []);

  return (
    <div className="pb-10">
      <Header 
        title="Partages" 
        subtitle="Surveillance de tous les accès et liens publics générés" 
      />

      <div className="px-8 mb-8 flex justify-end">
        <button 
          onClick={fetchShares}
          className="flex items-center gap-2 px-6 py-3.5 bg-white hover:bg-[--color-win-nav] text-[--color-win-text] border border-[--color-win-stroke] font-semibold rounded-md transition-all active:scale-95 shadow-sm"
        >
          <RefreshCw size={16} className={loading ? "animate-spin" : ""} />
          Actualiser
        </button>
      </div>

      <div className="px-8 space-y-5">
        {loading && shares.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-20 opacity-40">
             <div className="w-10 h-10 border-4 border-[--color-accent]/20 border-t-blue rounded-full animate-spin mb-4" />
             <p className="text-[--color-win-text3] font-semibold uppercase text-[10px] tracking-widest">Scan des partages...</p>
          </div>
        ) : shares.length === 0 ? (
          <div className="fluent-card flex flex-col items-center justify-center py-32 border-dashed border-2 border-[--color-win-stroke]  shadow-none opacity-50">
            <Share2 size={64} className="text-surface2 mb-4 opacity-20" />
            <p className="text-[--color-win-text3] font-bold">Aucun partage actif</p>
          </div>
        ) : (
          shares.map((share, i) => {
            const isLink = share.type === 'link';
            return (
              <div key={i} className="fluent-card flex flex-col lg:flex-row lg:items-center justify-between gap-6 group transition-all py-8 bg-white shadow-sm border border-[--color-win-stroke]/50">
                <div className="flex items-center gap-7">
                  <div className={cn(
                    "w-16 h-16 rounded-lg flex items-center justify-center shadow-lg transition-all",
                    isLink ? "bg-[--color-win-nav] text-[--color-win-text2]" : "bg-[--color-accent]/5 text-[--color-accent]"
                  )}>
                    {isLink ? <LinkIcon size={28} /> : <User size={28} />}
                  </div>
                  <div>
                    <div className="flex items-center gap-3 mb-2">
                      <h4 className="font-semibold text-xl tracking-tighter truncate max-w-md flex items-center gap-2 text-[--color-win-text]">
                        <FileText size={18} className="opacity-20" />
                        {share.filename}
                      </h4>
                      <AroChip 
                        label={isLink ? "Lien Public" : "Privé"} 
                        color={isLink ? "overlay2" : "blue"} 
                      />
                      {share.is_active === false && <AroChip label="Inactif" color="red" />}
                    </div>
                    <div className="flex flex-wrap items-center gap-6 text-sm text-[--color-win-text3] font-medium">
                      <span className="flex items-center gap-2">
                        <ShieldCheck size={14} className="text-[--color-success] opacity-60" />
                        Par : <strong className="text-[--color-win-text] font-semibold tracking-tight">{share.owner}</strong>
                      </span>
                      <span className="flex items-center gap-2">
                        <ExternalLink size={14} className="opacity-40" />
                        Pour : <strong className={cn("font-semibold tracking-tight", isLink ? "text-[--color-win-text2]" : "text-[--color-win-text]")}>{share.grantee}</strong>
                      </span>
                      <span className="flex items-center gap-1.5 opacity-60">
                        <Calendar size={14} /> 
                        {share.expires_at ? `Expire : ${new Date(share.expires_at).toLocaleDateString()}` : 'Illimité'}
                      </span>
                    </div>
                  </div>
                </div>
                
                <div className="flex items-center gap-4 self-end lg:self-center">
                  <div className="flex flex-col items-end mr-4">
                     <p className="text-[10px] font-semibold uppercase tracking-widest text-[--color-win-text3] opacity-40 mb-1">Permissions</p>
                     <div className="flex gap-1">
                        <span className={cn("px-2 py-0.5 rounded text-[9px] font-semibold", share.can_read ? "bg-[--color-success]/10 text-[--color-success]" : "bg-[--color-error]/10 text-[--color-error]")}>READ</span>
                        <span className={cn("px-2 py-0.5 rounded text-[9px] font-semibold", share.can_write ? "bg-[--color-warning]/10 text-[--color-warning]" : "bg-[--color-error]/10 text-[--color-error]")}>WRITE</span>
                     </div>
                  </div>
                  <button 
                    onClick={() => handleRevoke(share)}
                    className="w-14 h-14 flex items-center justify-center bg-white text-[--color-win-text3] hover:text-[--color-error] hover:bg-[--color-error]/5 rounded-lg transition-all active:scale-90 border border-[--color-win-stroke] shadow-sm"
                    title="Révoquer"
                  >
                    <Trash2 size={24} />
                  </button>
                </div>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
};
