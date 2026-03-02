<script lang="ts">
  import { Plus, X } from 'lucide-svelte';

  interface TabItem {
    id: string;
    name: string;
  }

  let {
    tabs = [],
    activeTabId,
    onSelect,
    onAdd,
    onRemove,
  }: {
    tabs: TabItem[];
    activeTabId: string;
    onSelect: (id: string) => void;
    onAdd: () => void;
    onRemove: (id: string) => void;
  } = $props();
</script>

<header class="h-10 bg-white dark:bg-[#0c0c0e] border-b border-slate-200 dark:border-white/5 flex items-end px-4 select-none z-10 print:hidden">
  {#each tabs as tab (tab.id)}
    <div class="group relative flex items-center h-8 min-w-[140px] max-w-[200px]">
      <button
        onclick={() => onSelect(tab.id)}
        class={`w-full h-full pl-3 pr-8 rounded-t-lg text-xs font-medium cursor-pointer transition-all border-t border-x flex items-center gap-2
          ${
            activeTabId === tab.id
              ? 'bg-slate-50 dark:bg-[#18181b] border-slate-300 dark:border-white/10 text-blue-600 dark:text-blue-400 border-b-slate-50 dark:border-b-[#18181b] mb-[-1px]'
              : 'bg-transparent border-transparent text-slate-500 hover:bg-slate-100 dark:hover:bg-white/5 mb-0'
          }`}
      >
        <span class="truncate">{tab.name}</span>
      </button>
      <button
        onclick={(e: MouseEvent) => { e.stopPropagation(); onRemove(tab.id); }}
        class="absolute right-1 top-1/2 -translate-y-1/2 p-1 rounded opacity-0 group-hover:opacity-100 hover:bg-red-100 dark:hover:bg-red-900/30 hover:text-red-600 transition-all"
      >
        <X size={12} strokeWidth={2.5} />
      </button>
    </div>
  {/each}
  <button onclick={onAdd} class="h-7 w-7 mb-0.5 flex items-center justify-center rounded-lg hover:bg-slate-100 dark:hover:bg-white/5 text-slate-500 transition-colors">
    <Plus size={16} />
  </button>
</header>

