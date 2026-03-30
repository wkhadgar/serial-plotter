<script lang="ts">
  import WorkspaceTabs, { type WorkspaceTabItem } from '../ui/WorkspaceTabs.svelte';

  interface TabItem {
    id: string;
    name: string;
  }

  let {
    tabs = [],
    activeTabId,
    onSelect,
    onAdd,
    onRemove,
  }: {
    tabs: TabItem[];
    activeTabId: string;
    onSelect: (id: string) => void;
    onAdd: () => void;
    onRemove: (id: string) => void;
  } = $props();

  const items = $derived.by<WorkspaceTabItem[]>(() =>
    tabs.map((tab) => ({
      id: tab.id,
      name: tab.name,
      closable: true,
    }))
  );
</script>

<WorkspaceTabs
  {items}
  activeId={activeTabId}
  onSelect={onSelect}
  onAdd={onAdd}
  onRemove={onRemove}
  addLabel="Nova análise"
/>
