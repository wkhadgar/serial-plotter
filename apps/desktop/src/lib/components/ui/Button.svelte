<script lang="ts">
  type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'success';
  type ButtonSize = 'sm' | 'md' | 'lg';

  interface Props {
    variant?: ButtonVariant;
    size?: ButtonSize;
    disabled?: boolean;
    ariaLabel?: string;
    onclick?: (e: MouseEvent) => void;
    children?: any;
  }

  let { variant = 'primary', size = 'md', disabled = false, ariaLabel, onclick, children } = $props();

  const sizeClasses = {
    sm: 'px-2.5 py-1.5 text-xs',
    md: 'px-4 py-2 text-sm',
    lg: 'px-6 py-3 text-base'
  } as const;

  const variantClasses = {
    primary: 'bg-blue-600 hover:bg-blue-700 text-white border border-blue-600 disabled:bg-slate-300 disabled:border-slate-300',
    secondary: 'bg-slate-100 dark:bg-white/10 hover:bg-slate-200 dark:hover:bg-white/15 text-slate-900 dark:text-white border border-slate-200 dark:border-white/20 disabled:opacity-50',
    ghost: 'bg-transparent text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-white/5 border border-transparent disabled:opacity-50',
    danger: 'bg-red-50 dark:bg-red-900/20 hover:bg-red-100 dark:hover:bg-red-900/30 text-red-600 dark:text-red-400 border border-red-200 dark:border-red-900/30 disabled:opacity-50',
    success: 'bg-emerald-50 dark:bg-emerald-900/20 hover:bg-emerald-100 dark:hover:bg-emerald-900/30 text-emerald-600 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-900/30 disabled:opacity-50'
  } as const;

  const classes = $derived(`${sizeClasses[size as ButtonSize]} ${variantClasses[variant as ButtonVariant]} rounded-lg font-medium transition-all focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 dark:focus:ring-offset-0 disabled:cursor-not-allowed`);
</script>

<button {disabled} {onclick} aria-label={ariaLabel} class={classes}>
  {#if children}
    {@render children()}
  {/if}
</button>
