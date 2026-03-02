<script lang="ts">
  import { Plus, X } from 'lucide-svelte';
  import type { Plant } from '$lib/types/plant';
  import PlantAddMenu from './PlantAddMenu.svelte';

  let { 
    plants,
    activePlantId,
    onSelect,
    onOpenFile,
    onCreateNew,
    onRemove
  }: {
    plants: Plant[];
    activePlantId: string;
    onSelect: (id: string) => void;
    onOpenFile: () => void;
    onCreateNew: () => void;
    onRemove: (id: string) => void;
  } = $props();

  // Menu state
  let menuVisible = $state(false);
  let menuX = $state(0);
  let menuY = $state(0);
  let addButtonRef: HTMLButtonElement;

  function handleAddClick(e: MouseEvent) {
    const rect = addButtonRef.getBoundingClientRect();
    menuX = rect.left;
    menuY = rect.bottom + 4;
    menuVisible = true;
  }
</script>

<header class="h-10 bg-white dark:bg-[#0c0c0e] border-b border-slate-200 dark:border-white/5 flex items-end px-4 select-none z-10 print:hidden">
  {#each plants as plant (plant.id)}
    <div class="group relative flex items-center h-8 min-w-[140px] max-w-[200px]">
      <button
        onclick={() => onSelect(plant.id)}
        class={`w-full h-full pl-3 pr-8 rounded-t-lg text-xs font-medium cursor-pointer transition-all border-t border-x flex items-center gap-2
          ${activePlantId === plant.id
            ? 'bg-slate-50 dark:bg-[#18181b] border-slate-300 dark:border-white/10 text-blue-600 dark:text-blue-400 border-b-slate-50 dark:border-b-[#18181b] mb-[-1px]'
            : 'bg-transparent border-transparent text-slate-500 hover:bg-slate-100 dark:hover:bg-white/5 mb-0'}`}
      >
        <div class={`w-1.5 h-1.5 rounded-full ${plant.connected ? 'bg-emerald-500' : 'bg-slate-300 dark:bg-zinc-700'}`}></div>
        <span class="truncate">{plant.name}</span>
      </button>
      <button
        onclick={(e: MouseEvent) => { e.stopPropagation(); onRemove(plant.id); }}
        class="absolute right-1 top-1/2 -translate-y-1/2 p-1 rounded opacity-0 group-hover:opacity-100 hover:bg-red-100 dark:hover:bg-red-900/30 hover:text-red-600 transition-all"
      >
        <X size={12} strokeWidth={2.5} />
      </button>
    </div>
  {/each}
  <button 
    bind:this={addButtonRef}
    onclick={handleAddClick} 
    class="h-7 w-7 mb-0.5 flex items-center justify-center rounded-lg hover:bg-slate-100 dark:hover:bg-white/5 text-slate-500 transition-colors"
  >
    <Plus size={16} />
  </button>
</header>

<PlantAddMenu
  visible={menuVisible}
  x={menuX}
  y={menuY}
  onClose={() => menuVisible = false}
  {onOpenFile}
  {onCreateNew}
/>
