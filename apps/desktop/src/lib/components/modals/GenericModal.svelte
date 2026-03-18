<script lang="ts">
  import { AlertCircle, CheckCircle, Info, XCircle } from 'lucide-svelte';

  let {
    visible,
    type = 'info',
    title,
    message,
    confirmLabel = 'OK',
    onConfirm,
    onClose
  }: {
    visible: boolean;
    type?: 'info' | 'error' | 'warning' | 'success';
    title: string;
    message: string;
    confirmLabel?: string;
    onConfirm?: () => void;
    onClose?: () => void;
  } = $props();

  const colors = {
    info: { bg: 'bg-blue-50 dark:bg-blue-900/10', icon: 'text-blue-600 dark:text-blue-400', button: 'bg-blue-600 hover:bg-blue-700' },
    error: { bg: 'bg-red-50 dark:bg-red-900/10', icon: 'text-red-600 dark:text-red-400', button: 'bg-red-600 hover:bg-red-700' },
    warning: { bg: 'bg-amber-50 dark:bg-amber-900/10', icon: 'text-amber-600 dark:text-amber-400', button: 'bg-amber-600 hover:bg-amber-700' },
    success: { bg: 'bg-emerald-50 dark:bg-emerald-900/10', icon: 'text-emerald-600 dark:text-emerald-400', button: 'bg-emerald-600 hover:bg-emerald-700' }
  };

  const icons = {
    info: Info,
    error: XCircle,
    warning: AlertCircle,
    success: CheckCircle
  };

  const Icon = $derived(icons[type]);
  const color = $derived(colors[type]);

  function handleConfirm() {
    if (onConfirm) onConfirm();
    if (onClose) onClose();
  }

  function handleClose() {
    if (onClose) onClose();
  }
</script>

{#if visible}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 animate-in fade-in duration-200">
    <div class="bg-white dark:bg-zinc-900 rounded-2xl shadow-2xl w-full max-w-lg mx-4 animate-in zoom-in-95 duration-200">
      <div class={`${color.bg} p-6 rounded-t-2xl border-b border-slate-200 dark:border-white/5`}>
        <div class="flex items-start gap-4">
          <div class={`${color.icon} flex-shrink-0`}>
            <Icon size={28} />
          </div>
          <div class="flex-1">
            <h3 class="text-lg font-bold text-slate-800 dark:text-white mb-1">{title}</h3>
            <p class="max-h-[45vh] overflow-y-auto pr-1 text-sm text-slate-600 dark:text-zinc-300 whitespace-pre-line break-words">{message}</p>
          </div>
        </div>
      </div>
      <div class="p-6 flex justify-end gap-3">
        {#if onClose}
          <button
            onclick={handleClose}
            class="px-4 py-2 rounded-lg border border-slate-300 dark:border-white/10 bg-white dark:bg-zinc-800 text-slate-700 dark:text-zinc-300 hover:bg-slate-50 dark:hover:bg-zinc-700 transition-colors font-medium text-sm"
          >
            Cancelar
          </button>
        {/if}
        <button
          onclick={handleConfirm}
          class={`${color.button} text-white px-6 py-2 rounded-lg transition-colors font-medium text-sm`}
        >
          {confirmLabel}
        </button>
      </div>
    </div>
  </div>
{/if}
