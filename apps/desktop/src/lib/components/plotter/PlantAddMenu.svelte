<script lang="ts">
  import { FolderOpen, FilePlus } from 'lucide-svelte';

  interface Props {
    visible: boolean;
    x: number;
    y: number;
    onClose: () => void;
    onOpenFile: () => void;
    onCreateNew: () => void;
  }

  let {
    visible,
    x,
    y,
    onClose,
    onOpenFile,
    onCreateNew,
  }: Props = $props();

  function handleOpenFile() {
    onOpenFile();
    onClose();
  }

  function handleCreateNew() {
    onCreateNew();
    onClose();
  }
</script>

{#if visible}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="fixed inset-0 z-50" onclick={onClose}>
    <div
      class="absolute bg-white dark:bg-[#18181b] border border-slate-200 dark:border-white/10 rounded-xl shadow-2xl py-2 min-w-[200px] overflow-hidden"
      style="left: {x}px; top: {y}px;"
      onclick={(e) => e.stopPropagation()}
    >
      <button
        onclick={handleOpenFile}
        class="w-full flex items-center gap-3 px-4 py-2.5 text-sm text-slate-700 dark:text-zinc-200 hover:bg-slate-100 dark:hover:bg-white/5 transition-colors text-left"
      >
        <FolderOpen size={18} class="text-blue-500" />
        <div>
          <div class="font-medium">Abrir Planta</div>
          <div class="text-xs text-slate-400 dark:text-zinc-500">Carregar de arquivo</div>
        </div>
      </button>
      
      <div class="mx-3 my-1 border-t border-slate-100 dark:border-white/5"></div>
      
      <button
        onclick={handleCreateNew}
        class="w-full flex items-center gap-3 px-4 py-2.5 text-sm text-slate-700 dark:text-zinc-200 hover:bg-slate-100 dark:hover:bg-white/5 transition-colors text-left"
      >
        <FilePlus size={18} class="text-emerald-500" />
        <div>
          <div class="font-medium">Criar Nova Planta</div>
          <div class="text-xs text-slate-400 dark:text-zinc-500">Configurar do zero</div>
        </div>
      </button>
    </div>
  </div>
{/if}
