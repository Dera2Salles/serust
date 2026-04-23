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
      <div className="w-24 md:w-64 flex flex-col bg-mantle border-r border-surface0 items-center md:items-stretch py-8 px-4 transition-all">
        <div className="px-4 mb-12 hidden md:block">
          <h2 className="text-2xl font-bold text-blue tracking-tighter">TCP Admin</h2>
        </div>
        
        <nav className="flex-1 space-y-2">
          {tabs.map((tab) => {
            const Icon = tab.icon;
            const active = activeTab === tab.id;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={cn(
                  "w-full flex items-center gap-4 px-4 py-3 rounded-2xl transition-all duration-200",
                  active 
                    ? "bg-blue/10 text-blue font-semibold scale-105" 
                    : "text-subtext0 hover:bg-surface0 hover:text-text"
                )}
              >
                <Icon size={24} />
                <span className="hidden md:block">{tab.label}</span>
              </button>
            );
          })}
        </nav>

        <div className="mt-auto px-4 py-4 bg-surface0 rounded-2xl hidden md:flex items-center gap-3">
          <div className="w-10 h-10 rounded-full bg-mauve flex items-center justify-center text-crust font-bold">
            A
          </div>
          <div className="min-w-0">
            <p className="text-sm font-semibold truncate">Admin User</p>
            <p className="text-xs text-subtext0 truncate">admin@local</p>
          </div>
        </div>
      </div>

      {/* Main Content Area */}
      <main className="flex-1 overflow-y-auto overflow-x-hidden">
        {activeTab === 'dashboard' && <AdminDashboard />}
        {activeTab === 'users' && <UserManagement />}
        {activeTab === 'files' && <FileExplorer />}
        {activeTab === 'settings' && (
          <div className="p-12 text-center text-subtext0">
            <Settings size={48} className="mx-auto mb-4 opacity-20" />
            <h3 className="text-xl">Settings coming soon</h3>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
