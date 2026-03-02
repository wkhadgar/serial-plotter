<script lang="ts">
  import { ChevronsRight, Check } from 'lucide-svelte';
  import type { AnalyzerVariable } from '$lib/types/analyzer';

  let {
    visible = $bindable(false),
    variables,
    onToggleVariable,
    onVisibleChange
  }: {
    visible: boolean;
    variables: AnalyzerVariable[];
    onToggleVariable: (index: number) => void;
    onVisibleChange?: (visible: boolean) => void;
  } = $props();

  function handleClose() {
    visible = false;
    onVisibleChange?.(false);
  }
</script>

<div class={`${visible ? 'w-80 translate-x-0' : 'w-0 translate-x-full'} bg-white dark:bg-[#0c0c0e] border-l border-slate-200 dark:border-white/5 flex flex-col transition-all duration-300 ease-in-out shadow-xl relative z-30 print:hidden`}>
  <div class="h-14 border-b border-slate-100 dark:border-white/5 flex justify-between items-center px-5 bg-slate-50 dark:bg-white/[0.02]">
    <h3 class="font-bold text-slate-700 dark:text-white text-sm">Variáveis</h3>
    <button onclick={handleClose} class="text-slate-400 hover:text-slate-600 dark:hover:text-white" title="Recolher Painel">
      <ChevronsRight size={20} />
    </button>
  </div>

  <div class="flex-1 overflow-y-auto p-5 space-y-3 min-w-[320px]">
    {#if variables.length === 0}
      <div class="text-center text-sm text-slate-500 dark:text-zinc-500 py-8">
        Nenhuma variável encontrada
      </div>
    {:else}
      {#each variables as variable (variable.index)}
        <button
          onclick={() => onToggleVariable(variable.index)}
          class={`w-full p-4 rounded-xl border transition-all text-left ${
            variable.selected
              ? 'bg-blue-50 dark:bg-blue-900/20 border-blue-300 dark:border-blue-700 shadow-sm'
              : 'bg-slate-50 dark:bg-[#121215] border-slate-200 dark:border-white/5 hover:bg-slate-100 dark:hover:bg-[#16161a] shadow-sm'
          }`}
        >
          <div class="flex items-center justify-between mb-2">
            <span class="text-sm font-bold text-slate-700 dark:text-zinc-300">
              {variable.sensorName}
            </span>
            {#if variable.selected}
              <div class="w-5 h-5 rounded-full bg-blue-600 dark:bg-blue-500 flex items-center justify-center">
                <Check size={14} class="text-white" />
              </div>
            {:else}
              <div class="w-5 h-5 rounded-full border-2 border-slate-300 dark:border-white/20"></div>
            {/if}
          </div>
          <div class="text-xs text-slate-500 dark:text-zinc-500 space-y-1">
            <div class="flex items-center gap-1">
              <span class="w-2 h-2 rounded-full bg-blue-500"></span>
              <span>Sensor: {variable.sensorName} ({variable.sensorUnit})</span>
            </div>
            <div class="flex items-center gap-1">
              <span class="w-2 h-2 rounded-full bg-amber-500"></span>
              <span>Setpoint: {variable.setpointId}</span>
            </div>
            {#if variable.actuators.length > 0}
              {#each variable.actuators as act}
                <div class="flex items-center gap-1">
                  <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
                  <span>Atuador: {act.name} ({act.unit})</span>
                </div>
              {/each}
            {:else}
              <div class="flex items-center gap-1">
                <span class="w-2 h-2 rounded-full bg-slate-400"></span>
                <span>Sem atuadores vinculados</span>
              </div>
            {/if}
          </div>
        </button>
      {/each}
    {/if}
  </div>
</div>
