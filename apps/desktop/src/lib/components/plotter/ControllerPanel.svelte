<script lang="ts">
  import { ChevronsRight, LoaderCircle, Pencil, Save, Trash2, ChevronDown, ChevronUp } from 'lucide-svelte';
  import SimpleToggle from '../ui/SimpleToggle.svelte';
  import DynamicParamInput from '../ui/DynamicParamInput.svelte';
  import type { Plant } from '$lib/types/plant';
  import type { Controller, ControllerParam } from '$lib/types/controller';

  type SaveControllerResult = {
    success: boolean;
    deferred?: boolean;
    error?: string;
  };

  let {
    visible = $bindable(false),
    plant,
    onAddController,
    onDeleteController,
    onEditControllerBindings,
    onSaveControllerConfig,
    onToggleControllerActive,
    onUpdateControllerMeta,
    onUpdateControllerParam,
    onUpdateSetpoint
  }: {
    visible: boolean;
    plant: Plant | undefined;
    onAddController: () => void;
    onDeleteController: (id: string) => void;
    onEditControllerBindings: (id: string) => void;
    onSaveControllerConfig: (id: string) => Promise<SaveControllerResult>;
    onToggleControllerActive: (id: string, nextActive: boolean) => void;
    onUpdateControllerMeta: (id: string, field: string, value: any) => void;
    onUpdateControllerParam: (id: string, paramKey: string, value: any) => void;
    onUpdateSetpoint: (varIndex: number, value: number) => void | Promise<void>;
  } = $props();

  let setpointsExpanded = $state(true);
  let savedSnapshots = $state<Record<string, string>>({});
  let invalidFields = $state<Record<string, boolean>>({});
  let savingControllers = $state<Record<string, boolean>>({});
  let saveFeedback = $state<Record<string, { tone: 'success' | 'warning' | 'error'; message: string }>>({});

  const sensorVariables = $derived(
    plant?.variables
      .map((v, idx) => ({ variable: v, index: idx }))
      .filter(({ variable }) => variable.type === 'sensor') ?? []
  );
  function getControllerKey(controllerId: string): string {
    return `${plant?.id ?? 'no-plant'}:${controllerId}`;
  }

  function serializeController(controller: Controller): string {
    return JSON.stringify({
      name: controller.name,
      active: controller.active,
      inputVariableIds: [...(controller.inputVariableIds ?? [])].sort(),
      outputVariableIds: [...(controller.outputVariableIds ?? [])].sort(),
      params: Object.entries(controller.params ?? {})
        .sort(([left], [right]) => left.localeCompare(right))
        .map(([key, param]) => [key, { type: param.type, label: param.label, value: param.value }]),
    });
  }

  function isControllerDirty(controller: Controller): boolean {
    const controllerKey = getControllerKey(controller.id);
    return savedSnapshots[controllerKey] !== undefined && savedSnapshots[controllerKey] !== serializeController(controller);
  }

  function controllerHasInvalidFields(controllerId: string): boolean {
    const prefix = `${getControllerKey(controllerId)}:`;
    return Object.entries(invalidFields).some(([key, isInvalid]) => key.startsWith(prefix) && isInvalid);
  }

  function updateParamValidity(controllerId: string, paramKey: string, isValid: boolean) {
    const stateKey = `${getControllerKey(controllerId)}:${paramKey}`;
    const nextValue = !isValid;

    if (invalidFields[stateKey] === nextValue) {
      return;
    }

    invalidFields = {
      ...invalidFields,
      [stateKey]: nextValue,
    };
  }

  async function handleSaveController(controller: Controller) {
    const controllerKey = getControllerKey(controller.id);
    if (controllerHasInvalidFields(controller.id) || savingControllers[controllerKey]) {
      return;
    }

    savingControllers = {
      ...savingControllers,
      [controllerKey]: true,
    };

    try {
      const result = await onSaveControllerConfig(controller.id);
      if (result.success) {
        savedSnapshots = {
          ...savedSnapshots,
          [controllerKey]: serializeController(controller),
        };
      }

      saveFeedback = {
        ...saveFeedback,
        [controllerKey]: result.success
          ? {
              tone: result.deferred ? 'warning' : 'success',
              message: result.deferred
                ? 'Ajustes salvos. Este controlador precisa de restart da planta para entrar em execucao.'
                : 'Ajustes salvos.',
            }
          : {
              tone: 'error',
              message: result.error || 'Nao foi possivel salvar os ajustes.',
            },
      };
    } finally {
      savingControllers = {
        ...savingControllers,
        [controllerKey]: false,
      };
    }
  }

  $effect(() => {
    if (!plant) return;

    const activeControllerKeys = new Set(plant.controllers.map((controller) => getControllerKey(controller.id)));
    const nextSnapshots = { ...savedSnapshots };
    const nextInvalidFields = { ...invalidFields };
    const nextFeedback = { ...saveFeedback };
    const nextSaving = { ...savingControllers };
    let snapshotsChanged = false;
    let invalidChanged = false;
    let feedbackChanged = false;
    let savingChanged = false;

    for (const controller of plant.controllers) {
      const controllerKey = getControllerKey(controller.id);
      if (!(controllerKey in nextSnapshots)) {
        nextSnapshots[controllerKey] = serializeController(controller);
        snapshotsChanged = true;
      }

      if (
        controllerKey in nextFeedback &&
        nextFeedback[controllerKey]?.tone !== 'error' &&
        nextSnapshots[controllerKey] !== serializeController(controller)
      ) {
        delete nextFeedback[controllerKey];
        feedbackChanged = true;
      }
    }

    for (const key of Object.keys(nextSnapshots)) {
      const [plantId, controllerId] = key.split(':');
      if (plantId === plant.id && !activeControllerKeys.has(`${plantId}:${controllerId}`)) {
        delete nextSnapshots[key];
        snapshotsChanged = true;
        delete nextFeedback[key];
        feedbackChanged = true;
        delete nextSaving[key];
        savingChanged = true;
      }
    }

    for (const key of Object.keys(nextInvalidFields)) {
      if (key.startsWith(`${plant.id}:`)) {
        const controllerKey = key.split(':').slice(0, 2).join(':');
        if (!activeControllerKeys.has(controllerKey)) {
          delete nextInvalidFields[key];
          invalidChanged = true;
        }
      }
    }

    if (snapshotsChanged) {
      savedSnapshots = nextSnapshots;
    }
    if (invalidChanged) {
      invalidFields = nextInvalidFields;
    }
    if (feedbackChanged) {
      saveFeedback = nextFeedback;
    }
    if (savingChanged) {
      savingControllers = nextSaving;
    }
  });
