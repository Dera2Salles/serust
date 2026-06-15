import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Lock, Mail, RefreshCcw, ShieldCheck } from 'lucide-react';
import { ModernTextField, Button } from './OneUI';

interface LoginProps {
  onLoginSuccess: (user: any) => void;
}

export const Login: React.FC<LoginProps> = ({ onLoginSuccess }) => {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [rememberMe, setRememberMe] = useState(true);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const user = await invoke<any>('login_admin_db', { email, passwordRaw: password });
      
      // If rememberMe is false, we could pass a flag or handle it differently, 
      // but in this app we'll just respect the user's choice for localStorage.
      if (rememberMe) {
        localStorage.setItem('admin_user', JSON.stringify(user));
      } else {
        sessionStorage.setItem('admin_user', JSON.stringify(user));
      }
      
      onLoginSuccess(user);
    } catch (err: any) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex items-center justify-center min-h-screen" style={{ background: 'var(--color-win-bg)', backgroundImage: 'radial-gradient(circle at 50% 50%, var(--color-accent-subtle) 0%, transparent 70%)', opacity: 0.95 }}>
      <div className="fluent-card w-full max-w-md p-10 animate-in fade-in zoom-in duration-500 shadow-2xl" style={{ borderRadius: 20, border: '1px solid var(--color-win-stroke2)' }}>
        
        <div className="flex flex-col items-center mb-10 text-center">
          <div style={{ 
            width: 72, height: 72, borderRadius: 18, background: 'var(--color-win-surface)', 
            display: 'flex', alignItems: 'center', justifyContent: 'center', marginBottom: 20,
            boxShadow: '0 10px 25px -5px rgba(37, 99, 235, 0.2)', border: '1px solid var(--color-win-stroke)'
          }}>
            <img 
              src="/logo.png" 
              alt="Logo" 
              style={{ width: 48, height: 48, objectFit: 'contain' }} 
            />
          </div>
          <h1 style={{ fontSize: 28, fontWeight: 800, color: 'var(--color-win-text)', margin: 0, letterSpacing: '-0.6px' }}>
            Kajy Admin
          </h1>
          <p style={{ color: 'var(--color-win-text3)', marginTop: 8, fontSize: 14, fontWeight: 500 }}>
            Panel d'administration sécurisé
          </p>
        </div>

        <form onSubmit={handleLogin} className="space-y-6">
          <ModernTextField
            label="Adresse Email"
            prefixIcon={<Mail size={18} />}
            type="email"
            value={email}
            onChange={(e: any) => setEmail(e.target.value)}
            placeholder="admin@kajy.mg"
            required
            autoFocus
          />

          <ModernTextField
            label="Mot de passe"
            prefixIcon={<Lock size={18} />}
            type="password"
            value={password}
            onChange={(e: any) => setPassword(e.target.value)}
            placeholder="••••••••"
            required
          />

          <div className="flex items-center justify-between">
            <label className="flex items-center gap-2 cursor-pointer group">
              <input 
                type="checkbox" 
                checked={rememberMe} 
                onChange={(e) => setRememberMe(e.target.checked)}
                className="w-4 h-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
              />
              <span style={{ fontSize: 13, color: 'var(--color-win-text2)', fontWeight: 500 }} className="group-hover:text-[--color-accent] transition-colors">
                Se souvenir de moi
              </span>
            </label>
            <a href="#" style={{ fontSize: 13, color: 'var(--color-accent)', fontWeight: 600, textDecoration: 'none' }} className="hover:underline">
              Besoin d'aide ?
            </a>
          </div>

          {error && (
            <div style={{
              padding: '12px 16px',
              borderRadius: 10,
              background: 'rgba(220, 38, 38, 0.1)',
              border: '1px solid rgba(220, 38, 38, 0.2)',
              color: '#dc2626',
              fontSize: 13,
              fontWeight: 600,
              display: 'flex',
              alignItems: 'center',
              gap: 10,
              animation: 'shake 0.4s ease-in-out'
            }}>
              <div style={{ width: 6, height: 6, borderRadius: '50%', background: '#dc2626' }} />
              {error}
            </div>
          )}

          <Button
            type="submit"
            disabled={loading}
            className="w-full py-3.5 text-[15px] font-bold"
            style={{ height: 52, borderRadius: 12, boxShadow: '0 4px 12px rgba(37, 99, 235, 0.2)' }}
            icon={loading ? <RefreshCcw size={18} className="animate-spin" /> : <ShieldCheck size={18} />}
          >
            {loading ? "Authentification..." : "Connexion sécurisée"}
          </Button>
        </form>

        <div className="mt-12 pt-8 border-t border-[--color-win-stroke] text-center">
          <p style={{ fontSize: 12, color: 'var(--color-win-text4)', letterSpacing: '0.02em' }}>
            SYSTÈME DE GESTION DE FICHIERS · VERSION 2.1.4
            <br />
            <span style={{ opacity: 0.6 }}>© 2026 Kajy Cloud Computing</span>
          </p>
        </div>
      </div>
    </div>
  );
};

