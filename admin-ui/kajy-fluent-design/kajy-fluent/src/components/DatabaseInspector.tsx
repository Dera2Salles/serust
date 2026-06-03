import React, { useState, useEffect } from 'react';
import { Header, SoftCard, cn } from './OneUI';
import { Database, Table, Search, RefreshCcw } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

export const DatabaseInspector: React.FC = () => {
  const [tables, setTables] = useState<string[]>([]);
  const [selectedTable, setSelectedTable] = useState<string | null>(null);
  const [rows, setRows] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchTables = async () => {
    setLoading(true);
    try {
      setTables(['users', 'admins', 'files', 'share_links', 'share_grants', 'access_log', 'read_counters']);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchTables();
  }, []);

  return (
    <div className="pb-10">
      <header className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
        <div>
          <h2 className="text-[26px] font-semibold text-win-text">Inspecteur <span className="text-[--color-accent]">DB</span></h2>
          <p className="text-[--color-win-text3] font-bold text-xs uppercase tracking-[0.2em] opacity-60">Consultation directe des tables SQLite</p>
        </div>
        <button onClick={fetchTables} className="p-4 bg-[--color-win-surface] text-[--color-win-text3] rounded-full hover:bg-[--color-win-nav] transition-all active:scale-90 border border-[--color-win-stroke] shadow-sm">
          <RefreshCcw size={20} />
        </button>
      </header>

      <div className="flex-1 flex flex-col lg:flex-row gap-8 overflow-hidden px-2">
        {/* Table List */}
        <div className="lg:w-72 space-y-2 overflow-y-auto no-scrollbar pb-10">
          <h3 className="text-[10px] font-semibold uppercase tracking-[0.2em] text-[--color-win-text3] opacity-40 px-5 mb-4">Tables Système</h3>
          {tables.map(table => (
            <button
              key={table}
              onClick={() => setSelectedTable(table)}
              className={cn(
                "w-full flex items-center gap-4 px-6 py-4 rounded-md font-semibold text-sm transition-all border",
                selectedTable === table 
                  ? "bg-[--color-accent] text-white border-[--color-accent] shadow-lg shadow-blue/20" 
                  : "bg-[--color-win-surface] text-[--color-win-text3] border-[--color-win-stroke]/50 hover:bg-[--color-win-nav]"
              )}
            >
              <Table size={18} className={selectedTable === table ? "opacity-100" : "opacity-30"} />
              {table}
            </button>
          ))}
        </div>

        {/* Table Content */}
        <div className="flex-1 bg-[--color-win-surface] rounded-[40px] border border-[--color-win-stroke]/50 overflow-hidden flex flex-col shadow-md">
          {selectedTable ? (
            <div className="h-full flex flex-col">
              <div className="p-8 border-b border-[--color-win-stroke] flex items-center justify-between bg-[--color-win-bg]/50">
                <div className="flex flex-col">
                  <h4 className="text-xl font-semibold tabular-nums tracking-tighter text-[--color-win-text]">
                    Structure : <span className="text-[--color-accent]">{selectedTable}</span>
                  </h4>
                  <p className="text-[10px] font-semibold text-[--color-win-text3] uppercase tracking-widest mt-1 opacity-60">Visualisation des enregistrements</p>
                </div>
                <div className="flex items-center gap-2 px-5 py-2.5 bg-[--color-win-surface] border border-[--color-win-stroke] rounded-full text-[10px] font-semibold text-[--color-win-text3] uppercase tracking-widest shadow-sm">
                  <Database size={14} className="text-[--color-accent]" /> SQLite v3
                </div>
              </div>
              <div className="flex-1 overflow-auto p-10">
                <div className="h-full flex flex-col items-center justify-center text-center opacity-30 py-20">
                  <div className="w-20 h-20 bg-[--color-win-bg] rounded-xl flex items-center justify-center mb-6">
                    <Search size={40} className="text-[--color-win-text3]" />
                  </div>
                  <p className="font-semibold text-xl text-[--color-win-text] tracking-tight mb-2">Aperçu du contenu indisponible</p>
                  <p className="text-sm font-medium text-[--color-win-text3] max-w-sm">L'accès direct aux colonnes via l'API bridge sera activé dans la prochaine mise à jour.</p>
                </div>
              </div>
            </div>
          ) : (
            <div className="h-full flex flex-col items-center justify-center opacity-10 py-32">
              <Database size={80} className="mb-6" />
              <p className="text-[18px] font-semibold text-center px-20 text-[--color-win-text]">Sélectionnez une table pour en inspecter les données brutes</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
