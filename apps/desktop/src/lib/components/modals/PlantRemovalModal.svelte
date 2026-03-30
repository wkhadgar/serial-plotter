<script lang="ts">
  import { AlertTriangle, X } from 'lucide-svelte';

  let {
    visible = $bindable(false),
    plantName,
    reason,
    onConfirm,
    onCancel
  }: {
    visible: boolean;
    plantName: string;
    reason: 'confirm' | 'min-units' | '';
    onConfirm: () => void;
    onCancel: () => void;
  } = $props();
</script>

{#if visible}
  <div
    class="fixed inset-0 bg-black/60 flex items-center justify-center z-[100] print:hidden"
    onclick={onCancel}
    onkeydown={(e: KeyboardEvent) => e.key === 'Escape' && onCancel()}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="bg-white dark:bg-[#18181b] rounded-xl shadow-2xl border border-slate-200 dark:border-white/10 p-6 max-w-md w-full mx-4"
      onclick={(e: MouseEvent) => e.stopPropagation()}
      onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && e.stopPropagation()}
      role="document"
    >
      <div class="flex items-start gap-4 mb-4">
        <div class={`p-3 rounded-lg ${reason === 'min-units' ? 'bg-amber-100 dark:bg-amber-900/20' : 'bg-slate-100 dark:bg-slate-800/60'}`}>
          {#if reason === 'min-units'}
            <AlertTriangle size={24} class="text-amber-600 dark:text-amber-400" />
          {:else}
            <X size={24} class="text-slate-600 dark:text-slate-300" />
          {/if}
        </div>
        <div class="flex-1">
          <h3 class="text-lg font-bold text-slate-900 dark:text-white mb-1">
            {reason === 'min-units' ? 'Ação não permitida' : 'Fechar planta'}
          </h3>
          <p class="text-sm text-slate-600 dark:text-slate-300">
            {#if reason === 'min-units'}
              É necessário manter ao menos uma unidade ativa no sistema.
            {:else}
              Deseja fechar a planta <strong class="font-semibold text-slate-900 dark:text-white">{plantName}</strong>? A runtime será encerrada, a planta sairá da sessão atual e continuará salva nos arquivos.
            {/if}
          </p>
        </div>
      </div>
      <div class="flex gap-3 justify-end">
        {#if reason === 'min-units'}
          <button
            onclick={onCancel}
            class="px-4 py-2 rounded-lg text-sm font-medium bg-blue-600 hover:bg-blue-700 text-white transition-colors"
          >
            Entendi
          </button>
        {:else}
          <button
            onclick={onCancel}
            class="px-4 py-2 rounded-lg text-sm font-medium bg-slate-100 hover:bg-slate-200 dark:bg-white/5 dark:hover:bg-white/10 text-slate-700 dark:text-slate-300 transition-colors"
          >
            Cancelar
          </button>
          <button
            onclick={onConfirm}
            class="px-4 py-2 rounded-lg text-sm font-medium bg-slate-900 hover:bg-slate-800 dark:bg-white dark:text-slate-900 dark:hover:bg-slate-100 text-white transition-colors flex items-center gap-2"
          >
            <X size={14} />
            Fechar aba
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}
