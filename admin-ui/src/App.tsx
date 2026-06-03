import { useState } from 'react';
import { AdminDashboard } from './components/AdminDashboard';
import { UserManagement } from './components/UserManagement';
import { FileExplorer } from './components/FileExplorer';
import { ServerControl } from './components/ServerControl';
import { SystemLogs } from './components/SystemLogs';
import { ActiveSessions } from './components/ActiveSessions';
import { ShareManagement } from './components/ShareManagement';
import { GlobalSettings } from './components/GlobalSettings';
import {
  LayoutDashboard, Users, FolderTree, Settings, Activity,
  Share2, ScrollText, Cpu, ChevronLeft, ChevronRight
} from 'lucide-react';
import { cn } from './components/OneUI';

function App() {
  const [activeTab, setActiveTab] = useState('dashboard');
  const [collapsed, setCollapsed] = useState(false);

  const tabs = [
    { id: 'dashboard', label: 'Dashboard',    icon: LayoutDashboard, group: 'main' },
    { id: 'system',    label: 'System',        icon: Cpu,             group: 'main' },
    { id: 'users',     label: 'Utilisateurs',  icon: Users,           group: 'main' },
    { id: 'sessions',  label: 'Connexions',    icon: Activity,        group: 'main' },
    { id: 'shares',    label: 'Partages',      icon: Share2,          group: 'data' },
    { id: 'files',     label: 'Fichiers',      icon: FolderTree,      group: 'data' },
    { id: 'logs',      label: 'Journaux',      icon: ScrollText,      group: 'data' },
    { id: 'settings',  label: 'Paramètres',    icon: Settings,        group: 'settings' },
  ];

  const navW = collapsed ? 56 : 240;

  return (
    <div
      className="flex w-full h-screen overflow-hidden"
      style={{ background: 'var(--color-win-bg)' }}
    >
      {/* ── Sidebar / Navigation Pane ── */}
      <aside
        className="flex flex-col flex-shrink-0 transition-all duration-200"
        style={{
          width: navW,
          background: 'var(--color-win-nav)',
          borderRight: '1px solid var(--color-win-stroke)',
          paddingTop: 8,
          paddingBottom: 8,
          overflow: 'hidden',
        }}
      >
        {/* App header */}
        <div
          className="flex items-center gap-3 px-3 mb-3"
          style={{ height: 48, flexShrink: 0 }}
        >
          <img 
            src="/logo.png" 
            alt="Kajy Logo"
            style={{ width: 40, height: 40, borderRadius: 8, flexShrink: 0, objectFit: 'contain' }}
          />
          {!collapsed && (
            <span style={{ fontSize: 15, fontWeight: 600, color: 'var(--color-win-text)', whiteSpace: 'nowrap' }}>
              Kajy Admin
            </span>
          )}
        </div>

        {/* Collapse toggle */}
        <button
          onClick={() => setCollapsed(!collapsed)}
          className="fluent-btn mx-2 mb-4"
          style={{ padding: '5px 8px', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 6, fontSize: 12 }}
        >
          {collapsed ? <ChevronRight size={14} /> : <><ChevronLeft size={14} /><span>Réduire</span></>}
        </button>

        {/* Nav groups */}
        <div className="flex-1 overflow-y-auto no-scrollbar px-2 space-y-0.5">
          {tabs.map((tab, i) => {
            const Icon = tab.icon;
            const active = activeTab === tab.id;
            const prevGroup = i > 0 ? tabs[i - 1].group : tab.group;
            return (
              <div key={tab.id}>
                {/* Group separator */}
                {tab.group !== prevGroup && i !== 0 && (
                  <div className="fluent-divider mx-1 my-2" />
                )}
                <button
                  onClick={() => setActiveTab(tab.id)}
                  className={cn('fluent-nav-item', active && 'active')}
                  title={collapsed ? tab.label : undefined}
                  style={{ justifyContent: collapsed ? 'center' : 'flex-start', paddingLeft: collapsed ? 0 : undefined }}
                >
                  <Icon size={18} style={{ flexShrink: 0 }} />
                  {!collapsed && <span>{tab.label}</span>}
                </button>
              </div>
            );
          })}
        </div>

        {/* Footer user strip */}
        {!collapsed && (
          <div
            className="mx-2 mt-2 p-2 flex items-center gap-2"
            style={{
              borderRadius: 8,
              background: 'var(--color-win-surface)',
              border: '1px solid var(--color-win-stroke)',
              flexShrink: 0,
            }}
          >
            <div
              style={{
                width: 28, height: 28, borderRadius: '50%', flexShrink: 0,
                background: 'var(--color-accent)',
                display: 'flex', alignItems: 'center', justifyContent: 'center',
                color: '#fff', fontWeight: 700, fontSize: 13,
              }}
            >
              A
            </div>
            <div style={{ minWidth: 0 }}>
              <p style={{ fontSize: 13, fontWeight: 600, color: 'var(--color-win-text)', margin: 0, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                Admin User
              </p>
              <p style={{ fontSize: 11, color: 'var(--color-win-text3)', margin: 0 }}>Administrateur</p>
            </div>
          </div>
        )}
      </aside>

      {/* ── Main Content ── */}
      <main
        className="flex-1 overflow-y-auto overflow-x-hidden"
        style={{ background: 'var(--color-win-bg)' }}
      >
        {activeTab === 'dashboard'  && <AdminDashboard />}
        {activeTab === 'system'     && <ServerControl />}
        {activeTab === 'users'      && <UserManagement />}
        {activeTab === 'sessions'   && <ActiveSessions />}
        {activeTab === 'shares'     && <ShareManagement />}
        {activeTab === 'files'      && <FileExplorer />}
        {activeTab === 'logs'       && <SystemLogs />}
        {activeTab === 'settings'   && <GlobalSettings />}
      </main>
    </div>
  );
}

export default App;
