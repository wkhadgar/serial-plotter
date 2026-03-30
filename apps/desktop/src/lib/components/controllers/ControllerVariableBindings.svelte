<script lang="ts">
  import type { PlantVariable } from '$lib/types/plant';

  interface Props {
    label: string;
    helper: string;
    variables: PlantVariable[];
    selectedIds: string[];
    emptyLabel: string;
    tone: 'sensor' | 'atuador';
    onChange: (ids: string[]) => void;
  }

  let {
    label,
    helper,
    variables,
    selectedIds,
    emptyLabel,
    tone,
    onChange,
  }: Props = $props();

  function toggleVariable(variableId: string) {
    const nextIds = selectedIds.includes(variableId)
      ? selectedIds.filter((id) => id !== variableId)
      : [...selectedIds, variableId];

    onChange(nextIds);
  }
</script>

<div class="space-y-2">
  <div>
    <div class="text-[10px] font-bold uppercase tracking-wide text-slate-500 dark:text-zinc-400">{label}</div>
    <div class="mt-1 text-[11px] text-slate-500 dark:text-zinc-400">{helper}</div>
  </div>

  {#if variables.length === 0}
    <div class="rounded-lg border border-dashed border-slate-200 bg-slate-50 px-3 py-2 text-xs text-slate-500 dark:border-white/10 dark:bg-white/[0.03] dark:text-zinc-400">
      {emptyLabel}
    </div>
  {:else}
    <div class="flex flex-wrap gap-2">
      {#each variables as variable (variable.id)}
        {@const selected = selectedIds.includes(variable.id)}
        <button
          type="button"
          onclick={() => toggleVariable(variable.id)}
          class={`rounded-lg border px-3 py-1.5 text-xs font-medium transition-colors ${
            selected
              ? tone === 'sensor'
                ? 'border-cyan-300 bg-cyan-100 text-cyan-700 dark:border-cyan-700 dark:bg-cyan-900/30 dark:text-cyan-300'
                : 'border-orange-300 bg-orange-100 text-orange-700 dark:border-orange-700 dark:bg-orange-900/30 dark:text-orange-300'
              : 'border-slate-200 bg-white text-slate-600 hover:border-slate-300 dark:border-white/10 dark:bg-[#18181b] dark:text-zinc-300 dark:hover:border-white/20'
          }`}
        >
          {variable.name}
        </button>
      {/each}
    </div>
  {/if}
</div>
