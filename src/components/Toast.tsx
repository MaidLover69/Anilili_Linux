import React from 'react';
import { X, CheckCircle, AlertTriangle, AlertCircle, Info } from 'lucide-react';
import { useUIStore } from '../stores/uiStore';

export const ToastContainer: React.FC = () => {
  const { toasts, removeToast } = useUIStore();

  if (toasts.length === 0) return null;

  return (
    <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-[9999] flex flex-col gap-2.5 max-w-sm w-full items-center pointer-events-none">
      {toasts.map((toast) => {
        const iconMap = {
          success: <CheckCircle className="w-5 h-5 text-state-success flex-shrink-0" />,
          warning: <AlertTriangle className="w-5 h-5 text-state-warning flex-shrink-0" />,
          error: <AlertCircle className="w-5 h-5 text-state-error flex-shrink-0" />,
          info: <Info className="w-5 h-5 text-accent-300 flex-shrink-0" />,
        };

        return (
          <div
            key={toast.id}
            className="pointer-events-auto flex items-start gap-3 p-3.5 rounded-xl bg-bg-card/95 border border-border shadow-2xl backdrop-blur-xl animate-slide-up"
          >
            {iconMap[toast.type]}
            <div className="flex flex-col flex-1 min-w-0">
              <span className="text-xs font-bold text-text-primary">
                {toast.title}
              </span>
              {toast.message && (
                <p className="text-[11px] text-text-muted mt-0.5 leading-snug">
                  {toast.message}
                </p>
              )}
            </div>
            <button
              onClick={() => removeToast(toast.id)}
              className="p-1 rounded-md text-text-muted hover:text-white hover:bg-bg-elevated transition-colors"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          </div>
        );
      })}
    </div>
  );
};
