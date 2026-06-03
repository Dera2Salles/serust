import React, { useState, useEffect } from 'react';
import { Header, ModernTextField, cn } from './OneUI';
import { Globe, Shield, Save, RefreshCcw, AlertTriangle, Lock } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface GlobalSettingsData { 
  default_storage_quota_gb: number;
  allow_public_registration: boolean;
  allow_public_links: boolean;
  server_maintenance_mode: boolean;
  max_upload_size_mb: number;
}

const SettingRow = ({ label, desc, checked, onChange, danger = false }: { label: string; desc: string; checked: boolean; onChange: () => void; danger?: boolean }) => (
  <div
    style={{
      display: 'flex', alignItems: 'center', justifyContent: 'space-between',
      padding: '14px 16px', borderRadius: 8,
      background: danger && checked ? 'var(--color-error-bg)' : 'var(--color-win-nav)',
      border: `1px solid ${danger && checked ? 'var(--color-error)' : 'var(--color-win-stroke)'}`,
      marginBottom: 8,
    }}
  >
    <div>
      <p style={{ fontWeight: 600, fontSize: 14, margin: 0, color: danger && checked ? 'var(--color-error)' : 'var(--color-win-text)' }}>{label}</p>
      <p style={{ fontSize: 12, color: 'var(--color-win-text3)', margin: 0 }}>{desc}</p>
    </div>
    <div className={cn('fluent-toggle', checked && 'on')} onClick={onChange} />
  </div>
);

export const GlobalSettings: React.FC = () => {
  const [settings, setSettings] = useState<GlobalSettingsData | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<{ text: string; type: 'success' | 'error' } | null>(null);

  const fetchSettings = async () => {
    setLoading(true);
    try { setSettings(await invoke<GlobalSettingsData>('get_global_settings')); }
    catch (e) { console.error(e); } finally { setLoading(false); }
  };
  useEffect(() => { fetchSettings(); }, []);

  const handleSave = async () => {
    if (!settings) return;
    setSaving(true); setMessage(null);
    try { await invoke('save_global_settings', { settings }); setMessage({ text: 'Paramètres sauvegardés', type: 'success' }); }
    catch (e) { setMessage({ text: `Erreur: ${e}`, type: 'error' }); }
    finally { setSaving(false); }
  };

  return (
    <div style={{ paddingBottom: 40 }}>
      <Header title="Paramètres" subtitle="Configuration système et politiques de sécurité Kajy" />

      <div style={{ padding: '0 32px', display: 'grid', gridTemplateColumns: '1fr 300px', gap: 20, maxWidth: 900 }}>
        {/* Main settings */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>

          {/* Toggles */}
          <div className="fluent-card">
            <div className="flex items-center gap-3 mb-4">
              <div style={{ width: 32, height: 32, borderRadius: 8, background: 'var(--color-success-bg)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                <Shield size={16} style={{ color: 'var(--color-success)' }} />
              </div>
              <div>
                <p style={{ fontWeight: 600, fontSize: 15, margin: 0 }}>Politiques d'accès</p>
                <p style={{ fontSize: 12, color: 'var(--color-win-text3)', margin: 0 }}>Inscriptions et disponibilité</p>
              </div>
            </div>
            <SettingRow
              label="Inscriptions ouvertes"
              desc="Autoriser la création de nouveaux comptes par le public"
              checked={settings?.allow_public_registration ?? false}
              onChange={() => settings && setSettings({ ...settings, allow_public_registration: !settings.allow_public_registration })}
            />
            <SettingRow
              label="Partages publics"
              desc="Autoriser la création de liens de partage anonymes"
              checked={settings?.allow_public_links ?? false}
              onChange={() => settings && setSettings({ ...settings, allow_public_links: !settings.allow_public_links })}
            />
            <SettingRow
              label="Mode Maintenance"
              desc="Bloquer tous les accès client"
              checked={settings?.server_maintenance_mode ?? false}
              onChange={() => settings && setSettings({ ...settings, server_maintenance_mode: !settings.server_maintenance_mode })}
              danger
            />
          </div>

          {/* Quotas */}
          <div className="fluent-card">
            <div className="flex items-center gap-3 mb-4">
              <div style={{ width: 32, height: 32, borderRadius: 8, background: 'var(--color-warning-bg)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                <Lock size={16} style={{ color: 'var(--color-warning)' }} />
              </div>
              <p style={{ fontWeight: 600, fontSize: 15, margin: 0 }}>Limites de ressources</p>
            </div>
            <div className="space-y-4">
              <div>
                <label style={{ fontSize: 12, fontWeight: 600, color: 'var(--color-win-text2)', display: 'block', marginBottom: 6 }}>
                  Quota par défaut (GB)
                </label>
                <input
                  type="number"
                  className="fluent-input"
                  style={{ maxWidth: 200 }}
                  value={settings?.default_storage_quota_gb || 0}
                  onChange={e => settings && setSettings({ ...settings, default_storage_quota_gb: Number(e.target.value) })}
                />
              </div>
              <div>
                <label style={{ fontSize: 12, fontWeight: 600, color: 'var(--color-win-text2)', display: 'block', marginBottom: 6 }}>
                  Taille max upload (MB)
                </label>
                <input
                  type="number"
                  className="fluent-input"
                  style={{ maxWidth: 200 }}
                  value={settings?.max_upload_size_mb || 0}
                  onChange={e => settings && setSettings({ ...settings, max_upload_size_mb: Number(e.target.value) })}
                />
              </div>
            </div>
          </div>
        </div>

        {/* Action panel */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
          <div className="fluent-card" style={{ background: 'var(--color-accent)', border: '1px solid var(--color-accent-light)' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 10 }}>
              <Save size={16} style={{ color: 'white' }} />
              <p style={{ fontWeight: 600, fontSize: 14, color: 'white', margin: 0 }}>Sauvegarder</p>
            </div>
            <p style={{ fontSize: 12, color: 'rgba(255,255,255,0.8)', marginBottom: 16, lineHeight: 1.5 }}>
              Les modifications sont appliquées immédiatement sur le serveur.
            </p>
            <button
              onClick={handleSave}
              disabled={saving || !settings}
              style={{
                width: '100%', padding: '8px 0', borderRadius: 6, border: '1px solid rgba(255,255,255,0.3)',
                background: 'rgba(255,255,255,0.15)', color: 'white', fontWeight: 600, fontSize: 13,
                cursor: saving ? 'wait' : 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 6,
              }}
            >
              {saving ? <RefreshCcw size={14} style={{ animation: 'spin 0.8s linear infinite' }} /> : <Save size={14} />}
              {saving ? 'Enregistrement...' : 'Sauvegarder les modifications'}
            </button>

            {message && (
              <div style={{
                marginTop: 10, padding: '8px 12px', borderRadius: 6, fontSize: 12, fontWeight: 600,
                background: message.type === 'success' ? 'rgba(255,255,255,0.2)' : 'rgba(255,0,0,0.3)',
                color: 'white',
              }}>
                {message.text}
              </div>
            )}
          </div>

          <div className="fluent-card">
            <p style={{ fontWeight: 600, fontSize: 13, marginBottom: 8 }}>À propos</p>
            <p style={{ fontSize: 12, color: 'var(--color-win-text3)', lineHeight: 1.6, margin: 0 }}>
              Le mode maintenance coupe proprement les services avant une mise à jour système. Les connexions actives sont préservées.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};
