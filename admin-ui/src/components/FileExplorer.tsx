import React, { useState, useEffect } from 'react';
import { Header, SoftCard, AroIconButton, cn } from './OneUI';
import { Folder, FileText, ChevronLeft, HardDrive, RefreshCw, MoreVertical } from 'lucide-react';
import { callMcpTool } from '../lib/mcp';

export const FileExplorer = () => {
  const [path, setPath] = useState('/');
  const [entries, setEntries] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const [username] = useState('admin_dev');

  const fetchDirectory = async (targetPath: string) => {
    setLoading(true);
    try {
      const result = await callMcpTool('list_directory', { username, path: targetPath });
      const text = result.content[0].text;
      
      const lines = text.split('\n').slice(1);
      const parsed = lines.map((line: string) => {
        const isDir = line.includes('📁');
        const namePart = line.replace('  📁 ', '').replace('  📄 ', '');
        const name = namePart.split(' (')[0];
        return { name, isDir };
      }).filter((e: any) => e.name);
      
      setEntries(parsed);
      setPath(targetPath);
    } catch (err) {
      console.error("Failed to list directory:", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchDirectory('/');
  }, []);

  const navigateUp = () => {
    if (path === '/') return;
    const parts = path.split('/').filter(Boolean);
    parts.pop();
    fetchDirectory('/' + parts.join('/'));
  };

  const getFileIcon = (name: string, isDir: boolean) => {
    if (isDir) return <Folder size={40} className="text-yellow fill-yellow/10" />;
    
    const ext = name.split('.').last?.toLowerCase() || '';
    if (['jpg', 'jpeg', 'png', 'gif'].includes(ext)) return <FileText size={40} className="text-green" />;
    if (['pdf'].includes(ext)) return <FileText size={40} className="text-red" />;
    if (['txt', 'md'].includes(ext)) return <FileText size={40} className="text-blue" />;
    return <FileText size={40} className="text-overlay2" />;
  };

  return (
    <div className="flex flex-col w-full min-h-screen bg-transparent pb-20">
      <Header 
        title="Explorateur" 
        subtitle="Parcourir le stockage du serveur" 
      />

      <div className="px-10 flex items-center gap-4 mb-8">
        <AroIconButton 
          icon={<ChevronLeft size={24} />}
          onClick={navigateUp}
          disabled={path === '/'}
          className={path === '/' ? "opacity-30 cursor-not-allowed" : ""}
        />
        <div className="flex-1 bg-mantle border border-surface0 rounded-[24px] px-6 py-4 flex items-center gap-4 shadow-inner">
          <HardDrive size={18} className="text-blue" />
          <div className="flex items-center gap-1 font-mono text-sm overflow-hidden">
             {path.split('/').map((seg, i) => (
                <React.Fragment key={i}>
                  {i > 0 && <span className="text-surface2">/</span>}
                  <span className={cn(i === path.split('/').length - 1 ? "text-text font-bold" : "text-subtext0 opacity-60")}>{seg || 'root'}</span>
                </React.Fragment>
             ))}
          </div>
        </div>
        <AroIconButton 
          icon={<RefreshCw size={20} className={loading ? "animate-spin" : ""} />}
          onClick={() => fetchDirectory(path)}
        />
      </div>

      <div className="px-10 grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
        {loading ? (
          <div className="col-span-full py-32 flex flex-col items-center opacity-50">
             <div className="w-12 h-12 border-4 border-blue/20 border-t-blue rounded-full animate-spin mb-4" />
             <p className="text-subtext0 font-bold tracking-tight">Chargement du dossier...</p>
          </div>
        ) : entries.length === 0 ? (
          <SoftCard className="col-span-full py-32 flex flex-col items-center justify-center border-dashed opacity-50">
            <Folder size={64} className="text-surface2 mb-4 opacity-20" />
            <p className="text-subtext0 font-bold">Ce dossier est vide</p>
          </SoftCard>
        ) : (
          entries.map((entry, i) => (
            <SoftCard 
              key={i} 
              onClick={() => entry.isDir && fetchDirectory(path === '/' ? `/${entry.name}` : `${path}/${entry.name}`)}
              className={cn(
                "flex flex-col items-center justify-center gap-4 py-10 cursor-pointer group hover:border-blue/30 relative overflow-hidden",
                !entry.isDir && "cursor-default opacity-90"
              )}
            >
              <div className="absolute top-4 right-4 opacity-0 group-hover:opacity-100 transition-opacity">
                <MoreVertical size={16} className="text-subtext0" />
              </div>
              <div className="p-4 bg-surface0/50 rounded-3xl group-hover:scale-110 transition-transform duration-300">
                {getFileIcon(entry.name, entry.isDir)}
              </div>
              <span className="text-sm font-black text-center truncate w-full px-5 tracking-tight group-hover:text-blue transition-colors">
                {entry.name}
              </span>
            </SoftCard>
          ))
        )}
      </div>
    </div>
  );
};

// Extension for array access
declare global {
  interface Array<T> {
    last: T | undefined;
  }
}

if (!Array.prototype.hasOwnProperty('last')) {
  Object.defineProperty(Array.prototype, 'last', {
    get: function() {
      return this[this.length - 1];
    }
  });
}
