<script lang="ts">
  import { onMount, onDestroy, untrack } from 'svelte';
  import uPlot from 'uplot';
  import 'uplot/dist/uPlot.min.css';
  import type { ChartSeries, ChartConfig } from '$lib/types/chart';

  interface Props {
    series: ChartSeries[];
    config: ChartConfig;
    theme: 'dark' | 'light';
    onRangeChange?: (xMin: number, xMax: number) => void;
  }

  let { series, config, theme, onRangeChange }: Props = $props();

  let wrapper: HTMLDivElement;
  let chart: uPlot | null = null;
  let renderTimer: ReturnType<typeof setInterval> | null = null;
  let prevLen = -1;
  let _mounted = false;
  let _panning = false;
  let _panStartX = 0;
  let _panScaleMin = 0;
  let _panScaleMax = 0;
  let tooltipEl: HTMLDivElement;

  const _scaleRef = {
    xMode: 'auto' as string,
    xMin: null as number | null,
    xMax: null as number | null,
    windowSize: 30,
    yMode: 'manual' as string,
    yMin: 0,
    yMax: 100,
  };

  function syncScaleRef() {
    _scaleRef.xMode = config.xMode;
    _scaleRef.xMin = config.xMin ?? null;
    _scaleRef.xMax = config.xMax ?? null;
    _scaleRef.windowSize = config.windowSize;
    _scaleRef.yMode = config.yMode;
    _scaleRef.yMin = config.yMin;
    _scaleRef.yMax = config.yMax;
  }

  const colors = $derived({
    bg: theme === 'dark' ? '#0c0c0e' : '#ffffff',
    grid: theme === 'dark' ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.07)',
    axis: theme === 'dark' ? 'rgba(255,255,255,0.12)' : 'rgba(0,0,0,0.12)',
    ticks: theme === 'dark' ? '#71717a' : '#6b7280',
  });

  function buildData(): uPlot.AlignedData {
    if (!series.length || !series[0].data.length) {
      return [[], ...series.map(() => [])] as uPlot.AlignedData;
    }
    const src = series[0].data;
    const n = src.length;
    const xs: number[] = new Array(n);
    for (let i = 0; i < n; i++) xs[i] = src[i].time;
    const cols: (number | null)[][] = [xs];
    for (const s of series) {
      if (s.visible && s.data.length > 0) {
        const col: number[] = new Array(n);
        for (let i = 0; i < n; i++) col[i] = s.data[i]?.[s.dataKey] ?? 0;
        cols.push(col);
      } else {
        cols.push(new Array(n).fill(null));
      }
    }
    return cols as uPlot.AlignedData;
  }

  function buildSeries(): uPlot.Series[] {
    const uSeries: uPlot.Series[] = [
      {
        label: 'Tempo',
        value: (_u: uPlot, v: number) => (v != null ? v.toFixed(2) + 's' : '--'),
      },
    ];
    for (const s of series) {
      const ser: uPlot.Series = {
        label: s.label,
        stroke: s.color,
        width: (s.strokeWidth || 2) / devicePixelRatio,
        dash: s.dashed ? [6, 4] : undefined,
        show: s.visible,
        value: (_u: uPlot, v: number | null) => (v != null ? v.toFixed(2) : '--'),
        points: { show: false },
      };
      if (s.type === 'step') ser.paths = uPlot.paths.stepped!({ align: 1 });
      if (s.type === 'area') ser.fill = s.color + '30';
      uSeries.push(ser);
    }
    return uSeries;
  }

  function buildOpts(w: number, h: number): uPlot.Options {
    return {
      width: w,
      height: h,
      padding: [8, 12, 0, 0],
      cursor: {
        show: config.showHover !== false,
        drag: { x: true, y: false, setScale: false },
      },
      legend: { show: false },
      scales: {
        x: {
          time: false,
          auto: true,
          range: (_u: uPlot, dataMin: number, dataMax: number): [number, number] => {
            const xPad = Math.max((dataMax - dataMin) * 0.03, 0.5);
            if (_scaleRef.xMode === 'sliding') {
              return [Math.max(0, dataMax - _scaleRef.windowSize), dataMax + xPad];
            }
            if (_scaleRef.xMode === 'manual' && _scaleRef.xMin != null && _scaleRef.xMax != null) {
              return [_scaleRef.xMin, _scaleRef.xMax];
            }
            return [Math.min(dataMin, 0), Math.max(dataMax, 1) + xPad];
          },
        },
        y: {
          auto: true,
          range: (_u: uPlot, dataMin: number, dataMax: number): [number, number] => {
            if (_scaleRef.yMode === 'manual') {
              return [_scaleRef.yMin, _scaleRef.yMax];
            }
            const pad = (dataMax - dataMin) * 0.05 || 1;
            return [dataMin - pad, dataMax + pad];
          },
        },
      },
      axes: [
        {
          stroke: colors.ticks,
          grid: { stroke: colors.grid, width: 1, show: config.showGrid },
          ticks: { stroke: colors.axis, width: 1 },
          font: '11px system-ui, sans-serif',
          values: (_u: uPlot, splits: number[]) => splits.map((v) => v.toFixed(1) + 's'),
          gap: 6,
          size: 32,
        },
        {
          stroke: colors.ticks,
          grid: { stroke: colors.grid, width: 1, show: config.showGrid },
          ticks: { stroke: colors.axis, width: 1 },
          font: '11px system-ui, sans-serif',
          gap: 6,
          size: 50,
        },
      ],
      series: buildSeries(),
      hooks: {
        setSelect: [
          (u: uPlot) => {
            const sel = u.select;
            if (sel.width > 2) {
              const xMin = u.posToVal(sel.left, 'x');
              const xMax = u.posToVal(sel.left + sel.width, 'x');
              onRangeChange?.(xMin, xMax);
            }
            u.setSelect({ left: 0, top: 0, width: 0, height: 0 }, false);
          },
        ],
        setCursor: [
          (u: uPlot) => {
            if (!tooltipEl) return;
            const idx = u.cursor.idx;
            if (idx == null || idx < 0) {
              tooltipEl.style.display = 'none';
              return;
            }
            const xVal = u.data[0][idx];
            if (xVal == null) {
              tooltipEl.style.display = 'none';
              return;
            }
            let html = `<div class="tt-time">${xVal.toFixed(2)}s</div>`;
            for (let i = 0; i < series.length; i++) {
              const s = series[i];
              if (!s.visible) continue;
              const val = u.data[i + 1]?.[idx];
              if (val == null) continue;
              html += `<div class="tt-row"><span class="tt-dot" style="background:${s.color}"></span><span class="tt-label">${s.label}</span><span class="tt-val">${val.toFixed(2)}</span></div>`;
            }
            tooltipEl.innerHTML = html;
            tooltipEl.style.display = 'block';
            const left = u.valToPos(xVal, 'x');
            const wrapRect = wrapper.getBoundingClientRect();
            const ttW = tooltipEl.offsetWidth;
            const ttH = tooltipEl.offsetHeight;
            const plotLeft = u.bbox.left / devicePixelRatio;
            const px = plotLeft + left;
            const flipX = px + ttW + 12 > wrapRect.width;
            const cursorTop = u.cursor.top ?? 0;
            const flipY = cursorTop - ttH - 12 < 0;
            tooltipEl.style.left = flipX ? `${px - ttW - 12}px` : `${px + 12}px`;
            tooltipEl.style.top = flipY ? `${cursorTop + 12}px` : `${cursorTop - ttH - 12}px`;
          },
        ],
      },
    };
  }

  function handleWheel(e: WheelEvent) {
    if (!chart || !wrapper) return;
    e.preventDefault();
    const rect = wrapper.getBoundingClientRect();
    const cursorX = e.clientX - rect.left;
    const xMin = chart.scales.x.min ?? 0;
    const xMax = chart.scales.x.max ?? 1;
    const range = xMax - xMin;
    const factor = e.deltaY > 0 ? 1.15 : 1 / 1.15;
    const newRange = range * factor;
    const ratio = Math.max(0, Math.min(1, cursorX / rect.width));
    const centerTime = xMin + range * ratio;
    let nMin = centerTime - newRange * ratio;
    let nMax = centerTime + newRange * (1 - ratio);
    const data = series[0]?.data;
    if (data?.length) {
      const dataMax = data[data.length - 1].time;
      if (nMin < 0) {
        nMax += -nMin;
        nMin = 0;
      }
      if (nMax > dataMax) {
        nMin -= nMax - dataMax;
        nMax = dataMax;
      }
      nMin = Math.max(0, nMin);
    }
    onRangeChange?.(nMin, nMax);
  }

  function handlePointerDown(e: PointerEvent) {
    if (!chart || !wrapper) return;
    if (e.button === 1 || (e.button === 0 && e.shiftKey)) {
      e.preventDefault();
      _panning = true;
      _panStartX = e.clientX;
      _panScaleMin = chart.scales.x.min ?? 0;
      _panScaleMax = chart.scales.x.max ?? 1;
      wrapper.setPointerCapture(e.pointerId);
      wrapper.style.cursor = 'grabbing';
    }
  }

  function handlePointerMove(e: PointerEvent) {
    if (!_panning || !chart || !wrapper) return;
    const rect = wrapper.getBoundingClientRect();
    const pxRange = rect.width;
    const scaleRange = _panScaleMax - _panScaleMin;
    const dx = e.clientX - _panStartX;
    const dt = -(dx / pxRange) * scaleRange;
    let nMin = _panScaleMin + dt;
    let nMax = _panScaleMax + dt;
    const data = series[0]?.data;
    if (data?.length) {
      const dataMax = data[data.length - 1].time;
      if (nMin < 0) {
        nMax += -nMin;
        nMin = 0;
      }
      if (nMax > dataMax) {
        nMin -= nMax - dataMax;
        nMax = dataMax;
      }
      nMin = Math.max(0, nMin);
    }
    onRangeChange?.(nMin, nMax);
  }

  function handlePointerUp(e: PointerEvent) {
    if (!_panning) return;
    _panning = false;
    if (wrapper) {
      wrapper.releasePointerCapture(e.pointerId);
      wrapper.style.cursor = '';
    }
  }

  function initChart() {
    if (!wrapper) return;
    if (chart) {
      chart.destroy();
      chart = null;
    }
    const rect = wrapper.getBoundingClientRect();
    if (rect.width < 10 || rect.height < 10) return;
    syncScaleRef();
    const opts = buildOpts(Math.floor(rect.width), Math.floor(rect.height));
    const data = buildData();
    chart = new uPlot(opts, data, wrapper);
    prevLen = series.length > 0 ? series[0].data.length : 0;
  }


  function updateChart() {
    if (!chart || !wrapper) return;
    const rect = wrapper.getBoundingClientRect();
    if (rect.width < 10 || rect.height < 10) return;

    syncScaleRef();
    const data = buildData();
    chart.setData(data, true);

    const w = Math.floor(rect.width);
    const h = Math.floor(rect.height);
    if (Math.abs(chart.width - w) > 2 || Math.abs(chart.height - h) > 2) {
      chart.setSize({ width: w, height: h });
    }
  }

  onMount(() => {
    initChart();
    _mounted = true;
    renderTimer = setInterval(() => {
      const n = series.length > 0 ? series[0].data.length : 0;
      if (n !== prevLen) {
        prevLen = n;
        updateChart();
      }
    }, 33);
    const ro = new ResizeObserver(() => {
      if (chart && wrapper) {
        const rect = wrapper.getBoundingClientRect();
        const w = Math.floor(rect.width);
        const h = Math.floor(rect.height);
        if (w > 10 && h > 10) {
          chart.setSize({ width: w, height: h });
        }
      }
    });
    if (wrapper) ro.observe(wrapper);
    return () => ro.disconnect();
  });

  onDestroy(() => {
    if (renderTimer) clearInterval(renderTimer);
    if (chart) {
      chart.destroy();
      chart = null;
    }
  });

  $effect(() => {
    const _ = [
      config.yMode,
      config.showGrid,
      theme,
      ...series.map((s) => `${s.visible}|${s.color}`),
    ];
    untrack(() => {
      if (_mounted && chart) initChart();
    });
  });

  $effect(() => {
    const _ = [
      config.xMode,
      config.xMin,
      config.xMax,
      config.windowSize,
      config.yMin,
      config.yMax,
    ];
    untrack(() => {
      if (_mounted && chart) updateChart();
    });
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  bind:this={wrapper}
  class="w-full h-full uplot-wrapper"
  style="min-height: 100px; background: {colors.bg};"
  onwheel={handleWheel}
  onpointerdown={handlePointerDown}
  onpointermove={handlePointerMove}
  onpointerup={handlePointerUp}
>
  <div bind:this={tooltipEl} class="chart-tooltip"></div>
</div>

<style>
  .uplot-wrapper {
    position: relative;
  }
  .uplot-wrapper :global(.u-over) {
    cursor: crosshair !important;
  }
  .uplot-wrapper :global(.u-legend) {
    display: none !important;
  }
  .uplot-wrapper :global(.u-cursor-pt) {
    border-radius: 50%;
  }
  .uplot-wrapper :global(.u-select) {
    background: rgba(59, 130, 246, 0.1) !important;
    border: 1px solid rgba(59, 130, 246, 0.3) !important;
  }
  .chart-tooltip {
    display: none;
    position: absolute;
    z-index: 100;
    pointer-events: none;
    padding: 8px 10px;
    border-radius: 8px;
    font-size: 11px;
    font-family: ui-monospace, monospace;
    line-height: 1.5;
    white-space: nowrap;
    background: rgba(24, 24, 27, 0.92);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: #e4e4e7;
    backdrop-filter: blur(8px);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  }
  :global(.chart-tooltip .tt-time) {
    font-weight: 700;
    color: #a1a1aa;
    margin-bottom: 4px;
    font-size: 10px;
    letter-spacing: 0.05em;
  }
  :global(.chart-tooltip .tt-row) {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  :global(.chart-tooltip .tt-dot) {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  :global(.chart-tooltip .tt-label) {
    color: #a1a1aa;
    flex: 1;
  }
  :global(.chart-tooltip .tt-val) {
    font-weight: 600;
    color: #fafafa;
  }
</style>
