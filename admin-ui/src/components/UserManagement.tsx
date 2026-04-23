import React, { useState, useEffect } from 'react';
import { Header, SoftCard, Button, ModernTextField, AroChip, cn } from './OneUI';
import { Search, Shield, MoreVertical, Mail, Calendar } from 'lucide-react';
import { callMcpTool } from '../lib/mcp';

export const UserManagement = () => {
  const [query, setQuery] = useState('');
  const [users, setUsers] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);

  const handleSearch = async (e?: React.FormEvent) => {
    e?.preventDefault();
    setLoading(true);
    try {
      const result = await callMcpTool('search_users', { query });
      const data = JSON.parse(result.content[0].text);
      setUsers(data.users || []);
    } catch (err) {
      console.error("Search failed:", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    handleSearch();
  }, []);

  return (
    <div className="flex flex-col w-full min-h-screen bg-transparent pb-20">
      <Header 
        title="Users" 
        subtitle="Gérer les comptes et permissions" 
      />

      <div className="px-10 mb-10">
        <form onSubmit={handleSearch} className="max-w-3xl">
          <ModernTextField 
            placeholder="Rechercher par nom d'utilisateur..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            prefixIcon={<Search size={20} />}
          />
        </form>
      </div>

      <div className="px-10 space-y-4">
        {loading ? (
          <div className="flex flex-col items-center justify-center py-20 opacity-50">
             <div className="w-10 h-10 border-4 border-blue/20 border-t-blue rounded-full animate-spin mb-4" />
             <p className="text-subtext0 font-bold tracking-tight">Recherche en cours...</p>
          </div>
        ) : users.length === 0 ? (
          <SoftCard className="flex flex-col items-center justify-center py-20 border-dashed">
            <Search size={48} className="text-surface2 mb-4 opacity-20" />
            <p className="text-subtext0 font-bold">Aucun utilisateur trouvé</p>
          </SoftCard>
        ) : (
          users.map((user, i) => (
            <SoftCard key={i} className="flex items-center justify-between group hover:border-blue/20 transition-all py-5">
              <div className="flex items-center gap-6">
                <div className="w-16 h-16 rounded-3xl bg-gradient-to-br from-blue to-mauve text-crust flex items-center justify-center font-black text-2xl shadow-lg shadow-blue/10">
                  {user.username.charAt(0).toUpperCase()}
                </div>
                <div>
                  <div className="flex items-center gap-3 mb-1">
                    <h4 className="font-black text-xl tracking-tighter">{user.username}</h4>
                    <AroChip label="Actif" color="green" />
                  </div>
                  <div className="flex items-center gap-4 text-sm text-subtext0 font-medium">
                    <span className="flex items-center gap-1.5"><Mail size={14} className="opacity-50" /> {user.email || 'Pas d\'email'}</span>
                    <span className="flex items-center gap-1.5"><Calendar size={14} className="opacity-50" /> Membre depuis 2024</span>
                  </div>
                </div>
              </div>
              
              <div className="flex items-center gap-3">
                <Button variant="secondary" className="px-5 py-2 font-bold text-sm hidden sm:flex">
                  Statistiques
                </Button>
                <button className="w-12 h-12 flex items-center justify-center hover:bg-surface0 rounded-2xl transition-colors">
                  <MoreVertical size={20} className="text-subtext0" />
                </button>
              </div>
            </SoftCard>
          ))
        )}
      </div>
    </div>
  );
};
