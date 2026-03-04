<script lang="ts">
  import { X, Search, Plus, Trash2, ChevronDown, ChevronUp, Check, Settings, Cpu, Activity, Gauge, Zap, Link, Upload, Code } from 'lucide-svelte';
  import { onMount } from 'svelte';
  import type { PlantVariable, VariableType } from '$lib/types/plant';
  import type { Controller } from '$lib/types/controller';
  import type { PluginDefinition, PluginInstance } from '$lib/types/plugin';
  import { PLUGIN_KIND_LABELS, PLUGIN_RUNTIME_LABELS } from '$lib/types/plugin';
  import { createDefaultVariable, VARIABLE_TYPE_LABELS } from '$lib/types/plant';
  import { listControllerTemplates, createPlant, type CreatePlantRequest } from '$lib/services/plantBackend';
  import { listPlugins, validatePluginFile, registerPlugin } from '$lib/services/pluginBackend';
  import { openFileDialog, readFileAsJSON, FILE_FILTERS } from '$lib/services/fileDialog';
  import { generateId } from '$lib/utils/format';
  import CreatePluginModal from './CreatePluginModal.svelte';
  import PluginInstanceConfigModal from './PluginInstanceConfigModal.svelte';

  interface Props {
    visible: boolean;
    onClose: () => void;
    onPlantCreated: (plant: any) => void;
  }

  let {
    visible = $bindable(),
    onClose,
    onPlantCreated,
  }: Props = $props();

  let plantName = $state('');
  let driverInstance = $state<PluginInstance | null>(null);
  let variables = $state<PlantVariable[]>([createDefaultVariable(0, 'Variável 1')]);
  let selectedControllers = $state<Controller[]>([]);

  let isLoading = $state(false);
  let error = $state<string | null>(null);
  let currentStep = $state<'info' | 'driver' | 'variables' | 'controllers'>('info');
  
  let availablePlugins = $state<PluginDefinition[]>([]);
  let controllerTemplates = $state<Controller[]>([]);
  let driverSearch = $state('');
  let controllerSearch = $state('');
  
  let showCreatePlugin = $state(false);
  let showInstanceConfig = $state(false);
  let pluginToConfig = $state<PluginDefinition | null>(null);
  let importError = $state<string | null>(null);

  const filteredPlugins = $derived(
    availablePlugins.filter(d => 
      d.name.toLowerCase().includes(driverSearch.toLowerCase()) ||
      PLUGIN_RUNTIME_LABELS[d.runtime].toLowerCase().includes(driverSearch.toLowerCase())
    )
  );

  const filteredTemplates = $derived(
    controllerTemplates.filter(c => 
      c.name.toLowerCase().includes(controllerSearch.toLowerCase()) ||
      c.type.toLowerCase().includes(controllerSearch.toLowerCase())
    )
  );

  const sensorVariables = $derived(
    variables.filter(v => v.type === 'sensor')
  );

  onMount(async () => {
    availablePlugins = await listPlugins('driver');
    controllerTemplates = await listControllerTemplates();
  });

  async function handleImportPlugin() {
    importError = null;
    try {
      const result = await openFileDialog({
        title: 'Importar Plugin JSON',
        filters: FILE_FILTERS.json,
      });
      if (!result) return;

      const json = await readFileAsJSON(result.file);
      const validation = await validatePluginFile(json);

      if (!validation.success || !validation.plugin) {
        importError = validation.error || 'Plugin inválido';
        return;
      }

      const reg = await registerPlugin(validation.plugin);
      if (!reg.success || !reg.plugin) {
        importError = reg.error || 'Erro ao registrar plugin';
        return;
      }

      availablePlugins = await listPlugins('driver');
      pluginToConfig = reg.plugin;
      showInstanceConfig = true;
    } catch (e) {
      importError = e instanceof Error ? e.message : 'Erro ao importar arquivo';
    }
  }

  function handlePluginCreated(plugin: PluginDefinition) {
    availablePlugins = [...availablePlugins, plugin];
    pluginToConfig = plugin;
    showInstanceConfig = true;
  }

  function handleSelectPlugin(plugin: PluginDefinition) {
    pluginToConfig = plugin;
    showInstanceConfig = true;
  }

  function handleInstanceConfigured(instance: PluginInstance) {
    driverInstance = instance;
    currentStep = 'variables';
  }

  function addVariable() {
    const nextIndex = variables.length;
    variables = [...variables, createDefaultVariable(nextIndex, `Variável ${nextIndex + 1}`)];
  }

  function removeVariable(index: number) {
    if (variables.length <= 1) return;
    variables = variables.filter((_, i) => i !== index);
  }

  function updateVariable(index: number, field: keyof PlantVariable, value: any) {
    variables = variables.map((v, i) => {
      if (i !== index) return v;
      if (field === 'type' && value === 'atuador') {
        return { ...v, [field]: value, linkedSensorIds: [] };
      }
      if (field === 'type' && value === 'sensor') {
        const { linkedSensorIds, ...rest } = v;
        return { ...rest, [field]: value };
      }
      return { ...v, [field]: value };
    });
  }

  function toggleLinkedSensor(actuatorIdx: number, sensorId: string) {
    variables = variables.map((v, i) => {
      if (i !== actuatorIdx) return v;
      const linked = v.linkedSensorIds ?? [];
      const newLinked = linked.includes(sensorId)
        ? linked.filter(id => id !== sensorId)
        : [...linked, sensorId];
      return { ...v, linkedSensorIds: newLinked };
    });
  }

  function addController(template: Controller) {
    const newController: Controller = {
      ...template,
      id: generateId(),
      name: `${template.name} ${selectedControllers.length + 1}`,
      active: true,
    };
    selectedControllers = [...selectedControllers, newController];
  }

  function removeController(id: string) {
    selectedControllers = selectedControllers.filter(c => c.id !== id);
  }

  async function handleSubmit() {
    error = null;
    
    if (!plantName.trim()) {
      error = 'Nome da planta é obrigatório';
      currentStep = 'info';
      return;
    }

    if (!driverInstance) {
      error = 'Configure um driver de comunicação';
      currentStep = 'driver';
      return;
    }

    if (variables.length === 0) {
      error = 'Adicione pelo menos uma variável';
      currentStep = 'variables';
      return;
    }

    isLoading = true;

    try {
      const request: CreatePlantRequest = {
        name: plantName.trim(),
        driverId: driverInstance.pluginId,
        variables,
        controllers: selectedControllers,
      };

      const response = await createPlant(request);

      if (response.success && response.plant) {
        onPlantCreated(response.plant);
        resetForm();
        onClose();
      } else {
        error = response.error || 'Erro desconhecido ao criar planta';
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erro ao criar planta';
    } finally {
      isLoading = false;
    }
  }

  function resetForm() {
    plantName = '';
    driverInstance = null;
    variables = [createDefaultVariable(0, 'Variável 1')];
    selectedControllers = [];
    currentStep = 'info';
    error = null;
    importError = null;
    pluginToConfig = null;
  }

  function handleClose() {
    resetForm();
    onClose();
  }
</script>

{#if visible}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div 
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
    onclick={handleClose}
  >
    <div 
      class="bg-white dark:bg-[#0c0c0e] rounded-2xl shadow-2xl w-full max-w-2xl max-h-[85vh] flex flex-col overflow-hidden border border-slate-200 dark:border-white/10"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="flex items-center justify-between px-6 py-4 border-b border-slate-200 dark:border-white/5 shrink-0">
        <div>
          <h2 class="text-lg font-bold text-slate-800 dark:text-white">Criar Nova Planta</h2>
          <p class="text-xs text-slate-500 dark:text-zinc-400 mt-0.5">Configure os parâmetros da nova unidade de controle</p>
        </div>
        <button 
          onclick={handleClose}
          class="p-2 rounded-lg hover:bg-slate-100 dark:hover:bg-white/5 text-slate-400 transition-colors"
        >
          <X size={20} />
        </button>
      </div>

      <div class="flex border-b border-slate-200 dark:border-white/5 px-6 shrink-0">
        <button 
          onclick={() => currentStep = 'info'}
          class="px-4 py-3 text-sm font-medium border-b-2 transition-colors {currentStep === 'info' ? 'border-blue-500 text-blue-600 dark:text-blue-400' : 'border-transparent text-slate-500 hover:text-slate-700 dark:hover:text-zinc-300'}"
        >
          Informações
        </button>
        <button 
          onclick={() => currentStep = 'driver'}
          class="px-4 py-3 text-sm font-medium border-b-2 transition-colors {currentStep === 'driver' ? 'border-blue-500 text-blue-600 dark:text-blue-400' : 'border-transparent text-slate-500 hover:text-slate-700 dark:hover:text-zinc-300'}"
        >
          Driver
          {#if driverInstance}
            <span class="ml-1.5 w-2 h-2 rounded-full bg-emerald-500 inline-block"></span>
          {/if}
        </button>
        <button 
          onclick={() => currentStep = 'variables'}
          class="px-4 py-3 text-sm font-medium border-b-2 transition-colors {currentStep === 'variables' ? 'border-blue-500 text-blue-600 dark:text-blue-400' : 'border-transparent text-slate-500 hover:text-slate-700 dark:hover:text-zinc-300'}"
        >
          Variáveis ({variables.length})
        </button>
        <button 
          onclick={() => currentStep = 'controllers'}
          class="px-4 py-3 text-sm font-medium border-b-2 transition-colors {currentStep === 'controllers' ? 'border-blue-500 text-blue-600 dark:text-blue-400' : 'border-transparent text-slate-500 hover:text-slate-700 dark:hover:text-zinc-300'}"
        >
          Controladores ({selectedControllers.length})
        </button>
      </div>

      <div class="flex-1 overflow-y-auto p-6">
        {#if error}
          <div class="mb-4 p-3 rounded-lg bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-900/50 text-red-700 dark:text-red-400 text-sm">
            {error}
          </div>
        {/if}

        {#if currentStep === 'info'}
          <div class="space-y-4">
            <label class="block">
              <span class="block text-sm font-medium text-slate-700 dark:text-zinc-300 mb-2">
                Nome da Planta *
              </span>
              <input
                type="text"
                bind:value={plantName}
                placeholder="Ex: Tanque Misturador T-200"
                class="w-full px-4 py-3 rounded-xl border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-slate-800 dark:text-white placeholder-slate-400 dark:placeholder-zinc-500 focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500"
              />
            </label>

            {#if driverInstance}
              <div class="p-4 rounded-xl bg-slate-50 dark:bg-white/5 border border-slate-200 dark:border-white/10">
                <div class="flex items-center gap-3">
                  <div class="w-10 h-10 rounded-lg bg-blue-100 dark:bg-blue-900/30 flex items-center justify-center">
                    <Cpu size={20} class="text-blue-600 dark:text-blue-400" />
                  </div>
                  <div class="flex-1">
                    <div class="font-medium text-slate-800 dark:text-white">{driverInstance.pluginName}</div>
                    <div class="text-xs text-slate-500 dark:text-zinc-400">Configurado · {Object.keys(driverInstance.config).length} parâmetro(s)</div>
                  </div>
                  <button 
                    onclick={() => currentStep = 'driver'}
                    class="text-xs text-blue-600 dark:text-blue-400 hover:underline"
                  >
                    Alterar
                  </button>
                </div>
              </div>
            {:else}
              <button
                onclick={() => currentStep = 'driver'}
                class="w-full p-4 rounded-xl border-2 border-dashed border-slate-200 dark:border-white/10 hover:border-blue-400 dark:hover:border-blue-500 transition-colors text-slate-500 dark:text-zinc-400 hover:text-blue-600 dark:hover:text-blue-400"
              >
                <Cpu size={24} class="mx-auto mb-2 opacity-50" />
                <div class="text-sm font-medium">Selecionar Driver de Comunicação</div>
              </button>
            {/if}
          </div>

        {:else if currentStep === 'driver'}
          <div class="space-y-4">
            {#if importError}
              <div class="p-3 rounded-lg bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-900/50 text-red-700 dark:text-red-400 text-sm">
                {importError}
              </div>
            {/if}

            {#if driverInstance}
              <div class="p-4 rounded-xl border border-emerald-300 dark:border-emerald-700 bg-emerald-50 dark:bg-emerald-900/20">
                <div class="flex items-center gap-3">
                  <div class="w-10 h-10 rounded-lg bg-emerald-500 flex items-center justify-center">
                    <Check size={20} class="text-white" />
                  </div>
                  <div class="flex-1 min-w-0">
                    <div class="font-medium text-slate-800 dark:text-white">{driverInstance.pluginName}</div>
                    <div class="text-xs text-emerald-600 dark:text-emerald-400">{Object.keys(driverInstance.config).length} parâmetro(s) configurados</div>
                  </div>
                  <button
                    onclick={() => { driverInstance = null; }}
                    class="text-xs text-red-500 hover:underline"
                  >
                    Remover
                  </button>
                </div>
              </div>
            {/if}

            <div class="relative">
              <Search size={18} class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
              <input
                type="text"
                bind:value={driverSearch}
                placeholder="Buscar plugin driver..."
                class="w-full pl-10 pr-4 py-2.5 rounded-xl border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-slate-800 dark:text-white placeholder-slate-400 dark:placeholder-zinc-500 focus:outline-none focus:ring-2 focus:ring-blue-500/50"
              />
            </div>

            <div class="space-y-2">
              {#each filteredPlugins as plugin (plugin.id)}
                <button
                  onclick={() => handleSelectPlugin(plugin)}
                  class="w-full p-4 rounded-xl border text-left transition-all {driverInstance?.pluginId === plugin.id 
                    ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20' 
                    : 'border-slate-200 dark:border-white/10 hover:border-slate-300 dark:hover:border-white/20 bg-white dark:bg-[#18181b]'}"
                >
                  <div class="flex items-center gap-3">
                    <div class="w-10 h-10 rounded-lg {driverInstance?.pluginId === plugin.id ? 'bg-blue-500' : 'bg-slate-100 dark:bg-white/10'} flex items-center justify-center">
                      <Cpu size={20} class={driverInstance?.pluginId === plugin.id ? 'text-white' : 'text-slate-500 dark:text-zinc-400'} />
                    </div>
                    <div class="flex-1 min-w-0">
                      <div class="font-medium text-slate-800 dark:text-white truncate">{plugin.name}</div>
                      <div class="text-xs text-slate-500 dark:text-zinc-400">
                        {PLUGIN_RUNTIME_LABELS[plugin.runtime]} · {plugin.schema.length} parâmetro(s)
                      </div>
                    </div>
                    {#if driverInstance?.pluginId === plugin.id}
                      <Check size={20} class="text-blue-500 shrink-0" />
                    {/if}
                  </div>
                  {#if plugin.description}
                    <p class="mt-2 text-xs text-slate-500 dark:text-zinc-500 truncate">{plugin.description}</p>
                  {/if}
                </button>
              {/each}

              {#if filteredPlugins.length === 0}
                <div class="text-center py-8 text-slate-400 dark:text-zinc-500">
                  <Cpu size={32} class="mx-auto mb-2 opacity-50" />
                  <p class="text-sm">Nenhum plugin driver encontrado</p>
                </div>
              {/if}
            </div>

            <div class="grid grid-cols-2 gap-3">
              <button
                onclick={handleImportPlugin}
                class="p-3 rounded-xl border-2 border-dashed border-slate-200 dark:border-white/10 hover:border-blue-400 dark:hover:border-blue-500 transition-colors text-slate-500 dark:text-zinc-400 hover:text-blue-600 dark:hover:text-blue-400 flex items-center justify-center gap-2"
              >
                <Upload size={18} />
                <span class="text-sm font-medium">Importar .json</span>
              </button>
              <button
                onclick={() => { showCreatePlugin = true; }}
                class="p-3 rounded-xl border-2 border-dashed border-slate-200 dark:border-white/10 hover:border-emerald-400 dark:hover:border-emerald-500 transition-colors text-slate-500 dark:text-zinc-400 hover:text-emerald-600 dark:hover:text-emerald-400 flex items-center justify-center gap-2"
              >
                <Code size={18} />
                <span class="text-sm font-medium">Criar Novo</span>
              </button>
            </div>
          </div>

        {:else if currentStep === 'variables'}
          <div class="space-y-4">
            {#each variables as variable, idx (variable.id)}
              <div class="p-5 rounded-xl border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b]">
                <div class="flex items-start gap-4">
                  <div class={`w-10 h-10 rounded-xl flex items-center justify-center shrink-0 ${variable.type === 'sensor' ? 'bg-cyan-100 dark:bg-cyan-900/30' : 'bg-orange-100 dark:bg-orange-900/30'}`}>
                    {#if variable.type === 'sensor'}
                      <Gauge size={20} class="text-cyan-600 dark:text-cyan-400" />
                    {:else}
                      <Zap size={20} class="text-orange-600 dark:text-orange-400" />
                    {/if}
                  </div>
                  <div class="flex-1 space-y-4">
                    <div class="grid grid-cols-[120px_1fr] gap-3">
                      <label class="block">
                        <span class="text-[10px] text-slate-400 dark:text-zinc-500 uppercase mb-1.5 block">Tipo</span>
                        <select
                          value={variable.type}
                          onchange={(e) => updateVariable(idx, 'type', (e.target as HTMLSelectElement).value as VariableType)}
                          class="w-full h-10 px-3 rounded-lg border border-slate-200 dark:border-white/10 bg-white dark:bg-zinc-900 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50 cursor-pointer"
                        >
                          {#each Object.entries(VARIABLE_TYPE_LABELS) as [value, label]}
                            <option {value} class="dark:bg-zinc-900">{label}</option>
                          {/each}
                        </select>
                      </label>
                      <label class="block">
                        <span class="text-[10px] text-slate-400 dark:text-zinc-500 uppercase mb-1.5 block">Nome</span>
                        <input
                          type="text"
                          value={variable.name}
                          oninput={(e) => updateVariable(idx, 'name', (e.target as HTMLInputElement).value)}
                          placeholder="Nome da variável"
                          class="w-full h-10 px-3 rounded-lg border border-slate-200 dark:border-white/10 bg-transparent text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                        />
                      </label>
                    </div>
                    <div class={`grid gap-3 ${variable.type === 'sensor' ? 'grid-cols-4' : 'grid-cols-3'}`}>
                      <label class="block">
                        <span class="text-[10px] text-slate-400 dark:text-zinc-500 uppercase mb-1.5 block">Unidade</span>
                        <input
                          type="text"
                          value={variable.unit}
                          oninput={(e) => updateVariable(idx, 'unit', (e.target as HTMLInputElement).value)}
                          placeholder="%"
                          class="w-full h-10 px-3 rounded-lg border border-slate-200 dark:border-white/10 bg-transparent text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                        />
                      </label>
                      {#if variable.type === 'sensor'}
                        <label class="block">
                          <span class="text-[10px] text-slate-400 dark:text-zinc-500 uppercase mb-1.5 block">Setpoint</span>
                          <input
                            type="number"
                            value={variable.setpoint}
                            oninput={(e) => updateVariable(idx, 'setpoint', Number((e.target as HTMLInputElement).value))}
                            class="w-full h-10 px-3 rounded-lg border border-slate-200 dark:border-white/10 bg-transparent text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                          />
                        </label>
                      {/if}
                      <label class="block">
                        <span class="text-[10px] text-slate-400 dark:text-zinc-500 uppercase mb-1.5 block">{variable.type === 'sensor' ? 'PV Min' : 'MV Min'}</span>
                        <input
                          type="number"
                          value={variable.pvMin}
                          oninput={(e) => updateVariable(idx, 'pvMin', Number((e.target as HTMLInputElement).value))}
                          class="w-full h-10 px-3 rounded-lg border border-slate-200 dark:border-white/10 bg-transparent text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                        />
                      </label>
                      <label class="block">
                        <span class="text-[10px] text-slate-400 dark:text-zinc-500 uppercase mb-1.5 block">{variable.type === 'sensor' ? 'PV Max' : 'MV Max'}</span>
                        <input
                          type="number"
                          value={variable.pvMax}
                          oninput={(e) => updateVariable(idx, 'pvMax', Number((e.target as HTMLInputElement).value))}
                          class="w-full h-10 px-3 rounded-lg border border-slate-200 dark:border-white/10 bg-transparent text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                        />
                      </label>
                    </div>
                    {#if variable.type === 'atuador'}
                      <div class="pt-3 border-t border-slate-100 dark:border-white/5">
                        <div class="flex items-center gap-2 mb-2">
                          <Link size={14} class="text-orange-500" />
                          <span class="text-[10px] text-slate-400 dark:text-zinc-500 uppercase">Vincular a Sensores</span>
                        </div>
                        {#if sensorVariables.length === 0}
                          <p class="text-xs text-slate-400 dark:text-zinc-500 italic">Nenhum sensor disponível. Adicione sensores primeiro.</p>
                        {:else}
                          <div class="flex flex-wrap gap-2">
                            {#each sensorVariables as sensor (sensor.id)}
                              <button
                                type="button"
                                onclick={() => toggleLinkedSensor(idx, sensor.id)}
                                class={`px-3 py-1.5 rounded-lg text-xs font-medium border transition-all ${
                                  (variable.linkedSensorIds ?? []).includes(sensor.id)
                                    ? 'bg-cyan-100 dark:bg-cyan-900/30 border-cyan-300 dark:border-cyan-700 text-cyan-700 dark:text-cyan-300'
                                    : 'bg-slate-50 dark:bg-zinc-900 border-slate-200 dark:border-white/10 text-slate-500 dark:text-zinc-400 hover:border-cyan-300 dark:hover:border-cyan-700'
                                }`}
                              >
                                <span class="flex items-center gap-1.5">
                                  <Gauge size={12} />
                                  {sensor.name}
                                </span>
                              </button>
                            {/each}
                          </div>
                        {/if}
                      </div>
                    {/if}
                  </div>
                  {#if variables.length > 1}
                    <button
                      onclick={() => removeVariable(idx)}
                      class="p-2.5 rounded-lg hover:bg-red-100 dark:hover:bg-red-900/30 text-slate-400 hover:text-red-600 transition-colors shrink-0 mt-6"
                    >
                      <Trash2 size={18} />
                    </button>
                  {/if}
                </div>
              </div>
            {/each}

            <button
              onclick={addVariable}
              class="w-full p-3 rounded-xl border-2 border-dashed border-slate-200 dark:border-white/10 hover:border-blue-400 dark:hover:border-blue-500 transition-colors text-slate-500 dark:text-zinc-400 hover:text-blue-600 dark:hover:text-blue-400 flex items-center justify-center gap-2"
            >
              <Plus size={18} />
              <span class="text-sm font-medium">Adicionar Variável</span>
            </button>
          </div>

        {:else if currentStep === 'controllers'}
          <div class="space-y-4">
            {#if selectedControllers.length > 0}
              <div class="space-y-2">
                <h4 class="text-xs font-bold text-slate-500 dark:text-zinc-400 uppercase tracking-wide">Controladores Adicionados</h4>
                {#each selectedControllers as controller (controller.id)}
                  <div class="flex items-center gap-3 p-3 rounded-xl border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b]">
                    <div class="w-8 h-8 rounded-lg bg-emerald-100 dark:bg-emerald-900/30 flex items-center justify-center">
                      <Settings size={16} class="text-emerald-600 dark:text-emerald-400" />
                    </div>
                    <div class="flex-1 min-w-0">
                      <div class="font-medium text-sm text-slate-800 dark:text-white truncate">{controller.name}</div>
                      <div class="text-xs text-slate-500 dark:text-zinc-400">{controller.type}</div>
                    </div>
                    <button
                      onclick={() => removeController(controller.id)}
                      class="p-1.5 rounded hover:bg-red-100 dark:hover:bg-red-900/30 text-slate-400 hover:text-red-600 transition-colors"
                    >
                      <Trash2 size={14} />
                    </button>
                  </div>
                {/each}
              </div>
            {/if}

            <div>
              <h4 class="text-xs font-bold text-slate-500 dark:text-zinc-400 uppercase tracking-wide mb-2">Templates Disponíveis</h4>
              
              <div class="relative mb-3">
                <Search size={16} class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
                <input
                  type="text"
                  bind:value={controllerSearch}
                  placeholder="Buscar template..."
                  class="w-full pl-9 pr-4 py-2 rounded-lg border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50"
                />
              </div>

              <div class="grid grid-cols-2 gap-2">
                {#each filteredTemplates as template (template.id)}
                  <button
                    onclick={() => addController(template)}
                    class="p-3 rounded-xl border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] hover:border-emerald-400 dark:hover:border-emerald-500 transition-all text-left"
                  >
                    <div class="flex items-center gap-2 mb-1">
                      <Settings size={14} class="text-slate-400" />
                      <span class="text-sm font-medium text-slate-800 dark:text-white">{template.name}</span>
                    </div>
                    <span class="text-xs text-slate-500 dark:text-zinc-400">{template.type}</span>
                  </button>
                {/each}
              </div>
            </div>
          </div>
        {/if}
      </div>

      <div class="flex items-center justify-between px-6 py-4 border-t border-slate-200 dark:border-white/5 bg-slate-50 dark:bg-white/[0.02] shrink-0">
        <button
          onclick={handleClose}
          class="px-4 py-2 rounded-lg text-sm font-medium text-slate-600 dark:text-zinc-400 hover:bg-slate-200 dark:hover:bg-white/10 transition-colors"
        >
          Cancelar
        </button>
        <button
          onclick={handleSubmit}
          disabled={isLoading}
          class="px-6 py-2 rounded-lg text-sm font-bold bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white transition-colors flex items-center gap-2"
        >
          {#if isLoading}
            <div class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
            Criando...
          {:else}
            Criar Planta
          {/if}
        </button>
      </div>
    </div>
  </div>

  <CreatePluginModal
    bind:visible={showCreatePlugin}
    forceKind="driver"
    onClose={() => { showCreatePlugin = false; }}
    onPluginCreated={handlePluginCreated}
  />

  <PluginInstanceConfigModal
    bind:visible={showInstanceConfig}
    plugin={pluginToConfig}
    onClose={() => { showInstanceConfig = false; }}
    onConfigured={handleInstanceConfigured}
  />
{/if}
