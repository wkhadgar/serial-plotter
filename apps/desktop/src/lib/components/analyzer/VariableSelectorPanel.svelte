<script lang="ts">
  import { ChevronsRight, Check, Plus } from 'lucide-svelte';
  import type { AnalyzerVariable } from '$lib/types/analyzer';
  import type { AnalysisMethod } from '$lib/stores/analyzerStore.svelte';

  let {
    visible = $bindable(false),
    variables,
    selectedAnalysisMethod,
    onToggleVariable,
    onSelectAnalysisMethod,
    onVisibleChange
  }: {
    visible: boolean;
    variables: AnalyzerVariable[];
    selectedAnalysisMethod: AnalysisMethod | null;
    onToggleVariable: (index: number) => void;
    onSelectAnalysisMethod: (method: AnalysisMethod) => void;
    onVisibleChange?: (visible: boolean) => void;
  } = $props();

  const analysisMethods: Array<{ id: AnalysisMethod; label: string; description: string }> = [
    {
      id: 'open_loop',
      label: 'Malha aberta',
      description: 'Visualização para experimentos em malha aberta',
    },
    {
      id: 'closed_loop',
      label: 'Malha fechada',
      description: 'Visualização para experimentos em malha fechada',
    },
  ];

  function handleClose() {
    visible = false;
    onVisibleChange?.(false);
  }
</script>

<div class={`${visible
  ? 'w-full max-h-[58vh] translate-y-0 md:w-80 md:max-h-none md:translate-y-0'
  : 'w-full max-h-0 translate-y-2 md:w-0 md:max-h-none md:translate-y-0 md:translate-x-full'
} bg-white dark:bg-[#0c0c0e] border-t md:border-t-0 md:border-l border-slate-200 dark:border-white/5 flex flex-col overflow-hidden transition-[width,max-height,transform] duration-300 ease-in-out shadow-xl relative z-30 print:hidden`}>
  <div class="h-14 border-b border-slate-100 dark:border-white/5 flex justify-between items-center px-5 bg-slate-50 dark:bg-white/[0.02]">
    <h3 class="font-bold text-slate-700 dark:text-white text-sm">Variáveis</h3>
    <button onclick={handleClose} class="text-slate-400 hover:text-slate-600 dark:hover:text-white" title="Recolher Painel">
      <ChevronsRight size={20} />
    </button>
  </div>

  <div class="flex-1 overflow-y-auto p-4 space-y-3 sm:p-5">
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

  <div class="shrink-0 border-t border-slate-200 bg-slate-50/70 p-4 dark:border-white/10 dark:bg-white/[0.02] sm:p-5">
    <h4 class="text-xs font-semibold uppercase tracking-wide text-slate-500 dark:text-zinc-400">
      Métodos de análise
    </h4>
    <div class="mt-3 space-y-2">
      {#each analysisMethods as method (method.id)}
        <button
          type="button"
          onclick={() => onSelectAnalysisMethod(method.id)}
          class={`w-full rounded-lg border px-3 py-2 text-left transition-all ${
            selectedAnalysisMethod === method.id
              ? 'border-blue-500 bg-blue-50 text-blue-700 dark:border-blue-600 dark:bg-blue-900/20 dark:text-blue-300'
              : 'border-slate-200 bg-white text-slate-600 hover:bg-slate-100 dark:border-white/10 dark:bg-[#121215] dark:text-zinc-300 dark:hover:bg-[#16161a]'
          }`}
          title={method.label}
        >
          <div class="flex items-start justify-between gap-2">
            <div>
              <p class="text-sm font-semibold">{method.label}</p>
              <p class="mt-0.5 text-[11px] leading-4 opacity-80">{method.description}</p>
            </div>
            {#if selectedAnalysisMethod === method.id}
              <div class="mt-0.5 flex h-5 w-5 items-center justify-center rounded-full bg-blue-600 text-white dark:bg-blue-500">
                <Check size={12} />
              </div>
            {/if}
          </div>
        </button>
      {/each}
    </div>

    <button
      type="button"
      disabled
      class="mt-3 flex w-full items-center justify-center gap-2 rounded-lg border border-dashed border-slate-300 px-3 py-2 text-xs font-semibold text-slate-400 opacity-80 dark:border-white/20 dark:text-zinc-500"
      title="Em breve"
    >
      <Plus size={14} />
      Adicionar método (Plugin personalizado)
    </button>
  </div>
</div>
