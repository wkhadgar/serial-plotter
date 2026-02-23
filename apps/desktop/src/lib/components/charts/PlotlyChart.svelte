<script lang="ts">
  import { onMount, onDestroy, untrack } from 'svelte';
  // @ts-ignore
  import Plotly from 'plotly.js-dist-min';
  import type { Config, PlotRelayoutEvent } from 'plotly.js';

  interface DataPoint { time: number; [key: string]: number; }

  interface DataSeries {
    key: string;
    label: string;
    color: string;
    visible: boolean;
    data: DataPoint[];
    dataKey: string;
    type?: 'line' | 'step' | 'area';
    strokeWidth?: number;
    dashed?: boolean;
  }

  interface ChartConfig {
    yMin: number;
    yMax: number;
    yMode: 'auto' | 'manual';
    xMode: 'auto' | 'sliding' | 'manual';
    windowSize: number;
    xMin?: number | null;
    xMax?: number | null;
    showGrid: boolean;
    showLimits: boolean;
    limitHigh?: number;
    limitLow?: number;
    showHover?: boolean;
    showLegend?: boolean;
    showModeBar?: boolean;
  }

  interface Props {
    series: DataSeries[];
    config: ChartConfig;
    theme: 'dark' | 'light';
    height?: string;
    onZoom?: (factor: number, centerTime: number) => void;
    onPan?: (deltaTime: number) => void;
    onRangeChange?: (xMin: number, xMax: number) => void;
  }

  let { series, config, theme, height = '100%', onZoom, onPan, onRangeChange }: Props = $props();

  let container: HTMLDivElement;
  let plotReady = false;
  let renderTimer: ReturnType<typeof setInterval> | null = null;
  let prevLen = -1;

  // ── Theme colors ──
  const colors = $derived({
    bg:    theme === 'dark' ? '#0c0c0e' : '#ffffff',
    grid:  theme === 'dark' ? '#1f1f23' : '#e5e7eb',
    text:  theme === 'dark' ? '#71717a' : '#6b7280',
    hover: theme === 'dark' ? '#27272a' : '#f8fafc',
    hText: theme === 'dark' ? '#e4e4e7' : '#1e293b'
  });

  // ── Fast stride downsample ──
  const MAX_PTS = 600;
  function ds(data: DataPoint[], key: string): { x: number[]; y: number[] } {
    const n = data.length;
    if (n === 0) return { x: [], y: [] };
    if (n <= MAX_PTS) {
      const x = new Array(n), y = new Array(n);
      for (let i = 0; i < n; i++) { x[i] = data[i].time; y[i] = data[i][key]; }
      return { x, y };
    }
    const step = (n - 1) / (MAX_PTS - 1);
    const x = new Array(MAX_PTS), y = new Array(MAX_PTS);
    for (let i = 0; i < MAX_PTS - 1; i++) {
      const j = Math.round(i * step);
      x[i] = data[j].time; y[i] = data[j][key];
    }
    x[MAX_PTS - 1] = data[n - 1].time;
    y[MAX_PTS - 1] = data[n - 1][key];
    return { x, y };
  }

  // ── X range based on mode ──
  function xRange(): [number, number] | undefined {
    if (!series.length || !series[0].data.length) return undefined;
    const last = series[0].data[series[0].data.length - 1].time;
    if (config.xMode === 'sliding') return [Math.max(0, last - config.windowSize), last];
    if (config.xMode === 'manual' && config.xMin != null && config.xMax != null) return [config.xMin, config.xMax];
    return [0, Math.max(last, 1)];
  }

  // ── Build Plotly traces ──
  function traces(): Partial<Plotly.PlotData>[] {
    return series.filter(s => s.visible && s.data.length > 0).map(s => {
      const { x, y } = ds(s.data, s.dataKey);
      const t: Partial<Plotly.PlotData> = {
        x, y, name: s.label,
        type: 'scattergl', mode: 'lines',
        line: {
          color: s.color, width: s.strokeWidth || 2,
          dash: s.dashed ? 'dash' : 'solid',
          shape: s.type === 'step' ? 'hv' : 'linear'
        },
        hovertemplate: config.showHover !== false
          ? `<b>${s.label}</b><br>%{x:.2f}s → %{y:.2f}<extra></extra>` : undefined,
        hoverinfo: config.showHover !== false ? 'all' : 'skip',
        showlegend: config.showLegend ?? false
      };
      if (s.type === 'area') { t.fill = 'tozeroy'; t.fillcolor = s.color + '30'; }
      return t;
    });
  }

  // ── Limit shapes ──
  function shapes(): Partial<Plotly.Shape>[] {
    if (!config.showLimits) return [];
    const r = xRange(); if (!r) return [];
    const out: Partial<Plotly.Shape>[] = [];
    const line = (yv: number) => out.push({
      type: 'line', x0: r[0], x1: r[1], y0: yv, y1: yv,
      line: { color: '#ef4444', width: 1, dash: 'dash' }
    });
    if (config.limitHigh != null) line(config.limitHigh);
    if (config.limitLow != null) line(config.limitLow);
    return out;
  }

  // ── Layout ──
  function layout(): Partial<Plotly.Layout> {
    return {
      autosize: true,
      margin: { l: 50, r: 16, t: 6, b: 32 },
      paper_bgcolor: colors.bg, plot_bgcolor: colors.bg,
      font: { color: colors.text, size: 11, family: 'system-ui, sans-serif' },
      xaxis: {
        range: xRange(),
        gridcolor: colors.grid, gridwidth: 0.5,
        showgrid: config.showGrid, zeroline: false,
        tickformat: '.1f', ticksuffix: 's',
        fixedrange: false,
        showline: true, linewidth: 1, linecolor: colors.grid
      },
      yaxis: {
        range: config.yMode === 'manual' ? [config.yMin, config.yMax] : undefined,
        autorange: config.yMode === 'auto',
        gridcolor: colors.grid, gridwidth: 0.5,
        showgrid: config.showGrid, zeroline: false,
        fixedrange: false,
        showline: true, linewidth: 1, linecolor: colors.grid
      },
      shapes: shapes(),
      hovermode: config.showHover !== false ? 'x unified' : false,
      hoverlabel: {
        bgcolor: colors.hover,
        font: { color: colors.hText, size: 11, family: 'monospace' },
        bordercolor: colors.grid
      },
      dragmode: 'pan',
      legend: { orientation: 'h', y: 1.02, x: 0.5, xanchor: 'center', font: { size: 10, color: colors.text } }
    };
  }

  const cfg: Partial<Config> = {
    responsive: true, displayModeBar: false,
    scrollZoom: true, doubleClick: 'reset',
    staticPlot: false, displaylogo: false
  };

  // ── Render: Plotly.react does internal diffing, safe to call often ──
  function render() {
    if (!container) return;
    const t = traces();
    if (!t.length && !plotReady) return;
    const l = layout();
    if (!plotReady) {
      Plotly.newPlot(container, t, l, cfg).then(() => {
        plotReady = true;
        (container as any).on('plotly_relayout', (ev: PlotRelayoutEvent) => {
          if (ev['xaxis.range[0]'] != null && ev['xaxis.range[1]'] != null) {
            onRangeChange?.(ev['xaxis.range[0]'] as number, ev['xaxis.range[1]'] as number);
          }
          if (ev['xaxis.autorange'] === true) {
            const d = series[0]?.data;
            if (d?.length) onRangeChange?.(0, d[d.length - 1].time);
          }
        });
      });
    } else {
      Plotly.react(container, t, l, cfg);
    }
  }

  // ── Lifecycle ──
  onMount(() => {
    render();
    // 5 Hz render loop: cheap length check, only render when data changed
    renderTimer = setInterval(() => {
      const n = series.length > 0 ? series[0].data.length : 0;
      if (n !== prevLen) { prevLen = n; render(); }
    }, 200);
    const ro = new ResizeObserver(() => { if (plotReady && container) Plotly.Plots.resize(container); });
    if (container) ro.observe(container);
    return () => ro.disconnect();
  });

  onDestroy(() => {
    if (renderTimer) clearInterval(renderTimer);
    if (plotReady && container) { Plotly.purge(container); plotReady = false; }
  });

  // Config/theme changed → re-render immediately (but do NOT track data reads
  // inside render, otherwise every data.push triggers an extra Plotly.react).
  $effect(() => {
    const _ = [config.xMode, config.yMode, config.yMin, config.yMax,
               config.windowSize, config.showGrid, config.showLimits,
               config.xMin, config.xMax, theme];
    if (plotReady) untrack(() => render());
  });
</script>

<div bind:this={container} class="w-full h-full" style="min-height: 100px;"></div>