<script lang="ts">
  import type { Plant } from '$lib/types/plant';
  import PlantAddMenu from './PlantAddMenu.svelte';
  import WorkspaceTabs, { type WorkspaceTabItem } from '../ui/WorkspaceTabs.svelte';

  let { 
    plants,
    activePlantId,
    onSelect,
    onOpenFile,
    onCreateNew,
    onRemove
  }: {
    plants: Plant[];
    activePlantId: string;
    onSelect: (id: string) => void;
    onOpenFile: () => void;
    onCreateNew: () => void;
    onRemove: (id: string) => void;
  } = $props();

  let menuVisible = $state(false);
  let menuX = $state(0);
  let menuY = $state(0);
  let addButtonRef = $state<HTMLButtonElement | undefined>(undefined);
  const EMPTY_TAB_ID = '__unamed__';

  const tabItems = $derived.by<WorkspaceTabItem[]>(() => {
    if (plants.length === 0) {
      return [
        {
          id: EMPTY_TAB_ID,
          name: 'Unamed',
          closable: false,
          placeholder: true,
          indicatorClass: 'bg-slate-300 dark:bg-zinc-600',
        },
      ];
    }

    return plants.map((plant) => ({
      id: plant.id,
      name: plant.name,
      closable: true,
      indicatorClass: plant.connected ? 'bg-emerald-500' : 'bg-slate-300 dark:bg-zinc-700',
    }));
  });

  const resolvedActiveId = $derived(plants.length === 0 ? EMPTY_TAB_ID : activePlantId);

  function handleAddClick() {
    if (!addButtonRef) return;
    const rect = addButtonRef.getBoundingClientRect();
    const menuWidth = 200;
    const viewportWidth = window.innerWidth;
    const preferredLeft = rect.left;
    const overflowRight = preferredLeft + menuWidth > viewportWidth - 12;

    menuX = overflowRight ? Math.max(12, rect.right - menuWidth) : preferredLeft;
    menuY = rect.bottom + 4;
    menuVisible = true;
  }
</script>

<WorkspaceTabs
  items={tabItems}
  activeId={resolvedActiveId}
  onSelect={onSelect}
  onAdd={handleAddClick}
  onRemove={onRemove}
  addLabel="Nova planta"
  bind:addButtonRef
/>

<PlantAddMenu
  visible={menuVisible}
  x={menuX}
  y={menuY}
  onClose={() => menuVisible = false}
  {onOpenFile}
  {onCreateNew}
/>
