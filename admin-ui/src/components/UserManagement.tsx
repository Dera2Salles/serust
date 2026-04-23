import React, { useState, useEffect } from 'react';
import { Header, Card, Button, cn } from './OneUI';
import { Search, User as UserIcon, Shield, MoreVertical } from 'lucide-react';
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
      // result.content[0].text is a JSON string of { users: [...] }
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
    <div className="flex flex-col w-full min-h-screen bg-base pb-20">
      <Header 
        title="Users" 
        subtitle="Manage accounts and permissions" 
      />

      <div className="px-8 mb-8">
        <form onSubmit={handleSearch} className="relative max-w-2xl">
          <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-overlay0" size={20} />
          <input 
            type="text"
            placeholder="Search users by username..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            className="w-full bg-surface0 border-none rounded-2xl py-4 pl-12 pr-4 text-text focus:ring-2 focus:ring-blue transition-all"
          />
        </form>
      </div>

      <div className="px-8 space-y-4">
        {loading ? (
          <p className="text-center py-10 text-subtext0">Searching users...</p>
        ) : users.length === 0 ? (
          <p className="text-center py-10 text-subtext0">No users found.</p>
        ) : (
          users.map((user, i) => (
            <Card key={i} className="flex items-center justify-between group">
              <div className="flex items-center gap-4">
                <div className="w-12 h-12 rounded-full bg-blue/10 text-blue flex items-center justify-center font-bold text-xl">
                  {user.username.charAt(0).toUpperCase()}
                </div>
                <div>
                  <h4 className="font-semibold text-lg">{user.username}</h4>
                  <p className="text-sm text-subtext0 flex items-center gap-1">
                    <Shield size={14} className="text-green" /> Standard User
                  </p>
                </div>
              </div>
              
              <div className="flex items-center gap-2">
                <Button variant="secondary" className="hidden group-hover:block px-4 py-2">View Stats</Button>
                <button className="p-2 hover:bg-surface1 rounded-full transition-colors">
                  <MoreVertical size={20} className="text-overlay0" />
                </button>
              </div>
            </Card>
          ))
        )}
      </div>
    </div>
  );
};
