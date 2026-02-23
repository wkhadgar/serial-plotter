<script lang="ts">
  import { appStore } from '$lib/stores/data.svelte';
  import SidebarBtn from './SidebarBtn.svelte';
  import { TrendingUp, Activity, Sun, Moon, Settings as SettingsIcon } from 'lucide-svelte';
  import { MODULE_TABS } from '$lib/types/ui';

  let { theme, sidebarCollapsed, activeModule } = $props();

  function toggleSidebar() {
    appStore.toggleSidebar();
  }

  function toggleTheme() {
    appStore.toggleTheme();
  }

  function setShowGlobalSettings(show: boolean) {
    appStore.setShowGlobalSettings(show);
  }
</script>

<aside
  class={`${sidebarCollapsed ? 'w-16' : 'w-64'} bg-white dark:bg-zinc-900 border-r border-slate-200 dark:border-white/5 flex flex-col transition-all duration-300 z-50 shadow-sm print:hidden`}
>
  <button
    onclick={toggleSidebar}
    aria-label={sidebarCollapsed ? 'Expandir menu' : 'Recolher menu'}
    class={`h-16 flex items-center border-b border-slate-100 dark:border-white/5 transition-all duration-300 cursor-pointer hover:bg-slate-50 dark:hover:bg-white/5 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:focus:ring-blue-400 ${sidebarCollapsed ? 'justify-center' : 'px-4 gap-3'}`}
  >
    <div class="w-9 h-9 rounded-lg bg-blue-600 flex-shrink-0 flex items-center justify-center text-white shadow-md hover:scale-105 transition-transform">
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M9 20a3 3 0 0 1-3-3v-2a4 4 0 0 1 4-4h4a4 4 0 0 0 4-4V5" />
        <path d="M18 5a2 2 0 1 0 0-4 2 2 0 0 0 0 4z" />
        <path d="M18 1v-1" />
      </svg>
    </div>
    {#if !sidebarCollapsed}
      <span class="font-bold tracking-tight text-lg text-slate-800 dark:text-white whitespace-nowrap">
        SCA<span class="text-blue-600">DA</span>.js
      </span>
    {/if}
  </button>

  <nav class="flex-1 py-6 px-3 space-y-2">
    <SidebarBtn
      icon={TrendingUp}
      label={MODULE_TABS.plotter.label}
      active={activeModule === 'plotter'}
      collapsed={sidebarCollapsed}
      onclick={() => appStore.setActiveModule('plotter')}
    />
    <SidebarBtn
      icon={Activity}
      label={MODULE_TABS.poles.label}
      active={activeModule === 'poles'}
      collapsed={sidebarCollapsed}
      onclick={() => appStore.setActiveModule('poles')}
    />
  </nav>

  <div class="p-3 space-y-2 border-t border-slate-200 dark:border-white/5 bg-slate-50 dark:bg-white/[0.02]">
    <SidebarBtn
      icon={theme === 'dark' ? Sun : Moon}
      label={theme === 'dark' ? 'Modo Claro' : 'Modo Escuro'}
      collapsed={sidebarCollapsed}
      onclick={toggleTheme}
    />
    <SidebarBtn
      icon={SettingsIcon}
      label="Ajustes"
      collapsed={sidebarCollapsed}
      onclick={() => setShowGlobalSettings(true)}
    />
  </div>
</aside>

