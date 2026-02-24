<script lang="ts">
  import { ChevronsRight, Trash2 } from 'lucide-svelte';
  import SimpleToggle from '../ui/SimpleToggle.svelte';
  import DynamicParamInput from '../ui/DynamicParamInput.svelte';
  import type { Plant } from '$lib/types/plant';
  import type { ControllerParam } from '$lib/types/controller';

  let {
    visible = $bindable(false),
    plant,
    onAddController,
    onDeleteController,
    onUpdateControllerMeta,
    onUpdateControllerParam,
    onUpdateSetpoint
  }: {
    visible: boolean;
    plant: Plant | undefined;
    onAddController: () => void;
    onDeleteController: (id: string) => void;
    onUpdateControllerMeta: (id: string, field: string, value: any) => void;
    onUpdateControllerParam: (id: string, paramKey: string, value: any) => void;
    onUpdateSetpoint: (value: number) => void;
  } = $props();
</script>

<div class={`${visible ? 'w-80 translate-x-0' : 'w-0 translate-x-full'} bg-white dark:bg-[#0c0c0e] border-l border-slate-200 dark:border-white/5 flex flex-col transition-all duration-300 ease-in-out shadow-xl relative z-30 print:hidden`}>
  <div class="h-14 border-b border-slate-100 dark:border-white/5 flex justify-between items-center px-5 bg-slate-50 dark:bg-white/[0.02]">
    <h3 class="font-bold text-slate-700 dark:text-white text-sm">Malhas de Controle</h3>
    <button onclick={() => visible = false} class="text-slate-400 hover:text-slate-600 dark:hover:text-white" title="Recolher Painel">
      <ChevronsRight size={20} />
    </button>
  </div>
  <div class="flex-1 overflow-y-auto p-5 space-y-6 min-w-[320px]">
    {#if plant}
      <div class="bg-slate-50 dark:bg-[#121215] rounded-xl p-4 border border-slate-200 dark:border-white/5 shadow-sm">
        <div class="flex justify-between items-end mb-2">
          <span class="text-xs font-bold text-slate-500 uppercase tracking-wide">Setpoint</span>
          <span class="text-xl font-mono font-bold text-blue-600 dark:text-blue-400">{plant.setpoint.toFixed(1)}%</span>
        </div>
        <input
          type="range"
          min="0"
          max="100"
          step="0.5"
          value={plant.setpoint}
          onchange={(e: Event) => onUpdateSetpoint(Number((e.target as HTMLInputElement).value))}
          class="w-full h-1.5 bg-slate-300 dark:bg-zinc-700 rounded-lg appearance-none cursor-pointer accent-blue-600"
        />
      </div>
      
      <div class="border-t border-slate-100 dark:border-white/5"></div>
      
      <div>
        <div class="flex justify-between items-center mb-4">
          <span class="text-xs font-bold text-slate-500 uppercase">Controladores</span>
          <button onclick={onAddController} class="text-xs font-medium bg-blue-50 text-blue-600 hover:bg-blue-100 dark:bg-blue-900/20 dark:text-blue-400 dark:hover:bg-blue-900/30 px-3 py-1.5 rounded-full transition-colors">
            + Adicionar
          </button>
        </div>
        <div class="space-y-4">
          {#each plant.controllers as ctrl (ctrl.id)}
            <div class="border border-slate-200 dark:border-white/10 rounded-xl overflow-hidden shadow-sm bg-white dark:bg-[#0c0c0e]">
              <div class="bg-slate-50 dark:bg-white/[0.02] p-3 border-b border-slate-100 dark:border-white/5 flex items-center justify-between">
                <div class="flex items-center gap-3">
                  <SimpleToggle checked={ctrl.active} ariaLabel="Toggle controller" onchange={() => onUpdateControllerMeta(ctrl.id, 'active', !ctrl.active)} />
                  <input
                    value={ctrl.name}
                    oninput={(e: Event) => onUpdateControllerMeta(ctrl.id, 'name', (e.target as HTMLInputElement).value)}
                    class="bg-transparent text-sm font-semibold text-slate-700 dark:text-zinc-200 w-32 focus:text-blue-600 dark:focus:text-blue-400 transition-colors"
                    style="border: none; outline: none; box-shadow: none;"
                  />
                </div>
                <button onclick={() => onDeleteController(ctrl.id)} class="text-slate-400 hover:text-red-500 p-1">
                  <Trash2 size={14} />
                </button>
              </div>
              <div class={`p-4 space-y-3 ${ctrl.active ? '' : 'opacity-40 pointer-events-none'}`}>
                {#each Object.entries(ctrl.params) as [key, param]}
                  <DynamicParamInput
                    label={(param as ControllerParam).label || key}
                    type={(param as ControllerParam).type}
                    value={(param as ControllerParam).value}
                    onChange={(newValue: any) => onUpdateControllerParam(ctrl.id, key, newValue)}
                  />
                {/each}
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>
