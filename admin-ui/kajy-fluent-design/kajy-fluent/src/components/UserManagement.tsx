import React, { useState, useEffect } from 'react';
import { Header, AroChip, ModernTextField, cn } from './OneUI';
import { Users, UserPlus, Search, Edit3, Trash2, Shield, HardDrive, Mail, Key, X, Check } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface User { username: string; email: string; quota_gb: number; is_admin: boolean; created_at: string; }

export const UserManagement = () => {
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  const [query, setQuery] = useState('');
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [formData, setFormData] = useState({ username: '', email: '', password: '', quota_gb: 10, is_admin: false });

  const fetchUsers = async () => {
    setLoading(true);
    try { setUsers(await invoke<User[]>('get_users_from_db') || []); } catch {} finally { setLoading(false); }
  };
  useEffect(() => { fetchUsers(); }, []);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await invoke('create_user_direct', { username: formData.username, email: formData.email, password: formData.password, quotaGb: Number(formData.quota_gb), isAdmin: formData.is_admin });
      setIsCreateOpen(false);
      fetchUsers();
    } catch (e) { alert(e); }
  };

  const filtered = users.filter(u =>
    u.username.toLowerCase().includes(query.toLowerCase()) ||
    u.email.toLowerCase().includes(query.toLowerCase())
  );

  return (
    <div style={{ paddingBottom: 40 }}>
      <Header title="Utilisateurs" subtitle="Gérer les comptes et quotas de stockage" />

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
          display: 'grid', gridTemplateColumns: '2fr 2fr 80px 80px 88px',
          padding: '6px 16px', marginBottom: 4,
          fontSize: 12, fontWeight: 600, color: 'var(--color-win-text3)',
        }}>
          <span>Utilisateur</span><span>Email</span><span>Quota</span><span>Rôle</span><span></span>
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
            <div key={i}>
              {i > 0 && <div className="fluent-divider" style={{ margin: '0 16px' }} />}
              <div style={{
                display: 'grid', gridTemplateColumns: '2fr 2fr 80px 80px 88px',
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
                    background: 'var(--color-accent-subtle)',
                    display: 'flex', alignItems: 'center', justifyContent: 'center',
                    color: 'var(--color-accent)', fontWeight: 700, fontSize: 13,
                    position: 'relative',
                  }}>
                    {user.username.charAt(0).toUpperCase()}
                    {user.is_admin && (
                      <div style={{
                        position: 'absolute', bottom: -2, right: -2,
                        width: 14, height: 14, borderRadius: '50%',
                        background: 'var(--color-accent)', border: '2px solid white',
                        display: 'flex', alignItems: 'center', justifyContent: 'center',
                      }}>
                        <Shield size={7} style={{ color: 'white' }} />
                      </div>
                    )}
                  </div>
                  <span style={{ fontSize: 14, fontWeight: 600, color: 'var(--color-win-text)' }}>{user.username}</span>
                </div>
                {/* Email */}
                <span style={{ fontSize: 13, color: 'var(--color-win-text2)' }}>{user.email}</span>
                {/* Quota */}
                <span style={{ fontSize: 13, color: 'var(--color-win-text2)' }}>{user.quota_gb} GB</span>
                {/* Role */}
                <span>
                  <AroChip label={user.is_admin ? 'Admin' : 'User'} color={user.is_admin ? 'blue' : 'overlay2'} />
                </span>
                {/* Actions */}
                <div className="flex items-center gap-1">
                  <button className="fluent-btn" style={{ padding: '4px 8px' }}>
                    <Edit3 size={14} />
                  </button>
                  <button className="fluent-btn fluent-btn-danger" style={{ padding: '4px 8px' }}>
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

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
                <div>
                  <label style={{ fontSize: 12, fontWeight: 600, color: 'var(--color-win-text2)', display: 'block', marginBottom: 6 }}>Quota (GB)</label>
                  <input type="number" className="fluent-input"
                    value={formData.quota_gb}
                    onChange={e => setFormData({...formData, quota_gb: Number(e.target.value)})} />
                </div>
                <div style={{ display: 'flex', flexDirection: 'column', justifyContent: 'flex-end' }}>
                  <label style={{ fontSize: 12, fontWeight: 600, color: 'var(--color-win-text2)', display: 'block', marginBottom: 6 }}>Rôle Administrateur</label>
                  <button type="button"
                    onClick={() => setFormData({...formData, is_admin: !formData.is_admin})}
                    className={cn('fluent-btn flex items-center gap-2', formData.is_admin && 'fluent-btn-accent')}
                    style={{ width: '100%', justifyContent: 'center' }}
                  >
                    <Shield size={14} /> {formData.is_admin ? 'Admin activé' : 'Utilisateur'}
                  </button>
                </div>
              </div>

              <div className="fluent-divider" />
              <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
                <button type="button" className="fluent-btn" onClick={() => setIsCreateOpen(false)}>Annuler</button>
                <button type="submit" className="fluent-btn fluent-btn-accent flex items-center gap-2">
                  <Check size={14} /> Créer l'utilisateur
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
