<script lang="ts">
  import { appStore } from '$lib/stores/data.svelte';
  import SidebarBtn from './SidebarBtn.svelte';
  import { TrendingUp, BarChart3, Puzzle, Sun, Moon, Settings as SettingsIcon, X } from 'lucide-svelte';
  import { MODULE_TABS } from '$lib/types/ui';

  interface Props {
    theme: 'dark' | 'light';
    sidebarCollapsed: boolean;
    activeModule: string;
    isMobile?: boolean;
    mobileOpen?: boolean;
    onNavigate?: () => void;
    onRequestClose?: () => void;
  }

  let {
    theme,
    sidebarCollapsed,
    activeModule,
    isMobile = false,
    mobileOpen = false,
    onNavigate,
    onRequestClose,
  }: Props = $props();

  const isCollapsed = $derived(isMobile ? false : sidebarCollapsed);

  function toggleSidebar() {
    appStore.toggleSidebar();
  }

  function toggleTheme() {
    appStore.toggleTheme();
  }

  function setShowGlobalSettings(show: boolean) {
    appStore.setShowGlobalSettings(show);
  }

  function handleHeaderAction() {
    if (isMobile) {
      onRequestClose?.();
      return;
    }

    toggleSidebar();
  }

  function selectModule(module: 'plotter' | 'analyzer' | 'plugins') {
    appStore.setActiveModule(module);
    onNavigate?.();
  }
</script>

<aside
  class={`print:hidden ${
    isMobile
      ? `fixed inset-y-0 left-0 z-50 w-72 transform border-r border-slate-200 bg-white/95 shadow-xl backdrop-blur transition-transform duration-300 dark:border-white/10 dark:bg-zinc-900/95 ${mobileOpen ? 'translate-x-0' : '-translate-x-full'}`
      : `${isCollapsed ? 'w-16' : 'w-64'} z-30 flex-shrink-0 border-r border-slate-200 bg-white/90 shadow-sm backdrop-blur transition-all duration-300 dark:border-white/5 dark:bg-zinc-900/90`
  } flex flex-col`}
>
  <button
    onclick={handleHeaderAction}
    aria-label={isMobile ? 'Fechar menu' : isCollapsed ? 'Expandir menu' : 'Recolher menu'}
    class={`h-16 flex items-center border-b border-slate-100 transition-all duration-300 cursor-pointer hover:bg-slate-50 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-white/5 dark:hover:bg-white/5 dark:focus:ring-blue-400 ${isCollapsed ? 'justify-center' : 'justify-between px-4 gap-3'}`}
  >
    <div class={`flex items-center ${isCollapsed ? '' : 'gap-3'}`}>
      <div class="w-9 h-9 rounded-lg bg-gradient-to-br from-blue-500 to-blue-700 flex-shrink-0 flex items-center justify-center text-white shadow-md hover:scale-105 transition-transform">
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
        </svg>
      </div>
      {#if !isCollapsed}
        <span class="font-bold tracking-tight text-lg text-slate-800 dark:text-white whitespace-nowrap">
          Sena<span class="text-blue-600">mby</span>
        </span>
      {/if}
    </div>

    {#if isMobile}
      <span class="rounded-lg p-2 text-slate-400 transition-colors hover:bg-slate-100 hover:text-slate-700 dark:hover:bg-white/10 dark:hover:text-zinc-200">
        <X size={18} />
      </span>
    {/if}
  </button>

  <nav class="flex-1 py-3 px-2 space-y-1">
    <SidebarBtn
      icon={TrendingUp}
      label={MODULE_TABS.plotter.label}
      active={activeModule === 'plotter'}
      collapsed={isCollapsed}
      onclick={() => selectModule('plotter')}
    />
    <SidebarBtn
      icon={BarChart3}
      label={MODULE_TABS.analyzer.label}
      active={activeModule === 'analyzer'}
      collapsed={isCollapsed}
      onclick={() => selectModule('analyzer')}
    />
    <SidebarBtn
      icon={Puzzle}
      label={MODULE_TABS.plugins.label}
      active={activeModule === 'plugins'}
      collapsed={isCollapsed}
      onclick={() => selectModule('plugins')}
    />
  </nav>

  <div class="p-2 space-y-1 border-t border-slate-200 dark:border-white/5 bg-slate-50/80 dark:bg-white/[0.02]">
    <SidebarBtn
      icon={theme === 'dark' ? Sun : Moon}
      label={theme === 'dark' ? 'Modo Claro' : 'Modo Escuro'}
      collapsed={isCollapsed}
      onclick={toggleTheme}
    />
    <SidebarBtn
      icon={SettingsIcon}
      label="Ajustes"
      collapsed={isCollapsed}
      onclick={() => setShowGlobalSettings(true)}
    />
  </div>
</aside>
