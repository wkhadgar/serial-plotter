<script lang="ts">
  interface Props {
    isOpen: boolean;
    onClose: () => void;
    title?: string;
    size?: 'sm' | 'md' | 'lg';
    children?: any;
  }

  let { isOpen, onClose, title, size = 'md', children } = $props();

  const sizeClasses = {
    sm: 'max-w-sm',
    md: 'max-w-md',
    lg: 'max-w-lg'
  } as const;

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) onClose();
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
  }
</script>

{#if isOpen}
  <div
    role="dialog"
    aria-modal="true"
    aria-labelledby={title ? 'modal-title' : undefined}
    onkeydown={handleKeyDown}
    tabindex="0"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm p-4"
    onclick={handleBackdropClick}
  >
    <div class="w-full {sizeClasses[size as 'sm' | 'md' | 'lg']} bg-white dark:bg-zinc-900 rounded-xl shadow-2xl border border-slate-200 dark:border-white/10 overflow-hidden">
      {#if title}
        <div class="px-6 py-4 border-b border-slate-200 dark:border-white/5">
          <h2 id="modal-title" class="text-lg font-bold text-slate-900 dark:text-white">{title}</h2>
        </div>
      {/if}
      <div class="p-6">
        {#if children}
          {@render children()}
        {/if}
      </div>
    </div>
  </div>
{/if}
