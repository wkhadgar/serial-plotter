<script lang="ts">
  import SimpleToggle from './SimpleToggle.svelte';
  import type { ParamType } from '$lib/types/controller';

  interface Props {
    label: string;
    type: ParamType;
    value: number | boolean | string;
    onChange?: (value: number | boolean | string) => void;
    onValidityChange?: (isValid: boolean) => void;
  }

  let { label, type, value, onChange, onValidityChange } = $props();
  const id = `param-${crypto.randomUUID().substring(0, 8)}`;

  let draftValue = $state('');
  let error = $state<string | null>(null);
  let lastReportedValidity = $state<boolean | null>(null);

  function notifyValidity(isValid: boolean) {
    if (lastReportedValidity === isValid) {
      return;
    }

    lastReportedValidity = isValid;
    onValidityChange?.(isValid);
  }

  function syncDraft(nextValue: number | boolean | string) {
    const normalized = nextValue === null || nextValue === undefined ? '' : String(nextValue);
    if (draftValue !== normalized) {
      draftValue = normalized;
    }
  }

  $effect(() => {
    if (type === 'boolean') {
      error = null;
      notifyValidity(true);
      return;
    }

    syncDraft(value);
    error = null;
    notifyValidity(true);
  });

  function handleToggleChange() {
    error = null;
    notifyValidity(true);
    onChange?.(!(value as boolean));
  }

  function handleTextInput(event: Event) {
    const nextValue = (event.target as HTMLInputElement).value;
    draftValue = nextValue;
    error = null;
    notifyValidity(true);
    onChange?.(nextValue);
  }

  function handleNumberInput(event: Event) {
    const nextValue = (event.target as HTMLInputElement).value;
    draftValue = nextValue;

    if (!nextValue.trim()) {
      error = 'Informe um numero valido';
      notifyValidity(false);
      return;
    }

    const parsed = Number(nextValue);
    if (!Number.isFinite(parsed)) {
      error = 'Informe um numero valido';
      notifyValidity(false);
      return;
    }

    error = null;
    notifyValidity(true);
    onChange?.(parsed);
  }
</script>

{#if type === 'boolean'}
  <div class="flex items-center justify-between py-1">
    <label for={id} class="text-xs font-medium text-slate-600 dark:text-slate-400">{label}</label>
    <SimpleToggle
      checked={value as boolean}
      onchange={handleToggleChange}
      ariaLabel={label}
    />
  </div>
{:else}
  <div class="flex items-start justify-between gap-3 group">
    <label for={id} class="pt-2 text-xs font-medium text-slate-600 dark:text-slate-400 truncate flex-1">{label}</label>
    <div class="relative w-28">
      <input
        {id}
        type={type === 'number' ? 'number' : 'text'}
        step={type === 'number' ? '0.01' : undefined}
        value={draftValue}
        oninput={type === 'number' ? handleNumberInput : handleTextInput}
        class={`
          w-full bg-white dark:bg-zinc-800 border border-slate-300 dark:border-white/10 rounded-md px-2.5 py-1.5 text-xs text-slate-900 dark:text-zinc-200 outline-none
          focus:border-blue-500 focus:ring-1 focus:ring-blue-500/20 transition-all shadow-sm
          ${error ? 'border-red-400 focus:border-red-500 focus:ring-red-500/20' : ''}
          ${type === 'number' ? 'font-mono text-center' : 'text-left'}
        `}
      />
      {#if error}
        <div class="mt-1 text-[10px] text-red-500 dark:text-red-400 text-right">
          {error}
        </div>
      {/if}
    </div>
  </div>
{/if}
