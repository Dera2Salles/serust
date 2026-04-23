import React, { useState } from 'react';
import { AdminDashboard } from './components/AdminDashboard';
import { UserManagement } from './components/UserManagement';
import { FileExplorer } from './components/FileExplorer';
import { LayoutDashboard, Users, FolderTree, Settings, Menu } from 'lucide-react';
import { cn } from './components/OneUI';

function App() {
  const [activeTab, setActiveTab] = useState('dashboard');

  const tabs = [
    { id: 'dashboard', label: 'Dashboard', icon: LayoutDashboard },
    { id: 'users', label: 'Users', icon: Users },
    { id: 'files', label: 'Files', icon: FolderTree },
    { id: 'settings', label: 'Settings', icon: Settings },
  ];

  return (
    <div className="flex w-full h-screen bg-base overflow-hidden">
      {/* Side Navigation (One UI Rail) */}
      <div className="w-24 md:w-72 flex flex-col bg-mantle border-r border-surface0 items-center md:items-stretch py-10 px-5 transition-all">
        <div className="px-4 mb-14 hidden md:flex items-center gap-3">
          <div className="w-10 h-10 bg-blue rounded-xl flex items-center justify-center">
            <FolderTree className="text-crust" size={24} />
          </div>
          <h2 className="text-2xl font-black text-text tracking-tighter">AroSaina <span className="text-blue">Admin</span></h2>
        </div>
        
        <nav className="flex-1 space-y-3">
          {tabs.map((tab) => {
            const Icon = tab.icon;
            const active = activeTab === tab.id;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={cn(
                  "w-full flex items-center gap-4 px-5 py-4 rounded-[20px] transition-all duration-300 group",
                  active 
                    ? "bg-blue/10 text-blue scale-[1.02]" 
                    : "text-subtext0 hover:bg-surface0 hover:text-text"
                )}
              >
                <Icon size={24} className={cn("transition-transform group-active:scale-90", active && "stroke-[2.5px]")} />
                <span className={cn("hidden md:block font-bold tracking-tight", active && "text-blue")}>{tab.label}</span>
              </button>
            );
          })}
        </nav>

        <div className="mt-auto p-5 bg-surface0/30 rounded-[32px] hidden md:flex items-center gap-4 border border-surface0/50">
          <div className="w-12 h-12 rounded-full bg-mauve flex items-center justify-center text-crust font-black text-xl shadow-lg shadow-mauve/20">
            A
          </div>
          <div className="min-w-0">
            <p className="text-sm font-extrabold text-text truncate">Admin User</p>
            <p className="text-xs text-subtext0 font-medium truncate opacity-70">admin@arosaina.io</p>
          </div>
        </div>
      </div>

      {/* Main Content Area */}
      <main className="flex-1 overflow-y-auto overflow-x-hidden relative">
        <div className="absolute inset-0 bg-gradient-to-br from-blue/5 via-transparent to-mauve/5 pointer-events-none opacity-50" />
        <div className="relative z-10 h-full">
          {activeTab === 'dashboard' && <AdminDashboard />}
          {activeTab === 'users' && <UserManagement />}
          {activeTab === 'files' && <FileExplorer />}
          {activeTab === 'settings' && (
            <div className="flex flex-col items-center justify-center h-full text-center p-12">
              <div className="w-24 h-24 bg-surface0 rounded-[32px] flex items-center justify-center mb-8 border border-surface1">
                <Settings size={48} className="text-subtext0 opacity-50" />
              </div>
              <h3 className="text-3xl font-black tracking-tighter text-text mb-3">Settings</h3>
              <p className="text-subtext0 max-w-sm font-medium">Server configuration and administrative settings will be available in a future update.</p>
            </div>
          )}
        </div>
      </main>
    </div>
  );
}

export default App;
