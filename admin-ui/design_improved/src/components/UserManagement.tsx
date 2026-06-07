import React, { useState, useEffect } from 'react';
import { Header, AroChip, ModernTextField } from './OneUI';
import { Users, UserPlus, Search, Edit3, Trash2, Mail, Key, X, Check, RefreshCw } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface User {
  id: string;
  username: string;
  email: string;
  storage_quota_bytes: number;
  storage_used_bytes: number;
  is_active: boolean;
  created_at: string;
}

const formatSize = (bytes: number) => {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let val = bytes, idx = 0;
  while (val >= 1024 && idx < units.length - 1) { val /= 1024; idx++; }
  return `${val.toFixed(1)} ${units[idx]}`;
};

const AVATAR_COLORS = [
  { bg: '#eff4ff', text: '#2563eb' },
  { bg: '#ecfdf5', text: '#059669' },
  { bg: '#f5f3ff', text: '#7c3aed' },
  { bg: '#fff7ed', text: '#ea580c' },
  { bg: '#fdf2f8', text: '#db2777' },
];

export const UserManagement = () => {
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  const [query, setQuery] = useState('');
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [formData, setFormData] = useState({ username: '', email: '', password: '', quota_gb: 5 });

  const fetchUsers = async () => {
    setLoading(true);
    try {
      const data = await invoke<User[]>('get_users_from_db');
      setUsers(data || []);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  const fetchDefaultQuota = async () => {
    try {
      const settings = await invoke<any>('get_global_settings');
      if (settings?.default_storage_quota_gb) {
        setFormData(prev => ({ ...prev, quota_gb: settings.default_storage_quota_gb }));
      }
    } catch (e) { console.error(e); }
  };

  useEffect(() => { fetchUsers(); fetchDefaultQuota(); }, []);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await invoke('create_user_db', {
        username: formData.username,
        email: formData.email,
        passwordRaw: formData.password,
        quota: Number(formData.quota_gb) * 1024 * 1024 * 1024,
      });
      setIsCreateOpen(false);
      setFormData({ username: '', email: '', password: '', quota_gb: 10 });
      fetchUsers();
    } catch (e) { alert(e); }
  };

  const handleApprove = async (user: User) => {
    try {
      await invoke('update_user_db', { id: user.id, email: user.email, quota: user.storage_quota_bytes, isActive: true });
      fetchUsers();
    } catch (e) { alert(e); }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Supprimer cet utilisateur ?')) return;
    try { await invoke('delete_user_db', { id }); fetchUsers(); } catch (e) { alert(e); }
  };

  const filtered = users.filter(u =>
    u.username.toLowerCase().includes(query.toLowerCase()) ||
    u.email.toLowerCase().includes(query.toLowerCase())
  );

  return (
    <div style={{ paddingBottom: 48 }}>
      <Header title="Utilisateurs" subtitle="Gérer les comptes et quotas de stockage" />

      {/* Stats */}
      <div style={{ padding: '0 28px 22px', display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 12 }}>
        {[
          { label: 'Total Utilisateurs', value: users.length,                        icon: Users,     bg: 'var(--color-accent-light)',  color: 'var(--color-accent)' },
          { label: 'Actifs',             value: users.filter(u => u.is_active).length, icon: Check,   bg: 'var(--color-success-bg)',    color: 'var(--color-success)' },
          { label: 'En attente',         value: users.filter(u => !u.is_active).length, icon: RefreshCw, bg: 'var(--color-warning-bg)', color: 'var(--color-warning)' },
        ].map((s, i) => {
          const Icon = s.icon;
          return (
            <div className="fluent-stat" key={i}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <span style={{ fontSize: 11.5, color: 'var(--color-win-text3)', fontWeight: 500 }}>{s.label}</span>
                <div style={{ width: 30, height: 30, borderRadius: 7, background: s.bg, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                  <Icon size={15} style={{ color: s.color }} />
                </div>
              </div>
              <p style={{ fontSize: 28, fontWeight: 700, color: 'var(--color-win-text)', margin: 0, letterSpacing: '-0.5px', lineHeight: 1.1 }}>{s.value}</p>
            </div>
          );
        })}
      </div>

      {/* Toolbar */}
      <div style={{ padding: '0 28px 16px', display: 'flex', gap: 10, alignItems: 'center' }}>
        <div style={{
          flex: 1, maxWidth: 340,
          display: 'flex', alignItems: 'center', gap: 8,
          background: 'var(--color-win-surface)',
          border: '1px solid var(--color-win-stroke2)',
          borderRadius: 8, padding: '8px 12px',
          boxShadow: 'var(--shadow-2)',
        }}>
          <Search size={14} style={{ color: 'var(--color-win-text3)', flexShrink: 0 }} />
          <input
            style={{ border: 'none', outline: 'none', background: 'transparent', flex: 1, fontSize: 13.5, fontFamily: 'inherit', color: 'var(--color-win-text)' }}
            placeholder="Rechercher un utilisateur..."
            value={query}
            onChange={e => setQuery(e.target.value)}
          />
        </div>
        <button className="fluent-btn fluent-btn-accent" onClick={() => setIsCreateOpen(true)}>
          <UserPlus size={14} />
          Ajouter un utilisateur
        </button>
      </div>

      {/* Table */}
      <div style={{ padding: '0 28px' }}>
        {/* Header */}
        <div style={{
          display: 'grid',
          gridTemplateColumns: '1.5fr 1.8fr 180px 100px 130px',
          padding: '0 16px 8px',
          fontSize: 11.5,
          fontWeight: 600,
          color: 'var(--color-win-text3)',
          textTransform: 'uppercase',
          letterSpacing: '0.06em',
        }}>
          <span>Utilisateur</span>
          <span>Email</span>
          <span>Stockage</span>
          <span>Statut</span>
          <span>Actions</span>
        </div>

        {/* Rows */}
        <div className="fluent-card" style={{ padding: 0, overflow: 'hidden' }}>
          {loading ? (
            <div className="flex items-center justify-center py-16">
              <div style={{ width: 24, height: 24, border: '2px solid var(--color-win-stroke2)', borderTopColor: 'var(--color-accent)', borderRadius: '50%', animation: 'spin 0.8s linear infinite' }} />
            </div>
          ) : filtered.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 gap-3" style={{ opacity: 0.3 }}>
              <Users size={40} />
              <span style={{ fontSize: 14, fontWeight: 600 }}>Aucun utilisateur trouvé</span>
            </div>
          ) : filtered.map((user, i) => {
            const avatarStyle = AVATAR_COLORS[i % AVATAR_COLORS.length];
            const usedPct = Math.min(100, (user.storage_used_bytes / Math.max(user.storage_quota_bytes, 1)) * 100);
            return (
              <div key={user.id}>
                {i > 0 && <div style={{ height: 1, background: 'var(--color-win-stroke)', margin: '0 16px' }} />}
                <div
                  style={{
                    display: 'grid',
                    gridTemplateColumns: '1.5fr 1.8fr 180px 100px 130px',
                    padding: '12px 16px',
                    alignItems: 'center',
                    transition: 'background 0.1s',
                    cursor: 'default',
                  }}
                  onMouseEnter={e => (e.currentTarget.style.background = 'var(--color-accent-subtle)')}
                  onMouseLeave={e => (e.currentTarget.style.background = '')}
                >
                  {/* Name + avatar */}
                  <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                    <div style={{
                      width: 34, height: 34, borderRadius: '50%', flexShrink: 0,
                      background: avatarStyle.bg,
                      display: 'flex', alignItems: 'center', justifyContent: 'center',
                      color: avatarStyle.text,
                      fontWeight: 700, fontSize: 13,
                    }}>
                      {user.username.charAt(0).toUpperCase()}
                    </div>
                    <div>
                      <p style={{ fontSize: 13.5, fontWeight: 600, margin: 0, color: 'var(--color-win-text)' }}>{user.username}</p>
                      <p style={{ fontSize: 11, color: 'var(--color-win-text3)', margin: 0 }}>
                        {new Date(user.created_at).toLocaleDateString('fr-FR', { day: '2-digit', month: 'short', year: 'numeric' })}
                      </p>
                    </div>
                  </div>

                  {/* Email */}
                  <span style={{ fontSize: 13, color: 'var(--color-win-text2)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {user.email}
                  </span>

                  {/* Storage bar */}
                  <div style={{ paddingRight: 16 }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 4 }}>
                      <span style={{ fontSize: 11.5, fontWeight: 500, color: 'var(--color-win-text2)' }}>{formatSize(user.storage_used_bytes)}</span>
                      <span style={{ fontSize: 11, color: 'var(--color-win-text4)' }}>/ {formatSize(user.storage_quota_bytes)}</span>
                    </div>
                    <div className="fluent-progress">
                      <div
                        className="fluent-progress-fill"
                        style={{
                          width: `${usedPct}%`,
                          background: usedPct > 90 ? 'var(--color-error)' : usedPct > 70 ? 'var(--color-warning)' : 'var(--color-accent)',
                        }}
                      />
                    </div>
                  </div>

                  {/* Status */}
                  <span>
                    <AroChip label={user.is_active ? 'Actif' : 'En attente'} color={user.is_active ? 'green' : 'yellow'} />
                  </span>

                  {/* Actions */}
                  <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                    {!user.is_active && (
                      <button
                        className="fluent-btn fluent-btn-accent"
                        style={{ padding: '4px 8px', gap: 4, fontSize: 12 }}
                        onClick={() => handleApprove(user)}
                        title="Approuver"
                      >
                        <Check size={12} />
                      </button>
                    )}
                    <button className="fluent-btn" style={{ padding: '4px 8px' }} title="Modifier">
                      <Edit3 size={13} />
                    </button>
                    <button
                      className="fluent-btn fluent-btn-danger"
                      style={{ padding: '4px 8px' }}
                      onClick={() => handleDelete(user.id)}
                      title="Supprimer"
                    >
                      <Trash2 size={13} />
                    </button>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Create dialog */}
      {isCreateOpen && (
        <div className="fluent-dialog-overlay" onClick={e => { if (e.target === e.currentTarget) setIsCreateOpen(false); }}>
          <div className="fluent-dialog">
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 22 }}>
              <div>
                <p className="fluent-dialog-title" style={{ margin: 0 }}>Nouvel utilisateur</p>
                <p style={{ fontSize: 12, color: 'var(--color-win-text3)', margin: '2px 0 0' }}>Le compte sera créé en attente d'approbation</p>
              </div>
              <button className="fluent-btn" style={{ padding: '4px 8px' }} onClick={() => setIsCreateOpen(false)}>
                <X size={14} />
              </button>
            </div>
            <form onSubmit={handleCreate} style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
              <ModernTextField label="Nom d'utilisateur" prefixIcon={<Users size={14} />} value={formData.username}
                onChange={(e: any) => setFormData({ ...formData, username: e.target.value })} placeholder="alice_kajy" />
              <ModernTextField label="Adresse Email" prefixIcon={<Mail size={14} />} value={formData.email}
                onChange={(e: any) => setFormData({ ...formData, email: e.target.value })} placeholder="alice@kajy.mg" />
              <ModernTextField label="Mot de passe" prefixIcon={<Key size={14} />} type="password" value={formData.password}
                onChange={(e: any) => setFormData({ ...formData, password: e.target.value })} placeholder="••••••••" />
              <div>
                <label style={{ fontSize: 12, fontWeight: 600, color: 'var(--color-win-text2)', display: 'block', marginBottom: 6 }}>Quota (GB)</label>
                <input type="number" className="fluent-input" value={formData.quota_gb}
                  onChange={e => setFormData({ ...formData, quota_gb: Number(e.target.value) })} />
              </div>
              <div className="fluent-divider" />
              <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
                <button type="button" className="fluent-btn" onClick={() => setIsCreateOpen(false)}>Annuler</button>
                <button type="submit" className="fluent-btn fluent-btn-accent">
                  <Check size={14} /> Créer
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
