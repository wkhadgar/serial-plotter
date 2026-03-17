<script lang="ts">
  import { onDestroy } from 'svelte';
  import { MoreVertical, Code, Settings, Trash2 } from 'lucide-svelte';
  import type { PluginDefinition, PluginKind } from '$lib/types/plugin';
  import { getPluginKindLabel, PLUGIN_RUNTIME_LABELS } from '$lib/types/plugin';

  interface Props {
    plugin: PluginDefinition;
    onEdit?: (plugin: PluginDefinition) => void;
    onDelete?: (plugin: PluginDefinition) => void;
    onViewCode?: (plugin: PluginDefinition) => void;
  }

  let { plugin, onEdit, onDelete, onViewCode }: Props = $props();

  let menuOpen = $state(false);
  let closeMenuTimeout: ReturnType<typeof setTimeout> | null = null;
  let menuRootElement = $state<HTMLDivElement | null>(null);

  const kindColors: Record<PluginKind, { bg: string; text: string; border: string }> = {
    driver: { bg: 'bg-blue-500/10', text: 'text-blue-600 dark:text-blue-400', border: 'border-blue-500/20' },
    controller: { bg: 'bg-purple-500/10', text: 'text-purple-600 dark:text-purple-400', border: 'border-purple-500/20' },
  };

  const fallbackColors = { bg: 'bg-slate-500/10', text: 'text-slate-600 dark:text-slate-300', border: 'border-slate-500/20' };
  const colors = $derived(kindColors[plugin.kind] ?? fallbackColors);

  function handleMenuToggle(e: MouseEvent) {
    e.stopPropagation();
    cancelScheduledClose();
    menuOpen = !menuOpen;
  }

  function handleAction(action: () => void) {
    closeMenu();
    action();
  }

  function cancelScheduledClose() {
    if (closeMenuTimeout) {
      clearTimeout(closeMenuTimeout);
      closeMenuTimeout = null;
    }
  }

  function scheduleMenuClose() {
    cancelScheduledClose();
    closeMenuTimeout = setTimeout(() => {
      menuOpen = false;
      closeMenuTimeout = null;
    }, 180);
  }

  function handleMenuMouseEnter() {
    cancelScheduledClose();
  }

  function handleMenuMouseLeave() {
    scheduleMenuClose();
  }

  function closeMenu() {
    cancelScheduledClose();
    menuOpen = false;
  }

  onDestroy(() => {
    cancelScheduledClose();
  });

  $effect(() => {
    if (!menuOpen) return;

    const handlePointerDown = (event: PointerEvent) => {
      const target = event.target as Node | null;
      if (!target) return;
      if (menuRootElement && !menuRootElement.contains(target)) {
        closeMenu();
      }
    };

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        closeMenu();
      }
    };

    window.addEventListener('pointerdown', handlePointerDown);
    window.addEventListener('keydown', handleKeyDown);

    return () => {
      window.removeEventListener('pointerdown', handlePointerDown);
      window.removeEventListener('keydown', handleKeyDown);
    };
  });
</script>

<div
  class="group relative h-full min-h-[170px] rounded-[22px] border border-slate-200 bg-white p-5 transition-all duration-200 hover:-translate-y-0.5 hover:border-slate-300 hover:shadow-md dark:border-white/5 dark:bg-zinc-900 dark:hover:border-white/10"
