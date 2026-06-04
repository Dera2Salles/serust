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
      if (settings && settings.default_storage_quota_gb) {
        setFormData(prev => ({ ...prev, quota_gb: settings.default_storage_quota_gb }));
      }
    } catch (e) {
      console.error("Failed to fetch default quota:", e);
    }
  };

  useEffect(() => { 
    fetchUsers(); 
    fetchDefaultQuota();
  }, []);
const handleCreate = async (e: React.FormEvent) => {
  e.preventDefault();
  try {
    await invoke('create_user_db', { 
      username: formData.username, 
      email: formData.email, 
      passwordRaw: formData.password, 
      quota: Number(formData.quota_gb) * 1024 * 1024 * 1024 
    });
    setIsCreateOpen(false);
    setFormData({ username: '', email: '', password: '', quota_gb: 10 });
    fetchUsers();
  } catch (e) { 
    alert(e); 
  }
};

const handleApprove = async (user: User) => {
  try {
    await invoke('update_user_db', { 
      id: user.id, 
      email: user.email, 
      quota: user.storage_quota_bytes, 
      isActive: true 
    });
    fetchUsers();
  } catch (e) { 
    alert(e); 
  }
};

  const handleDelete = async (id: string) => {
    if (!confirm('Supprimer cet utilisateur ?')) return;
    try {
      await invoke('delete_user_db', { id });
      fetchUsers();
    } catch (e) { 
      alert(e); 
    }
  };

  const filtered = users.filter(u =>
    u.username.toLowerCase().includes(query.toLowerCase()) ||
    u.email.toLowerCase().includes(query.toLowerCase())
  );

  return (
    <div style={{ paddingBottom: 40 }}>
      <Header title="Utilisateurs" subtitle="Gérer les comptes et quotas de stockage" />

      {/* User Stats */}
      <div style={{ padding: '0 32px 24px', display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 12 }}>
        <div className="fluent-stat">
          <div className="flex items-center justify-between">
            <span style={{ fontSize: 12, color: 'var(--color-win-text3)', fontWeight: 400 }}>Total Utilisateurs</span>
            <Users size={16} style={{ color: 'var(--color-accent)', opacity: 0.8 }} />
          </div>
          <p style={{ fontSize: 26, fontWeight: 600, color: 'var(--color-win-text)', margin: 0 }}>{users.length}</p>
        </div>
        <div className="fluent-stat">
          <div className="flex items-center justify-between">
            <span style={{ fontSize: 12, color: 'var(--color-win-text3)', fontWeight: 400 }}>Actifs</span>
            <Check size={16} style={{ color: 'var(--color-success)', opacity: 0.8 }} />
          </div>
          <p style={{ fontSize: 26, fontWeight: 600, color: 'var(--color-win-text)', margin: 0 }}>{users.filter(u => u.is_active).length}</p>
        </div>
        <div className="fluent-stat">
          <div className="flex items-center justify-between">
            <span style={{ fontSize: 12, color: 'var(--color-win-text3)', fontWeight: 400 }}>En attente</span>
            <RefreshCw size={16} style={{ color: 'var(--color-warning)', opacity: 0.8 }} />
          </div>
          <p style={{ fontSize: 26, fontWeight: 600, color: 'var(--color-win-text)', margin: 0 }}>{users.filter(u => !u.is_active).length}</p>
        </div>
      </div>

      {/* Toolbar */}
      <div style={{ padding: '0 32px 20px', display: 'flex', gap: 12, alignItems: 'center', flexWrap: 'wrap' }}>
        <div className="fluent-commandbar flex-1" style={{ minWidth: 220, maxWidth: 360 }}>
          <Search size={16} style={{ color: 'var(--color-win-text3)', flexShrink: 0 }} />
          <input
            className="fluent-input"
            style={{ border: 'none', boxShadow: 'none', padding: '4px 8px', flex: 1, background: 'transparent' }}
            placeholder="Rechercher un utilisateur..."
            value={query}
            onChange={e => setQuery(e.target.value)}
          />
        </div>
        <button className="fluent-btn fluent-btn-accent flex items-center gap-2" onClick={() => setIsCreateOpen(true)}>
          <UserPlus size={14} /> Ajouter un utilisateur
        </button>
      </div>

      {/* Table header */}
      <div style={{ padding: '0 32px' }}>
        <div style={{
          display: 'grid', gridTemplateColumns: '1.5fr 1.8fr 180px 100px 120px',
          padding: '6px 16px', marginBottom: 4,
          fontSize: 12, fontWeight: 600, color: 'var(--color-win-text3)',
        }}>
          <span>Utilisateur</span><span>Email</span><span>Stockage (Utilisé/Total)</span><span>Statut</span><span>Actions</span>
        </div>

        {/* Rows */}
        <div className="fluent-card" style={{ padding: 0, overflow: 'hidden' }}>
          {loading ? (
            <div className="flex items-center justify-center py-16" style={{ color: 'var(--color-win-text3)' }}>
              <div style={{ width: 24, height: 24, border: '2px solid var(--color-win-stroke2)', borderTopColor: 'var(--color-accent)', borderRadius: '50%', animation: 'spin 0.8s linear infinite' }} />
            </div>
          ) : filtered.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 gap-3" style={{ opacity: 0.35 }}>
              <Users size={40} />
              <span style={{ fontSize: 14, fontWeight: 600 }}>Aucun utilisateur trouvé</span>
            </div>
          ) : filtered.map((user, i) => (
            <div key={user.id}>
              {i > 0 && <div className="fluent-divider" style={{ margin: '0 16px' }} />}
              <div style={{
                display: 'grid', gridTemplateColumns: '1.5fr 1.8fr 180px 100px 120px',
                padding: '10px 16px', alignItems: 'center',
                transition: 'background 0.1s', cursor: 'default',
              }}
                onMouseEnter={e => (e.currentTarget.style.background = 'var(--color-win-nav)')}
                onMouseLeave={e => (e.currentTarget.style.background = '')}
              >
                {/* Name */}
                <div className="flex items-center gap-2.5">
                  <div style={{
                    width: 32, height: 32, borderRadius: '50%', flexShrink: 0,
                    background: user.is_active ? 'var(--color-accent-subtle)' : 'var(--color-warning-bg)',
                    display: 'flex', alignItems: 'center', justifyContent: 'center',
                    color: user.is_active ? 'var(--color-accent)' : 'var(--color-warning)', 
                    fontWeight: 700, fontSize: 13,
                    position: 'relative',
                  }}>
                    {user.username.charAt(0).toUpperCase()}
                  </div>
                  <span style={{ fontSize: 14, fontWeight: 600, color: 'var(--color-win-text)' }}>{user.username}</span>
                </div>
                {/* Email */}
                <span style={{ fontSize: 13, color: 'var(--color-win-text2)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{user.email}</span>
                {/* Storage */}
                <div className="flex flex-col gap-1 pr-4">
                  <div className="flex justify-between text-[11px] font-medium text-[--color-win-text2]">
                    <span>{formatSize(user.storage_used_bytes)}</span>
                    <span className="opacity-50">/ {formatSize(user.storage_quota_bytes)}</span>
                  </div>
                  <div className="fluent-progress">
                    <div 
                      className="fluent-progress-fill" 
                      style={{ 
                        width: `${Math.min(100, (user.storage_used_bytes / user.storage_quota_bytes) * 100)}%`,
                        background: (user.storage_used_bytes / user.storage_quota_bytes) > 0.9 ? 'var(--color-error)' : 'var(--color-accent)'
                      }} 
                    />
                  </div>
                </div>
                {/* Status */}
                <span>
                  <AroChip 
                    label={user.is_active ? 'Actif' : 'En attente'} 
                    color={user.is_active ? 'green' : 'yellow'} 
                  />
                </span>
                {/* Actions */}
                <div className="flex items-center gap-1">
                  {!user.is_active && (
                    <button 
                      className="fluent-btn fluent-btn-accent" 
                      style={{ padding: '4px 8px' }}
                      onClick={() => handleApprove(user)}
                      title="Approuver"
                    >
                      <Check size={14} />
                    </button>
                  )}
                  <button className="fluent-btn" style={{ padding: '4px 8px' }}>
                    <Edit3 size={14} />
                  </button>
                  <button className="fluent-btn fluent-btn-danger" style={{ padding: '4px 8px' }} onClick={() => handleDelete(user.id)}>
                    <Trash2 size={14} />
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Create dialog */}
      {isCreateOpen && (
        <div className="fluent-dialog-overlay" onClick={e => { if (e.target === e.currentTarget) setIsCreateOpen(false); }}>
          <div className="fluent-dialog">
            <div className="flex items-center justify-between mb-5">
              <p className="fluent-dialog-title" style={{ margin: 0 }}>Nouvel utilisateur</p>
              <button className="fluent-btn" style={{ padding: '4px 8px' }} onClick={() => setIsCreateOpen(false)}><X size={14} /></button>
            </div>
            <form onSubmit={handleCreate} style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
              <ModernTextField label="Nom d'utilisateur" prefixIcon={<Users size={14} />} value={formData.username}
                onChange={(e: any) => setFormData({...formData, username: e.target.value})} placeholder="alice_kajy" />
              <ModernTextField label="Adresse Email" prefixIcon={<Mail size={14} />} value={formData.email}
                onChange={(e: any) => setFormData({...formData, email: e.target.value})} placeholder="alice@kajy.mg" />
              <ModernTextField label="Mot de passe" prefixIcon={<Key size={14} />} type="password" value={formData.password}
                onChange={(e: any) => setFormData({...formData, password: e.target.value})} placeholder="••••••••" />

              <div>
                <label style={{ fontSize: 12, fontWeight: 600, color: 'var(--color-win-text2)', display: 'block', marginBottom: 6 }}>Quota (GB)</label>
                <input type="number" className="fluent-input"
                  value={formData.quota_gb}
                  onChange={e => setFormData({...formData, quota_gb: Number(e.target.value)})} />
              </div>

              <div className="fluent-divider" />
              <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
                <button type="button" className="fluent-btn" onClick={() => setIsCreateOpen(false)}>Annuler</button>
                <button type="submit" className="fluent-btn fluent-btn-accent flex items-center gap-2">
                  <Check size={14} /> Créer (Attente d'approbation)
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
