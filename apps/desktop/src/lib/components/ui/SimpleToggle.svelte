<script lang="ts">
  let { 
    checked = $bindable(false), 
    onchange = undefined as (() => void) | undefined,
    ariaLabel 
  } = $props();

  function toggle() {
    checked = !checked;
    if (onchange) onchange();
  }
</script>

<button
  type="button"
  onclick={toggle}
  role="switch"
  aria-checked={checked}
  aria-label={ariaLabel || (checked ? 'Desabilitar' : 'Habilitar')}
  class={`w-9 h-5 rounded-full relative transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:focus:ring-blue-400 ${checked ? 'bg-emerald-500' : 'bg-slate-300 dark:bg-zinc-600'}`}
>
  <span
    class={`absolute top-0.5 left-0.5 w-4 h-4 bg-white rounded-full shadow-sm transition-transform duration-200 ${checked ? 'translate-x-4' : 'translate-x-0'}`}
  ></span>
</button>

