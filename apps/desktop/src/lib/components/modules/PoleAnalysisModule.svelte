<script lang="ts">
  interface Props {
    theme: 'dark' | 'light';
  }

  let { theme } = $props();

  const data = [
    { x: -0.5, y: 0.5, type: 'Polo', z: 100 },
    { x: -0.5, y: -0.5, type: 'Polo', z: 100 },
    { x: -2, y: 0, type: 'Zero', z: 100 }
  ];

  const gridColor = $derived(theme === 'dark' ? '#27272a' : '#e2e8f0');
  const axisColor = $derived(theme === 'dark' ? '#94a3b8' : '#94a3b8');
</script>

<div class="p-8 h-full flex flex-col bg-slate-50 dark:bg-zinc-950">
  <header class="mb-6 pb-4 border-b border-slate-200 dark:border-white/5">
    <h2 class="text-2xl font-bold text-slate-800 dark:text-zinc-100">Análise de Lugar das Raízes</h2>
  </header>

  <div class="flex-1 bg-white dark:bg-zinc-900 rounded-xl border border-slate-200 dark:border-white/5 relative flex items-center justify-center overflow-hidden shadow-sm">
    <div class="absolute top-6 right-6 bg-white/90 dark:bg-zinc-900/90 backdrop-blur p-5 rounded-xl border border-slate-200 dark:border-white/10 w-72 z-10 shadow-xl text-slate-900 dark:text-white">
      <h4 class="font-bold text-[10px] text-slate-600 dark:text-slate-400 uppercase tracking-widest mb-3">Função de Transferência</h4>
      <div class="font-mono text-sm text-slate-700 dark:text-zinc-300 bg-white dark:bg-white/5 p-3 rounded-lg text-center border border-slate-200 dark:border-white/5">
        G(s) = <span class="text-blue-500 font-bold">s + 2</span> / <span class="text-amber-500 font-bold">(s² + s + 1)</span>
      </div>
    </div>

    <div class="w-full h-full p-8">
      <svg viewBox="-6 -6 12 12" class="w-full h-full max-w-[600px] max-h-[600px] mx-auto">
        {#each [-5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5] as i}
          <line x1={i} y1="-5" x2={i} y2="5" stroke={gridColor} stroke-width="0.02" stroke-dasharray="0.1" />
          <line x1="-5" y1={i} x2="5" y2={i} stroke={gridColor} stroke-width="0.02" stroke-dasharray="0.1" />
        {/each}

        <line x1="-5" y1="0" x2="5" y2="0" stroke={axisColor} stroke-width="0.03" />
        <line x1="0" y1="-5" x2="0" y2="5" stroke={axisColor} stroke-width="0.03" />

        <text x="4.5" y="0.4" font-size="0.3" fill={axisColor}>Re</text>
        <text x="0.2" y="-4.5" font-size="0.3" fill={axisColor}>Im</text>

        <circle cx="0" cy="0" r="1" fill="none" stroke={theme === 'dark' ? '#374151' : '#d1d5db'} stroke-width="0.02" stroke-dasharray="0.1" />

        {#each data as point}
          {#if point.type === 'Polo'}
            <g transform="translate({point.x}, {-point.y})">
              <line x1="-0.15" y1="-0.15" x2="0.15" y2="0.15" stroke="#fbbf24" stroke-width="0.08" />
              <line x1="-0.15" y1="0.15" x2="0.15" y2="-0.15" stroke="#fbbf24" stroke-width="0.08" />
            </g>
          {:else}
            <circle cx={point.x} cy={-point.y} r="0.15" fill="none" stroke="#3b82f6" stroke-width="0.08" />
          {/if}
        {/each}
      </svg>
    </div>

    <div class="absolute bottom-6 left-6 bg-white/90 dark:bg-zinc-900/90 backdrop-blur p-4 rounded-xl border border-slate-200 dark:border-white/10 z-10 shadow-xl">
      <h4 class="font-bold text-[10px] text-slate-600 dark:text-slate-400 uppercase tracking-widest mb-3">Legenda</h4>
      <div class="space-y-2 text-sm">
        {#each data as point}
          <div class="flex items-center gap-3">
            {#if point.type === 'Polo'}
              <span class="text-amber-400 font-bold text-lg">×</span>
            {:else}
              <span class="w-3 h-3 rounded-full border-2 border-blue-400"></span>
            {/if}
            <span class="text-slate-600 dark:text-slate-300">{point.type}: ({point.x}, {point.y}j)</span>
          </div>
        {/each}
      </div>
    </div>
  </div>
</div>