>
  <div class="flex h-full flex-col gap-3.5">
    <div class="flex items-start justify-between gap-3">
      <div class="flex-1 min-w-0">
        <div class="flex flex-wrap items-center gap-2 mb-2">
          <h3 class="truncate text-[15px] font-semibold leading-5 text-slate-800 dark:text-white">
            {plugin.name}
          </h3>
          <span class={`rounded-full border px-2 py-0.5 text-[11px] font-medium ${colors.bg} ${colors.text} ${colors.border}`}>
            {getPluginKindLabel(plugin.kind)}
          </span>
          <span class="rounded-full border border-slate-200 bg-slate-100 px-2 py-0.5 text-[10px] font-medium uppercase tracking-wide text-slate-500 dark:border-white/10 dark:bg-zinc-800 dark:text-zinc-400">
            {plugin.source === 'backend' ? 'Padrão' : 'Personalizado'}
          </span>
        </div>

        <div class="min-h-[46px]">
          {#if plugin.description}
            <p class="mt-0.5 line-clamp-2 text-sm text-slate-500 dark:text-zinc-400">
              {plugin.description}
            </p>
          {:else}
            <p class="text-sm text-slate-400 dark:text-zinc-500">
              Sem descrição registrada para este plugin.
            </p>
          {/if}
        </div>
      </div>

      <div
        bind:this={menuRootElement}
        class="relative shrink-0"
        role="group"
        aria-label="Ações do plugin"
        onmouseenter={handleMenuMouseEnter}
        onmouseleave={handleMenuMouseLeave}
      >
        <button
          onclick={handleMenuToggle}
          class="rounded-lg p-1.5 text-slate-400 opacity-100 transition-colors hover:bg-slate-100 hover:text-slate-600 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500/50 dark:text-zinc-500 dark:hover:bg-white/5 dark:hover:text-zinc-300 md:opacity-0 md:group-hover:opacity-100"
          aria-label="Abrir ações do plugin"
        >
          <MoreVertical class="w-4 h-4" />
        </button>

        {#if menuOpen}
          <div class="absolute right-0 top-full pt-1 z-50">
            <div
              onclick={(event) => event.stopPropagation()}
              onkeydown={(event) => event.stopPropagation()}
              role="menu"
              tabindex="-1"
              class="w-40 rounded-lg border border-slate-200 bg-white py-1 shadow-lg backdrop-blur-sm dark:border-white/10 dark:bg-zinc-800"
            >
              {#if onViewCode && plugin.sourceCode}
                <button
                  onclick={() => handleAction(() => onViewCode?.(plugin))}
                  class="w-full px-3 py-2 text-left text-sm text-slate-700 dark:text-zinc-200 hover:bg-slate-100 dark:hover:bg-white/5 flex items-center gap-2"
                >
                  <Code class="w-4 h-4" />
                  Ver código
                </button>
              {/if}
              {#if onEdit}
                <button
                  onclick={() => handleAction(() => onEdit?.(plugin))}
                  class="w-full px-3 py-2 text-left text-sm text-slate-700 dark:text-zinc-200 hover:bg-slate-100 dark:hover:bg-white/5 flex items-center gap-2"
                >
                  <Settings class="w-4 h-4" />
                  Editar
                </button>
              {/if}
              {#if onDelete}
                <button
                  onclick={() => handleAction(() => onDelete?.(plugin))}
                  class="w-full px-3 py-2 text-left text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-500/10 flex items-center gap-2"
                >
                  <Trash2 class="w-4 h-4" />
                  Excluir
                </button>
              {/if}
            </div>
          </div>
        {/if}
      </div>
    </div>

    <div class="mt-auto flex flex-wrap items-center gap-2.5 border-t border-slate-100 pt-3 text-xs text-slate-400 dark:border-white/5 dark:text-zinc-500">
      <span class="inline-flex items-center gap-1 rounded-md bg-slate-100 px-2 py-1 dark:bg-white/5">
        <Code class="w-3.5 h-3.5" />
        {PLUGIN_RUNTIME_LABELS[plugin.runtime]}
      </span>
      {#if plugin.version}
        <span class="inline-flex items-center rounded-md bg-slate-100 px-2 py-1 dark:bg-white/5">v{plugin.version}</span>
      {/if}
      {#if plugin.schema.length > 0}
        <span class="inline-flex items-center rounded-md bg-slate-100 px-2 py-1 dark:bg-white/5">{plugin.schema.length} ajustes</span>
      {/if}
    </div>
  </div>
</div>
