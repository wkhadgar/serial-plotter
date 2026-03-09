<script lang="ts">
  import { onMount } from 'svelte';
  import { Plus, X } from 'lucide-svelte';

  export interface WorkspaceTabItem {
    id: string;
    name: string;
    closable?: boolean;
    indicatorClass?: string;
    placeholder?: boolean;
  }

  let {
    items,
    activeId,
    onSelect,
    onAdd,
    onRemove,
    addLabel = 'Adicionar',
    addButtonRef = $bindable(),
  }: {
    items: WorkspaceTabItem[];
    activeId: string;
    onSelect: (id: string) => void;
    onAdd: () => void;
    onRemove?: (id: string) => void;
    addLabel?: string;
    addButtonRef?: HTMLButtonElement;
  } = $props();

  let tabsViewportRef = $state<HTMLDivElement | undefined>(undefined);
  let tabsTrackRef = $state<HTMLDivElement | undefined>(undefined);
  let tabsItemsRef = $state<HTMLDivElement | undefined>(undefined);
  let inlineAddButtonRef = $state<HTMLButtonElement | undefined>(undefined);
  let fixedAddButtonRef = $state<HTMLButtonElement | undefined>(undefined);
  let useFixedAddButton = $state(false);

  function syncBoundAddButtonRef(): void {
    addButtonRef = useFixedAddButton ? fixedAddButtonRef : inlineAddButtonRef;
  }

  function updateAddButtonMode(): void {
    if (!tabsViewportRef || !tabsTrackRef || !tabsItemsRef) {
      useFixedAddButton = false;
      syncBoundAddButtonRef();
      return;
    }

    const addButtonWidth = (inlineAddButtonRef?.offsetWidth ?? fixedAddButtonRef?.offsetWidth ?? 32) + 8;
    useFixedAddButton = tabsItemsRef.scrollWidth + addButtonWidth > tabsViewportRef.clientWidth;
    syncBoundAddButtonRef();
  }

  onMount(() => {
    updateAddButtonMode();

    if (!tabsViewportRef || !tabsTrackRef || !tabsItemsRef || typeof ResizeObserver === 'undefined') {
      return;
    }

    const observer = new ResizeObserver(() => {
      updateAddButtonMode();
    });

    observer.observe(tabsViewportRef);
    observer.observe(tabsTrackRef);
    observer.observe(tabsItemsRef);

    return () => observer.disconnect();
  });

  $effect(() => {
    items.length;
    activeId;
    queueMicrotask(() => updateAddButtonMode());
  });

  $effect(() => {
    useFixedAddButton;
    inlineAddButtonRef;
    fixedAddButtonRef;
    tabsItemsRef;
    syncBoundAddButtonRef();
  });
</script>

<header class="flex h-11 items-center border-b border-slate-200 bg-white px-4 select-none print:hidden dark:border-white/5 dark:bg-[#0c0c0e]">
  <div bind:this={tabsViewportRef} class="flex min-w-0 flex-1 self-end overflow-x-auto pb-0.5">
    <div bind:this={tabsTrackRef} class="flex min-w-max items-end gap-1">
      <div bind:this={tabsItemsRef} class="flex min-w-max items-end gap-1">
        {#each items as item (item.id)}
          <div class="group relative flex h-9 min-w-[150px] max-w-[220px] items-center">
            <button
              onclick={() => !item.placeholder && onSelect(item.id)}
              class={`flex h-full w-full items-center gap-2 rounded-t-xl border-x border-t px-3 pr-8 text-xs font-semibold transition-all ${
                activeId === item.id
                  ? 'mb-[-1px] border-slate-300 border-b-slate-50 bg-slate-50 text-slate-800 dark:border-white/10 dark:border-b-[#18181b] dark:bg-[#18181b] dark:text-white'
                  : 'border-transparent bg-transparent text-slate-500 hover:bg-slate-100 dark:text-zinc-400 dark:hover:bg-white/5'
              } ${item.placeholder ? 'italic text-slate-400 dark:text-zinc-500' : ''}`}
            >
              {#if item.indicatorClass}
                <span class={`h-2 w-2 rounded-full ${item.indicatorClass}`}></span>
              {/if}
              <span class="truncate">{item.name}</span>
            </button>

            {#if item.closable && onRemove}
              <button
                onclick={(event: MouseEvent) => { event.stopPropagation(); onRemove(item.id); }}
                class="absolute right-1 top-1/2 -translate-y-1/2 rounded p-1 opacity-0 transition-all hover:bg-red-100 hover:text-red-600 group-hover:opacity-100 dark:hover:bg-red-900/30"
                aria-label={`Fechar ${item.name}`}
              >
                <X size={12} strokeWidth={2.5} />
              </button>
            {/if}
          </div>
        {/each}
      </div>

      {#if !useFixedAddButton}
        <button
          bind:this={inlineAddButtonRef}
          onclick={onAdd}
          class="ml-2 flex h-9 w-9 shrink-0 items-center justify-center self-center rounded-lg border border-transparent text-slate-500 transition-colors hover:bg-slate-100 dark:text-zinc-400 dark:hover:bg-white/5"
          title={addLabel}
          aria-label={addLabel}
        >
          <Plus size={16} />
        </button>
      {/if}
    </div>
  </div>

  {#if useFixedAddButton}
    <button
      bind:this={fixedAddButtonRef}
      onclick={onAdd}
      class="ml-2 flex h-9 w-9 shrink-0 items-center justify-center self-center rounded-lg border border-transparent text-slate-500 transition-colors hover:bg-slate-100 dark:text-zinc-400 dark:hover:bg-white/5"
      title={addLabel}
      aria-label={addLabel}
    >
      <Plus size={16} />
    </button>
  {/if}
</header>
