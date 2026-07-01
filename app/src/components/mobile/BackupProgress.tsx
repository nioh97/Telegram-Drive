import { useEffect, useState } from 'react';
import { CloudUpload, RefreshCw, CheckCircle } from 'lucide-react';

export function BackupProgress() {
  const [status, setStatus] = useState<'idle' | 'scanning' | 'uploading' | 'synced'>('scanning');
  const [progress, setProgress] = useState(0);

  // Mocking real-time updates for now
  useEffect(() => {
    let timeout: ReturnType<typeof setTimeout>;
    if (status === 'scanning') {
      timeout = setTimeout(() => {
        setStatus('uploading');
        setProgress(0);
      }, 3000);
    } else if (status === 'uploading') {
      if (progress < 100) {
        timeout = setTimeout(() => {
          setProgress(p => Math.min(100, p + 5));
        }, 500);
      } else {
        timeout = setTimeout(() => {
          setStatus('synced');
        }, 1000);
      }
    }
    return () => clearTimeout(timeout);
  }, [status, progress]);

  return (
    <div className="p-4 rounded-2xl bg-telegram-hover/20 border border-telegram-border/30 space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-bold text-telegram-primary tracking-wide uppercase text-[10px] flex items-center gap-1.5">
          <CloudUpload className="w-3 h-3" />
          Backup Status
        </h3>
        {status === 'synced' && (
          <span className="text-[10px] font-medium text-emerald-400 flex items-center gap-1">
            <CheckCircle className="w-3 h-3" /> Up to date
          </span>
        )}
      </div>

      <div className="space-y-1">
        {status === 'scanning' && (
          <div className="flex items-center gap-2">
            <RefreshCw className="w-3.5 h-3.5 text-telegram-primary animate-spin" />
            <p className="text-xs font-medium text-telegram-text">Scanning MediaStore...</p>
          </div>
        )}
        
        {status === 'uploading' && (
          <div className="space-y-2">
            <div className="flex justify-between text-xs font-medium text-telegram-text">
              <span>Uploading 45 / 120</span>
              <span>{progress}%</span>
            </div>
            <div className="w-full h-1.5 bg-telegram-border/30 rounded-full overflow-hidden">
              <div 
                className="h-full bg-telegram-primary rounded-full transition-all duration-300"
                style={{ width: `${progress}%` }}
              />
            </div>
          </div>
        )}

        {status === 'synced' && (
          <p className="text-xs font-medium text-telegram-text">
            All selected media is backed up to Telegram Drive.
          </p>
        )}

        <div className="pt-2 flex justify-between text-[10px] text-telegram-subtext">
          <span>Last backup: Today 3:45 PM</span>
          <span>Next scan in 15 mins</span>
        </div>
      </div>
    </div>
  );
}
