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
  let renderPollTimer: number | null = null;
  let prevDataSignature = '';
  let _mounted = false;
  let _panning = false;
  let _pointerInside = false;
  let _pendingDeferredUpdate = false;
  let _localManualRange: { min: number; max: number } | null = null;
  let _lastResolvedXMode: 'auto' | 'sliding' | 'manual' = 'auto';
  let _panStartX = 0;
  let _panScaleMin = 0;
  let _panScaleMax = 0;
  let tooltipEl: HTMLDivElement;
  const MAX_RENDER_POINTS = 2400;
  let rangeChangeRaf: number | null = null;
  let pendingRangeChange: { xMin: number; xMax: number } | null = null;
  let lastAppliedXRange: { min: number; max: number } | null = null;
  let lastAppliedYRange: { min: number; max: number } | null = null;

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
    const localRange = _localManualRange;

    _scaleRef.xMode = localRange ? 'manual' : config.xMode;
    _scaleRef.xMin = localRange?.min ?? config.xMin ?? null;
    _scaleRef.xMax = localRange?.max ?? config.xMax ?? null;
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

  function getDataSignature(): string {
    const source = series[0]?.data;

    if (!source?.length) {
      return '0';
    }

    syncScaleRef();
    const indices = buildRenderIndices(source);
    if (!indices.length) {
      return `${_scaleRef.xMode}:${_scaleRef.xMin ?? 'na'}:${_scaleRef.xMax ?? 'na'}:${_scaleRef.windowSize}:0`;
    }

    const firstIndex = indices[0];
    const lastIndex = indices[indices.length - 1];
    return [
      _scaleRef.xMode,
      _scaleRef.xMin ?? 'na',
      _scaleRef.xMax ?? 'na',
      _scaleRef.windowSize,
      indices.length,
      source[firstIndex]?.time ?? 0,
      source[lastIndex]?.time ?? 0,
    ].join(':');
  }

  const seriesVisualSignature = $derived.by(() =>
    series.map((s) => `${s.key}:${s.visible ? 1 : 0}:${s.color}:${s.label}`).join('|')
  );

  function queueRangeChange(xMin: number, xMax: number) {
    if (!onRangeChange) return;

    const nextMin = Math.min(xMin, xMax);
    const nextMax = Math.max(xMin, xMax);
    if (!Number.isFinite(nextMin) || !Number.isFinite(nextMax) || nextMax - nextMin <= 0.0001) {
      return;
    }

    pendingRangeChange = { xMin: nextMin, xMax: nextMax };
    if (rangeChangeRaf !== null) {
      return;
    }

    rangeChangeRaf = requestAnimationFrame(() => {
      rangeChangeRaf = null;
      const nextRange = pendingRangeChange;
      pendingRangeChange = null;
      if (nextRange) {
        onRangeChange?.(nextRange.xMin, nextRange.xMax);
      }
    });
  }

  function findLowerBound(data: typeof series[number]['data'], target: number): number {
    let low = 0;
    let high = data.length;

    while (low < high) {
      const mid = Math.floor((low + high) / 2);

      if ((data[mid]?.time ?? 0) < target) {
        low = mid + 1;
      } else {
        high = mid;
      }
    }

    return low;
  }

  function findUpperBound(data: typeof series[number]['data'], target: number): number {
    let low = 0;
    let high = data.length;

    while (low < high) {
      const mid = Math.floor((low + high) / 2);

      if ((data[mid]?.time ?? 0) <= target) {
        low = mid + 1;
      } else {
        high = mid;
      }
    }

    return low;
  }

  function buildRenderIndices(source: typeof series[number]['data']): number[] {
    if (source.length === 0) {
      return [];
    }

    if (_scaleRef.xMode === 'manual') {
      const count = source.length;

      if (count <= MAX_RENDER_POINTS) {
        const indices = new Array<number>(count);
        for (let index = 0; index < count; index += 1) {
          indices[index] = index;
        }
        return indices;
      }

      const step = Math.max(1, Math.floor(count / (MAX_RENDER_POINTS - 1)));
      const indices: number[] = [];

      for (let index = 0; index < count; index += step) {
        indices.push(index);
      }

      if (indices[indices.length - 1] !== count - 1) {
        indices.push(count - 1);
      }

      return indices;
    }

    const firstTime = source[0].time;
    const lastTime = source[source.length - 1].time;
    const rangeStart =
      _scaleRef.xMode === 'sliding'
        ? Math.max(firstTime, lastTime - _scaleRef.windowSize)
        : _scaleRef.xMode === 'manual' && _scaleRef.xMin != null
          ? _scaleRef.xMin
          : firstTime;
    const rangeEnd =
      _scaleRef.xMode === 'manual' && _scaleRef.xMax != null
        ? _scaleRef.xMax
        : lastTime;

    const startIndex = Math.min(source.length - 1, findLowerBound(source, rangeStart));
    const endIndex = Math.min(source.length, Math.max(startIndex + 1, findUpperBound(source, rangeEnd)));
    const count = endIndex - startIndex;

    if (count <= MAX_RENDER_POINTS) {
      const indices = new Array<number>(count);
      for (let index = 0; index < count; index += 1) {
        indices[index] = startIndex + index;
      }
      return indices;
    }

    const step = Math.max(1, Math.floor(count / (MAX_RENDER_POINTS - 1)));
    const indices: number[] = [];

    for (let index = startIndex; index < endIndex; index += step) {
      indices.push(index);
    }

    if (indices[indices.length - 1] !== endIndex - 1) {
      indices.push(endIndex - 1);
    }

    return indices;
  }

  function buildData(): uPlot.AlignedData {
    if (!series.length || !series[0].data.length) {
      return [[], ...series.map(() => [])] as uPlot.AlignedData;
    }

    syncScaleRef();
    const src = series[0].data;
    const indices = buildRenderIndices(src);
    const n = indices.length;
    const xs: number[] = new Array(n);
    for (let i = 0; i < n; i++) xs[i] = src[indices[i]].time;
    const cols: (number | null)[][] = [xs];
    for (const s of series) {
      if (s.visible && s.data.length > 0) {
        const col: (number | null)[] = new Array(n);
        for (let i = 0; i < n; i++) col[i] = s.data[indices[i]]?.[s.dataKey] ?? null;
        cols.push(col);
      } else {
        cols.push(new Array(n).fill(null));
      }
    }
    return cols as uPlot.AlignedData;
  }

  function rangesEqual(
    current: { min: number; max: number } | null,
    next: { min: number; max: number } | null,
    epsilon = 0.0001,
  ): boolean {
    if (current === next) return true;
    if (!current || !next) return false;

    return (
      Math.abs(current.min - next.min) <= epsilon &&
      Math.abs(current.max - next.max) <= epsilon
    );
  }

  function resolveXRange(data: uPlot.AlignedData): { min: number; max: number } | null {
    const xs = data[0] as number[];
    if (!xs.length) return null;

    const dataMin = xs[0];
    const dataMax = xs[xs.length - 1];
    const xPad = Math.max((dataMax - dataMin) * 0.03, 0.5);

    if (_scaleRef.xMode === 'sliding') {
      return {
        min: Math.max(0, dataMax - _scaleRef.windowSize),
        max: dataMax + xPad,
      };
    }

    if (_scaleRef.xMode === 'manual' && _scaleRef.xMin != null && _scaleRef.xMax != null) {
      return {
        min: _scaleRef.xMin,
        max: _scaleRef.xMax,
      };
    }

    return {
      min: Math.min(dataMin, 0),
      max: Math.max(dataMax, 1) + xPad,
    };
  }

  function resolveYRange(data: uPlot.AlignedData): { min: number; max: number } | null {
    if (_scaleRef.yMode === 'manual') {
      return {
        min: _scaleRef.yMin,
        max: _scaleRef.yMax,
      };
    }

    let dataMin = Number.POSITIVE_INFINITY;
    let dataMax = Number.NEGATIVE_INFINITY;

    for (let columnIndex = 1; columnIndex < data.length; columnIndex += 1) {
      const values = data[columnIndex] as Array<number | null>;
      for (const value of values) {
        if (value == null || !Number.isFinite(value)) continue;
        if (value < dataMin) dataMin = value;
        if (value > dataMax) dataMax = value;
      }
    }

    if (!Number.isFinite(dataMin) || !Number.isFinite(dataMax)) {
      return { min: 0, max: 1 };
    }

    const pad = (dataMax - dataMin) * 0.05 || 1;
    return {
      min: dataMin - pad,
      max: dataMax + pad,
    };
  }

  function applyResolvedScales(data: uPlot.AlignedData) {
    if (!chart) return;

    const nextXRange = resolveXRange(data);
    const nextYRange = resolveYRange(data);

    const batch = (chart as unknown as { batch?: (fn: () => void) => void }).batch;
    const run = () => {
      if (nextXRange && !rangesEqual(lastAppliedXRange, nextXRange)) {
        chart?.setScale('x', nextXRange);
        lastAppliedXRange = nextXRange;
      }

      if (nextYRange && !rangesEqual(lastAppliedYRange, nextYRange)) {
        chart?.setScale('y', nextYRange);
        lastAppliedYRange = nextYRange;
      }
    };

    if (typeof batch === 'function') {
      batch.call(chart, run);
      return;
    }

    run();
  }

  function shouldDeferRuntimeRefresh(): boolean {
    return _pointerInside && !_panning && _scaleRef.xMode === 'manual';
  }

  function setLocalManualRange(xMin: number, xMax: number, syncChart = true) {
    const nextMin = Math.min(xMin, xMax);
    const nextMax = Math.max(xMin, xMax);

    if (!Number.isFinite(nextMin) || !Number.isFinite(nextMax) || nextMax - nextMin <= 0.0001) {
      return;
    }

    _localManualRange = { min: nextMin, max: nextMax };
    syncScaleRef();

    if (syncChart && chart) {
      chart.setScale('x', { min: nextMin, max: nextMax });
      lastAppliedXRange = { min: nextMin, max: nextMax };
    }
  }

  function clearLocalManualRange() {
    _localManualRange = null;
    syncScaleRef();
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
    const compactHeight = h < 180;
    const ultraCompactHeight = h < 120;
    const axisFont = ultraCompactHeight ? '9px system-ui, sans-serif' : compactHeight ? '10px system-ui, sans-serif' : '11px system-ui, sans-serif';
    const xAxisSize = ultraCompactHeight ? 24 : compactHeight ? 28 : 32;
    const yAxisSize = ultraCompactHeight ? 40 : compactHeight ? 44 : 50;
    const axisGap = ultraCompactHeight ? 4 : 6;

    return {
      width: w,
      height: h,
      padding: ultraCompactHeight ? [6, 8, 6, 0] : compactHeight ? [7, 10, 6, 0] : [8, 12, 8, 0],
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
          font: axisFont,
          values: (_u: uPlot, splits: number[]) => splits.map((v) => v.toFixed(1) + 's'),
          gap: axisGap,
          size: xAxisSize,
        },
        {
          stroke: colors.ticks,
          grid: { stroke: colors.grid, width: 1, show: config.showGrid },
          ticks: { stroke: colors.axis, width: 1 },
          font: axisFont,
          gap: axisGap,
          size: yAxisSize,
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
              setLocalManualRange(xMin, xMax);
              queueRangeChange(xMin, xMax);
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
    setLocalManualRange(nMin, nMax);
    queueRangeChange(nMin, nMax);
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
    setLocalManualRange(nMin, nMax);
    queueRangeChange(nMin, nMax);
  }

  function handlePointerUp(e: PointerEvent) {
    if (!_panning) return;
    _panning = false;
    if (wrapper) {
      wrapper.releasePointerCapture(e.pointerId);
      wrapper.style.cursor = '';
    }
    if (!_pointerInside) {
      flushDeferredUpdate();
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
    lastAppliedXRange = null;
    lastAppliedYRange = null;
    applyResolvedScales(data);
    prevDataSignature = getDataSignature();
    _pendingDeferredUpdate = false;
    _lastResolvedXMode = _scaleRef.xMode as 'auto' | 'sliding' | 'manual';
  }


  function updateChart(options?: { deferIfHovering?: boolean }) {
    if (!chart || !wrapper) return;
    const rect = wrapper.getBoundingClientRect();
    if (rect.width < 10 || rect.height < 10) return;

    syncScaleRef();
    if (options?.deferIfHovering && shouldDeferRuntimeRefresh()) {
      _pendingDeferredUpdate = true;
      prevDataSignature = getDataSignature();
      return;
    }

    const data = buildData();
    if (tooltipEl) {
      tooltipEl.style.display = 'none';
    }
    chart.setData(data, false);
    applyResolvedScales(data);
    prevDataSignature = getDataSignature();
    _pendingDeferredUpdate = false;

    const w = Math.floor(rect.width);
    const h = Math.floor(rect.height);
    if (Math.abs(chart.width - w) > 2 || Math.abs(chart.height - h) > 2) {
      chart.setSize({ width: w, height: h });
    }
  }

  function pollDataUpdates() {
    if (chart && wrapper) {
      syncScaleRef();
      if (_scaleRef.xMode === 'manual') {
        return;
      }

      const signature = getDataSignature();
      if (signature !== prevDataSignature) {
        updateChart({ deferIfHovering: true });
      }
    }
  }

  function flushDeferredUpdate() {
    if (!_pendingDeferredUpdate) return;
    updateChart();
  }

  onMount(() => {
    initChart();
    _mounted = true;
    renderPollTimer = window.setInterval(pollDataUpdates, 75);

    let resizeRaf: number | null = null;
    const ro = new ResizeObserver(() => {
      if (!wrapper) return;
      if (resizeRaf) cancelAnimationFrame(resizeRaf);
      resizeRaf = requestAnimationFrame(() => {
        if (!wrapper) return;
        const rect = wrapper.getBoundingClientRect();
        const w = Math.floor(rect.width);
        const h = Math.floor(rect.height);
        if (w > 10 && h > 10) {
          if (chart) {
            chart.setSize({ width: w, height: h });
            prevDataSignature = '';
          } else {
            initChart();
          }
        }
      });
    });
    if (wrapper) ro.observe(wrapper);
    return () => {
      if (resizeRaf) cancelAnimationFrame(resizeRaf);
      ro.disconnect();
    };
  });

  onDestroy(() => {
    if (renderPollTimer) window.clearInterval(renderPollTimer);
    if (rangeChangeRaf !== null) cancelAnimationFrame(rangeChangeRaf);
    if (chart) {
      chart.destroy();
      chart = null;
    }
  });

  $effect(() => {
    const _ = [config.yMode, config.showGrid, theme, seriesVisualSignature];
    untrack(() => {
      if (_mounted && chart) initChart();
    });
  });

  $effect(() => {
    const _ = [config.yMin, config.yMax];
    untrack(() => {
      if (_mounted && chart) updateChart();
    });
  });

  $effect(() => {
    const _ = [config.xMode, config.xMin, config.xMax, config.windowSize];
    untrack(() => {
      if (!_mounted || !chart) return;

      if (config.xMode !== 'manual') {
        clearLocalManualRange();
      } else if (config.xMin != null && config.xMax != null) {
        setLocalManualRange(config.xMin, config.xMax, false);
      }

      syncScaleRef();
      const resolvedMode = _scaleRef.xMode as 'auto' | 'sliding' | 'manual';
      const modeChanged = resolvedMode !== _lastResolvedXMode;
      _lastResolvedXMode = resolvedMode;

      if (modeChanged) {
        updateChart();
        return;
      }

      if (resolvedMode === 'manual') {
        const data = (chart.data ?? buildData()) as uPlot.AlignedData;
        applyResolvedScales(data);
        prevDataSignature = getDataSignature();
        return;
      }

      updateChart();
    });
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  bind:this={wrapper}
  class="plotly-surface w-full h-full uplot-wrapper"
  style="background: {colors.bg};"
  onwheel={handleWheel}
  onpointerdown={handlePointerDown}
  onpointermove={handlePointerMove}
  onpointerup={handlePointerUp}
  onpointerenter={() => {
    _pointerInside = true;
  }}
  onpointerleave={() => {
    _pointerInside = false;
    if (!_panning) {
      flushDeferredUpdate();
    }
  }}
>
  <div bind:this={tooltipEl} class="chart-tooltip"></div>
</div>

<style>
  .plotly-surface {
    min-height: 96px;
    overflow: hidden;
  }

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

  @media (max-height: 860px) {
    .plotly-surface {
      min-height: 80px;
    }
  }

  @media (max-height: 700px) {
    .plotly-surface {
      min-height: 68px;
    }
  }
</style>
