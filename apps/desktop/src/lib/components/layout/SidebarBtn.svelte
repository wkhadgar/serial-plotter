<script lang="ts">
  import type { ComponentType } from 'svelte';

  interface Props {
    icon: ComponentType;
    label: string;
    active?: boolean;
    collapsed?: boolean;
    onclick?: () => void;
  }

  let { icon: Icon, label, active = false, collapsed = false, onclick }: Props = $props();
</script>

<button
  {onclick}
  aria-current={active ? 'page' : 'false'}
  title={collapsed ? label : ''}
  class={`
    w-full flex items-center rounded-xl transition-all relative group
    ${collapsed ? 'justify-center p-2.5' : 'gap-3 px-3.5 py-2.5'}
    ${active
      ? 'bg-blue-50 text-blue-700 shadow-sm ring-1 ring-blue-100 dark:bg-blue-500/10 dark:text-blue-300 dark:ring-blue-500/20'
      : 'text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-white/5 hover:text-slate-900 dark:hover:text-zinc-200'
    }
    focus:outline-none focus:ring-2 focus:ring-blue-500 dark:focus:ring-blue-400 focus:ring-offset-2 dark:focus:ring-offset-zinc-900
  `}
>
  <div class={`flex-shrink-0 ${active ? 'text-blue-600 dark:text-blue-400' : 'text-slate-500 dark:text-slate-500'}`}>
    <Icon size={20} />
  </div>
  {#if !collapsed}
    <span class="text-sm font-medium truncate">{label}</span>
  {/if}
</button>
