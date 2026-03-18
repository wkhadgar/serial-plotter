<script lang="ts">
  import PlotterWorkspaceModule from './PlotterWorkspaceModule.svelte';

  let { plants, activePlantId, theme, active = true, showControllerPanel = $bindable(false) } = $props();
</script>

<PlotterWorkspaceModule
  {plants}
  {activePlantId}
  {theme}
  {active}
  bind:showControllerPanel
/>
