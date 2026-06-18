import { useState, useEffect, useMemo } from "react";
import { Header, cn } from "./OneUI";
import {
  Share2,
  Link as LinkIcon,
  User,
  Trash2,
  Calendar,
  RefreshCw,
  ExternalLink,
  ShieldCheck,
  Search,
  Copy,
  CheckCircle2,
  AlertCircle,
  Clock,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

interface Share {
  id: string;
  type: "direct" | "link";
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
  const [query, setQuery] = useState("");
  const [copiedId, setCopyId] = useState<string | null>(null);

  const fetchShares = async () => {
    setLoading(true);
    try {
      const data = await invoke<Share[]>("get_all_shares_db");
      setShares(data || []);
    } catch (err) {
      console.error("Failed to fetch shares:", err);
    } finally {
      setLoading(false);
    }
  };

  const handleRevoke = async (share: Share) => {
    if (
      !confirm(
        `Voulez-vous vraiment révoquer définitivement le partage de "${share.filename}" ?`,
      )
    )
      return;
    try {
      if (share.type === "direct") {
        await invoke("revoke_share_grant_db", { id: share.id });
      } else {
        await invoke("revoke_share_link_db", { id: share.id });
      }
      fetchShares();
    } catch (err) {
      alert("Erreur lors de la révocation : " + err);
    }
  };

  const copyToken = async (share: Share) => {
    if (!share.token) return;
    try {
      await navigator.clipboard.writeText(share.token);
      setCopyId(share.id);
      setTimeout(() => setCopyId(null), 2000);
    } catch {
      // Clipboard access denied or not available
      console.error("Failed to copy to clipboard");
    }
  };

  useEffect(() => {
    fetchShares();
  }, []);

  const filteredShares = useMemo(() => {
    return shares.filter(
      (s) =>
        s.filename.toLowerCase().includes(query.toLowerCase()) ||
        s.owner.toLowerCase().includes(query.toLowerCase()) ||
        s.grantee.toLowerCase().includes(query.toLowerCase()),
    );
  }, [shares, query]);

  const stats = {
    total: shares.length,
    links: shares.filter((s) => s.type === "link").length,
    grants: shares.filter((s) => s.type === "direct").length,
    expired: shares.filter(
      (s) =>
        (s.expires_at && new Date(s.expires_at) < new Date()) ||
        s.is_active === false,
    ).length,
  };

  return (
    <div className="pb-10">
      <Header
        title="Gestion des Partages"
        subtitle="Surveillance des accès collaboratifs et des liens de diffusion"
      />

      {/* Stats row */}
      <div className="px-8 mb-8 grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="fluent-stat">
          <div className="flex items-center justify-between">
            <span className="text-[11px] font-semibold uppercase tracking-wider text-[--color-win-text3]">
              Total Partages
            </span>
            <Share2 size={14} className="text-[--color-accent]" />
          </div>
          <p className="text-2xl font-bold text-[--color-win-text]">
            {stats.total}
          </p>
        </div>
        <div className="fluent-stat">
          <div className="flex items-center justify-between">
            <span className="text-[11px] font-semibold uppercase tracking-wider text-[--color-win-text3]">
              Liens Publics
            </span>
            <LinkIcon size={14} className="text-[--color-accent]" />
          </div>
          <p className="text-2xl font-bold text-[--color-win-text]">
            {stats.links}
          </p>
        </div>
        <div className="fluent-stat">
          <div className="flex items-center justify-between">
            <span className="text-[11px] font-semibold uppercase tracking-wider text-[--color-win-text3]">
              Accès Directs
            </span>
            <User size={14} className="text-[--color-accent]" />
          </div>
          <p className="text-2xl font-bold text-[--color-win-text]">
            {stats.grants}
          </p>
        </div>
        <div className="fluent-stat">
          <div className="flex items-center justify-between">
            <span className="text-[11px] font-semibold uppercase tracking-wider text-[--color-win-text3]">
              Inactifs / Expirés
            </span>
            <Clock size={14} className="text-[--color-error]" />
          </div>
          <p className="text-2xl font-bold text-[--color-error]">
            {stats.expired}
          </p>
        </div>
      </div>

      {/* Toolbar */}
      <div className="px-8 mb-6 flex items-center justify-between gap-4">
        <div className="fluent-commandbar flex-1 max-w-md">
          <Search size={16} className="text-[--color-win-text3] ml-1" />
          <input
            type="text"
            placeholder="Filtrer par fichier, propriétaire..."
            className="fluent-input border-none shadow-none bg-transparent"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
        </div>
        <button
          onClick={fetchShares}
          className="fluent-btn flex items-center gap-2"
        >
          <RefreshCw size={14} className={loading ? "animate-spin" : ""} />
          Actualiser
        </button>
      </div>

      <div className="px-8 space-y-3">
        {loading && shares.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-20 opacity-40">
            <div className="w-10 h-10 border-4 border-[--color-win-stroke2] border-t-[--color-accent] rounded-full animate-spin mb-4" />
            <p className="text-xs font-bold uppercase tracking-widest text-[--color-win-text3]">
              Chargement des partages...
            </p>
          </div>
        ) : filteredShares.length === 0 ? (
          <div className="fluent-card flex flex-col items-center justify-center py-24 border-dashed border-2 opacity-50">
            <AlertCircle size={48} className="text-[--color-win-text4] mb-4" />
            <p className="text-[--color-win-text] font-semibold">
              Aucun partage ne correspond à votre recherche
            </p>
          </div>
        ) : (
          filteredShares.map((share) => {
            const isLink = share.type === "link";
            const isExpired =
              share.expires_at && new Date(share.expires_at) < new Date();
            const isActive = share.is_active !== false && !isExpired;

            return (
              <div
                key={share.id}
                className={cn(
                  "fluent-card group flex items-center gap-6 p-4 transition-all hover:bg-[--color-win-surface]",
                  !isActive && "opacity-60 grayscale-[0.5]",
                )}
              >
                {/* Type Icon */}
                <div
                  className={cn(
                    "w-12 h-12 rounded-lg flex items-center justify-center flex-shrink-0",
                    isLink
                      ? "bg-[--color-accent-subtle] text-[--color-accent]"
                      : "bg-[--color-success-bg] text-[--color-success]",
                  )}
                >
                  {isLink ? <LinkIcon size={20} /> : <User size={20} />}
                </div>

                {/* File Info */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-1">
                    <h4 className="text-[15px] font-bold text-[--color-win-text] truncate">
                      {share.filename}
                    </h4>
                    <span
                      className={cn(
                        "px-2 py-0.5 rounded text-[9px] font-bold uppercase tracking-tighter",
                        isLink
                          ? "bg-[--color-accent] text-white"
                          : "bg-[--color-win-nav] text-[--color-win-text2]",
                      )}
                    >
                      {isLink ? "Lien Public" : "Direct"}
                    </span>
                    {!isActive && (
                      <span className="bg-[--color-error-bg] text-[--color-error] px-2 py-0.5 rounded text-[9px] font-bold uppercase">
                        {isExpired ? "Expiré" : "Inactif"}
                      </span>
                    )}
                  </div>

                  <div className="flex flex-wrap items-center gap-x-5 gap-y-1 text-[12px] text-[--color-win-text3]">
                    <span className="flex items-center gap-1.5">
                      <ShieldCheck
                        size={13}
                        className="text-[--color-accent] opacity-60"
                      />
                      De{" "}
                      <span className="font-bold text-[--color-win-text2]">
                        {share.owner}
                      </span>
                    </span>
                    <span className="flex items-center gap-1.5">
                      <ExternalLink size={13} className="opacity-40" />
                      Pour{" "}
                      <span className="font-bold text-[--color-win-text2]">
                        {share.grantee}
                      </span>
                    </span>
                    {share.expires_at && (
                      <span className="flex items-center gap-1.5">
                        <Calendar size={13} className="opacity-40" />
                        Expire :{" "}
                        {new Date(share.expires_at).toLocaleDateString()}
                      </span>
                    )}
                  </div>
                </div>

                {/* Permissions & Actions */}
                <div className="flex items-center gap-6">
                  <div className="hidden xl:flex flex-col items-end gap-1">
                    <span className="text-[9px] font-bold uppercase tracking-widest opacity-40">
                      Permissions
                    </span>
                    <div className="flex gap-1">
                      <div
                        className={cn(
                          "w-2 h-2 rounded-full",
                          share.can_read
                            ? "bg-[--color-success]"
                            : "bg-[--color-error]",
                        )}
                        title="Lecture"
                      />
                      <div
                        className={cn(
                          "w-2 h-2 rounded-full",
                          share.can_write
                            ? "bg-[--color-warning]"
                            : "bg-[--color-error]",
                        )}
                        title="Écriture"
                      />
                    </div>
                  </div>

                  {isLink && (
                    <button
                      onClick={() => copyToken(share)}
                      className="flex items-center gap-2 px-3 py-1.5 bg-[--color-win-nav] hover:bg-[--color-win-nav-hover] text-[--color-win-text] rounded-md border border-[--color-win-stroke] transition-all active:scale-95"
                    >
                      {copiedId === share.id ? (
                        <CheckCircle2
                          size={14}
                          className="text-[--color-success]"
                        />
                      ) : (
                        <Copy size={14} />
                      )}
                      <span className="text-[11px] font-bold uppercase tracking-tighter">
                        Token
                      </span>
                    </button>
                  )}

                  <button
                    onClick={() => handleRevoke(share)}
                    className="w-10 h-10 flex items-center justify-center bg-transparent text-[--color-win-text3] hover:text-[--color-error] hover:bg-[--color-error-bg] rounded-lg transition-all active:scale-90"
                    title="Révoquer"
                  >
                    <Trash2 size={18} />
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
