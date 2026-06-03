import React, { useState, useEffect } from 'react';
import { Header, SoftCard, AroIconButton, cn, Button, ModernTextField } from './OneUI';
import { Folder, FileText, ChevronLeft, HardDrive, RefreshCw, MoreVertical, Plus, Upload, Trash2, Edit3, X, Eye, Search, Share2, Users } from 'lucide-react';
import { callMcpTool } from '../lib/mcp';
import { invoke } from '@tauri-apps/api/core';

interface FileEntry {
  name: string;
  is_dir: boolean;
  size: number;
  checksum: string | null;
}

export const FileExplorer = () => {
  const [path, setPath] = useState('/');
  const [entries, setEntries] = useState<FileEntry[]>([]);
  const [loading, setLoading] = useState(false);
  
  // Admin Context
  const [targetUsername, setTargetUsername] = useState('admin_dev');
  const [allUsers, setAllUsers] = useState<string[]>([]);
  
  // Modals / UI State
  const [showNewFolder, setShowNewFolder] = useState(false);
  const [showUpload, setShowUpload] = useState(false);
  const [showRead, setShowRead] = useState(false);
  const [showShare, setShowShare] = useState(false);
  const [folderName, setFolderName] = useState('');
  const [fileName, setFileName] = useState('');
  const [fileContent, setFileContent] = useState('');
  const [viewingFile, setViewingFile] = useState<{name: string, content: string} | null>(null);
  const [menuOpen, setMenuOpen] = useState<string | null>(null);
  
  // Sharing state
  const [selectedEntry, setSelectedEntry] = useState<FileEntry | null>(null);
  const [shareUserQuery, setShareUserQuery] = useState('');
  const [userResults, setUserResults] = useState<any[]>([]);
  const [shareTargetUser, setShareTargetUser] = useState<string | null>(null);
  const [canWrite, setCanWrite] = useState(false);

  const fetchUsers = async () => {
    try {
      const users = await invoke<any[]>('get_users_from_db');
      const names = users.map(u => u.username);
      setAllUsers(names);
      if (names.length > 0 && !names.includes(targetUsername)) {
        setTargetUsername(names[0]);
      }
    } catch (e) {
      console.error("Failed to fetch users list:", e);
    }
  };

  const fetchDirectory = async (targetPath: string) => {
    setLoading(true);
    setMenuOpen(null);
    try {
      const result = await callMcpTool('list_directory', { username: targetUsername, path: targetPath });
      const data = JSON.parse(result.content[0].text);
      setEntries(data.entries || []);
      setPath(targetPath);
    } catch (err) {
      console.error("Failed to list directory:", err);
      setEntries([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchUsers();
  }, []);

  useEffect(() => {
    fetchDirectory('/');
  }, [targetUsername]);

  const navigateUp = () => {
    if (path === '/') return;
    const parts = path.split('/').filter(Boolean);
    parts.pop();
    fetchDirectory('/' + parts.join('/'));
  };

  const handleCreateFolder = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!folderName) return;
    try {
      await callMcpTool('create_folder', { username: targetUsername, path, name: folderName });
      setFolderName('');
      setShowNewFolder(false);
      fetchDirectory(path);
    } catch (err) {
      alert("Failed to create folder: " + err);
    }
  };

  const handleUploadFile = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!fileName || !fileContent) return;
    try {
      await callMcpTool('create_file', { username: targetUsername, path, filename: fileName, content: fileContent });
      setFileName('');
      setFileContent('');
      setShowUpload(false);
      fetchDirectory(path);
    } catch (err) {
      alert("Failed to upload file: " + err);
    }
  };

  const handleDelete = async (entry: FileEntry) => {
    if (!confirm(`Are you sure you want to delete ${entry.name}?`)) return;
    try {
      await callMcpTool('delete_file', { username: targetUsername, path, filename: entry.name });
      fetchDirectory(path);
    } catch (err) {
      alert("Failed to delete: " + err);
    }
  };

  const handleRename = async (entry: FileEntry) => {
    const newName = prompt(`Rename ${entry.name} to:`, entry.name);
    if (!newName || newName === entry.name) return;
    try {
      await callMcpTool('rename_file', { username: targetUsername, path, old_name: entry.name, new_name: newName });
      fetchDirectory(path);
    } catch (err) {
      alert("Failed to rename: " + err);
    }
  };

  const handleReadFile = async (entry: FileEntry) => {
    try {
      setLoading(true);
      const result = await callMcpTool('read_file', { username: targetUsername, path, filename: entry.name });
      setViewingFile({ name: entry.name, content: result.content[0].text });
      setShowRead(true);
    } catch (err) {
      alert("Failed to read file: " + err);
    } finally {
      setLoading(false);
    }
  };

  const handleSearchUsers = async (query: string) => {
    setShareUserQuery(query);
    if (query.length < 2) {
      setUserResults([]);
      return;
    }
    try {
      const result = await callMcpTool('search_users', { query });
      const data = JSON.parse(result.content[0].text);
      setUserResults(data.users || []);
    } catch (err) {
      console.error(err);
    }
  };

  const handleGrantShare = async () => {
    if (!shareTargetUser || !selectedEntry) return;
    try {
      setLoading(true);
      const filePath = path === '/' ? selectedEntry.name : `${path}/${selectedEntry.name}`;
      await callMcpTool('create_share_grant', { 
        username: targetUsername, 
        path: filePath, 
        target_user: shareTargetUser,
        can_read: true,
        can_write: canWrite
      });
      alert(`Successfully shared ${selectedEntry.name} with ${shareTargetUser}`);
      setShowShare(false);
    } catch (err) {
      alert("Sharing failed: " + err);
    } finally {
      setLoading(false);
    }
  };

  const getFileIcon = (name: string, isDir: boolean) => {
    if (isDir) return <Folder size={40} className="text-yellow opacity-80" />;
    const ext = name.split('.').pop()?.toLowerCase() || '';
    if (['jpg', 'jpeg', 'png', 'gif', 'webp'].includes(ext)) return <FileText size={40} className="text-[--color-success] opacity-70" />;
    if (['pdf', 'zip', 'rar', 'gz'].includes(ext)) return <FileText size={40} className="text-[--color-error] opacity-70" />;
    return <FileText size={40} className="text-[--color-accent] opacity-70" />;
  };

  return (
    <div className="pb-10">
      <Header 
        title="Fichiers" 
        subtitle="Explorateur de fichiers multi-utilisateurs" 
      />

      <div className="px-8 flex flex-col md:flex-row items-stretch md:items-center gap-4 mb-8">
        <div className="flex items-center gap-3">
            <AroIconButton 
                icon={<ChevronLeft size={24} />}
                onClick={navigateUp}
                disabled={path === '/'}
                className={path === '/' ? "opacity-30 cursor-not-allowed" : "bg-white"}
            />
            
            <div className="relative group">
                <div className="absolute left-4 top-1/2 -translate-y-1/2 text-[--color-win-text3]">
                    <Users size={18} />
                </div>
                <select
                    value={targetUsername}
                    onChange={(e) => setTargetUsername(e.target.value)}
                    className="bg-white border border-[--color-win-stroke] text-[--color-win-text] pl-12 pr-10 py-3 rounded-md font-semibold text-sm transition-all focus:outline-none focus:border-[--color-accent] cursor-pointer appearance-none shadow-sm"
                >
                    {allUsers.map(u => (
                        <option key={u} value={u}>{u}</option>
                    ))}
                </select>
                <div className="absolute right-4 top-1/2 -translate-y-1/2 pointer-events-none text-[--color-win-text3]">
                    <ChevronLeft size={16} className="-rotate-90" />
                </div>
            </div>
        </div>

        <div className="flex-1 bg-white border border-[--color-win-stroke] rounded-md px-6 py-3 flex items-center gap-4 shadow-sm min-w-0">
          <HardDrive size={18} className="text-[--color-accent] flex-shrink-0" />
          <div className="flex items-center gap-1 font-mono text-sm overflow-hidden whitespace-nowrap">
             {path.split('/').map((seg, i) => (
                <React.Fragment key={i}>
                  {i > 0 && <span className="text-surface2">/</span>}
                  <span className={cn(i === path.split('/').length - 1 ? "text-[--color-win-text] font-bold" : "text-[--color-win-text3]")}>{seg || 'root'}</span>
                </React.Fragment>
             ))}
          </div>
        </div>

        <div className="flex gap-2">
            <AroIconButton 
              icon={<RefreshCw size={18} className={loading ? "animate-spin" : ""} />}
              onClick={() => fetchDirectory(path)}
              className="bg-white"
            />
            <AroIconButton 
              icon={<Plus size={18} />}
              onClick={() => setShowNewFolder(true)}
              className="text-[--color-success] border-green/20 bg-white"
            />
            <AroIconButton 
              icon={<Upload size={18} />}
              onClick={() => setShowUpload(true)}
              className="text-[--color-accent] border-[--color-accent]/20 bg-white"
            />
        </div>
      </div>

      <div className="px-8 grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-6">
        {loading && entries.length === 0 ? (
          <div className="col-span-full py-32 flex flex-col items-center opacity-40">
             <div className="w-12 h-12 border-4 border-[--color-accent]/20 border-t-blue rounded-full animate-spin mb-4" />
             <p className="text-[--color-win-text] font-bold tracking-tight">Accès aux fichiers...</p>
          </div>
        ) : entries.length === 0 ? (
          <div className="fluent-card col-span-full py-32 flex flex-col items-center justify-center border-dashed border-2 border-[--color-win-stroke]  shadow-none">
            <Folder size={64} className="text-surface2 mb-4 opacity-10" />
            <p className="text-[--color-win-text3] font-bold">Dossier vide</p>
          </div>
        ) : (
          entries.map((entry, i) => (
            <div 
              key={i} 
              onClick={() => entry.is_dir ? fetchDirectory(path === '/' ? `/${entry.name}` : `${path}/${entry.name}`) : handleReadFile(entry)}
              className={cn(
                "flex flex-col items-center justify-center gap-5 py-12 cursor-pointer group relative overflow-hidden bg-white shadow-sm border border-[--color-win-stroke]/50",
              )}
            >
              <div 
                className="absolute top-4 right-4 z-20 opacity-0 group-hover:opacity-100 transition-opacity"
                onClick={(e) => { e.stopPropagation(); setMenuOpen(menuOpen === entry.name ? null : entry.name); }}
              >
                <div className="p-2 hover:bg-[--color-win-nav] rounded-xl transition-colors">
                    <MoreVertical size={18} className="text-[--color-win-text3]" />
                </div>
              </div>

              {menuOpen === entry.name && (
                  <div className="absolute top-12 right-6 bg-white border border-[--color-win-stroke2] rounded-lg shadow-2xl z-30 py-2 min-w-[160px] animate-in zoom-in duration-150">
                      <button onClick={(e) => { e.stopPropagation(); setSelectedEntry(entry); setShowShare(true); setMenuOpen(null); }} className="w-full text-left px-5 py-3 hover:bg-[--color-win-nav] flex items-center gap-3 text-sm font-bold text-[--color-win-text]">
                        <Share2 size={16} className="text-[--color-accent]" /> Partager
                      </button>
                      <button onClick={(e) => { e.stopPropagation(); handleRename(entry); }} className="w-full text-left px-5 py-3 hover:bg-[--color-win-nav] flex items-center gap-3 text-sm font-bold text-[--color-win-text]">
                        <Edit3 size={16} /> Renommer
                      </button>
                      <button onClick={(e) => { e.stopPropagation(); handleDelete(entry); }} className="w-full text-left px-5 py-3 hover:bg-[--color-error]/5 flex items-center gap-3 text-sm font-bold text-[--color-error]">
                        <Trash2 size={16} /> Supprimer
                      </button>
                  </div>
              )}

              <div className="p-6 bg-[--color-win-nav] rounded-lg group-hover:scale-105 transition-transform duration-500 shadow-inner">
                {getFileIcon(entry.name, entry.is_dir)}
              </div>
              
              <div className="flex flex-col items-center gap-1.5 w-full px-4">
                <span className="text-sm font-semibold text-center truncate w-full tracking-tight text-[--color-win-text] group-hover:text-[--color-accent] transition-colors">
                    {entry.name}
                </span>
                {!entry.is_dir && (
                    <span className="text-[10px] font-semibold font-mono text-[--color-win-text3] opacity-60 uppercase">
                        {(entry.size / 1024).toFixed(1)} KB
                    </span>
                )}
              </div>
            </div>
          ))
        )}
      </div>

      {/* New Folder Modal */}
      {showNewFolder && (
          <div className="fixed inset-0 bg-text/40 backdrop-blur-xl z-50 flex items-center justify-center p-6 animate-in fade-in duration-200">
              <div className="fluent-card max-w-md w-full p-8 border border-[--color-win-stroke] shadow-2xl relative bg-white">
                  <button onClick={() => setShowNewFolder(false)} className="absolute top-6 right-6 text-[--color-win-text3] hover:text-[--color-win-text]"><X /></button>
                  <h3 className="text-2xl font-semibold text-[--color-win-text] mb-6 tracking-tight">Nouveau Dossier</h3>
                  <form onSubmit={handleCreateFolder} className="space-y-6">
                      <ModernTextField 
                        label="Nom du répertoire"
                        autoFocus
                        value={folderName}
                        onChange={(e) => setFolderName(e.target.value)}
                        placeholder="ex: Photos"
                      />
                      <div className="flex gap-3 pt-4">
                          <button type="button" className="flex-1 px-5 py-4 bg-[--color-win-nav] text-[--color-win-text] font-semibold rounded-md" onClick={() => setShowNewFolder(false)}>Annuler</button>
                          <button type="submit" className="flex-[2] px-5 py-4 bg-[--color-success] text-white font-semibold rounded-md shadow-lg shadow-green/20">Créer</button>
                      </div>
                  </form>
              </div>
          </div>
      )}

      {/* Upload Modal */}
      {showUpload && (
          <div className="fixed inset-0 bg-text/40 backdrop-blur-xl z-50 flex items-center justify-center p-6 animate-in fade-in duration-200">
              <div className="fluent-card max-w-xl w-full p-8 border border-[--color-win-stroke] shadow-2xl relative bg-white">
                  <button onClick={() => setShowUpload(false)} className="absolute top-6 right-6 text-[--color-win-text3] hover:text-[--color-win-text]"><X /></button>
                  <h3 className="text-2xl font-semibold text-[--color-win-text] mb-6 tracking-tight">Nouveau fichier texte</h3>
                  <form onSubmit={handleUploadFile} className="space-y-4">
                      <ModernTextField 
                        label="Nom du fichier"
                        autoFocus
                        value={fileName}
                        onChange={(e) => setFileName(e.target.value)}
                        placeholder="ex: notes.txt"
                      />
                      <div className="flex flex-col gap-2">
                        <label className="text-[10px] font-semibold uppercase tracking-widest text-[--color-win-text3] mb-1 px-1">Contenu</label>
                        <textarea 
                            className="w-full bg-[--color-win-nav] border-2 border-[--color-win-stroke] rounded-md p-5 text-[--color-win-text] placeholder:text-surface2 focus:border-[--color-accent] transition-all outline-none min-h-[180px] font-mono text-sm shadow-inner"
                            value={fileContent}
                            onChange={(e) => setFileContent(e.target.value)}
                            placeholder="Tapez votre texte ici..."
                        />
                      </div>
                      <div className="mt-8 flex gap-3">
                          <button type="button" className="flex-1 px-5 py-4 bg-[--color-win-nav] text-[--color-win-text] font-semibold rounded-md" onClick={() => setShowUpload(false)}>Annuler</button>
                          <button type="submit" className="flex-[2] px-5 py-4 bg-[--color-accent] text-white font-semibold rounded-md shadow-lg shadow-blue/20">Uploader</button>
                      </div>
                  </form>
              </div>
          </div>
      )}

      {/* Read File Modal */}
      {showRead && viewingFile && (
          <div className="fixed inset-0 bg-[--color-win-surface]/90 backdrop-blur-xl z-50 flex items-center justify-center p-6 sm:p-12 animate-in slide-in-from-bottom duration-300">
              <div className="fluent-card max-w-5xl w-full h-full flex flex-col p-0 border border-[--color-win-stroke2] overflow-hidden bg-white">
                  <div className="p-8 border-b border-[--color-win-stroke] flex items-center justify-between bg-[--color-win-nav]/30">
                      <div className="flex items-center gap-4">
                        <div className="p-3 bg-[--color-accent]/10 rounded-lg text-[--color-accent] shadow-sm"><FileText size={24} /></div>
                        <h3 className="text-[18px] font-semibold text-[--color-win-text]">{viewingFile.name}</h3>
                      </div>
                      <button onClick={() => setShowRead(false)} className="p-3 hover:bg-[--color-win-nav] rounded-full transition-colors text-[--color-win-text3]"><X /></button>
                  </div>
                  <div className="flex-1 p-10 overflow-auto font-mono text-sm leading-relaxed text-[--color-win-text]/80 selection:bg-[--color-accent]/10 scrollbar-thin">
                      <pre className="whitespace-pre-wrap">{viewingFile.content}</pre>
                  </div>
                  <div className="p-6 bg-[--color-win-nav]/30 border-t border-[--color-win-stroke] flex justify-end">
                      <button onClick={() => setShowRead(false)} className="px-8 py-3 bg-text text-white font-semibold rounded-full shadow-lg active:scale-95 transition-all">Fermer</button>
                  </div>
              </div>
          </div>
      )}
    </div>
  );
};
