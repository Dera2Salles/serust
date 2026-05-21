import React, { useState, useEffect } from 'react';
import { Header, SoftCard, AroIconButton, cn, Button, ModernTextField } from './OneUI';
import { Folder, FileText, ChevronLeft, HardDrive, RefreshCw, MoreVertical, Plus, Upload, Trash2, Edit3, X, Eye, Search, Share2 } from 'lucide-react';
import { callMcpTool } from '../lib/mcp';

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
  const [username] = useState('admin_dev');
  
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
  const [targetUser, setTargetUser] = useState<string | null>(null);
  const [canWrite, setCanWrite] = useState(false);
  const [searchingUsers, setSearchingUsers] = useState(false);

  const fetchDirectory = async (targetPath: string) => {
    setLoading(true);
    setMenuOpen(null);
    try {
      const result = await callMcpTool('list_directory', { username, path: targetPath });
      const data = JSON.parse(result.content[0].text);
      setEntries(data.entries || []);
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

  const handleCreateFolder = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!folderName) return;
    try {
      await callMcpTool('create_folder', { username, path, name: folderName });
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
      await callMcpTool('create_file', { username, path, filename: fileName, content: fileContent });
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
      await callMcpTool('delete_file', { username, path, filename: entry.name });
      fetchDirectory(path);
    } catch (err) {
      alert("Failed to delete: " + err);
    }
  };

  const handleRename = async (entry: FileEntry) => {
    const newName = prompt(`Rename ${entry.name} to:`, entry.name);
    if (!newName || newName === entry.name) return;
    try {
      await callMcpTool('rename_file', { username, path, old_name: entry.name, new_name: newName });
      fetchDirectory(path);
    } catch (err) {
      alert("Failed to rename: " + err);
    }
  };

  const handleReadFile = async (entry: FileEntry) => {
    try {
      setLoading(true);
      const result = await callMcpTool('read_file', { username, path, filename: entry.name });
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
    setSearchingUsers(true);
    try {
      const result = await callMcpTool('search_users', { query });
      const data = JSON.parse(result.content[0].text);
      setUserResults(data.users || []);
    } catch (err) {
      console.error(err);
    } finally {
      setSearchingUsers(false);
    }
  };

  const handleGrantShare = async () => {
    if (!targetUser || !selectedEntry) return;
    try {
      setLoading(true);
      const filePath = path === '/' ? selectedEntry.name : `${path}/${selectedEntry.name}`;
      await callMcpTool('create_share_grant', { 
        username, 
        path: filePath, 
        target_user: targetUser,
        can_read: true,
        can_write: canWrite
      });
      alert(`Successfully shared ${selectedEntry.name} with ${targetUser}`);
      setShowShare(false);
    } catch (err) {
      alert("Sharing failed: " + err);
    } finally {
      setLoading(false);
    }
  };

  const getFileIcon = (name: string, isDir: boolean) => {
    if (isDir) return <Folder size={40} className="text-yellow fill-yellow/10" />;
    
    const ext = name.split('.').last?.toLowerCase() || '';
    if (['jpg', 'jpeg', 'png', 'gif'].includes(ext)) return <FileText size={40} className="text-green" />;
    if (['pdf'].includes(ext)) return <FileText size={40} className="text-red" />;
    if (['txt', 'md', 'json', 'rs', 'dart', 'js', 'ts'].includes(ext)) return <FileText size={40} className="text-blue" />;
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
        <div className="flex gap-2">
            <AroIconButton 
              icon={<RefreshCw size={20} className={loading ? "animate-spin" : ""} />}
              onClick={() => fetchDirectory(path)}
              title="Actualiser"
            />
            <AroIconButton 
              icon={<Plus size={20} />}
              onClick={() => setShowNewFolder(true)}
              title="Nouveau dossier"
              className="text-green border-green/20"
            />
            <AroIconButton 
              icon={<Upload size={20} />}
              onClick={() => setShowUpload(true)}
              title="Uploader un fichier"
              className="text-mauve border-mauve/20"
            />
        </div>
      </div>

      <div className="px-10 grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
        {loading && entries.length === 0 ? (
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
              onClick={() => entry.is_dir ? fetchDirectory(path === '/' ? `/${entry.name}` : `${path}/${entry.name}`) : handleReadFile(entry)}
              className={cn(
                "flex flex-col items-center justify-center gap-4 py-10 cursor-pointer group hover:border-blue/30 relative overflow-hidden",
              )}
            >
              <div 
                className="absolute top-4 right-4 z-20"
                onClick={(e) => { e.stopPropagation(); setMenuOpen(menuOpen === entry.name ? null : entry.name); }}
              >
                <MoreVertical size={16} className="text-subtext0 hover:text-text" />
              </div>

              {/* Simple Context Menu */}
              {menuOpen === entry.name && (
                  <div className="absolute top-10 right-4 bg-crust border border-surface0 rounded-2xl shadow-2xl z-30 py-2 min-w-[140px] animate-in fade-in zoom-in duration-200">
                      <button onClick={(e) => { e.stopPropagation(); setSelectedEntry(entry); setShowShare(true); setMenuOpen(null); }} className="w-full text-left px-4 py-2 hover:bg-surface0 flex items-center gap-2 text-sm font-bold text-text">
                        <Share2 size={14} className="text-mauve" /> Partager
                      </button>
                      <button onClick={(e) => { e.stopPropagation(); handleRename(entry); }} className="w-full text-left px-4 py-2 hover:bg-surface0 flex items-center gap-2 text-sm font-bold text-text">
                        <Edit3 size={14} /> Renommer
                      </button>
                      <button onClick={(e) => { e.stopPropagation(); handleDelete(entry); }} className="w-full text-left px-4 py-2 hover:bg-red/10 flex items-center gap-2 text-sm font-bold text-red">
                        <Trash2 size={14} /> Supprimer
                      </button>
                      {!entry.is_dir && (
                        <button onClick={(e) => { e.stopPropagation(); handleReadFile(entry); }} className="w-full text-left px-4 py-2 hover:bg-blue/10 flex items-center gap-2 text-sm font-bold text-blue">
                            <Eye size={14} /> Lire
                        </button>
                      )}
                  </div>
              )}

              <div className="p-4 bg-surface0/50 rounded-3xl group-hover:scale-110 transition-transform duration-300">
                {getFileIcon(entry.name, entry.is_dir)}
              </div>
              <div className="flex flex-col items-center gap-1 w-full px-5">
                <span className="text-sm font-black text-center truncate w-full tracking-tight group-hover:text-blue transition-colors">
                    {entry.name}
                </span>
                {!entry.is_dir && (
                    <span className="text-[10px] font-mono text-subtext0 opacity-50">
                        {(entry.size / 1024).toFixed(1)} KB
                    </span>
                )}
              </div>
            </SoftCard>
          ))
        )}
      </div>

      {/* New Folder Modal */}
      {showNewFolder && (
          <div className="fixed inset-0 bg-crust/80 backdrop-blur-md z-50 flex items-center justify-center p-6">
              <SoftCard className="max-w-md w-full animate-in zoom-in duration-300 border-green/20">
                  <div className="flex items-center justify-between mb-6">
                      <h3 className="text-2xl font-black tracking-tighter">Nouveau dossier</h3>
                      <button onClick={() => setShowNewFolder(false)} className="text-subtext0"><X /></button>
                  </div>
                  <form onSubmit={handleCreateFolder}>
                      <ModernTextField 
                        label="Nom du dossier"
                        autoFocus
                        value={folderName}
                        onChange={(e) => setFolderName(e.target.value)}
                        placeholder="ex: Photos de vacances"
                      />
                      <div className="mt-8 flex gap-3">
                          <Button variant="outline" className="flex-1" onClick={() => setShowNewFolder(false)}>Annuler</Button>
                          <Button variant="primary" className="flex-1 bg-green text-crust shadow-green/20" type="submit">Créer</Button>
                      </div>
                  </form>
              </SoftCard>
          </div>
      )}

      {/* Upload Modal (Text-based for now via create_file) */}
      {showUpload && (
          <div className="fixed inset-0 bg-crust/80 backdrop-blur-md z-50 flex items-center justify-center p-6">
              <SoftCard className="max-w-xl w-full animate-in zoom-in duration-300 border-mauve/20">
                  <div className="flex items-center justify-between mb-6">
                      <h3 className="text-2xl font-black tracking-tighter">Uploader un fichier texte</h3>
                      <button onClick={() => setShowUpload(false)} className="text-subtext0"><X /></button>
                  </div>
                  <form onSubmit={handleUploadFile} className="space-y-4">
                      <ModernTextField 
                        label="Nom du fichier"
                        autoFocus
                        value={fileName}
                        onChange={(e) => setFileName(e.target.value)}
                        placeholder="ex: notes.txt"
                      />
                      <div className="flex flex-col gap-2">
                        <label className="text-xs font-bold tracking-widest text-subtext0 px-2 uppercase">Contenu</label>
                        <textarea 
                            className="w-full bg-mantle border-2 border-transparent rounded-[24px] p-6 text-text placeholder:text-surface2 focus:border-text transition-all outline-none min-h-[200px] font-mono text-sm"
                            value={fileContent}
                            onChange={(e) => setFileContent(e.target.value)}
                            placeholder="Écrivez le contenu ici..."
                        />
                      </div>
                      <div className="mt-8 flex gap-3">
                          <Button variant="outline" className="flex-1" onClick={() => setShowUpload(false)}>Annuler</Button>
                          <Button variant="primary" className="flex-1 bg-mauve text-crust shadow-mauve/20" type="submit">Uploader</Button>
                      </div>
                  </form>
              </SoftCard>
          </div>
      )}

      {/* Share Modal */}
      {showShare && selectedEntry && (
          <div className="fixed inset-0 bg-crust/80 backdrop-blur-md z-50 flex items-center justify-center p-6">
              <SoftCard className="max-w-md w-full animate-in zoom-in duration-300 border-mauve/20">
                  <div className="flex items-center justify-between mb-6">
                      <h3 className="text-2xl font-black tracking-tighter">Partager {selectedEntry.name}</h3>
                      <button onClick={() => setShowShare(false)} className="text-subtext0"><X /></button>
                  </div>
                  
                  <div className="space-y-6">
                    <ModernTextField 
                        label="Rechercher un utilisateur"
                        placeholder="Nom d'utilisateur..."
                        value={shareUserQuery}
                        onChange={(e) => handleSearchUsers(e.target.value)}
                        prefixIcon={<Search size={18} />}
                    />

                    {userResults.length > 0 && (
                        <div className="bg-mantle border border-surface0 rounded-2xl overflow-hidden max-h-40 overflow-y-auto">
                            {userResults.map((u, idx) => (
                                <button 
                                    key={idx}
                                    onClick={() => { setTargetUser(u.username); setShareUserQuery(u.username); setUserResults([]); }}
                                    className={cn(
                                        "w-full px-4 py-3 text-left hover:bg-surface0 text-sm font-bold transition-colors border-b border-surface0 last:border-0",
                                        targetUser === u.username ? "text-blue bg-blue/5" : "text-text"
                                    )}
                                >
                                    {u.username}
                                </button>
                            ))}
                        </div>
                    )}

                    <div className="flex items-center justify-between px-2">
                        <div>
                            <p className="text-sm font-bold text-text">Autoriser l'écriture</p>
                            <p className="text-[10px] text-subtext0 opacity-70">Permet de modifier ou supprimer</p>
                        </div>
                        <input 
                            type="checkbox" 
                            checked={canWrite}
                            onChange={(e) => setCanWrite(e.target.checked)}
                            className="w-5 h-5 accent-blue cursor-pointer"
                        />
                    </div>

                    <div className="mt-8 flex gap-3">
                        <Button variant="outline" className="flex-1" onClick={() => setShowShare(false)}>Annuler</Button>
                        <Button 
                            variant="primary" 
                            className="flex-1 bg-mauve text-crust shadow-mauve/20" 
                            onClick={handleGrantShare}
                            disabled={!targetUser}
                        >
                            Partager
                        </Button>
                    </div>
                  </div>
              </SoftCard>
          </div>
      )}

      {/* Read File Modal */}
      {showRead && viewingFile && (
          <div className="fixed inset-0 bg-crust/80 backdrop-blur-md z-50 flex items-center justify-center p-6">
              <SoftCard className="max-w-4xl w-full h-[80vh] flex flex-col animate-in slide-in-from-bottom duration-300">
                  <div className="flex items-center justify-between mb-6">
                      <div className="flex items-center gap-3">
                        <FileText className="text-blue" />
                        <h3 className="text-2xl font-black tracking-tighter">{viewingFile.name}</h3>
                      </div>
                      <button onClick={() => setShowRead(false)} className="p-2 hover:bg-surface0 rounded-full transition-colors text-subtext0"><X /></button>
                  </div>
                  <div className="flex-1 bg-crust/50 rounded-[24px] p-8 overflow-auto border border-surface0 shadow-inner">
                      <pre className="font-mono text-sm leading-relaxed whitespace-pre-wrap">{viewingFile.content}</pre>
                  </div>
                  <div className="mt-6 flex justify-end">
                      <Button variant="secondary" onClick={() => setShowRead(false)}>Fermer</Button>
                  </div>
              </SoftCard>
          </div>
      )}
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
