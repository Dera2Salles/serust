import React, { useState, useEffect } from 'react';
import { Header, Card, cn } from './OneUI';
import { Folder, FileText, ChevronLeft, ChevronRight, HardDrive } from 'lucide-react';
import { callMcpTool } from '../lib/mcp';

export const FileExplorer = () => {
  const [path, setPath] = useState('/');
  const [entries, setEntries] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const [username, setUsername] = useState('admin_dev');

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

  return (
    <div className="flex flex-col w-full min-h-screen bg-base pb-20">
      <Header 
        title="File Explorer" 
        subtitle="Browse server storage" 
      />

      <div className="px-8 flex items-center gap-4 mb-6">
        <button 
          onClick={navigateUp}
          disabled={path === '/'}
          className="p-3 bg-surface0 rounded-2xl hover:bg-surface1 disabled:opacity-30 disabled:cursor-not-allowed transition-all"
        >
          <ChevronLeft size={24} />
        </button>
        <div className="flex-1 bg-surface0 rounded-2xl px-6 py-3 flex items-center gap-3">
          <HardDrive size={18} className="text-blue" />
          <span className="font-mono text-sm">{path}</span>
        </div>
      </div>

      <div className="px-8 grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
        {loading ? (
          <div className="col-span-full py-20 text-center text-subtext0">Loading directory...</div>
        ) : entries.length === 0 ? (
          <div className="col-span-full py-20 text-center text-subtext0">This directory is empty.</div>
        ) : (
          entries.map((entry, i) => (
            <Card 
              key={i} 
              onClick={() => entry.isDir && fetchDirectory(path === '/' ? `/${entry.name}` : `${path}/${entry.name}`)}
              className={cn(
                "flex flex-col items-center justify-center gap-4 py-8 cursor-pointer hover:bg-surface1",
                !entry.isDir && "cursor-default opacity-80"
              )}
            >
              {entry.isDir ? (
                <Folder size={48} className="text-blue fill-blue/10" />
              ) : (
                <FileText size={48} className="text-overlay2" />
              )}
              <span className="text-sm font-medium text-center truncate w-full px-4">{entry.name}</span>
            </Card>
          ))
        )}
      </div>
    </div>
  );
};