</script>

<div class={`${visible
  ? 'w-full max-h-[58vh] translate-y-0 md:w-80 md:h-full md:max-h-full md:translate-y-0'
  : 'w-full max-h-0 translate-y-2 md:w-0 md:h-full md:max-h-full md:translate-y-0 md:translate-x-full'
} bg-white dark:bg-[#0c0c0e] border-t md:border-t-0 md:border-l border-slate-200 dark:border-white/5 flex flex-col min-h-0 overflow-hidden transition-[width,max-height,transform] duration-300 ease-in-out shadow-xl relative z-30 print:hidden`}>
  <div class="h-14 border-b border-slate-100 dark:border-white/5 flex justify-between items-center px-5 bg-slate-50 dark:bg-white/[0.02]">
    <h3 class="font-bold text-slate-700 dark:text-white text-sm">Malhas de Controle</h3>
    <button onclick={() => visible = false} class="text-slate-400 hover:text-slate-600 dark:hover:text-white" title="Recolher Painel">
      <ChevronsRight size={20} />
    </button>
  </div>
  <div class="flex-1 min-h-0 overflow-y-auto overscroll-y-contain p-4 space-y-5 sm:p-5 sm:space-y-6">
    {#if plant}
      <div class="bg-slate-50 dark:bg-[#121215] rounded-xl border border-slate-200 dark:border-white/5 shadow-sm overflow-hidden">
        <button 
          onclick={() => setpointsExpanded = !setpointsExpanded}
          class="w-full p-3 flex items-center justify-between hover:bg-slate-100 dark:hover:bg-white/5 transition-colors"
        >
          <span class="text-xs font-bold text-slate-500 dark:text-zinc-400 uppercase tracking-wide">
            Setpoints ({sensorVariables.length} sensores)
          </span>
          {#if setpointsExpanded}
            <ChevronUp size={16} class="text-slate-400" />
          {:else}
            <ChevronDown size={16} class="text-slate-400" />
          {/if}
        </button>
        
        {#if setpointsExpanded}
          <div class="px-3 pb-3 space-y-2">
            {#each sensorVariables as { variable, index } (variable.id)}
              <div class="flex items-center gap-2 p-2 rounded-lg bg-white dark:bg-black/20 border border-slate-100 dark:border-white/5">
                <div class="flex-1 min-w-0">
                  <span class="text-xs font-medium text-slate-600 dark:text-zinc-300 truncate block">
                    {variable.name}
                  </span>
                </div>
                <div class="flex items-center gap-1">
                  <input
                    type="number"
                    value={variable.setpoint}
                    min={variable.pvMin}
                    max={variable.pvMax}
                    step="0.1"
                    onchange={(e: Event) => onUpdateSetpoint(index, Number((e.target as HTMLInputElement).value))}
                    class="w-20 px-2 py-1 text-sm font-mono font-bold text-right text-blue-600 dark:text-blue-400 bg-transparent border border-slate-200 dark:border-white/10 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                  />
                  <span class="text-[10px] text-slate-400 dark:text-zinc-500 w-6">
                    {variable.unit}
                  </span>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
      
      <div class="border-t border-slate-100 dark:border-white/5"></div>
      
      <div>
        <div class="flex justify-between items-center mb-4">
          <span class="text-xs font-bold text-slate-500 uppercase">Controladores</span>
          <button onclick={onAddController} class="text-xs font-medium bg-blue-50 text-blue-600 hover:bg-blue-100 dark:bg-blue-900/20 dark:text-blue-400 dark:hover:bg-blue-900/30 px-3 py-1.5 rounded-full transition-colors">
            + Adicionar
          </button>
        </div>
        <div class="space-y-4 max-h-[700px] overflow-y-auto pr-2">
          {#each plant.controllers as ctrl (ctrl.id)}
            <div class="border border-slate-200 dark:border-white/10 rounded-xl overflow-hidden shadow-sm bg-white dark:bg-[#0c0c0e]">
              <div class="bg-slate-50 dark:bg-white/[0.02] p-3 border-b border-slate-100 dark:border-white/5 flex items-center justify-between">
                <div class="flex items-center gap-3">
                  <SimpleToggle checked={ctrl.active} ariaLabel="Toggle controller" onchange={() => onToggleControllerActive(ctrl.id, !ctrl.active)} />
                  <input
                    value={ctrl.name}
                    oninput={(e: Event) => onUpdateControllerMeta(ctrl.id, 'name', (e.target as HTMLInputElement).value)}
                    class="bg-transparent text-sm font-semibold text-slate-700 dark:text-zinc-200 w-32 focus:text-blue-600 dark:focus:text-blue-400 transition-colors"
                    style="border: none; outline: none; box-shadow: none;"
                  />
                  {#if ctrl.runtimeStatus === 'pending_restart'}
                    <span class="rounded-full bg-amber-100 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-amber-700 dark:bg-amber-900/20 dark:text-amber-300">
                      Pendente de restart
                    </span>
                  {/if}
                </div>
                <div class="flex items-center gap-1">
                  <button
                    type="button"
                    onclick={() => onEditControllerBindings(ctrl.id)}
                    title="Editar vínculos"
                    class="rounded p-1 text-slate-400 transition-colors hover:text-blue-500"
                  >
                    <Pencil size={14} />
                  </button>
                  <button
                    onclick={() => onDeleteController(ctrl.id)}
                    disabled={!!plant?.connected && ctrl.active && ctrl.runtimeStatus !== 'pending_restart'}
                    title={plant?.connected && ctrl.active && ctrl.runtimeStatus !== 'pending_restart'
                      ? 'Desative o controlador antes de removê-lo'
                      : 'Remover controlador'}
                    class="text-slate-400 hover:text-red-500 p-1 disabled:cursor-not-allowed disabled:opacity-40"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              </div>
              <div class={`p-4 space-y-3 ${ctrl.active ? '' : 'opacity-60'}`}>
                <div class="grid grid-cols-2 gap-2 rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 dark:border-white/10 dark:bg-white/[0.03]">
                  <div>
                    <div class="text-[10px] font-bold uppercase tracking-wide text-slate-500 dark:text-zinc-400">Entradas</div>
                    <div class="mt-1 text-xs text-slate-600 dark:text-zinc-300">{ctrl.inputVariableIds.length} selecionada(s)</div>
                  </div>
                  <div>
                    <div class="text-[10px] font-bold uppercase tracking-wide text-slate-500 dark:text-zinc-400">Saídas</div>
                    <div class="mt-1 text-xs text-slate-600 dark:text-zinc-300">{ctrl.outputVariableIds.length} selecionada(s)</div>
                  </div>
                </div>

                {#each Object.entries(ctrl.params) as [key, param]}
                  <DynamicParamInput
                    label={(param as ControllerParam).label || key}
                    type={(param as ControllerParam).type}
                    value={(param as ControllerParam).value}
                    onValidityChange={(isValid: boolean) => updateParamValidity(ctrl.id, key, isValid)}
                    onChange={(newValue: any) => onUpdateControllerParam(ctrl.id, key, newValue)}
                  />
                {/each}

                {#if isControllerDirty(ctrl) || saveFeedback[getControllerKey(ctrl.id)]}
                  <div class="rounded-lg border border-slate-200 dark:border-white/10 bg-slate-50 dark:bg-white/[0.02] p-3 space-y-2">
                    {#if saveFeedback[getControllerKey(ctrl.id)]}
                      {@const feedback = saveFeedback[getControllerKey(ctrl.id)]}
                      <div class={`text-[11px] ${
                        feedback.tone === 'success'
                          ? 'text-emerald-600 dark:text-emerald-400'
                          : feedback.tone === 'warning'
                            ? 'text-amber-600 dark:text-amber-400'
                            : 'text-red-500 dark:text-red-400'
                      }`}>
                        {feedback.message}
                      </div>
                    {/if}

                    {#if isControllerDirty(ctrl)}
                      <div class="flex items-center justify-between gap-2">
                        <span class="text-[11px] text-slate-500 dark:text-zinc-400">
                          Voce ainda nao salvou estas alteracoes.
                        </span>
                        <button
                          type="button"
                          onclick={() => handleSaveController(ctrl)}
                          disabled={controllerHasInvalidFields(ctrl.id) || savingControllers[getControllerKey(ctrl.id)]}
                          class="inline-flex items-center gap-2 rounded-lg bg-blue-600 px-3 py-1.5 text-[11px] font-semibold text-white transition-colors hover:bg-blue-700 disabled:cursor-not-allowed disabled:bg-blue-300 dark:disabled:bg-blue-900/40"
                        >
                          {#if savingControllers[getControllerKey(ctrl.id)]}
                            <LoaderCircle size={12} class="animate-spin" />
                            Salvando...
                          {:else}
                            <Save size={12} />
                            Salvar ajustes
                          {/if}
                        </button>
                      </div>
                    {/if}
                  </div>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>
