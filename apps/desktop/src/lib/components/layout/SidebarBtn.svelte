<script lang="ts">
  interface Props {
    icon: any;
    label: string;
    active?: boolean;
    collapsed?: boolean;
    onclick?: () => void;
  }

  let { icon: Icon, label, active = false, collapsed = false, onclick } = $props();
</script>

<button
  {onclick}
  aria-current={active ? 'page' : 'false'}
  title={collapsed ? label : ''}
  class={`
    w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all relative group
    ${active
      ? 'bg-blue-50 dark:bg-blue-900/10 text-blue-600 dark:text-blue-400 border border-blue-200 dark:border-blue-900/20'
      : 'text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-white/5 hover:text-slate-900 dark:hover:text-zinc-200 border border-transparent'
    }
    focus:outline-none focus:ring-2 focus:ring-blue-500 dark:focus:ring-blue-400 focus:ring-offset-2 dark:focus:ring-offset-zinc-900
  `}
>
  <div class={active ? 'text-blue-600 dark:text-blue-400' : 'text-slate-500 dark:text-slate-500'}>
    <Icon size={20} />
  </div>
  {#if !collapsed}
    <span class="text-sm font-medium">{label}</span>
  {/if}
</button>

