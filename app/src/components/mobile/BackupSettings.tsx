import { useState, useEffect } from 'react';
import { Camera, CheckCircle2, ChevronDown, Wifi, Battery, Zap } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface BackupPreferences {
  enabled: boolean;
  target_chat_id: number | null;
  target_folder_id: number | null;
  network_type: string;
  sources_photos: boolean;
  sources_videos: boolean;
  sources_screenshots: boolean;
  sources_downloads: boolean;
  sources_whatsapp: boolean;
}

export function BackupSettings({ folders, activeFolderId, activeChatId }: { folders: any[], activeFolderId: number | null, activeChatId: number | null }) {
  const [prefs, setPrefs] = useState<BackupPreferences | null>(null);

  useEffect(() => {
    invoke<BackupPreferences>('cmd_get_backup_preferences')
      .then(setPrefs)
      .catch(console.error);
  }, []);

  const savePrefs = (newPrefs: BackupPreferences) => {
    setPrefs(newPrefs);
    invoke('cmd_set_backup_preferences', { prefs: newPrefs }).catch(console.error);
  };

  const handleSourceToggle = (key: keyof BackupPreferences) => {
    if (prefs) savePrefs({ ...prefs, [key]: !prefs[key] });
  };

  if (!prefs) return null;

  return (
    <div className="p-4 rounded-2xl bg-telegram-hover/20 border border-telegram-border/30 space-y-4">
      <h3 className="text-sm font-bold text-telegram-primary tracking-wide uppercase text-[10px] flex items-center gap-1.5">
        <Camera className="w-3 h-3" />
        Auto Backup
      </h3>

      <div className="flex items-center justify-between py-2 border-b border-telegram-border/20">
        <div>
          <p className="text-xs font-medium">Enable Auto Backup</p>
          <p className="text-[10px] text-telegram-subtext">Automatically upload media to Telegram Drive</p>
        </div>
        <button
          onClick={() => savePrefs({ ...prefs, enabled: !prefs.enabled, target_folder_id: prefs.target_folder_id || activeFolderId, target_chat_id: prefs.target_chat_id || activeChatId })}
          className={`relative w-11 h-6 rounded-full transition-colors duration-200 flex-shrink-0 ${prefs.enabled ? 'bg-telegram-primary' : 'bg-telegram-border'}`}
        >
          <span className={`absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white shadow transition-transform duration-200 ${prefs.enabled ? 'translate-x-5' : 'translate-x-0'}`} />
        </button>
      </div>

      {prefs.enabled && (
        <>
          <div className="flex items-center justify-between py-2 border-b border-telegram-border/20">
            <div>
              <p className="text-xs font-medium">Backup Destination</p>
              <p className="text-[10px] text-telegram-subtext">Folder to save backups</p>
            </div>
            <div className="relative">
              <select
                value={prefs.target_folder_id || ''}
                onChange={e => savePrefs({ ...prefs, target_folder_id: e.target.value ? Number(e.target.value) : null, target_chat_id: activeChatId })}
                className="appearance-none bg-telegram-bg border border-telegram-border rounded-lg pl-2.5 pr-7 py-1.5 text-xs text-telegram-text focus:outline-none focus:border-telegram-primary/50 transition cursor-pointer max-w-[120px] truncate"
              >
                <option value="">Saved Messages</option>
                {folders.map(f => (
                  <option key={f.id} value={f.id}>{f.name}</option>
                ))}
              </select>
              <ChevronDown className="w-3.5 h-3.5 text-telegram-subtext absolute right-2 top-1/2 -translate-y-1/2 pointer-events-none" />
            </div>
          </div>

          <div className="py-2 border-b border-telegram-border/20 space-y-2">
            <p className="text-xs font-medium mb-1">Backup Sources</p>
            {[
              { key: 'sources_photos', label: 'Photos' },
              { key: 'sources_videos', label: 'Videos' },
              { key: 'sources_screenshots', label: 'Screenshots' },
              { key: 'sources_downloads', label: 'Downloads' },
              { key: 'sources_whatsapp', label: 'WhatsApp Images' },
            ].map(({ key, label }) => {
              const isEnabled = prefs[key as keyof BackupPreferences] as boolean;
              return (
                <label key={key} onClick={() => handleSourceToggle(key as keyof BackupPreferences)} className="flex items-center gap-3 cursor-pointer group">
                  <div className={`w-4 h-4 rounded-[4px] flex items-center justify-center transition-colors ${isEnabled ? 'bg-telegram-primary border-telegram-primary' : 'border border-telegram-border group-hover:border-telegram-primary/50'}`}>
                    {isEnabled && <CheckCircle2 className="w-3 h-3 text-black" />}
                  </div>
                  <span className="text-xs text-telegram-text capitalize">{label}</span>
                </label>
              );
            })}
          </div>

          <div className="py-2 border-b border-telegram-border/20 space-y-2">
            <p className="text-xs font-medium mb-1">Upload Constraints</p>
            <div className="flex flex-col gap-2">
              <label className="flex items-center gap-2 text-xs text-telegram-text cursor-pointer">
                <input type="radio" checked={prefs.network_type === 'wifi'} onChange={() => savePrefs({ ...prefs, network_type: 'wifi' })} className="accent-telegram-primary" />
                <Wifi className="w-3.5 h-3.5" /> Wi-Fi only
              </label>
              <label className="flex items-center gap-2 text-xs text-telegram-text cursor-pointer">
                <input type="radio" checked={prefs.network_type === 'cellular'} onChange={() => savePrefs({ ...prefs, network_type: 'cellular' })} className="accent-telegram-primary" />
                <Zap className="w-3.5 h-3.5" /> Wi-Fi + Cellular
              </label>
              <label className="flex items-center gap-2 text-xs text-telegram-text cursor-pointer">
                <input type="radio" checked={prefs.network_type === 'charging'} onChange={() => savePrefs({ ...prefs, network_type: 'charging' })} className="accent-telegram-primary" />
                <Battery className="w-3.5 h-3.5" /> Only while charging
              </label>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
