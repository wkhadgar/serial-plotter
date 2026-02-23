/**
 * Non-reactive data buffer for high-frequency plant simulation data.
 *
 * Storing large data arrays inside Svelte 5's $state() wraps them in a deep
 * reactive Proxy.  Every .push() / .splice() on that proxy fires change
 * notifications that cascade to every $derived and $effect that transitively
 * reads the array — even if those reads happen inside a Plotly render loop
 * that already has its own setInterval.  The result is O(n) proxy overhead
 * on every mutation plus redundant Plotly.react() calls driven by the
 * $effect subscription.
 *
 * By keeping the data in plain Maps outside the reactive graph we eliminate
 * that overhead entirely.  Chart components poll the plain arrays via their
 * own timers; display values (PV, MV, stats) are refreshed through a
 * lightweight $state tick counter at 5 Hz.
 */

import type { PlantDataPoint, PlantStats } from '$lib/types/plant';

// ── Plain (non-proxied) storage ──────────────────────────────────────────
const _data  = new Map<string, PlantDataPoint[]>();
const _stats = new Map<string, PlantStats>();

const DEFAULT_STATS: Readonly<PlantStats> = Object.freeze({
  errorAvg: 0,
  stability: 100,
  uptime: 0,
});

// ── Public API ───────────────────────────────────────────────────────────

/** Get (or lazily create) the data array for a plant. */
export function getPlantData(plantId: string): PlantDataPoint[] {
  let arr = _data.get(plantId);
  if (!arr) {
    arr = [];
    _data.set(plantId, arr);
  }
  return arr;
}

/** Get current stats snapshot for a plant. */
export function getPlantStats(plantId: string): PlantStats {
  return _stats.get(plantId) ?? { ...DEFAULT_STATS };
}

/** Replace stats for a plant. */
export function setPlantStats(plantId: string, stats: PlantStats): void {
  _stats.set(plantId, stats);
}

/** Wipe data + stats for a plant (e.g. on disconnect). */
export function clearPlant(plantId: string): void {
  _data.set(plantId, []);
  _stats.set(plantId, { ...DEFAULT_STATS });
}
