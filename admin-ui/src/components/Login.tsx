import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Lock, Mail, RefreshCcw } from 'lucide-react';
import { ModernTextField, Button } from './OneUI';

interface LoginProps {
  onLoginSuccess: (user: any) => void;
}

export const Login: React.FC<LoginProps> = ({ onLoginSuccess }) => {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const user = await invoke<any>('login_admin_db', { email, passwordRaw: password });
      onLoginSuccess(user);
    } catch (err: any) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex items-center justify-center min-h-screen" style={{ background: 'var(--color-win-bg)' }}>
      <div className="fluent-card w-full max-w-md p-10 animate-in fade-in zoom-in duration-300">
        <div className="flex flex-col items-center mb-10 text-center">
          <img 
            src="/logo.png" 
            alt="Logo" 
            style={{
              width: 64, height: 64,
              borderRadius: 14,
              objectFit: 'contain',
              marginBottom: 20,
              boxShadow: '0 8px 16px rgba(37, 99, 235, 0.1)'
            }} 
          />
          <h1 style={{ fontSize: 26, fontWeight: 800, color: 'var(--color-win-text)', margin: 0, letterSpacing: '-0.5px' }}>
            Kajy Admin
          </h1>
          <p style={{ color: 'var(--color-win-text3)', marginTop: 6, fontSize: 14 }}>
            Connectez-vous pour gérer votre serveur
          </p>
        </div>

        <form onSubmit={handleLogin} className="space-y-6">
          <ModernTextField
            label="Adresse Email"
            prefixIcon={<Mail size={16} />}
            type="email"
            value={email}
            onChange={(e: any) => setEmail(e.target.value)}
            placeholder="admin@local.mg"
            required
          />

          <ModernTextField
            label="Mot de passe"
            prefixIcon={<Lock size={16} />}
            type="password"
            value={password}
            onChange={(e: any) => setPassword(e.target.value)}
            placeholder="••••••••"
            required
          />

          {error && (
            <div style={{
              padding: '10px 14px',
              borderRadius: 8,
              background: 'var(--color-error-bg)',
              border: '1px solid var(--color-error)',
              color: 'var(--color-error)',
              fontSize: 13,
              fontWeight: 500,
            }}>
              {error}
            </div>
          )}

          <Button
            type="submit"
            disabled={loading}
            className="w-full py-3 text-[15px]"
            style={{ height: 48 }}
          >
            {loading ? <RefreshCcw size={18} className="animate-spin" /> : "Se connecter"}
          </Button>
        </form>

        <div className="mt-10 pt-8 border-t border-[--color-win-stroke] text-center">
          <p style={{ fontSize: 12, color: 'var(--color-win-text4)' }}>
            Système de gestion de fichiers sécurisé · v2.1
          </p>
        </div>
      </div>
    </div>
  );
};
