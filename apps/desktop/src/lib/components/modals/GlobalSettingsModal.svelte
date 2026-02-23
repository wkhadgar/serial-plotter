<script lang="ts">
  import { appStore } from '$lib/stores/data.svelte';
  import SimpleToggle from '../ui/SimpleToggle.svelte';
  import { X } from 'lucide-svelte';

  let { showGlobalSettings } = $props();

  let gridlinesEnabled = $state(true);
  let lockSetpoints = $state(false);

  function closeSettings() {
    appStore.setShowGlobalSettings(false);
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') closeSettings();
  }
</script>

{#if showGlobalSettings}
  <div
    role="dialog"
    aria-modal="true"
    aria-labelledby="settings-title"
    onkeydown={handleKeyDown}
    tabindex="0"
    class="fixed inset-0 z-50 bg-black/40 backdrop-blur-sm flex items-center justify-center p-4 print:hidden"
    onclick={(e: MouseEvent) => e.target === e.currentTarget && closeSettings()}
  >
    <div class="bg-white dark:bg-zinc-900 w-full max-w-lg rounded-xl shadow-2xl border border-slate-200 dark:border-white/10 overflow-hidden">
      <div class="p-5 border-b border-slate-200 dark:border-white/5 flex justify-between items-center">
        <h2 id="settings-title" class="text-lg font-bold text-slate-800 dark:text-white">Preferências</h2>
        <button
          onclick={closeSettings}
          aria-label="Fechar"
          class="text-slate-500 hover:text-slate-700 dark:hover:text-slate-300 focus:outline-none focus:ring-2 focus:ring-blue-500 rounded transition-colors"
        >
          <X size={20} />
        </button>
      </div>
      <div class="p-6 space-y-6">
        <div class="flex items-start justify-between">
          <div>
            <div class="text-sm font-medium text-slate-900 dark:text-white">Mostrar Gridlines</div>
            <div class="text-xs text-slate-500 dark:text-slate-400">Linhas de referência nos gráficos</div>
          </div>
          <SimpleToggle bind:checked={gridlinesEnabled} ariaLabel="Toggle gridlines" />
        </div>
        <div class="flex items-start justify-between">
          <div>
            <div class="text-sm font-medium text-slate-900 dark:text-white">Bloquear Setpoints</div>
            <div class="text-xs text-slate-500 dark:text-slate-400">Impede alterações manuais</div>
          </div>
          <SimpleToggle bind:checked={lockSetpoints} ariaLabel="Toggle setpoint lock" />
        </div>
      </div>
      <div class="p-4 bg-slate-50 dark:bg-black/20 text-right border-t border-slate-200 dark:border-white/5">
        <button
          onclick={closeSettings}
          class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 dark:focus:ring-blue-400"
        >
          Concluído
        </button>
      </div>
    </div>
  </div>
{/if}


