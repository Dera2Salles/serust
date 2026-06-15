import { useState, useEffect } from 'react';
import { AdminDashboard } from './components/AdminDashboard';
import { UserManagement } from './components/UserManagement';
import { ServerControl } from './components/ServerControl';
import { SystemLogs } from './components/SystemLogs';
import { ActiveSessions } from './components/ActiveSessions';
import { ShareManagement } from './components/ShareManagement';
import { GlobalSettings } from './components/GlobalSettings';
import { Login } from './components/Login';
import {
  LayoutDashboard, Users, Settings, Activity,
  Share2, ScrollText, Cpu, ChevronLeft, ChevronRight,
  LogOut
} from 'lucide-react';
import { cn } from './components/OneUI';

function App() {
  const [activeTab, setActiveTab] = useState('dashboard');
  const [collapsed, setCollapsed] = useState(false);
  const [user, setUser] = useState<any>(null);
  const [authLoading, setAuthLoading] = useState(true);
  const [showLogoutConfirm, setShowLogoutConfirm] = useState(false);

  useEffect(() => {
    const savedUser = localStorage.getItem('admin_user') || sessionStorage.getItem('admin_user');
    if (savedUser) {
      setUser(JSON.parse(savedUser));
    }
    setAuthLoading(false);
  }, []);

  const handleLoginSuccess = (userData: any) => {
    setUser(userData);
    // Note: Login.tsx already handles saving to localStorage/sessionStorage
  };

  const handleLogout = () => {
    setUser(null);
    localStorage.removeItem('admin_user');
    sessionStorage.removeItem('admin_user');
    setShowLogoutConfirm(false);
  };

  if (authLoading) return null;

  if (!user) {
    return <Login onLoginSuccess={handleLoginSuccess} />;
  }

  const tabs = [
    { id: 'dashboard', label: 'Dashboard',    icon: LayoutDashboard, group: 'main' },
    { id: 'system',    label: 'Système',       icon: Cpu,             group: 'main' },
    { id: 'users',     label: 'Utilisateurs',  icon: Users,           group: 'main' },
    { id: 'sessions',  label: 'Connexions',    icon: Activity,        group: 'main' },
    { id: 'shares',    label: 'Partages',      icon: Share2,          group: 'data' },
    { id: 'logs',      label: 'Journaux',      icon: ScrollText,      group: 'data' },
    { id: 'settings',  label: 'Paramètres',    icon: Settings,        group: 'settings' },
  ];

  const navW = collapsed ? 60 : 232;

  const tabLabels: Record<string, string> = {
    dashboard: "Console d'administration",
    system: "Système",
    users: "Utilisateurs",
    sessions: "Connexions actives",
    shares: "Partages",
    logs: "Journaux système",
    settings: "Paramètres globaux",
  };

  return (
    <div className="flex w-full h-screen overflow-hidden" style={{ background: 'var(--color-win-bg)' }}>

      {/* ── Sidebar ── */}
      <aside
        className="flex flex-col flex-shrink-0"
        style={{
          width: navW,
          background: 'var(--color-win-surface)',
          borderRight: '1px solid var(--color-win-stroke)',
          paddingTop: 0,
          paddingBottom: 12,
          overflow: 'hidden',
          transition: 'width 0.2s ease',
        }}
      >
        {/* Logo strip */}
        <div
          className="flex items-center gap-3"
          style={{
            height: 60,
            padding: collapsed ? '0 12px' : '0 16px',
            borderBottom: '1px solid var(--color-win-stroke)',
            flexShrink: 0,
            justifyContent: collapsed ? 'center' : 'flex-start',
          }}
        >
          {/* Logo mark — image logo.png */}
          <img 
            src="/logo.png" 
            alt="Logo" 
            style={{
              width: 32, height: 32,
              borderRadius: 8,
              objectFit: 'contain',
              flexShrink: 0,
            }} 
          />
          {!collapsed && (
            <div>
              <p style={{ fontSize: 14, fontWeight: 700, color: 'var(--color-win-text)', margin: 0, lineHeight: 1.2 }}>Kajy Admin</p>
              <p style={{ fontSize: 11, color: 'var(--color-win-text3)', margin: 0 }}>Panel de contrôle</p>
            </div>
          )}
        </div>

        {/* Nav items */}
        <div className="flex-1 overflow-y-auto no-scrollbar px-2 pt-3 space-y-0.5">
          {tabs.map((tab, i) => {
            const Icon = tab.icon;
            const active = activeTab === tab.id;
            const prevGroup = i > 0 ? tabs[i - 1].group : tab.group;
            return (
              <div key={tab.id}>
                {tab.group !== prevGroup && i !== 0 && (
                  <div style={{ margin: '6px 4px', height: 1, background: 'var(--color-win-stroke)' }} />
                )}
                <button
                  onClick={() => setActiveTab(tab.id)}
                  className={cn('fluent-nav-item', active && 'active')}
                  title={collapsed ? tab.label : undefined}
                  style={{
                    justifyContent: collapsed ? 'center' : 'flex-start',
                    paddingLeft: collapsed ? 0 : undefined,
                  }}
                >
                  <Icon size={17} style={{ flexShrink: 0 }} />
                  {!collapsed && <span>{tab.label}</span>}
                </button>
              </div>
            );
          })}
        </div>

        {/* Collapse toggle + user footer */}
        <div style={{ padding: '8px 8px 0' }}>
          <button
            onClick={() => setCollapsed(!collapsed)}
            className="fluent-btn"
            style={{
              width: '100%', padding: '6px 8px', fontSize: 12,
              justifyContent: collapsed ? 'center' : 'flex-start',
              color: 'var(--color-win-text3)',
              boxShadow: 'none', border: 'none', background: 'transparent',
            }}
          >
            {collapsed
              ? <ChevronRight size={14} />
              : <><ChevronLeft size={14} /><span>Réduire</span></>
            }
          </button>

          {!collapsed && (
            <div style={{
              marginTop: 6,
              padding: '10px 12px',
              borderRadius: 10,
              background: 'var(--color-win-bg)',
              border: '1px solid var(--color-win-stroke)',
              display: 'flex', alignItems: 'center', gap: 10,
            }}>
              <div style={{
                width: 30, height: 30, borderRadius: '50%', flexShrink: 0,
                background: 'var(--color-accent)',
                display: 'flex', alignItems: 'center', justifyContent: 'center',
                color: '#fff', fontWeight: 700, fontSize: 13,
              }}>
                {(user?.username || "A").charAt(0).toUpperCase()}
              </div>
              <div style={{ minWidth: 0, flex: 1 }}>
                <p style={{ fontSize: 13, fontWeight: 600, margin: 0, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                  {user?.username || "Admin User"}
                </p>
                <p style={{ fontSize: 11, color: 'var(--color-win-text3)', margin: 0 }}>Administrateur</p>
              </div>
              <button 
                onClick={handleLogout}
                className="hover:text-[--color-error] transition-colors"
                title="Déconnexion"
              >
                <LogOut size={16} />
              </button>
            </div>
          )}
        </div>
      </aside>

      {/* ── Main area ── */}
      <div className="flex-1 flex flex-col overflow-hidden">

        {/* Top bar */}
        <header style={{
          height: 60,
          background: 'var(--color-win-surface)',
          borderBottom: '1px solid var(--color-win-stroke)',
          display: 'flex',
          alignItems: 'center',
          padding: '0 28px',
          gap: 16,
          flexShrink: 0,
        }}>
          {/* Breadcrumb / page title */}
          <div style={{ flex: 1 }}>
            <p style={{ fontSize: 14, fontWeight: 600, color: 'var(--color-win-text)', margin: 0 }}>
              {tabLabels[activeTab]}
            </p>
          </div>

          {/* Avatar */}
          <div style={{
            width: 34, height: 34, borderRadius: '50%',
            background: 'var(--color-accent)',
            display: 'flex', alignItems: 'center', justifyContent: 'center',
            color: '#fff', fontWeight: 700, fontSize: 14, cursor: 'pointer',
            border: '2px solid var(--color-accent-bg)',
          }} onClick={() => setShowLogoutConfirm(true)} title="Cliquer pour se déconnecter">
            {(user?.username || "A").charAt(0).toUpperCase()}
          </div>
        </header>

        {/* Content */}
        <main className="flex-1 overflow-y-auto overflow-x-hidden" style={{ background: 'var(--color-win-bg)' }}>
          {activeTab === 'dashboard'  && <AdminDashboard />}
          {activeTab === 'system'     && <ServerControl />}
          {activeTab === 'users'      && <UserManagement />}
          {activeTab === 'sessions'   && <ActiveSessions />}
          {activeTab === 'shares'     && <ShareManagement />}
          {activeTab === 'logs'       && <SystemLogs />}
          {activeTab === 'settings'   && <GlobalSettings />}
        </main>
      </div>

      {/* ── Logout Confirmation Modal ── */}
      {showLogoutConfirm && (
        <div className="fluent-dialog-overlay" onClick={() => setShowLogoutConfirm(false)}>
          <div className="fluent-dialog" onClick={e => e.stopPropagation()} style={{ maxWidth: 400 }}>
            <div style={{ marginBottom: 20 }}>
              <p className="fluent-dialog-title" style={{ margin: 0, fontSize: 20 }}>Se déconnecter ?</p>
              <p style={{ fontSize: 14, color: 'var(--color-win-text3)', marginTop: 8 }}>
                Voulez-vous vraiment mettre fin à votre session d'administration ?
              </p>
            </div>
            
            <div className="flex justify-end gap-3 pt-2">
              <button 
                className="fluent-btn" 
                style={{ minWidth: 100 }}
                onClick={() => setShowLogoutConfirm(false)}
              >
                Annuler
              </button>
              <button 
                className="fluent-btn fluent-btn-danger" 
                style={{ minWidth: 100 }}
                onClick={handleLogout}
              >
                Déconnexion
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
