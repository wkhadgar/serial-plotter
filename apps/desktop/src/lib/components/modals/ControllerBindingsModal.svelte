<script lang="ts">
  import { Check, AlertCircle, X } from 'lucide-svelte';
  import type { PlantVariable } from '$lib/types/plant';
  import ControllerVariableBindings from '../controllers/ControllerVariableBindings.svelte';

  interface Props {
    visible: boolean;
    controllerName: string;
    sensorVariables: PlantVariable[];
    actuatorVariables: PlantVariable[];
    initialInputVariableIds: string[];
    initialOutputVariableIds: string[];
    onClose: () => void;
    onSave: (bindings: { inputVariableIds: string[]; outputVariableIds: string[] }) => string | null;
  }

  let {
    visible = $bindable(false),
    controllerName,
    sensorVariables,
    actuatorVariables,
    initialInputVariableIds,
    initialOutputVariableIds,
    onClose,
    onSave,
  }: Props = $props();

  let inputVariableIds = $state<string[]>([]);
  let outputVariableIds = $state<string[]>([]);
  let error = $state<string | null>(null);

  $effect(() => {
    if (!visible) return;

    inputVariableIds = [...initialInputVariableIds];
    outputVariableIds = [...initialOutputVariableIds];
    error = null;
  });

  function handleSubmit() {
    if (inputVariableIds.length === 0) {
      error = 'Selecione pelo menos uma variável de entrada';
      return;
    }

    if (outputVariableIds.length === 0) {
      error = 'Selecione pelo menos uma variável de saída';
      return;
    }

    const result = onSave({
      inputVariableIds,
      outputVariableIds,
    });

    if (result) {
      error = result;
      return;
    }

    visible = false;
    onClose();
  }

  function handleClose() {
    error = null;
    visible = false;
    onClose();
  }
</script>

{#if visible}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-[72] flex items-start justify-center overflow-y-auto bg-black/60 px-3 py-4 sm:items-center sm:px-4 sm:py-0"
    onclick={handleClose}
  >
    <div
      class="my-2 flex max-h-[90vh] w-full max-w-xl flex-col overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-2xl dark:border-white/10 dark:bg-[#0c0c0e] sm:my-0 sm:max-h-[85vh]"
      onclick={(event) => event.stopPropagation()}
    >
      <div class="flex items-center justify-between border-b border-slate-200 px-4 py-4 dark:border-white/5 sm:px-6">
        <div>
          <h2 class="text-lg font-bold text-slate-800 dark:text-white">Editar vínculos de {controllerName}</h2>
          <p class="mt-0.5 text-xs text-slate-500 dark:text-zinc-400">
            Atualize apenas as variáveis de entrada e saída do controlador.
          </p>
        </div>
        <button
          type="button"
          onclick={handleClose}
          class="rounded-lg p-2 text-slate-400 transition-colors hover:bg-slate-100 hover:text-slate-600 dark:hover:bg-white/5 dark:hover:text-white"
        >
          <X size={20} />
        </button>
      </div>

      <div class="flex-1 overflow-y-auto p-4 space-y-4 sm:p-6">
        {#if error}
          <div class="flex items-center gap-2 rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700 dark:border-red-900/50 dark:bg-red-900/20 dark:text-red-400">
            <AlertCircle size={16} class="shrink-0" />
            {error}
          </div>
        {/if}

        <ControllerVariableBindings
          label="Entradas"
          helper="Selecione um ou mais sensores."
          variables={sensorVariables}
          selectedIds={inputVariableIds}
          emptyLabel="Adicione sensores na planta para vincular entradas."
          tone="sensor"
          onChange={(ids) => inputVariableIds = ids}
        />

        <ControllerVariableBindings
          label="Saídas"
          helper="Selecione um ou mais atuadores."
          variables={actuatorVariables}
          selectedIds={outputVariableIds}
          emptyLabel="Adicione atuadores na planta para vincular saídas."
          tone="atuador"
          onChange={(ids) => outputVariableIds = ids}
        />
      </div>

      <div class="flex items-center justify-between border-t border-slate-200 bg-slate-50 px-4 py-4 dark:border-white/5 dark:bg-white/[0.02] sm:px-6">
        <button
          type="button"
          onclick={handleClose}
          class="rounded-lg px-4 py-2 text-sm font-medium text-slate-600 transition-colors hover:bg-slate-200 dark:text-zinc-400 dark:hover:bg-white/10"
        >
          Cancelar
        </button>
        <button
          type="button"
          onclick={handleSubmit}
          class="inline-flex items-center gap-2 rounded-lg bg-blue-600 px-6 py-2 text-sm font-bold text-white transition-colors hover:bg-blue-700"
        >
          <Check size={16} />
          Salvar vínculos
        </button>
      </div>
    </div>
  </div>
{/if}
