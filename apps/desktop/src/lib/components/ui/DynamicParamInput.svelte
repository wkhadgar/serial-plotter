<script lang="ts">
  import SimpleToggle from './SimpleToggle.svelte';
  import type { ParamType } from '$lib/types/controller';

  interface Props {
    label: string;
    type: ParamType;
    value: number | boolean | string;
    onChange?: (value: number | boolean | string) => void;
  }

  let { label, type, value, onChange } = $props();
  const id = `param-${crypto.randomUUID().substring(0, 8)}`;

  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  const DEBOUNCE_MS = 75;

  function handleChange(e: Event) {
    const target = e.target as HTMLInputElement;
    if (type === 'number') {
      if (debounceTimer) clearTimeout(debounceTimer);
      debounceTimer = setTimeout(() => {
        const floatVal = parseFloat(target.value);
        if (onChange) onChange(isNaN(floatVal) ? '' : floatVal);
      }, DEBOUNCE_MS);
    } else if (type === 'boolean') {
      if (onChange) onChange(target.checked);
    } else {
      if (onChange) onChange(target.value);
    }
  }
</script>

{#if type === 'boolean'}
  <div class="flex items-center justify-between py-1">
    <label for={id} class="text-xs font-medium text-slate-600 dark:text-slate-400">{label}</label>
    <SimpleToggle
      checked={value as boolean}
      onchange={() => {
        if (onChange) onChange(!(value as boolean));
      }}
      ariaLabel={label}
    />
  </div>
{:else}
  <div class="flex items-center justify-between gap-3 group">
    <label for={id} class="text-xs font-medium text-slate-600 dark:text-slate-400 truncate flex-1">{label}</label>
    <div class="relative w-24">
      <input
        {id}
        type={type === 'number' ? 'number' : 'text'}
        step={type === 'number' ? '0.01' : undefined}
        value={value === null || value === undefined || (typeof value === 'number' && isNaN(value)) ? '' : value}
        onchange={handleChange}
        class={`
          w-full bg-white dark:bg-zinc-800 border border-slate-300 dark:border-white/10 rounded-md px-2.5 py-1.5 text-xs text-slate-900 dark:text-zinc-200 outline-none
          focus:border-blue-500 focus:ring-1 focus:ring-blue-500/20 transition-all shadow-sm
          ${type === 'number' ? 'font-mono text-center' : 'text-left'}
        `}
      />
    </div>
  </div>
{/if}

