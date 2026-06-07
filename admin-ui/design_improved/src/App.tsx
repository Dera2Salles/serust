import { useState } from 'react';
import { AdminDashboard } from './components/AdminDashboard';
import { UserManagement } from './components/UserManagement';
import { ServerControl } from './components/ServerControl';
import { SystemLogs } from './components/SystemLogs';
import { ActiveSessions } from './components/ActiveSessions';
import { ShareManagement } from './components/ShareManagement';
import { GlobalSettings } from './components/GlobalSettings';
import {
  LayoutDashboard, Users, Settings, Activity,
  Share2, ScrollText, Cpu, ChevronLeft, ChevronRight,
  Bell, Search
} from 'lucide-react';
import { cn } from './components/OneUI';

function App() {
  const [activeTab, setActiveTab] = useState('dashboard');
  const [collapsed, setCollapsed] = useState(false);

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
          {/* Logo mark — bleu carré arrondi */}
          <div style={{
            width: 32, height: 32,
            borderRadius: 8,
            background: 'var(--color-accent)',
            display: 'flex', alignItems: 'center', justifyContent: 'center',
            flexShrink: 0,
            color: '#fff', fontWeight: 800, fontSize: 15, letterSpacing: '-0.5px',
          }}>K</div>
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
              }}>A</div>
              <div style={{ minWidth: 0 }}>
                <p style={{ fontSize: 13, fontWeight: 600, margin: 0, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>Admin User</p>
                <p style={{ fontSize: 11, color: 'var(--color-win-text3)', margin: 0 }}>Administrateur</p>
              </div>
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

          {/* Search */}
          <div style={{
            display: 'flex', alignItems: 'center', gap: 8,
            background: 'var(--color-win-bg)',
            border: '1px solid var(--color-win-stroke)',
            borderRadius: 8,
            padding: '7px 12px',
            width: 220,
          }}>
            <Search size={14} style={{ color: 'var(--color-win-text3)', flexShrink: 0 }} />
            <input
              style={{
                border: 'none', background: 'transparent', outline: 'none',
                fontSize: 13, color: 'var(--color-win-text)', flex: 1, fontFamily: 'inherit',
              }}
              placeholder="Rechercher..."
            />
          </div>

          {/* Notification */}
          <button
            className="fluent-btn"
            style={{ padding: '7px 10px', boxShadow: 'none', border: '1px solid var(--color-win-stroke)', position: 'relative' }}
          >
            <Bell size={16} style={{ color: 'var(--color-win-text2)' }} />
            <span style={{
              position: 'absolute', top: 6, right: 6,
              width: 7, height: 7, borderRadius: '50%',
              background: 'var(--color-accent)',
              border: '1.5px solid white',
            }} />
          </button>

          {/* Avatar */}
          <div style={{
            width: 34, height: 34, borderRadius: '50%',
            background: 'var(--color-accent)',
            display: 'flex', alignItems: 'center', justifyContent: 'center',
            color: '#fff', fontWeight: 700, fontSize: 14, cursor: 'pointer',
            border: '2px solid var(--color-accent-bg)',
          }}>A</div>
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
    </div>
  );
}

export default App;
