<script lang="ts">
  import { onMount } from 'svelte';

  interface DataPoint {
    time: number;
    [key: string]: number;
  }

  interface DataSeries {
    key: string;
    label: string;
    color: string;
    visible: boolean;
    data: DataPoint[];  // Raw data array with time and values
    dataKey: string;    // Which property to plot (pv, sp, mv)
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
  }

  interface Props {
    series: DataSeries[];
    config: ChartConfig;
    theme: 'dark' | 'light';
    height?: string;
    onZoom?: (factor: number, centerTime: number) => void;
    onPan?: (deltaTime: number) => void;
  }

  let { series, config, theme, height = '100%', onZoom, onPan }: Props = $props();

  let canvas: HTMLCanvasElement;
  let container: HTMLDivElement;
  let ctx: CanvasRenderingContext2D | null = null;
  let dpr = 1;
  
  // Throttling state - limit to 10 FPS for efficiency
  let lastDrawTime = 0;
  let pendingDraw = false;
  let animationFrameId: number | null = null;
  const TARGET_FPS = 10;
  const FRAME_INTERVAL = 1000 / TARGET_FPS;

  // Computed colors based on theme (simple object, rarely changes)
  const colors = $derived({
    background: theme === 'dark' ? '#0c0c0e' : '#ffffff',
    grid: theme === 'dark' ? '#1f1f23' : '#f1f5f9',
    gridMajor: theme === 'dark' ? '#27272a' : '#e2e8f0',
    axis: theme === 'dark' ? '#3f3f46' : '#cbd5e1',
    text: theme === 'dark' ? '#71717a' : '#94a3b8',
    limitLine: '#ef4444'
  });

  // Get raw data from first series (all series share the same data source)
  const rawData = $derived(series.length > 0 ? series[0].data : []);
  const dataLength = $derived(rawData.length);

  // Calculate visible data range
  const visibleRange = $derived.by(() => {
    if (!dataLength) return { start: 0, end: 0, timeStart: 0, timeEnd: 1 };
    
    const lastTime = rawData[dataLength - 1].time;
    const firstTime = rawData[0].time;
    
    let timeStart: number, timeEnd: number;
    
    if (config.xMode === 'sliding') {
      timeEnd = lastTime;
      timeStart = Math.max(0, lastTime - config.windowSize);
    } else if (config.xMode === 'manual' && config.xMin != null && config.xMax != null) {
      timeStart = config.xMin;
      timeEnd = config.xMax;
    } else {
      // Auto - show ALL data from time 0 to current time
      timeStart = 0;
      timeEnd = Math.max(lastTime, 1);
    }
    
    // Binary search for start index (more efficient)
    let start = 0;
    let end = dataLength - 1;
    
    // Simple linear search for small datasets, binary for large
    if (dataLength < 1000) {
      for (let i = 0; i < dataLength; i++) {
        if (rawData[i].time >= timeStart) {
          start = Math.max(0, i - 1);
          break;
        }
      }
      for (let i = dataLength - 1; i >= 0; i--) {
        if (rawData[i].time <= timeEnd) {
          end = Math.min(dataLength - 1, i + 1);
          break;
        }
      }
    } else {
      // Binary search for large datasets
      let lo = 0, hi = dataLength - 1;
      while (lo < hi) {
        const mid = (lo + hi) >> 1;
        if (rawData[mid].time < timeStart) lo = mid + 1;
        else hi = mid;
      }
      start = Math.max(0, lo - 1);
      
      lo = 0; hi = dataLength - 1;
      while (lo < hi) {
        const mid = (lo + hi + 1) >> 1;
        if (rawData[mid].time > timeEnd) hi = mid - 1;
        else lo = mid;
      }
      end = Math.min(dataLength - 1, hi + 1);
    }
    
    return { start, end, timeStart, timeEnd };
  });

  // Calculate Y bounds
  const yBounds = $derived.by(() => {
    if (config.yMode === 'manual') {
      return { min: config.yMin, max: config.yMax };
    }
    
    // Auto-calculate from visible data
    let min = Infinity;
    let max = -Infinity;
    
    const { start, end } = visibleRange;
    
    for (const s of series) {
      if (!s.visible) continue;
      const key = s.dataKey;
      for (let i = start; i <= end && i < s.data.length; i++) {
        const val = s.data[i][key];
        if (val < min) min = val;
        if (val > max) max = val;
      }
    }
    
    // Add padding
    const range = max - min || 1;
    min = min - range * 0.1;
    max = max + range * 0.1;
    
    return { min, max };
  });

  function setupCanvas() {
    if (!canvas || !container) return;
    
    dpr = window.devicePixelRatio || 1;
    const rect = container.getBoundingClientRect();
    
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    canvas.style.width = `${rect.width}px`;
    canvas.style.height = `${rect.height}px`;
    
    ctx = canvas.getContext('2d');
    if (ctx) {
      ctx.scale(dpr, dpr);
    }
  }

  function drawChart() {
    if (!ctx || !canvas || !container) return;
    
    const rect = container.getBoundingClientRect();
    const width = rect.width;
    const height = rect.height;
    
    const padding = { top: 20, right: 50, bottom: 30, left: 20 };
    const chartWidth = width - padding.left - padding.right;
    const chartHeight = height - padding.top - padding.bottom;
    
    // Clear canvas
    ctx.fillStyle = colors.background;
    ctx.fillRect(0, 0, width, height);
    
    if (!dataLength || !series.some(s => s.visible && s.data.length)) {
      // Draw empty state
      ctx.fillStyle = colors.text;
      ctx.font = '14px system-ui, sans-serif';
      ctx.textAlign = 'center';
      ctx.fillText('Aguardando dados...', width / 2, height / 2);
      return;
    }
    
    const { start, end, timeStart, timeEnd } = visibleRange;
    const { min: yMin, max: yMax } = yBounds;
    const yRange = yMax - yMin || 1;
    const xRange = timeEnd - timeStart || 1;
    
    // Transform functions
    const toX = (time: number) => padding.left + ((time - timeStart) / xRange) * chartWidth;
    const toY = (value: number) => padding.top + chartHeight - ((value - yMin) / yRange) * chartHeight;
    
    // Draw grid
    if (config.showGrid) {
      ctx.strokeStyle = colors.grid;
      ctx.lineWidth = 1;
      
      // Horizontal grid lines
      const ySteps = 5;
      for (let i = 0; i <= ySteps; i++) {
        const y = padding.top + (i / ySteps) * chartHeight;
        ctx.beginPath();
        ctx.moveTo(padding.left, y);
        ctx.lineTo(width - padding.right, y);
        ctx.stroke();
      }
      
      // Vertical grid lines
      const xSteps = Math.min(10, Math.floor(chartWidth / 60));
      for (let i = 0; i <= xSteps; i++) {
        const x = padding.left + (i / xSteps) * chartWidth;
        ctx.beginPath();
        ctx.moveTo(x, padding.top);
        ctx.lineTo(x, height - padding.bottom);
        ctx.stroke();
      }
    }
    
    // Draw limit lines
    if (config.showLimits) {
      ctx.strokeStyle = colors.limitLine;
      ctx.lineWidth = 1;
      ctx.setLineDash([4, 4]);
      
      if (config.limitHigh != null) {
        const y = toY(config.limitHigh);
        if (y >= padding.top && y <= height - padding.bottom) {
          ctx.beginPath();
          ctx.moveTo(padding.left, y);
          ctx.lineTo(width - padding.right, y);
          ctx.stroke();
        }
      }
      
      if (config.limitLow != null) {
        const y = toY(config.limitLow);
        if (y >= padding.top && y <= height - padding.bottom) {
          ctx.beginPath();
          ctx.moveTo(padding.left, y);
          ctx.lineTo(width - padding.right, y);
          ctx.stroke();
        }
      }
      
      ctx.setLineDash([]);
    }
    
    // Draw data series
    for (const s of series) {
      if (!s.visible || !s.data.length) continue;
      
      const dataKey = s.dataKey;
      
      ctx.strokeStyle = s.color;
      ctx.lineWidth = s.strokeWidth || 2;
      ctx.lineJoin = 'round';
      ctx.lineCap = 'round';
      
      if (s.dashed) {
        ctx.setLineDash([4, 4]);
      } else {
        ctx.setLineDash([]);
      }
      
      // Draw area fill for area type
      if (s.type === 'area') {
        ctx.beginPath();
        const baseY = toY(yMin);
        
        let firstPoint = true;
        for (let i = start; i <= end && i < s.data.length; i++) {
          const point = s.data[i];
          const x = toX(point.time);
          const y = toY(point[dataKey]);
          
          if (firstPoint) {
            ctx.moveTo(x, baseY);
            ctx.lineTo(x, y);
            firstPoint = false;
          } else {
            ctx.lineTo(x, y);
          }
        }
        
        // Close the area
        if (!firstPoint && end < s.data.length) {
          ctx.lineTo(toX(s.data[Math.min(end, s.data.length - 1)].time), baseY);
          ctx.closePath();
          
          // Create gradient fill
          const gradient = ctx.createLinearGradient(0, padding.top, 0, height - padding.bottom);
          gradient.addColorStop(0, s.color + '40');
          gradient.addColorStop(1, s.color + '05');
          ctx.fillStyle = gradient;
          ctx.fill();
        }
      }
      
      // Draw line
      ctx.beginPath();
      let firstPoint = true;
      let lastX = 0, lastY = 0;
      
      for (let i = start; i <= end && i < s.data.length; i++) {
        const point = s.data[i];
        const x = toX(point.time);
        const y = toY(point[dataKey]);
        
        if (firstPoint) {
          ctx.moveTo(x, y);
          firstPoint = false;
        } else {
          if (s.type === 'step') {
            ctx.lineTo(x, lastY);
            ctx.lineTo(x, y);
          } else {
            ctx.lineTo(x, y);
          }
        }
        
        lastX = x;
        lastY = y;
      }
      
      ctx.stroke();
    }
    
    // Draw Y axis labels
    ctx.fillStyle = colors.text;
    ctx.font = '10px ui-monospace, monospace';
    ctx.textAlign = 'right';
    ctx.textBaseline = 'middle';
    
    const ySteps = 5;
    for (let i = 0; i <= ySteps; i++) {
      const value = yMax - (i / ySteps) * yRange;
      const y = padding.top + (i / ySteps) * chartHeight;
      ctx.fillText(value.toFixed(1), width - padding.right + 40, y);
    }
    
    // Draw X axis labels
    ctx.textAlign = 'center';
    ctx.textBaseline = 'top';
    
    const xSteps = Math.min(6, Math.floor(chartWidth / 80));
    for (let i = 0; i <= xSteps; i++) {
      const time = timeStart + (i / xSteps) * xRange;
      const x = padding.left + (i / xSteps) * chartWidth;
      const minutes = Math.floor(time / 60);
      const label = formatTimeLabel(time);
      ctx.fillText(label, x, height - padding.bottom + 8);
    }
    
    ctx.setLineDash([]);
  }

  // Format time labels based on the time range
  function formatTimeLabel(time: number): string {
    if (time < 60) {
      return `${time.toFixed(1)}s`;
    } else if (time < 3600) {
      const minutes = Math.floor(time / 60);
      const seconds = Math.floor(time % 60);
      return `${minutes}:${seconds.toString().padStart(2, '0')}`;
    } else {
      const hours = Math.floor(time / 3600);
      const minutes = Math.floor((time % 3600) / 60);
      return `${hours}h${minutes.toString().padStart(2, '0')}`;
    }
  }

  // Handle wheel zoom
  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    
    if (!container || !dataLength || !onZoom) return;
    
    const rect = container.getBoundingClientRect();
    const padding = { left: 20, right: 50 };
    const chartWidth = rect.width - padding.left - padding.right;
    
    // Calculate mouse position as percentage of chart width
    const mouseX = e.clientX - rect.left - padding.left;
    const mousePercent = Math.max(0, Math.min(1, mouseX / chartWidth));
    
    // Calculate the time at mouse position
    const { timeStart, timeEnd } = visibleRange;
    const centerTime = timeStart + mousePercent * (timeEnd - timeStart);
    
    // Determine zoom factor (scroll up = zoom in, scroll down = zoom out)
    const zoomFactor = e.deltaY > 0 ? 1.2 : 0.8;
    
    onZoom(zoomFactor, centerTime);
  }

  // Handle mouse drag for panning
  let isDragging = false;
  let lastMouseX = 0;

  function handleMouseDown(e: MouseEvent) {
    if (e.button !== 0) return; // Only left mouse button
    isDragging = true;
    lastMouseX = e.clientX;
    canvas.style.cursor = 'grabbing';
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isDragging || !container || !onPan) return;
    
    const rect = container.getBoundingClientRect();
    const padding = { left: 20, right: 50 };
    const chartWidth = rect.width - padding.left - padding.right;
    
    const deltaX = e.clientX - lastMouseX;
    lastMouseX = e.clientX;
    
    // Convert pixel delta to time delta
    const { timeStart, timeEnd } = visibleRange;
    const timeRange = timeEnd - timeStart;
    const deltaTime = -(deltaX / chartWidth) * timeRange;
    
    onPan(deltaTime);
  }

  function handleMouseUp() {
    isDragging = false;
    if (canvas) canvas.style.cursor = 'crosshair';
  }

  // Throttled draw function - limits to TARGET_FPS
  function scheduleRedraw() {
    if (pendingDraw) return;
    
    const now = performance.now();
    const elapsed = now - lastDrawTime;
    
    if (elapsed >= FRAME_INTERVAL) {
      // Enough time passed, draw immediately
      lastDrawTime = now;
      drawChart();
    } else {
      // Schedule draw for next available frame
      pendingDraw = true;
      animationFrameId = requestAnimationFrame(() => {
        pendingDraw = false;
        lastDrawTime = performance.now();
        drawChart();
      });
    }
  }

  onMount(() => {
    setupCanvas();
    drawChart();
    
    const resizeObserver = new ResizeObserver(() => {
      setupCanvas();
      scheduleRedraw();
    });
    
    if (container) {
      resizeObserver.observe(container);
    }
    
    // Add global mouse listeners for dragging
    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
    
    return () => {
      resizeObserver.disconnect();
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
      if (animationFrameId) {
        cancelAnimationFrame(animationFrameId);
      }
    };
  });

  // Redraw only when data or config changes (reactive) with throttling
  $effect(() => {
    // Track only essential dependencies (primitive values)
    const len = dataLength;
    const xMode = config.xMode;
    const yMode = config.yMode;
    const themeVal = theme;
    
    // Only draw if canvas is ready
    if (ctx && canvas && container && len > 0) {
      scheduleRedraw();
    }
  });
</script>

<div bind:this={container} class="w-full" style="height: {height}">
  <canvas 
    bind:this={canvas} 
    class="w-full h-full cursor-crosshair"
    onwheel={handleWheel}
    onmousedown={handleMouseDown}
  ></canvas>
</div>
