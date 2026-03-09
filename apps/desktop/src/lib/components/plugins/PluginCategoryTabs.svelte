<script lang="ts">
  import { Cable, Cpu, Puzzle } from 'lucide-svelte';
  import type { PluginKind } from '$lib/types/plugin';
  import { getPluginKindLabel } from '$lib/types/plugin';
  import type { ComponentType } from 'svelte';

  export type PluginCategoryTab = {
    key: PluginKind;
    count: number;
  };

  interface Props {
    activeCategory: PluginKind;
    categories: PluginCategoryTab[];
    onCategoryChange: (category: PluginKind) => void;
  }

  let { activeCategory, categories, onCategoryChange }: Props = $props();

  function getCategoryIcon(kind: PluginKind): ComponentType {
    if (kind === 'driver') return Cable;
    if (kind === 'controller') return Cpu;
    return Puzzle;
  }
</script>

<div class="flex flex-wrap items-center gap-1.5 rounded-xl">
  {#each categories as category}
    {@const isActive = activeCategory === category.key}
    {@const Icon = getCategoryIcon(category.key)}
    <button
      onclick={() => onCategoryChange(category.key)}
      class={`
        flex min-w-0 items-center gap-2 px-3.5 py-2.5 rounded-xl text-sm font-medium transition-all duration-200
        ${isActive 
          ? 'bg-white dark:bg-zinc-800 text-slate-800 dark:text-white shadow-sm border border-slate-200 dark:border-white/10' 
          : 'text-slate-500 dark:text-zinc-400 border border-transparent hover:text-slate-700 dark:hover:text-zinc-200 hover:bg-white/70 dark:hover:bg-white/5'
        }
      `}
    >
      <Icon class="w-4 h-4 shrink-0" />
      <span class="truncate">{getPluginKindLabel(category.key)}</span>
      <span class={`
        text-[11px] px-1.5 py-0.5 rounded-full shrink-0
        ${isActive 
          ? 'bg-blue-100 dark:bg-blue-500/20 text-blue-600 dark:text-blue-400' 
          : 'bg-slate-200 dark:bg-zinc-700 text-slate-500 dark:text-zinc-400'
        }
      `}>
        {category.count}
      </span>
    </button>
  {/each}
</div>
