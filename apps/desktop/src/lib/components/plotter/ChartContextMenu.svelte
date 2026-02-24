<script lang="ts">
  import { Palette, Eye, EyeOff } from 'lucide-svelte';

  let {
    visible = $bindable(false),
    x,
    y,
    chartState,
    lineColors,
    onClose
  }: {
    visible: boolean;
    x: number;
    y: number;
    chartState: any;
    lineColors: { pv: string; sp: string; mv: string };
    onClose: () => void;
  } = $props();
</script>

{#if visible}
  <div
    class="absolute z-50 bg-white dark:bg-[#18181b] border border-slate-200 dark:border-white/10 rounded-lg shadow-2xl p-3 min-w-[240px] flex flex-col gap-2"
    style="top: {y}px; left: {x}px"
    onclick={(e: MouseEvent) => e.stopPropagation()}
    onkeydown={(e: KeyboardEvent) => e.key === 'Escape' && onClose()}
    onmouseleave={() => setTimeout(onClose, 1000)}
    role="menu"
    tabindex="-1"
  >
    <div>
      <div class="px-1 text-[10px] font-bold uppercase text-slate-400 tracking-wider mb-1 flex justify-between items-center">
        Eixo X (Tempo) <span class="text-[9px] bg-slate-100 dark:bg-white/5 px-1 rounded">{chartState.xMode}</span>
      </div>
      <div class="flex gap-1 mb-1">
        <button onclick={() => chartState.xMode = 'auto'} class={`flex-1 text-[10px] font-bold py-1 px-2 rounded border transition-colors ${chartState.xMode === 'auto' ? 'bg-blue-600 text-white border-blue-600' : 'bg-slate-50 dark:bg-white/5 text-slate-500 border-slate-200 dark:border-white/10 hover:bg-slate-100 dark:hover:bg-white/10'}`}>Auto</button>
        <button onclick={() => chartState.xMode = 'sliding'} class={`flex-1 text-[10px] font-bold py-1 px-2 rounded border transition-colors ${chartState.xMode === 'sliding' ? 'bg-blue-600 text-white border-blue-600' : 'bg-slate-50 dark:bg-white/5 text-slate-500 border-slate-200 dark:border-white/10 hover:bg-slate-100 dark:hover:bg-white/10'}`}>Janela</button>
        <button onclick={() => chartState.xMode = 'manual'} class={`flex-1 text-[10px] font-bold py-1 px-2 rounded border transition-colors ${chartState.xMode === 'manual' ? 'bg-blue-600 text-white border-blue-600' : 'bg-slate-50 dark:bg-white/5 text-slate-500 border-slate-200 dark:border-white/10 hover:bg-slate-100 dark:hover:bg-white/10'}`}>Manual</button>
      </div>
      {#if chartState.xMode === 'sliding'}
        <div class="flex items-center gap-2 px-1">
          <span class="text-xs text-slate-500">Janela (s):</span>
          <input type="number" class="w-16 h-6 text-xs bg-slate-50 dark:bg-black/20 border border-slate-200 dark:border-white/10 rounded px-1" bind:value={chartState.windowSize} />
        </div>
      {/if}
    </div>
    <div class="border-t border-slate-100 dark:border-white/5"></div>
    <div>
      <div class="px-1 text-[10px] font-bold uppercase text-slate-400 tracking-wider mb-1 flex items-center gap-2">
        <Palette size={12} /> Variáveis
      </div>
      <div class="space-y-1">
        <div class="flex items-center justify-between px-2 py-1.5 rounded hover:bg-slate-100 dark:hover:bg-white/5 group">
          <div class="flex items-center gap-2">
            <button onclick={() => chartState.visible.pv = !chartState.visible.pv} class="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 transition-colors">
              {#if chartState.visible.pv}<Eye size={14} />{:else}<EyeOff size={14} />{/if}
            </button>
            <span class="text-xs text-slate-500 dark:text-slate-400 font-medium">PV (Process)</span>
          </div>
          <div class="flex items-center gap-2">
            <input type="text" bind:value={lineColors.pv} class="w-16 h-5 text-[10px] font-mono bg-transparent border border-slate-200 dark:border-white/10 rounded px-1 text-slate-600 dark:text-slate-300 focus:outline-none focus:border-blue-500 text-right uppercase" />
            <div class="relative w-5 h-5 rounded-full overflow-hidden border border-slate-200 dark:border-white/20 shadow-sm cursor-pointer hover:scale-110 transition-transform" style="background-color: {lineColors.pv}">
              <input type="color" bind:value={lineColors.pv} class="absolute -top-1/2 -left-1/2 w-[200%] h-[200%] cursor-pointer p-0 m-0 border-0 opacity-0" />
            </div>
          </div>
        </div>
        <div class="flex items-center justify-between px-2 py-1.5 rounded hover:bg-slate-100 dark:hover:bg-white/5 group">
          <div class="flex items-center gap-2">
            <button onclick={() => chartState.visible.sp = !chartState.visible.sp} class="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 transition-colors">
              {#if chartState.visible.sp}<Eye size={14} />{:else}<EyeOff size={14} />{/if}
            </button>
            <span class="text-xs text-slate-500 dark:text-slate-400 font-medium">SP (Setpoint)</span>
          </div>
          <div class="flex items-center gap-2">
            <input type="text" bind:value={lineColors.sp} class="w-16 h-5 text-[10px] font-mono bg-transparent border border-slate-200 dark:border-white/10 rounded px-1 text-slate-600 dark:text-slate-300 focus:outline-none focus:border-blue-500 text-right uppercase" />
            <div class="relative w-5 h-5 rounded-full overflow-hidden border border-slate-200 dark:border-white/20 shadow-sm cursor-pointer hover:scale-110 transition-transform" style="background-color: {lineColors.sp}">
              <input type="color" bind:value={lineColors.sp} class="absolute -top-1/2 -left-1/2 w-[200%] h-[200%] cursor-pointer p-0 m-0 border-0 opacity-0" />
            </div>
          </div>
        </div>
        <div class="flex items-center justify-between px-2 py-1.5 rounded hover:bg-slate-100 dark:hover:bg-white/5 group">
          <div class="flex items-center gap-2">
            <button onclick={() => chartState.visible.mv = !chartState.visible.mv} class="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 transition-colors">
              {#if chartState.visible.mv}<Eye size={14} />{:else}<EyeOff size={14} />{/if}
            </button>
            <span class="text-xs text-slate-500 dark:text-slate-400 font-medium">MV (Output)</span>
          </div>
          <div class="flex items-center gap-2">
            <input type="text" bind:value={lineColors.mv} class="w-16 h-5 text-[10px] font-mono bg-transparent border border-slate-200 dark:border-white/10 rounded px-1 text-slate-600 dark:text-slate-300 focus:outline-none focus:border-blue-500 text-right uppercase" />
            <div class="relative w-5 h-5 rounded-full overflow-hidden border border-slate-200 dark:border-white/20 shadow-sm cursor-pointer hover:scale-110 transition-transform" style="background-color: {lineColors.mv}">
              <input type="color" bind:value={lineColors.mv} class="absolute -top-1/2 -left-1/2 w-[200%] h-[200%] cursor-pointer p-0 m-0 border-0 opacity-0" />
            </div>
          </div>
        </div>
      </div>
    </div>
    <div class="border-t border-slate-100 dark:border-white/5"></div>
    <div>
      <div class="px-1 text-[10px] font-bold uppercase text-slate-400 tracking-wider mb-1 flex justify-between items-center">
        Eixo Y <span class="text-[9px] bg-slate-100 dark:bg-white/5 px-1 rounded">{chartState.yMode}</span>
      </div>
      <div class="flex gap-1 mb-2">
        <button onclick={() => chartState.yMode = 'auto'} class={`flex-1 text-[10px] font-bold py-1 px-2 rounded border transition-colors ${chartState.yMode === 'auto' ? 'bg-blue-600 text-white border-blue-600' : 'bg-slate-50 dark:bg-white/5 text-slate-500 border-slate-200 dark:border-white/10 hover:bg-slate-100 dark:hover:bg-white/10'}`}>Auto</button>
        <button onclick={() => chartState.yMode = 'manual'} class={`flex-1 text-[10px] font-bold py-1 px-2 rounded border transition-colors ${chartState.yMode === 'manual' ? 'bg-blue-600 text-white border-blue-600' : 'bg-slate-50 dark:bg-white/5 text-slate-500 border-slate-200 dark:border-white/10 hover:bg-slate-100 dark:hover:bg-white/10'}`}>Manual</button>
      </div>
      {#if chartState.yMode === 'manual'}
        <div class="flex gap-2 px-1">
          <input type="number" placeholder="Min" class="w-full h-6 text-xs bg-slate-50 dark:bg-black/20 border border-slate-200 dark:border-white/10 rounded px-1" bind:value={chartState.yMin} />
          <input type="number" placeholder="Max" class="w-full h-6 text-xs bg-slate-50 dark:bg-black/20 border border-slate-200 dark:border-white/10 rounded px-1" bind:value={chartState.yMax} />
        </div>
      {/if}
    </div>
  </div>
{/if}
